use serde::Deserialize;
use std::io::BufReader;
use std::{fs::File, path::Path, process::Command};

use wasmer_pack_cli::Language;

use crate::{utils, TestFailure};

#[derive(Deserialize, Debug)]
struct PackageJson {
    name: String,
}

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

    // reading the package and getting the namespace and name of the javascript created package
    let package_path = dest.join("package");
    let package_json_path = package_path.join("package.json");

    assert!(package_json_path.is_file());

    let file = File::open(package_json_path).unwrap();
    let reader = BufReader::new(file);

    let package_json: PackageJson = serde_json::from_reader(reader).unwrap();

    let package_name = package_json.name;
    // Create a link to the created bindings
    utils::execute_command(Command::new("yarn").arg("link").current_dir(&package_path))
        .map_err(TestFailure::InitializingYarnLink)?;

    utils::execute_command(Command::new("yarn").current_dir(&package_path))
        .map_err(TestFailure::InstallingDependencies)?;

    // link the yarn package to current package
    utils::execute_command(
        Command::new("yarn")
            .arg("link")
            .arg(&package_name)
            .current_dir(script_dir),
    )
    .map_err(TestFailure::InitializingYarnLink)?;

    let test_filename = script_path
        .file_name()
        .ok_or(TestFailure::DeterminingScriptFilename)?;

    utils::execute_command(
        Command::new("node")
            .arg(test_filename)
            .current_dir(script_dir),
    )
    .map_err(TestFailure::TestFileExecution)?;

    utils::execute_command(
        Command::new("yarn")
            .arg("unlink")
            .arg(package_name)
            .current_dir(script_dir),
    )
    .map_err(TestFailure::InitializingYarnUnlink)?;

    utils::execute_command(
        Command::new("yarn")
            .arg("unlink")
            .current_dir(&package_path),
    )
    .map_err(TestFailure::InitializingYarnUnlink)?;
    Ok(())
}
