use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use wasmer_pack_cli::{Codegen, Language};

use crate::{CommandFailed, LoadError, TestFailure};

pub(crate) fn execute_command(cmd: &mut Command) -> Result<(), CommandFailed> {
    let command = format!("{cmd:?}");

    tracing::debug!(%command, "Executing");

    let Output {
        status,
        stdout,
        stderr,
    } = cmd
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| CommandFailed::Spawn {
            command: cmd.get_program().to_os_string(),
            error: e,
        })?;

    if status.success() {
        Ok(())
    } else {
        return Err(CommandFailed::CompletedUnsuccessfully {
            command,
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            exit_code: status.code(),
        });
    }
}

pub fn compile_rust_to_wapm_package(
    manifest_path: &Path,
    target_dir: impl AsRef<Path>,
) -> Result<PathBuf, LoadError> {
    let target_dir = target_dir.as_ref();

    let mut cmd = Command::new("cargo");
    cmd.arg("wapm")
        .arg("--dry-run")
        .arg("--manifest-path")
        .arg(manifest_path)
        .env("CARGO_TARGET_DIR", target_dir);

    if let Some(parent) = manifest_path.parent() {
        cmd.current_dir(parent);
    }

    execute_command(&mut cmd).map_err(LoadError::CargoWapmFailed)?;

    let wapm_dir = target_dir.join("wapm");

    let generated_package_dir =
        first_dir_in_folder(&wapm_dir).map_err(|e| LoadError::UnableToLocateBindings {
            dir: wapm_dir,
            error: e,
        })?;

    Ok(generated_package_dir)
}

fn first_dir_in_folder(dir: &Path) -> Result<PathBuf, std::io::Error> {
    let mut entries = dir.read_dir()?;

    let first_item = match entries.next() {
        Some(Ok(entry)) => entry.path(),
        Some(Err(e)) => return Err(e),
        None => todo!(),
    };

    if !first_item.is_dir() {
        return Err(std::io::Error::new(
            ErrorKind::Other,
            format!("Expected \"{}\" to be a directory", first_item.display(),),
        ));
    }

    Ok(first_item)
}

pub fn generate_bindings(dest: &Path, wapm_dir: &Path, lang: Language) -> Result<(), TestFailure> {
    tracing::info!(
        output_dir=%dest.display(),
        wapm_dir=%wapm_dir.display(),
        language=?lang,
        "Generating bindings",
    );
    let codegen = Codegen {
        out_dir: Some(dest.to_path_buf()),
        input: wapm_dir.to_path_buf(),
    };
    codegen.run(lang).map_err(TestFailure::BindingsGeneration)?;
    Ok(())
}
