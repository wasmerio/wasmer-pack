use std::{path::Path, process::Command};

use wasmer_pack_cli::Language;

use crate::{utils, TestFailure};

pub(crate) fn run(script_path: &Path, wapm_dir: &Path, temp_dir: &Path) -> Result<(), TestFailure> {
    let dest = temp_dir.join("javascript");
    tracing::info!("Preparing the JavaScript package");

    utils::generate_bindings(&dest, wapm_dir, Language::JavaScript)?;
    let script_dir = script_path
        .parent()
        .ok_or(TestFailure::DeterminingScriptDirectory)?;

    utils::execute_command(
        Command::new("yarn")
            .arg("init")
            .arg("--yes")
            .current_dir(script_dir),
    )
    .map_err(TestFailure::InitializingJavascriptEnvironment)?;

    let package_path = dest.join("package");
    utils::execute_command(
        Command::new("yarn")
            .arg("add")
            .arg(&package_path)
            .current_dir(script_dir),
    )
    .map_err(TestFailure::InstallingDependencies)?;

    let test_filename = script_path
        .file_name()
        .ok_or(TestFailure::DeterminingScriptFilename)?;

    utils::execute_command(
        Command::new("node")
            .arg(test_filename)
            .current_dir(script_dir),
    )
    .map_err(TestFailure::InstallingDependencies)?;

    utils::execute_command(
        Command::new("yarn")
            .arg("remove")
            .arg(&package_path)
            .current_dir(script_dir),
    )
    .map_err(TestFailure::InstallingDependencies)?;

    Ok(())
}
