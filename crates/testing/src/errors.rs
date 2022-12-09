use std::{
    ffi::OsString,
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

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
            CommandFailed::Spawn { command, .. } => write!(f, "Unable to spawn \"{command:?}\""),
            CommandFailed::CompletedUnsuccessfully {
                command,
                stdout,
                stderr,
                exit_code,
            } => {
                write!(f, "Compiling to a WAPM package with {command} failed")?;
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
pub enum TestFailure {
    BindingsGeneration(wasmer_pack_cli::Error),
    DeterminingScriptDirectory,
    InstallingDependencies(CommandFailed),
    CreatingVirtualEnvironment {
        venv_dir: PathBuf,
        error: CommandFailed,
    },
    TestScript(CommandFailed),
}

impl std::error::Error for TestFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TestFailure::BindingsGeneration(e) => Some(&**e),
            TestFailure::InstallingDependencies(e)
            | TestFailure::CreatingVirtualEnvironment { error: e, .. }
            | TestFailure::TestScript(e) => Some(e),
            TestFailure::DeterminingScriptDirectory => None,
        }
    }
}

impl Display for TestFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TestFailure::BindingsGeneration(_) => write!(f, "Unable to generate bindings"),
            TestFailure::DeterminingScriptDirectory => {
                write!(f, "Unable to determine the script directory")
            }
            TestFailure::InstallingDependencies(_) => write!(f, "Unable to install dependencies"),
            TestFailure::CreatingVirtualEnvironment { venv_dir, .. } => write!(
                f,
                "Unable to create a virtual environment in \"{}\"",
                venv_dir.display()
            ),
            TestFailure::TestScript(_) => write!(f, "The tests failed"),
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
            LoadError::ManifestNotFound { .. } | LoadError::CargoWapmFailed { .. } => None,
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
