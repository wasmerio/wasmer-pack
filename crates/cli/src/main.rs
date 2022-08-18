use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
use clap::Parser;
use semver::VersionReq;
use wapm_toml::{Manifest, Module};
use wit_pack::Bindings;

fn main() -> Result<(), Error> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::Js(js) => js.run(Language::JavaScript),
        Cmd::Python(py) => py.run(Language::Python),
    }
}

#[derive(Debug, Parser)]
enum Cmd {
    Js(Codegen),
    Python(Codegen),
}

#[derive(Debug, Parser)]
struct Codegen {
    /// Where to save the generated bindings.
    #[clap(short, long)]
    out_dir: Option<PathBuf>,
    /// The module to generate bindings for.
    #[clap(short, long)]
    module: Option<String>,
    /// The path to check when looking for a WAPM package manifest.
    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Codegen {
    fn run(self, language: Language) -> Result<(), Error> {
        let Codegen {
            path,
            out_dir,
            module,
        } = self;

        let manifest = Manifest::find_in_directory(&path)?;
        let module = get_desired_module(&manifest, module.as_deref())?;

        let wapm_toml::Bindings { wit, wit_bindgen } = module
            .bindings
            .as_ref()
            .context("The module doesn't declare any bindings")?;

        let compatible_version: VersionReq = wit_pack::WIT_PARSER_VERSION
            .parse()
            .expect("Should always be valid");

        anyhow::ensure!(
            compatible_version.matches(wit_bindgen),
            "wit-pack is not compatible with WIT format {wit_bindgen} (expected {compatible_version})"
        );

        let bindings = Bindings::from_disk(&wit, &module.source)?;

        let files = match language {
            Language::JavaScript => bindings.javascript()?,
            Language::Python => todo!(),
        };

        let out_dir = out_dir
            .as_deref()
            .unwrap_or_else(|| Path::new(bindings.package_name()));
        files.save_to_disk(out_dir)?;

        Ok(())
    }
}

fn get_desired_module<'m>(
    manifest: &'m Manifest,
    target_module: Option<&str>,
) -> Result<&'m Module, Error> {
    match target_module {
        Some(target_module) => manifest
            .module
            .as_ref()
            .into_iter()
            .flatten()
            .find(|m| m.name == *target_module)
            .with_context(|| format!("The manifest doesn't define a \"{target_module}\" module")),
        None => match manifest.module.as_deref() {
            Some([m]) => Ok(m),
            Some([]) | None => Err(Error::msg("The manifest doesn't include any modules")),
            Some([..]) => Err(Error::msg("The manifest defines more than one module, please specify which module to use with the --module flag")),
        },
    }
}

#[derive(Debug, Copy, Clone)]
enum Language {
    JavaScript,
    Python,
}
