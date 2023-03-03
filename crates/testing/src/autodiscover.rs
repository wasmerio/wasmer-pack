use std::{
    collections::{BTreeSet, HashSet},
    env,
    fmt::Display,
    fs,
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};

use anyhow::{Context, Error, Ok};
use ignore::{overrides::OverrideBuilder, Walk, WalkBuilder};
use insta::Settings;
use wasmer_pack_cli::Language;

const JEST_CONFIG: &str = include_str!("./jest.config.js");

pub fn autodiscover(crate_dir: impl AsRef<Path>) -> Result<(), Error> {
    let start = Instant::now();

    let crate_dir = crate_dir.as_ref();
    tracing::info!(dir = %crate_dir.display(), "Looking for tests");

    let manifest_path = crate_dir.join("Cargo.toml");
    let temp = tempfile::tempdir().context("Unable to create a temporary directory")?;

    tracing::info!(?temp, "Compiling the crate and generating a WAPM package");
    let wapm_package = crate::compile_rust_to_wapm_package(&manifest_path, temp.path())?;

    let generated_bindings = crate_dir.join("generated_bindings");

    if generated_bindings.exists() {
        tracing::info!("Deleting bindings from a previous run");
        std::fs::remove_dir_all(&generated_bindings)
            .context("Unable to delete the old generated bindings")?;
    }

    for language in detected_languages(crate_dir) {
        let bindings = generated_bindings.join(language.name());
        tracing::info!(
            bindings_dir = %bindings.display(),
            language = language.name(),
            "Generating bindings",
        );
        crate::generate_bindings(&bindings, &wapm_package, language)?;

        match language {
            Language::JavaScript => {
                setup_javascript(crate_dir, &bindings)?;
                run_jest(crate_dir)?;
            }
            Language::Python => {
                setup_python(crate_dir, &bindings)?;
                run_pytest(crate_dir)?;
            }
        }

        snapshot_generated_bindings(crate_dir, &bindings, language)?;
    }

    tracing::info!(duration = ?start.elapsed(), "Testing complete");

    Ok(())
}

fn detected_languages(crate_dir: &Path) -> HashSet<Language> {
    let mut languages = HashSet::new();

    for entry in Walk::new(crate_dir).filter_map(|entry| entry.ok()) {
        match entry.path().extension().and_then(|s| s.to_str()) {
            Some("py") => {
                languages.insert(Language::Python);
            }
            Some("mjs") | Some("js") | Some("ts") => {
                languages.insert(Language::JavaScript);
            }
            _ => {}
        }
    }
    languages
}

fn snapshot_generated_bindings(
    crate_dir: &Path,
    package_dir: &Path,
    language: Language,
) -> Result<(), Error> {
    tracing::info!(
        package_dir=%package_dir.display(),
        language=language.name(),
        "Creating snapshot tests for the generated bindings",
    );

    let snapshot_files: BTreeSet<_> = language_specific_matches(package_dir, language)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.into_path())
        .collect();

    let mut settings = Settings::clone_current();
    settings.set_snapshot_path(crate_dir.join("snapshots").join(language.name()));
    settings.set_prepend_module_to_snapshot(false);
    settings.set_input_file(package_dir);
    settings.set_omit_expression(true);
    // We want to ignore version strings because it makes tests fail when you
    // make new versions
    settings.add_filter(r#""\d+\.\d+\.\d+""#, r#""x.y.z""#);
    // Also ignore the generator version comments
    settings.add_filter(r"wasmer-pack v\d+\.\d+\.\d+", "wasmer-pack vX.Y.Z");

    let _guard = settings.bind_to_scope();

    insta::assert_debug_snapshot!(
        "all files",
        snapshot_files
            .iter()
            .map(|path| path.strip_prefix(crate_dir).expect("unreachable"))
            .collect::<Vec<_>>()
    );

    for path in snapshot_files {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Unable to read \"{}\"", path.display()))?;

        let mut settings = Settings::clone_current();
        let simplified_path = path.strip_prefix(package_dir)?;
        settings.set_input_file(&path);
        let _guard = settings.bind_to_scope();

        let snapshot_name = simplified_path.display().to_string();
        insta::assert_display_snapshot!(snapshot_name, &contents);
    }

    Ok(())
}

fn language_specific_matches(package_dir: &Path, language: Language) -> Result<Walk, Error> {
    let mut builder = OverrideBuilder::new(package_dir);

    let overrides = match language {
        Language::JavaScript => builder
            .add("!node_modules")?
            .add("*.ts")?
            .add("*.test.ts")?
            .add("*.mjs")?
            .add("*.test.mjs")?
            .add("*.js")?
            .add("*.test.js")?
            .build()?,
        Language::Python => builder
            .add("*.py")?
            .add("*.toml")?
            .add("*.in")?
            .add("py.typed")?
            .build()?,
    };

    let walk = WalkBuilder::new(package_dir)
        .parents(false)
        .overrides(overrides)
        .build();

    Ok(walk)
}

