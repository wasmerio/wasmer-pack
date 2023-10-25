use std::path::Path;

use anyhow::{Context, Error};
use webc::{
    compat::Container,
    metadata::{self, annotations::Wapm},
};

use crate::{Abi, Command, Interface, Library, Metadata, Module, Package, PackageName};

pub(crate) fn load_webc_binary(webc: &Container) -> Result<Package, Error> {
    let metadata = metadata(webc.manifest())?;
    let libraries = libraries(webc)?;
    let commands = commands(webc)?;

    Ok(Package::new(metadata, libraries, commands))
}

fn commands(webc: &Container) -> Result<Vec<Command>, Error> {
    let mut commands = Vec::new();

    for (name, command) in &webc.manifest().commands {
        if command
            .runner
            .starts_with(webc::metadata::annotations::WASI_RUNNER_URI)
        {
            let atom_name = command
                .wasi()
                .ok()
                .flatten()
                .map(|wasi| wasi.atom)
                .unwrap_or_else(|| name.clone());
            let wasm = webc.get_atom(&atom_name).with_context(|| {
                format!("Unable to get the \"{atom_name}\" atom for the \"{name}\" command")
            })?;
            commands.push(Command {
                name: name.to_string(),
                wasm: wasm.into(),
            });
        }
    }

    Ok(commands)
}

fn libraries(webc: &Container) -> Result<Vec<Library>, Error> {
    let metadata::Manifest { bindings, .. } = webc.manifest();
    let libraries = bindings
        .iter()
        .map(|b| load_library(webc, b))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(libraries)
}

fn metadata(manifest: &metadata::Manifest) -> Result<Metadata, Error> {
    let Wapm { name, version, .. } = manifest
        .wapm()?
        .context("Unable to find the wapm metadata")?;
    let package_name = PackageName::parse(&name).context("Unable to parse the package name")?;
    Ok(Metadata::new(package_name, version))
}

fn load_library(webc: &Container, bindings: &metadata::Binding) -> Result<Library, Error> {
    let bindings = bindings
        .get_bindings()
        .context("Unable to read the bindings metadata")?;

    let exports_path = bindings
        .exports()
        .context("The library doesn't have any exports")?;
    let exports =
        load_interface(webc, exports_path).context("Unable to load the exports interface")?;

    let imports_paths = match &bindings {
        metadata::BindingsExtended::Wit(_) => &[],
        metadata::BindingsExtended::Wai(w) => w.imports.as_slice(),
    };
    let imports = imports_paths
        .iter()
        .map(|path| load_interface(webc, path))
        .collect::<Result<Vec<_>, Error>>()?;

    let module_name = bindings.module().trim_start_matches("atoms://");
    let module = webc
        .get_atom(module_name)
        .with_context(|| format!("Unable to get the \"{}\" atom", bindings.module()))?;
    let module = Module {
        name: Path::new(module_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Unable to determine the module's name")?
            .to_string(),
        abi: wasm_abi(&module),
        wasm: module.to_vec(),
    };

    Ok(Library {
        module,
        exports,
        imports,
    })
}

fn load_interface(webc: &Container, exports_path: &str) -> Result<Interface, Error> {
    let (volume, exports_path) = exports_path.split_once("://").unwrap();
    let exports = get_file_from_volume(webc, volume, exports_path)?;
    let exports = std::str::from_utf8(&exports).context("The WIT file should be a UTF-8 string")?;
    Interface::from_wit(exports_path, exports).context("Unable to parse the WIT file")
}

fn get_file_from_volume(
    webc: &Container,
    volume_name: &str,
    exports_path: &str,
) -> Result<webc::compat::SharedBytes, Error> {
    let volume = webc
        .get_volume(volume_name)
        .with_context(|| format!("The container doesn't have a \"{volume_name}\" volume"))?;

    volume.read_file(exports_path).with_context(|| {
        format!("Unable to find \"{exports_path}\" in the \"{volume_name}\" volume")
    })
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
