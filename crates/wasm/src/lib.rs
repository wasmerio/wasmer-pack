extern crate wasmer_pack as upstream;

use std::cell::RefCell;

use anyhow::Error;
use upstream::SourceFile;
use wai_bindgen_rust::Handle;

wai_bindgen_rust::export!("wasmer-pack.exports.wai");

pub struct WasmerPack;

impl crate::wasmer_pack::WasmerPack for WasmerPack {
    fn generate_javascript(
        pkg: crate::wasmer_pack::Package,
    ) -> Result<Vec<wasmer_pack::File>, wasmer_pack::Error> {
        let pkg = pkg.into();
        let js = upstream::generate_javascript(&pkg)?;
        Ok(unwrap_files(js))
    }

    fn generate_python(
        pkg: crate::wasmer_pack::Package,
    ) -> Result<Vec<wasmer_pack::File>, wasmer_pack::Error> {
        let pkg = pkg.into();
        let py = upstream::generate_python(&pkg)?;
        Ok(unwrap_files(py))
    }
}

fn unwrap_files(files: upstream::Files) -> Vec<wasmer_pack::File> {
    files
        .into_iter()
        .map(|(path, SourceFile(contents))| wasmer_pack::File {
            filename: path.display().to_string(),
            contents,
        })
        .collect()
}

pub struct Interface(upstream::Interface);

impl crate::wasmer_pack::Interface for Interface {
    fn from_wit(name: String, contents: String) -> Result<Handle<Interface>, wasmer_pack::Error> {
        let exports = upstream::Interface::from_wit(&name, &contents)?;
        Ok(Handle::new(Interface(exports)))
    }

    fn from_path(path: String) -> Result<Handle<Interface>, wasmer_pack::Error> {
        let exports = upstream::Interface::from_path(path)?;
        Ok(Handle::new(Interface(exports)))
    }
}

pub struct Metadata(RefCell<upstream::Metadata>);

impl crate::wasmer_pack::Metadata for Metadata {
    fn new(
        package_name: String,
        version: String,
    ) -> Result<wai_bindgen_rust::Handle<crate::Metadata>, wasmer_pack::Error> {
        let meta = upstream::Metadata::new(package_name.parse()?, version);

        Ok(Handle::new(Metadata(RefCell::new(meta))))
    }

    fn set_description(&self, description: String) {
        self.0.borrow_mut().description = Some(description);
    }
}

impl From<crate::wasmer_pack::Abi> for ::wasmer_pack::Abi {
    fn from(abi: crate::wasmer_pack::Abi) -> Self {
        match abi {
            wasmer_pack::Abi::None => upstream::Abi::None,
            wasmer_pack::Abi::Wasi => upstream::Abi::Wasi,
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

impl From<crate::wasmer_pack::Package> for upstream::Package {
    fn from(pkg: crate::wasmer_pack::Package) -> Self {
        let crate::wasmer_pack::Package {
            metadata,
            libraries,
            commands,
        } = pkg;
        let metadata = metadata.0.borrow();
        upstream::Package::new(
            upstream::Metadata::clone(&metadata),
            libraries.into_iter().map(Into::into).collect(),
            commands.into_iter().map(Into::into).collect(),
        )
    }
}

impl From<wasmer_pack::Library> for upstream::Library {
    fn from(lib: wasmer_pack::Library) -> Self {
        let wasmer_pack::Library {
            interface,
            abi,
            wasm,
        } = lib;
        upstream::Library {
            module: upstream::Module {
                name: format!("{}.wasm", interface.0.name()),
                abi: abi.into(),
                wasm,
            },
            interface: interface.0.clone(),
        }
    }
}

impl From<wasmer_pack::Command> for upstream::Command {
    fn from(cmd: wasmer_pack::Command) -> Self {
        let wasmer_pack::Command { name, wasm } = cmd;
        upstream::Command { name, wasm }
    }
}
