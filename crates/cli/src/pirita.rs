use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
use wapm_targz_to_pirita::TransformManifestFunctions;
use wasmer_pack::{Command, Interface, Library, Metadata, Module, Package};
use webc::{DirOrFile, Manifest, ParseOptions, WebC, WebCOwned};

pub(crate) fn load(path: &Path) -> Result<Package, Error> {
    let raw_webc: Vec<u8> = if path.is_dir() {
        webc_from_dir(path)?
    } else if path.extension() == Some("webc".as_ref()) {
        std::fs::read(path).with_context(|| format!("Unable to read \"{}\"", path.display()))?
    } else {
        webc_from_tarball(path)?
    };

    let options = ParseOptions::default();
    let webc = WebCOwned::parse(raw_webc, &options)
        .with_context(|| format!("Unable to parse \"{}\" as a WEBC file", path.display()))?;

    let fully_qualified_package_name = webc.get_package_name();
    let metadata = metadata(&fully_qualified_package_name)?;
    let libraries = libraries(&webc, &fully_qualified_package_name)?;
    let commands = commands(&webc, &fully_qualified_package_name)?;

    Ok(Package::new(metadata, libraries, commands))
}

fn webc_from_dir(path: &Path) -> Result<Vec<u8>, Error> {
    if !path.join("wapm.toml").exists() {
        anyhow::bail!(
            "The \"{}\" directory doesn't contain a \"wapm.tom\" file",
            path.display()
        );
    }

    todo!("Read all files into a FileMap and call generate_webc_file()");
}

fn webc_from_tarball(path: &Path) -> Result<Vec<u8>, Error> {
    let tarball =
        std::fs::read(path).with_context(|| format!("Unable to read \"{}\"", path.display()))?;
    let files =
        wapm_targz_to_pirita::unpack_tar_gz(tarball).context("Unable to unpack the tarball")?;

    wapm_targz_to_pirita::generate_webc_file(
        files,
        &PathBuf::new(),
        None,
        &TransformManifestFunctions {
            get_atoms_wapm_toml: wapm_toml::get_wapm_atom_file_paths,
            get_dependencies: wapm_toml::get_dependencies,
            get_package_annotations: wapm_toml::get_package_annotations,
            get_modules: wapm_toml::get_modules,
            get_commands: wapm_toml::get_commands,
            get_manifest_file_names: wapm_toml::get_manifest_file_names,
            get_metadata_paths: wapm_toml::get_metadata_paths,
            get_bindings: wapm_toml::get_bindings,
            get_wapm_manifest_file_name: wapm_toml::get_wapm_manifest_file_name,
        },
    )
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

    let (volume, exports_path) = bindings.exports.split_once("://").unwrap();
    let exports: &[u8] = get_file_from_volume(
        webc,
        fully_qualified_package_name,
        volume,
        exports_path,
        &bindings,
    )?;
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

fn get_file_from_volume<'webc>(
    webc: &'webc WebC,
    fully_qualified_package_name: &str,
    volume: &str,
    exports_path: &str,
    bindings: &webc::WitBindings,
) -> Result<&'webc [u8], Error> {
    let volume = webc
        .get_volume(fully_qualified_package_name, volume)
        .with_context(|| format!("The container doesn't have a \"{volume}\" volume"))?;

    let result = volume
        .get_file(exports_path)
        .with_context(|| format!("Unable to find \"{}\"", bindings.exports));

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
            .get_all_file_entries()
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

fn wasm_abi(module: &[u8]) -> wasmer_pack::Abi {
    // TODO: use a proper method to guess the ABI
    if bytes_contain(module, b"wasi_snapshot_preview") {
        wasmer_pack::Abi::Wasi
    } else {
        wasmer_pack::Abi::None
    }
}

fn bytes_contain(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
