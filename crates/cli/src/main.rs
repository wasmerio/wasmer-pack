use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
use clap::Parser;
use webc::{Manifest, ParseOptions, WebCMmap};
use wit_pack::{Interface, Metadata, Module};

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
    #[clap(parse(from_os_str))]
    input: PathBuf,
}

impl Codegen {
    fn run(self, language: Language) -> Result<(), Error> {
        let Codegen { out_dir, input } = self;

        let (metadata, module, interface) = load_pirita_file(&input)?;

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

fn load_pirita_file(path: &Path) -> Result<(Metadata, Module, Interface), Error> {
    let options = ParseOptions::default();

    let webc = WebCMmap::parse(path.to_path_buf(), &options)
        .with_context(|| format!("Unable to load \"{}\" as a WEBC file", path.display()))?;
    let Manifest {
        package, bindings, ..
    } = webc.get_metadata();

    let bindings = match bindings.as_slice() {
        [b] => b
            .get_wit_bindings()
            .with_context(|| format!("Expected WIT bindings, but found \"{}\"", b.kind))?,
        [..] => {
            anyhow::bail!("Generating bindings for multiple modules isn't supported at the moment")
        }
    };

    let package = webc.get_package_name();

    dbg!(webc.get_volumes_for_package(&package));

    let exports = webc
        .get_file(&package, &bindings.exports)
        .with_context(|| format!("Unable to find the \"{}\" volume", bindings.exports))?;

    let module = webc
        .get_atom("Michael-F-Bryan/wit-pack@0.1.4", &bindings.module)
        .with_context(|| format!("Unable to get the \"{}\" atom", bindings.module))?;

    todo!();
}
