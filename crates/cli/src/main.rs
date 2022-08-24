use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
use clap::Parser;
use semver::{Version, VersionReq};
use wapm_toml::{Manifest, Module};

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

        let manifest = Manifest::find_in_directory(&path).context("Unable to find wapm.toml")?;
        let module = get_desired_module(&manifest, module.as_deref())?;

        let bindings_field = module
            .bindings
            .as_ref()
            .context("The module doesn't declare any bindings")?;

        let metadata = derive_metadata(&manifest);
        let interface = load_interface(bindings_field, &manifest.base_directory_path)?;
        let module = load_module(module, &manifest.base_directory_path)?;

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

fn load_interface(
    bindings: &wapm_toml::Bindings,
    base_dir: &Path,
) -> Result<wit_pack::Interface, Error> {
    let wapm_toml::Bindings {
        wit_exports,
        wit_bindgen,
    } = bindings;
    ensure_compatible(wit_bindgen)?;

    wit_pack::Interface::from_path(base_dir.join(wit_exports))
}

fn load_module(module: &Module, base_dir: &Path) -> Result<wit_pack::Module, Error> {
    let Module {
        name, source, abi, ..
    } = module;
    let path = base_dir.join(source);
    let wasm =
        std::fs::read(&path).with_context(|| format!("Unable to read \"{}\"", path.display()))?;

    let abi = match abi {
        wapm_toml::Abi::None => wit_pack::Abi::None,
        wapm_toml::Abi::Wasi => wit_pack::Abi::Wasi,
        other => anyhow::bail!("ABI not supported by wit-pack: {other:?}"),
    };

    Ok(wit_pack::Module {
        name: name.clone(),
        abi,
        wasm,
    })
}

fn derive_metadata(manifest: &Manifest) -> wit_pack::Metadata {
    let wapm_toml::Package {
        name,
        version,
        description,
        ..
    } = &manifest.package;

    wit_pack::Metadata::new(name, version.to_string()).with_description(description)
}

fn ensure_compatible(wit_bindgen: &Version) -> Result<(), Error> {
    let compatible_version: VersionReq = wit_pack::WIT_PARSER_VERSION
        .parse()
        .expect("Should always be valid");

    anyhow::ensure!(
        compatible_version.matches(wit_bindgen),
        "wit-pack is not compatible with WIT format {wit_bindgen} (expected {compatible_version})",
    );

    Ok(())
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