#[tracing::instrument(skip_all)]
fn setup_python(crate_dir: &Path, generated_bindings: &Path) -> Result<(), Error> {
    let pyproject = crate_dir.join("pyproject.toml");

    if pyproject.exists() {
        // Assume everything has been set up correctly. Now, we just need to
        // make sure the dependencies are available.

        tracing::info!("Installing dependencies");
        shell(
            crate_dir,
            [
                "poetry",
                "install",
                "--sync",
                "--no-interaction",
                "--no-root",
            ],
        )
        .context("Unable to install Python dependencies")?;

        return Ok(());
    }

    tracing::info!("Initializing the python package");

    shell(
        crate_dir,
        [
            "poetry",
            "init",
            "--name=tests",
            "--no-interaction",
            "--description=Python integration tests",
            "--dependency=pytest",
        ],
    )
    .context("Unable to initialize the Python package")?;

    tracing::info!("Adding the generated bindings as a dependency");
    shell(
        crate_dir,
        [
            "poetry",
            "add",
            "--no-interaction",
            "--editable",
            generated_bindings
                .strip_prefix(crate_dir)?
                .display()
                .to_string()
                .as_str(),
        ],
    )
    .context("Unable to add the generated bindings as a dependency")?;

    Ok(())
}

#[tracing::instrument(skip_all)]
fn run_pytest(crate_dir: &Path) -> Result<(), Error> {
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        tracing::warn!("Skipping Pytest. Wasmer Python doesn't work on M1 MacOS. For more, see <https://github.com/wasmerio/wasmer-python/issues/680>");
        return Ok(());
    }

    shell(crate_dir, ["poetry", "run", "pytest", "--verbose"]).context("pytest failed")?;

    Ok(())
}

fn shell<A, S>(cwd: &Path, args: A) -> Result<(), Error>
where
    A: IntoIterator<Item = S>,
    S: Display,
{
    let command = args
        .into_iter()
        .map(|s| {
            // quick'n'dirty shell-escape
            let mut s = s.to_string();
            if s.contains(' ') {
                s.insert(0, '"');
                s.push('"');
            }
            s
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut cmd = {
        if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").arg(&command);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(&command);
            cmd
        }
    };

    tracing::info!(
        command=?cmd,
        cwd=%cwd.display(),
        "Executing a shell command",
    );

    let status = cmd
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Unable to run yarn. Is it installed?")?;
    anyhow::ensure!(status.success(), "Unable to execute `{command}`");

    Ok(())
}

#[tracing::instrument(skip_all)]
fn setup_javascript(crate_dir: &Path, generated_bindings: &Path) -> Result<(), Error> {
    let package_path = generated_bindings.join("package");
    let yarn_lock = crate_dir.join("yarn.lock");

    if !yarn_lock.exists() {
        //need to install dependencies for generated package as yarn link
        //doesn't resolves the dependencies on it own

        tracing::info!("Initializing the Javascript package");
        shell(crate_dir, ["yarn", "init", "--yes"])
            .context("Unable to initialize the package.json")?;

        tracing::info!("Installing the Jest testing library");
        shell(crate_dir, ["yarn", "add", "--dev", "jest"])
            .context("Unable to add jest as a dev-dependency")?;

        let jest_config_file = crate_dir.join("jest.config.js");

        fs::write(&jest_config_file, JEST_CONFIG)?;

        tracing::info!("Adding the generated bindings as a dependency");
        let relative_path = package_path
            .strip_prefix(crate_dir)
            .unwrap_or(&package_path);
        shell(
            crate_dir,
            ["yarn", "add", &format!("file:{}", relative_path.display())],
        )
        .context("Unable to add the generated bindings as a dependency")?;
    }

    tracing::info!("Installing dependencies for generated bindings");
    shell(&package_path, ["yarn", "install"])
        .context("Unable to install dependencies for the generated bindings")?;

    tracing::info!("Installing dependencies for the test package");
    shell(crate_dir, ["yarn", "install"])
        .context("Unable to install the test package's dependencies")?;

    Ok(())
}

#[tracing::instrument(skip_all)]
fn run_jest(crate_dir: &Path) -> Result<(), Error> {
    tracing::info!("Running the jest tests");
    shell(crate_dir, ["yarn", "jest"]).context("Testing failed")?;

    Ok(())
}
