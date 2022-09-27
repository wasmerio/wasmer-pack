use std::path::Path;

use anyhow::{Context, Error};
use webc::{Manifest, ParseOptions, WebC, WebCOwned};
use wit_pack::{Command, Interface, Library, Metadata, Module, Package};

pub(crate) fn load_pirita_file(path: &Path) -> Result<Package, Error> {
    let options = ParseOptions::default();

    let raw =
        std::fs::read(path).with_context(|| format!("Unable to read \"{}\"", path.display()))?;
    let webc = WebCOwned::parse(raw, &options)
        .with_context(|| format!("Unable to parse \"{}\" as a WEBC file", path.display()))?;

    let fully_qualified_package_name = webc.get_package_name();
    let metadata = metadata(&fully_qualified_package_name)?;
    let libraries = libraries(&webc, &fully_qualified_package_name)?;
    let commands = commands(&webc, &fully_qualified_package_name);

    Ok(Package::new(metadata, libraries, commands))
}

fn commands(webc: &WebC<'_>, fully_qualified_package_name: &str) -> Vec<Command> {
    let mut commands = Vec::new();

    for name in webc.list_commands() {
        let wasm = webc.get_atom(fully_qualified_package_name, &name).unwrap();
        commands.push(Command {
            name: name.to_string(),
            wasm: wasm.to_vec(),
        });
    }

    commands
}

fn libraries(webc: &WebC<'_>, fully_qualified_package_name: &str) -> Result<Vec<Library>, Error> {
    let Manifest { bindings, .. } = webc.get_metadata();
    let libraries = bindings
        .iter()
        .map(|b| load_library(b, webc, fully_qualified_package_name))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(libraries)
}

fn metadata(fully_qualified_package_name: &str) -> Result<Metadata, Error> {
    let (unversioned_name, version) = fully_qualified_package_name.split_once('@').unwrap();
    let package_name = unversioned_name
        .parse()
        .context("Unable to parse the package name")?;
    Ok(Metadata::new(package_name, version))
}

fn load_library(
    bindings: &webc::Binding,
    webc: &WebC,
    fully_qualified_package_name: &str,
) -> Result<Library, Error> {
    let bindings = bindings
        .get_wit_bindings()
        .with_context(|| format!("Expected WIT bindings, but found \"{}\"", bindings.kind))?;

    let exports = bindings.exports.trim_start_matches("metadata://");
    let exports = webc
        .get_volume(fully_qualified_package_name, "metadata")
        .context("The container doesn't have a \"metadata\" volume")?
        .get_file(exports)
        .with_context(|| format!("Unable to find \"{}\"", bindings.exports))?;
    let exports = std::str::from_utf8(exports).context("The WIT file should be a UTF-8 string")?;
    let interface =
        Interface::from_wit(&bindings.exports, exports).context("Unable to parse the WIT file")?;
    let exports = bindings.module.trim_start_matches("atoms://");

    let module = webc
        .get_atom(fully_qualified_package_name, exports)
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

    Ok(Library { module, interface })
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
