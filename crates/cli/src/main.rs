use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
use clap::Parser;
use wit_pack::{Abi, Interface, Metadata, Module};

const POSSIBLE_ABIS: &[&str] = &["wasi", "none"];

fn main() -> Result<(), Error> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::Js(js) => js.run(Language::JavaScript),
        Cmd::Python(py) => py.run(Language::Python),
    }
}

#[derive(Debug, Parser)]
#[clap(version)]
enum Cmd {
    /// Generate bindings for use with NodeJS.
    Js(Codegen),
    /// Generate Python bindings.
    Python(Codegen),
}

#[derive(Debug, Parser)]
struct Codegen {
    /// Where to save the generated bindings.
    #[clap(short, long)]
    out_dir: Option<PathBuf>,
    /// A `*.wit` file defining the exported interface.
    #[clap(short, long)]
    exports: PathBuf,
    /// The name of the generated package.
    #[clap(short = 'n', long = "name")]
    package_name: String,
    /// The generated package's version number.
    #[clap(short = 'v', long = "version")]
    package_version: String,
    /// The `*.wasm` file bindings are being generated for.
    #[clap(short, long)]
    module: PathBuf,
    #[clap(short, long, default_value = "none", parse(try_from_str), possible_values = POSSIBLE_ABIS)]
    abi: Abi,
}

impl Codegen {
    fn run(self, language: Language) -> Result<(), Error> {
        let Codegen {
            out_dir,
            exports,
            package_name,
            package_version,
            module,
            abi,
        } = self;

        let metadata = Metadata::new(package_name, package_version);
        let module = Module::from_path(&module, abi)
            .with_context(|| format!("Unable to load \"{}\"", module.display()))?;
        let interface = Interface::from_path(&exports).with_context(|| {
            format!(
                "Unable to parse an interface definition from \"{}\"",
                exports.display()
            )
        })?;

        let files = match language {
            Language::JavaScript => wit_pack::generate_javascript(&metadata, &module, &interface)?,
            Language::Python => wit_pack::generate_python(&metadata, &module, &interface)?,
        };

        let out_dir = out_dir
            .as_deref()
            .unwrap_or_else(|| Path::new(&metadata.package_name));
        files.save_to_disk(out_dir)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum Language {
    JavaScript,
    Python,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn abi_options_are_all_usable() {
        for abi in POSSIBLE_ABIS {
            assert!(Abi::from_str(abi).is_ok(), "{abi}");
        }
    }
}
