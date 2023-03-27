use crate::{Abi, Command, Interface, Library, Metadata, Module, Package};
use anyhow::{Context, Error};
pub use serde_cbor::Value;
use std::path::Path;
use webc::metadata::annotations::Wapm;
use webc::metadata::{Binding, BindingsExtended, Manifest};
use webc::v1::{DirOrFile, ParseOptions, WebC};

pub(crate) fn load_webc_binary(raw_webc: &[u8]) -> Result<Package, Error> {
    let options = ParseOptions::default();
    let webc = WebC::parse(raw_webc, &options)?;

    let fully_qualified_package_name = webc.get_package_name();

    let metadata = metadata(&webc, &fully_qualified_package_name)?;
    let libraries = libraries(&webc, &fully_qualified_package_name)?;
    let commands = commands(&webc, &fully_qualified_package_name)?;

    Ok(Package::new(metadata, libraries, commands))
}

fn commands(webc: &WebC<'_>, fully_qualified_package_name: &str) -> Result<Vec<Command>, Error> {
    let mut commands = Vec::new();

    for name in webc.list_commands() {
        let atom_name = webc
            .get_atom_name_for_command("wasi", name)
            .map_err(Error::msg)?;
        let wasm = webc.get_atom(fully_qualified_package_name, &atom_name)?;

        commands.push(Command {
            name: name.to_string(),
            wasm: wasm.to_vec(),
        });
    }

    Ok(commands)
}

fn libraries(webc: &WebC<'_>, fully_qualified_package_name: &str) -> Result<Vec<Library>, Error> {
    let Manifest { bindings, .. } = webc.get_metadata();
    let libraries = bindings
        .iter()
        .map(|b| load_library(b.clone(), webc, fully_qualified_package_name))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(libraries)
}

fn metadata(webc: &WebC<'_>, fully_qualified_package_name: &str) -> Result<Metadata, Error> {
    let (unversioned_name, version) = fully_qualified_package_name.split_once('@').unwrap();
    let package_name = unversioned_name
        .parse()
        .context("Unable to parse the package name")?;
    let mut metadata = Metadata::new(package_name, version);
    if let Ok(Some(Wapm { description, .. })) = webc.manifest.package_annotation("wapm") {
        metadata = metadata.with_description(description);
    }
    Ok(metadata)
}

fn load_library(
    bindings: Binding,
    webc: &WebC,
    fully_qualified_package_name: &str,
) -> Result<Library, Error> {
    let bindings = bindings
        .get_bindings()
        .context("Unable to read the bindings metadata")?;

    let exports_path = bindings
        .exports()
        .context("The library doesn't have any exports")?;
    let exports = load_interface(webc, exports_path, fully_qualified_package_name)
        .context("Unable to load the exports interface")?;

    let imports_paths = match &bindings {
        BindingsExtended::Wit(_) => &[],
        BindingsExtended::Wai(w) => w.imports.as_slice(),
    };
    let imports = imports_paths
        .iter()
        .map(|path| load_interface(webc, path, fully_qualified_package_name))
        .collect::<Result<Vec<_>, Error>>()?;

    let module_name = bindings.module().trim_start_matches("atoms://");
    let module = webc
        .get_atom(fully_qualified_package_name, module_name)
        .with_context(|| format!("Unable to get the \"{}\" atom", bindings.module()))?;
    let module = Module {
        name: Path::new(module_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Unable to determine the module's name")?
            .to_string(),
        abi: wasm_abi(module),
        wasm: module.to_vec(),
    };

    Ok(Library {
        module,
        exports,
        imports,
    })
}

fn load_interface(
    webc: &WebC<'_>,
    exports_path: &str,
    fully_qualified_package_name: &str,
) -> Result<Interface, Error> {
    let (volume, exports_path) = exports_path.split_once("://").unwrap();
    let exports: &[u8] =
        get_file_from_volume(webc, fully_qualified_package_name, volume, exports_path)?;
    let exports = std::str::from_utf8(exports).context("The WIT file should be a UTF-8 string")?;
    Interface::from_wit(exports_path, exports).context("Unable to parse the WIT file")
}

fn get_file_from_volume<'webc>(
    webc: &'webc WebC,
    fully_qualified_package_name: &str,
    volume_name: &str,
    exports_path: &str,
) -> Result<&'webc [u8], Error> {
    let volume = webc
        .get_volume(fully_qualified_package_name, volume_name)
        .with_context(|| format!("The container doesn't have a \"{volume_name}\" volume"))?;

    let result = volume.get_file(exports_path).with_context(|| {
        format!("Unable to find \"{exports_path}\" in the \"{volume_name}\" volume")
    });

    if result.is_err() {
        // Older versions of wapm2pirita would create entries where the filename
        // section contained an internal `/` (i.e. the root directory has a file
        // called `path/to/foo.wasm`, rather than a `path/` directory that
        // contains a `to/` directory which contains a `foo.wasm` file).
        //
        // That means calls to volume.get_file() will always fail.
        // See https://github.com/wasmerio/pirita/issues/30 for more

        let path = DirOrFile::File(exports_path.into());
        if let Some(entry) = volume
            .get_all_file_and_dir_entries()
            .ok()
            .and_then(|entries| entries.get(&path).cloned())
        {
            let start = entry.offset_start as usize;
            let end = entry.offset_end as usize;
            return Ok(&volume.data[start..end]);
        }
    }

    result
}

/// Try to automatically detec the ABI for a WebAssembly module based in its
/// imports.
fn wasm_abi(module: &[u8]) -> Abi {
    let wasi_namespaces = &["wasi_unstable", "wasi_snapshot_preview1"];

    let imported_modules = wasmparser::Parser::new(0)
        .parse_all(module)
        .filter_map(|p| match p {
            Ok(wasmparser::Payload::ImportSection(imports)) => Some(imports),
            _ => None,
        })
        .flat_map(|s| s.into_iter().filter_map(|result| result.ok()))
        .map(|import| import.module);

    for imported_module in imported_modules {
        if wasi_namespaces.contains(&imported_module) {
            return Abi::Wasi;
        }
    }

    Abi::None
}
