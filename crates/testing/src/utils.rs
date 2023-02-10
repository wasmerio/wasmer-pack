use std::{
    ffi::OsString,
    fmt::{self, Display, Formatter},
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use wasmer_pack_cli::{Codegen, Language};

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

pub fn generate_bindings(
    dest: &Path,
    wapm_dir: &Path,
    lang: Language,
) -> Result<(), anyhow::Error> {
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
    codegen.run(lang)?;
    Ok(())
}

#[derive(Debug)]
pub enum CommandFailed {
    Spawn {
        command: OsString,
        error: std::io::Error,
    },
    CompletedUnsuccessfully {
        command: String,
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
    },
}

impl std::error::Error for CommandFailed {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CommandFailed::Spawn { error, .. } => Some(error),
            CommandFailed::CompletedUnsuccessfully { .. } => None,
        }
    }
}

impl Display for CommandFailed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CommandFailed::Spawn { command, .. } => write!(f, "Unable to spawn {command:?}"),
            CommandFailed::CompletedUnsuccessfully {
                command,
                stdout,
                stderr,
                exit_code,
            } => {
                write!(f, "Executing {command} failed")?;
                if let Some(exit_code) = exit_code {
                    write!(f, " (exit code: {exit_code})")?;
                }
                write!(f, ".")?;

                if !stdout.trim().is_empty() {
                    writeln!(f)?;
                    writeln!(f, "Stdout: {stdout}")?;
                }
                if !stderr.trim().is_empty() {
                    writeln!(f)?;
                    writeln!(f, "Stderr: {stderr}")?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    ManifestNotFound { path: PathBuf },
    TempDir(std::io::Error),
    SpawnFailed(std::io::Error),
    CargoWapmFailed(CommandFailed),
    UnableToLocateBindings { dir: PathBuf, error: std::io::Error },
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::TempDir(e)
            | LoadError::SpawnFailed(e)
            | LoadError::UnableToLocateBindings { error: e, .. } => Some(e),
            LoadError::CargoWapmFailed(e) => Some(e),
            LoadError::ManifestNotFound { .. } => None,
        }
    }
}

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::ManifestNotFound { path } => {
                write!(f, "\"{}\" doesn't exist", path.display())
            }
            LoadError::TempDir(_) => write!(f, "Unable to create a temporary directory"),
            LoadError::SpawnFailed(_) => {
                write!(f, "Unable to start \"cargo wapm\". Is it installed?")
            }
            LoadError::CargoWapmFailed(_) => {
                write!(f, "Generating a WAPM package with \"cargo wapm\" failed")
            }
            LoadError::UnableToLocateBindings { dir, .. } => write!(
                f,
                "Unable to locate the generated bindings in \"{}\"",
                dir.display()
            ),
        }
    }
}
