use std::{fmt::Display, io::Write, path::PathBuf, str::FromStr};

use anyhow::Error;
use clap::Parser;
use wasmer_pack::{Metadata, Package};

#[derive(Debug, Parser)]
pub struct Show {
    /// The format to use when emitting metadata.
    #[clap(short, long, default_value_t = Format::Text)]
    format: Format,
    /// The Pirita file to read.
    input: PathBuf,
}

impl Show {
    pub fn run(self) -> Result<(), Error> {
        let Show { format, input } = self;

        let pkg = crate::utils::load(&input)?;

        let summary: Summary = summarize(&pkg);

        let mut stdout = std::io::stdout();
        match format {
            Format::Json => {
                summary.write_json(stdout.lock())?;
                writeln!(stdout)?;
            }
            Format::Text => {
                summary.dump(stdout.lock())?;
            }
        }

        Ok(())
    }
}

fn summarize(pkg: &Package) -> Summary {
    let Metadata {
        description,
        package_name,
        version,
        ..
    } = pkg.metadata();

    let bindings = pkg
        .libraries()
        .iter()
        .map(|lib| Library {
            interface_name: lib.interface_name().to_string(),
            wasi: lib.requires_wasi(),
        })
        .collect();

    let commands = pkg
        .commands()
        .iter()
        .map(|cmd| Command {
            name: cmd.name.clone(),
        })
        .collect();

    Summary {
        description: description.clone(),
        name: package_name.to_string(),
        version: version.clone(),
        bindings,
        commands,
    }
}

#[derive(Debug, serde::Serialize)]
struct Summary {
    name: String,
    version: String,
    description: Option<String>,
    bindings: Vec<Library>,
    commands: Vec<Command>,
}

impl Summary {
    fn write_json(&self, writer: impl Write) -> Result<(), Error> {
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    fn dump(&self, mut writer: impl Write) -> Result<(), Error> {
        let Summary {
            name,
            version,
            description,
            commands,
            bindings,
        } = self;

        writeln!(writer, "{name} {version}")?;

        if let Some(description) = description {
            writeln!(writer, "{description}")?;
        }

        if !commands.is_empty() {
            writeln!(writer, "Commands:")?;
            for command in commands {
                command.dump(&mut writer)?;
            }
        }

        if !bindings.is_empty() {
            writeln!(writer, "Bindings:")?;
            for lib in bindings {
                lib.dump(&mut writer)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
struct Command {
    name: String,
}

impl Command {
    fn dump(&self, mut writer: impl Write) -> Result<(), Error> {
        let Command { name } = self;

        writeln!(writer, "- {name}")?;

        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
struct Library {
    interface_name: String,
    wasi: bool,
}

impl Library {
    fn dump(&self, mut writer: impl Write) -> Result<(), Error> {
        let Library {
            interface_name,
            wasi,
        } = self;

        write!(writer, "- {interface_name}")?;

        if *wasi {
            write!(writer, " (wasi)")?;
        }

        writeln!(writer)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    Json,
    Text,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Json => f.write_str("json"),
            Format::Text => f.write_str("text"),
        }
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Format::Json),
            "text" => Ok(Format::Text),
            other => anyhow::bail!("Expected \"json\" or \"text\", found \"{other}\""),
        }
    }
}
