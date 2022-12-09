use std::{
    path::{Path, PathBuf},
    process::Command,
};

use wasmer_pack_cli::{Codegen, Language};

use crate::{utils::execute_command, TestFailure};

pub(crate) fn run(script_path: &Path, wapm_dir: &Path, temp_dir: &Path) -> Result<(), TestFailure> {
    let dest = temp_dir.join("python");
    tracing::info!("Preparing the python package");

    generate_bindings(&dest, wapm_dir)?;

    let script_dir = script_path
        .parent()
        .ok_or(TestFailure::DeterminingScriptDirectory)?;

    let venv_dir = script_dir.join(".venv");

    if !venv_dir.exists() {
        tracing::debug!(
            venv = %venv_dir.display(),
            "Creating a new virtual environment",
        );
        initialize_python_virtual_environment(&venv_dir)?;
    }

    let pip = get_executable_from_venv(&venv_dir, "pip");

    pip_install_generated_bindings(&pip, &dest, script_dir)?;

    let pytest = get_executable_from_venv(&venv_dir, "pytest");
    if !pytest.exists() {
        install_python_package(&pip, "pytest")?;
    }

    run_test_suite(&pytest, script_path)?;

    Ok(())
}

fn run_test_suite(pytest: &Path, script_path: &Path) -> Result<(), TestFailure> {
    tracing::info!("Running the test suite");
    let mut cmd = Command::new(pytest);
    cmd.arg(script_path);
    execute_command(&mut cmd).map_err(TestFailure::TestScript)?;
    Ok(())
}

fn install_python_package(pip: &Path, package_name: &str) -> Result<(), TestFailure> {
    tracing::debug!(package = package_name, "Installing Package");

    let mut cmd = Command::new(pip);
    cmd.arg("install").arg(package_name);
    execute_command(&mut cmd).map_err(TestFailure::InstallingDependencies)?;
    Ok(())
}

fn pip_install_generated_bindings(
    pip: &Path,
    dest: &Path,
    script_dir: &Path,
) -> Result<(), TestFailure> {
    tracing::info!(
        pip = %pip.display(),
        bindings = %dest.display(),
        "Installing the bindings",
    );

    // TODO: check if this works on Windows. We might need to invoke pip
    // through cmd.exe
    let mut cmd = Command::new(pip);
    cmd.arg("install")
        .arg("-e")
        .arg(dest)
        .current_dir(script_dir);
    execute_command(&mut cmd).map_err(TestFailure::InstallingDependencies)?;
    Ok(())
}

fn generate_bindings(dest: &Path, wapm_dir: &Path) -> Result<(), TestFailure> {
    let codegen = Codegen {
        out_dir: Some(dest.to_path_buf()),
        input: wapm_dir.to_path_buf(),
    };
    codegen
        .run(Language::Python)
        .map_err(TestFailure::BindingsGeneration)?;
    Ok(())
}

fn initialize_python_virtual_environment(venv_dir: &Path) -> Result<(), TestFailure> {
    let python = if cfg!(windows) {
        "python.exe"
    } else {
        "python3"
    };

    let mut cmd = Command::new(python);
    cmd.arg("-m").arg("venv").arg(venv_dir);

    execute_command(&mut cmd).map_err(|e| TestFailure::CreatingVirtualEnvironment {
        venv_dir: venv_dir.to_path_buf(),
        error: e,
    })?;

    Ok(())
}

fn get_executable_from_venv(venv_dir: &Path, binary: &str) -> PathBuf {
    if cfg!(windows) {
        venv_dir.join("Scripts").join(binary).with_extension("exe")
    } else {
        venv_dir.join("bin").join(binary)
    }
}
