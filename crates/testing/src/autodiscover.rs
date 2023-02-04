use std::{
    collections::{BTreeSet, HashSet},
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};

use serde::Deserialize;

use anyhow::{Context, Error, Ok};
use ignore::{overrides::OverrideBuilder, Walk, WalkBuilder};
use insta::Settings;
use wasmer_pack_cli::Language;

pub fn autodiscover(crate_dir: impl AsRef<Path>) -> Result<(), Error> {
    let start = Instant::now();

    let crate_dir = crate_dir.as_ref();
    tracing::info!(dir = %crate_dir.display(), "Looking for tests");

    let manifest_path = crate_dir.join("Cargo.toml");
    let temp = tempfile::tempdir().context("Unable to create a temporary directory")?;

    tracing::info!("Compiling the crate and generating a WAPM package");
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

fn setup_python(crate_dir: &Path, generated_bindings: &Path) -> Result<(), Error> {
    let pyproject = crate_dir.join("pyproject.toml");

    if pyproject.exists() {
        // Assume everything has been set up correctly. Now, we just need to
        // make sure the dependencies are available.

        let mut cmd = Command::new("poetry");
        cmd.arg("install")
            .arg("--sync")
            .arg("--no-interaction")
            .arg("--no-root");
        tracing::info!(?cmd, "Installing dependencies");
        let status = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(crate_dir)
            .status()
            .context("Unable to run poetry. Is it installed?")?;
        anyhow::ensure!(status.success(), "Unable to install Python dependencies");

        return Ok(());
    }

    tracing::info!("Initializing the python package");

    let mut cmd = Command::new("poetry");
    cmd.arg("init")
        .arg("--name=tests")
        .arg("--no-interaction")
        .arg("--description=Python integration tests")
        .arg("--dependency=pytest");
    tracing::info!(?cmd, "Initializing the Python package");
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run poetry. Is it installed?")?;
    anyhow::ensure!(status.success(), "Unable to initialize the Python package");

    let mut cmd = Command::new("poetry");
    cmd.arg("add")
        .arg("--no-interaction")
        .arg("--editable")
        .arg(generated_bindings.strip_prefix(crate_dir)?);
    tracing::info!(?cmd, "Adding the generated bindings as a dependency");
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run poetry. Is it installed?")?;
    anyhow::ensure!(
        status.success(),
        "Unable to add the generated bindings as a dependency"
    );

    Ok(())
}

fn run_pytest(crate_dir: &Path) -> Result<(), Error> {
    let mut cmd = Command::new("poetry");
    cmd.arg("run").arg("pytest").arg("--verbose");
    tracing::info!(?cmd, "Running pytest");
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run poetry. Is it installed?")?;
    anyhow::ensure!(status.success(), "pytest failed");

    Ok(())
}

struct PlatformIdpCommand {
    inner: Command,
}

impl PlatformIdpCommand {
    pub fn new() -> PlatformIdpCommand {
        let cmd = match cfg!(target_os = "windows") {
            true => Command::new("cmd"),
            false => Command::new("sh"),
        };

        PlatformIdpCommand { inner: cmd }
    }
    pub fn eval_for_platform(&mut self) -> &mut Command {
        if cfg!(target_os = "windows") {
            self.inner.arg("/C");
        } else {
            self.inner.arg("-c");
        }
        &mut self.inner
    }
}

#[derive(Deserialize, Debug)]
struct PackageJson {
    name: String,
}
fn setup_javascript(crate_dir: &Path, generated_bindings: &Path) -> Result<(), Error> {
    // reading the package and getting the namespace and name of the javascript created package
    let package_path = generated_bindings.join("package");
    let generated_package_name = get_package_name(&package_path)?;
    let yarn_lock = crate_dir.join("yarn.lock");

    if yarn_lock.exists() {
        //need to install dependencies for generated package as yarn link doesn't resolves the dependencies on it own

        let mut cmd = PlatformIdpCommand::new();
        cmd.eval_for_platform()
            .arg("yarn")
            .current_dir(&package_path);
        tracing::info!(
            ?cmd.inner,
            "Installing the Javascript Dependencies for generated package"
        );

        let status = cmd
            .inner
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(&package_path)
            .status()
            .context("Unable to run yarn. Is it installed?")?;
        anyhow::ensure!(
            status.success(),
            "Unable to install JavaScript Dependencies for generated package"
        );

        let mut cmd = PlatformIdpCommand::new();
        cmd.eval_for_platform().arg("yarn").current_dir(crate_dir);
        tracing::info!(
            ?cmd.inner,
            "Found `yarn-lock`. Installing the Javascript Dependencies"
        );

        let status = cmd
            .inner
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(crate_dir)
            .status()
            .context("Unable to run yarn. Is it installed?")?;
        anyhow::ensure!(
            status.success(),
            "Unable to install JavaScript Dependencies"
        );
        return Ok(());
    }

    let mut cmd = PlatformIdpCommand::new();
    cmd.eval_for_platform()
        .arg("yarn")
        .arg("init")
        .arg("--yes")
        .current_dir(crate_dir);
    tracing::info!(?cmd.inner, "Initializing the Javascript package");

    let status = cmd
        .inner
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run yarn. Is it installed?")?;
    anyhow::ensure!(
        status.success(),
        "Unable to initialize the JavaScript package"
    );

    // install jest to crate dir

    let mut cmd = PlatformIdpCommand::new();
    cmd.eval_for_platform()
        .arg("yarn")
        .arg("add")
        .arg("--dev")
        .arg("jest")
        .current_dir(crate_dir);
    tracing::info!(?cmd.inner, "Installing the Jest testing library");

    let status = cmd
        .inner
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run yarn. Is it installed?")?;
    anyhow::ensure!(status.success(), "Unable to install jest testing library");

    let jest_file_name = "jest.config.js";
    let jest_config_file = crate_dir.join(jest_file_name);

    const JEST_CONFIG: &str = include_str!("../configs/jest.config.js");

    fs::write(&jest_config_file, JEST_CONFIG)?;
    anyhow::ensure!(crate_dir.join(&jest_config_file).exists());

    let mut cmd = PlatformIdpCommand::new();
    cmd.eval_for_platform()
        .arg("yarn")
        .current_dir(&package_path);

    tracing::info!(?cmd.inner, "Installing dependencies for generated bindings");
    let status = cmd
        .inner
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(&package_path)
        .status()
        .context("Unable to run yarn. Is it installed?")?;
    anyhow::ensure!(
        status.success(),
        "Unable to install dependencies for generated bindings"
    );

    let mut cmd = PlatformIdpCommand::new();
    cmd.eval_for_platform()
        .arg("yarn")
        .arg("link")
        .current_dir(&package_path);

    tracing::info!(?cmd.inner, "Linking the generated bindings as a `Yarn link`");
    let status = cmd
        .inner
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(&package_path)
        .status()
        .context("Unable to run yarn. Is it installed?")?;
    anyhow::ensure!(
        status.success(),
        "Unable to perform yarn link on generated bindings"
    );

    let mut cmd = PlatformIdpCommand::new();
    cmd.eval_for_platform()
        .arg("yarn")
        .arg("link")
        .arg(&generated_package_name)
        .current_dir(crate_dir);

    tracing::info!(?cmd.inner, "Linking the testing package to generated bindings");
    let status = cmd
        .inner
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run yarn. Is it installed?")?;
    anyhow::ensure!(
        status.success(),
        "Unable to initialize a link to the generated bindings from testing crate"
    );
    Ok(())
}

fn run_jest(crate_dir: &Path) -> Result<(), Error> {
    let mut cmd = PlatformIdpCommand::new();

    cmd.eval_for_platform()
        .current_dir(crate_dir)
        .arg("yarn")
        .arg("jest");
    tracing::info!(?cmd.inner,"Running the jest tests");

    let status = cmd
        .inner
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run yarn. Is it installed?")?;
    anyhow::ensure!(
        status.success(),
        "Unable to install JavaScript Dependencies for generated package"
    );

    Ok(())
}

fn get_package_name(package_path: &Path) -> Result<String, Error> {
    let package_json_path = package_path.join("package.json");

    anyhow::ensure!(
        package_json_path.is_file(),
        "Package Json file for generated package not found"
    );

    let file = File::open(package_json_path).unwrap();
    let reader = BufReader::new(file);
    let package_json: PackageJson = serde_json::from_reader(reader).unwrap();
    Ok(package_json.name)
}
