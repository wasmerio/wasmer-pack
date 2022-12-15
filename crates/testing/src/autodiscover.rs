use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Error};
use ignore::{overrides::OverrideBuilder, Walk, WalkBuilder};
use insta::Settings;
use wasmer_pack_cli::Language;

pub fn autodiscover(crate_dir: impl AsRef<Path>) -> Result<(), Error> {
    let crate_dir = crate_dir.as_ref();
    tracing::info!(dir = %crate_dir.display(), "Looking for tests");

    let manifest_path = crate_dir.join("Cargo.toml");
    let temp = tempfile::tempdir().context("Unable to create a temporary directory")?;

    tracing::debug!("Compiling the crate and generating a WAPM package");
    let wapm_package = crate::compile_rust_to_wapm_package(&manifest_path, temp.path())?;

    let generated_bindings = crate_dir.join("generated_bindings");

    if generated_bindings.exists() {
        tracing::debug!("Deleting bindings from a previous run");
        std::fs::remove_dir_all(&generated_bindings)
            .context("Unable to delete the old generated bindings")?;
    }

    for language in detected_languages(crate_dir) {
        let bindings = generated_bindings.join(language.name());
        tracing::debug!(
            bindings_dir = %bindings.display(),
            "Generating bindings",
        );
        crate::generate_bindings(&bindings, &wapm_package, language)?;

        match language {
            Language::JavaScript => todo!(),
            Language::Python => {
                setup_python(crate_dir, &bindings)?;
                run_pytest(crate_dir)?;
            }
        }

        snapshot_generated_bindings(crate_dir, &bindings, language)?;
    }

    Ok(())
}

fn detected_languages(crate_dir: &Path) -> HashSet<Language> {
    let mut languages = HashSet::new();

    for entry in Walk::new(crate_dir).filter_map(|entry| entry.ok()) {
        match entry.path().extension().and_then(|s| s.to_str()) {
            Some("py") => {
                languages.insert(Language::Python);
            }
            Some("js") | Some("ts") => {
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

    insta::assert_debug_snapshot!("all files", &snapshot_files);

    for path in snapshot_files {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Unable to read \"{}\"", path.display()))?;

        let mut settings = Settings::clone_current();
        let simplified_path = path.strip_prefix(package_dir)?;
        settings.set_input_file(simplified_path);
        let snapshot_name = simplified_path.display().to_string();
        insta::assert_display_snapshot!(snapshot_name, &contents);
    }

    Ok(())
}

fn language_specific_matches(package_dir: &Path, language: Language) -> Result<Walk, Error> {
    let mut builder = OverrideBuilder::new(package_dir);

    let overrides = match language {
        Language::JavaScript => todo!(),
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
        // Assume everything has been set up correctly
        return Ok(());
    }

    tracing::info!("Initializing the python package");

    let mut cmd = Command::new("poetry");
    cmd.arg("init").arg("--name=tests").arg("--no-interaction");
    tracing::debug!(?cmd, "Initializing the Python package");
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run poetry. Is it installed?")?;
    anyhow::ensure!(status.success(), "Unable to initialize the Python package");

    let mut cmd = Command::new("poetry");
    cmd.arg("add").arg("--no-interaction").arg("pytest");
    tracing::debug!(?cmd, "Adding pytest as a dependency");
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run poetry. Is it installed?")?;
    anyhow::ensure!(status.success(), "Unable to add pytest as a dependency");

    let mut cmd = Command::new("poetry");
    cmd.arg("add")
        .arg("--no-interaction")
        .arg("--editable")
        .arg(generated_bindings);
    tracing::debug!(?cmd, "Adding the generated bindings as a dependency");
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
    tracing::debug!(?cmd, "Running pytest");
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(crate_dir)
        .status()
        .context("Unable to run poetry. Is it installed?")?;
    anyhow::ensure!(status.success(), "Testing failed");

    Ok(())
}
