use std::cell::RefCell;

use anyhow::Error;
use original::SourceFile;
use wai_bindgen_rust::Handle;

wai_bindgen_rust::export!("wasmer-pack.exports.wai");

pub struct WasmerPack;

impl crate::wasmer_pack::WasmerPack for WasmerPack {}

pub struct Package(original::Package);

impl crate::wasmer_pack::Package for Package {
    fn new(
        metadata: Handle<Metadata>,
        libraries: Vec<wasmer_pack::Library>,
        commands: Vec<wasmer_pack::Command>,
    ) -> Handle<crate::Package> {
        let metadata = metadata.0.borrow().clone();
        let libraries = libraries.into_iter().map(|lib| lib.into()).collect();
        let commands = commands.into_iter().map(|cmd| cmd.into()).collect();
        let pkg = original::Package::new(metadata, libraries, commands);

        Handle::new(Package(pkg))
    }

    fn from_webc(bytes: Vec<u8>) -> Result<Handle<crate::Package>, wasmer_pack::Error> {
        let pkg = original::Package::from_webc(&bytes)?;
        Ok(Handle::new(Package(pkg)))
    }

    fn from_wapm_tarball(tarball: Vec<u8>) -> Result<Handle<crate::Package>, wasmer_pack::Error> {
        let pkg = original::Package::from_wapm_tarball(tarball)?;
        Ok(Handle::new(Package(pkg)))
    }

    fn generate_javascript(&self) -> Result<Vec<wasmer_pack::File>, wasmer_pack::Error> {
        let files = original::generate_javascript(&self.0)?;
        Ok(unwrap_files(files))
    }

    fn generate_python(&self) -> Result<Vec<wasmer_pack::File>, wasmer_pack::Error> {
        let files = original::generate_python(&self.0)?;
        Ok(unwrap_files(files))
    }
}

fn unwrap_files(files: original::Files) -> Vec<wasmer_pack::File> {
    files
        .into_iter()
        .map(|(path, SourceFile(contents))| wasmer_pack::File {
            filename: path.display().to_string(),
            contents,
        })
        .collect()
}

pub struct Interface(original::Interface);

impl crate::wasmer_pack::Interface for Interface {
    fn from_wit(name: String, contents: String) -> Result<Handle<Interface>, wasmer_pack::Error> {
        let exports = original::Interface::from_wit(&name, &contents)?;
        Ok(Handle::new(Interface(exports)))
    }

    fn from_path(path: String) -> Result<Handle<Interface>, wasmer_pack::Error> {
        let exports = original::Interface::from_path(path)?;
        Ok(Handle::new(Interface(exports)))
    }
}

pub struct Metadata(RefCell<original::Metadata>);

impl crate::wasmer_pack::Metadata for Metadata {
    fn new(
        package_name: String,
        version: String,
    ) -> Result<Handle<crate::Metadata>, wasmer_pack::Error> {
        let meta = original::Metadata::new(package_name.parse()?, version);

        Ok(Handle::new(Metadata(RefCell::new(meta))))
    }

    fn set_description(&self, description: String) {
        self.0.borrow_mut().description = Some(description);
    }
}

impl From<crate::wasmer_pack::Abi> for original::Abi {
    fn from(abi: crate::wasmer_pack::Abi) -> Self {
        match abi {
            wasmer_pack::Abi::None => original::Abi::None,
            wasmer_pack::Abi::Wasi => original::Abi::Wasi,
        }
    }
}

impl From<Error> for crate::wasmer_pack::Error {
    fn from(e: Error) -> Self {
        crate::wasmer_pack::Error {
            message: e.to_string(),
            verbose: format!("{e:?}"),
            causes: e.chain().map(|e| e.to_string()).collect(),
        }
    }
}

impl From<wasmer_pack::Library> for original::Library {
    fn from(lib: wasmer_pack::Library) -> Self {
        let wasmer_pack::Library {
            exports,
            imports,
            abi,
            wasm,
        } = lib;
        original::Library {
            module: original::Module {
                name: format!("{}.wasm", exports.0.name()),
                abi: abi.into(),
                wasm,
            },
            exports: exports.0.clone(),
            imports: imports.iter().map(|i| i.0.clone()).collect(),
        }
    }
}

impl From<wasmer_pack::Command> for original::Command {
    fn from(cmd: wasmer_pack::Command) -> Self {
        let wasmer_pack::Command { name, wasm } = cmd;
        original::Command { name, wasm }
    }
}
