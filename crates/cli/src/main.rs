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
    let Manifest { bindings, .. } = webc.get_metadata();

    let bindings = match bindings.as_slice() {
        [b] => b
            .get_wit_bindings()
            .with_context(|| format!("Expected WIT bindings, but found \"{}\"", b.kind))?,
        [..] => {
            anyhow::bail!("Generating bindings for multiple modules isn't supported at the moment")
        }
    };

    let package = webc.get_package_name();

    let exports = bindings.exports.trim_start_matches("metadata://");
    let exports = webc
        .get_volume(&package, "metadata")
        .context("The container doesn't have a \"metadata\" volume")?
        .get_file(exports)
        .with_context(|| format!("Unable to find the \"{}\" volume", bindings.exports))?;
    let exports = std::str::from_utf8(exports).context("The WIT file should be a UTF-8 string")?;
    let interface =
        Interface::from_wit(&bindings.exports, exports).context("Unable to parse the WIT file")?;

    let exports = bindings.module.trim_start_matches("atoms://");
    let module = webc
        .get_atom(&package, exports)
        .with_context(|| format!("Unable to get the \"{}\" atom", bindings.module))?;
    let module = Module {
        name: Path::new(exports)
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Unable to determine the module's name")?
            .to_string(),
        abi: wasm_abi(module),
        wasm: module.to_vec(),
    };

    let (unversioned_name, _) = package.split_once("@").unwrap();

    Ok((
        Metadata::new(format!("@{unversioned_name}"), "0.0.0"),
        module,
        interface,
    ))
}

fn wasm_abi(module: &[u8]) -> wit_pack::Abi {
    // TODO: use a proper method to guess the ABI
    if bytes_contain(module, b"wasi_snapshot_preview") {
        wit_pack::Abi::Wasi
    } else {
        wit_pack::Abi::None
    }
}

fn bytes_contain(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
