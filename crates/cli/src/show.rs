use std::{io::Write, path::PathBuf, str::FromStr};

use anyhow::{Context, Error};
use clap::Parser;
use wit_pack::{Metadata, Package};

#[derive(Debug, Parser)]
pub struct Show {
    /// The format to use when emitting metadata.
    #[clap(
        short,
        long,
        possible_values = ["json", "text"],
        default_value = "text",
        parse(try_from_str),
    )]
    format: Format,
    /// The Pirita file to read.
    #[clap(parse(from_os_str))]
    input: PathBuf,
}

impl Show {
    pub fn run(self) -> Result<(), Error> {
        let pkg = crate::load_pirita_file(&self.input).context("Unable to load the package")?;

        let summary: Summary = summarize(&pkg);

        let mut stdout = std::io::stdout();
        match self.format {
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
}

impl Library {
    fn dump(&self, mut writer: impl Write) -> Result<(), Error> {
        let Library { interface_name } = self;

        writeln!(writer, "- {interface_name}")?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Format {
    Json,
    Text,
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
