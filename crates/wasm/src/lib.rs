extern crate wit_pack as upstream;

use std::cell::RefCell;

use anyhow::Error;
use upstream::SourceFile;
use wit_bindgen_rust::Handle;

wit_bindgen_rust::export!("wit-pack.exports.wit");

pub struct WitPack;

impl crate::wit_pack::WitPack for WitPack {
    fn generate_javascript(
        pkg: crate::wit_pack::Package,
    ) -> Result<Vec<wit_pack::File>, wit_pack::Error> {
        let pkg = pkg.into();
        let js = upstream::generate_javascript(&pkg)?;
        Ok(unwrap_files(js))
    }

    fn generate_python(
        pkg: crate::wit_pack::Package,
    ) -> Result<Vec<wit_pack::File>, wit_pack::Error> {
        let pkg = pkg.into();
        let py = upstream::generate_python(&pkg)?;
        Ok(unwrap_files(py))
    }
}

fn unwrap_files(files: upstream::Files) -> Vec<wit_pack::File> {
    files
        .into_iter()
        .map(|(path, SourceFile(contents))| wit_pack::File {
            filename: path.display().to_string(),
            contents,
        })
        .collect()
}

pub struct Interface(upstream::Interface);

impl crate::wit_pack::Interface for Interface {
    fn from_wit(name: String, contents: String) -> Result<Handle<Interface>, wit_pack::Error> {
        let exports = upstream::Interface::from_wit(&name, &contents)?;
        Ok(Handle::new(Interface(exports)))
    }

    fn from_path(path: String) -> Result<Handle<Interface>, wit_pack::Error> {
        let exports = upstream::Interface::from_path(&path)?;
        Ok(Handle::new(Interface(exports)))
    }
}

pub struct Module(upstream::Module);

impl crate::wit_pack::Module for Module {
    fn new(name: String, abi: wit_pack::Abi, wasm: Vec<u8>) -> Handle<Module> {
        Handle::new(Module(upstream::Module {
            name,
            abi: abi.into(),
            wasm,
        }))
    }
}

pub struct Metadata(RefCell<upstream::Metadata>);

impl crate::wit_pack::Metadata for Metadata {
    fn new(package_name: String, version: String) -> wit_bindgen_rust::Handle<crate::Metadata> {
        Handle::new(Metadata(RefCell::new(upstream::Metadata::new(
            package_name.parse().expect("Invalid package name"),
            version,
        ))))
    }

    fn set_description(&self, description: String) {
        self.0.borrow_mut().description = Some(description);
    }
}

impl From<crate::wit_pack::Abi> for ::wit_pack::Abi {
    fn from(abi: crate::wit_pack::Abi) -> Self {
        match abi {
            wit_pack::Abi::None => upstream::Abi::None,
            wit_pack::Abi::Wasi => upstream::Abi::Wasi,
        }
    }
}

impl From<Error> for crate::wit_pack::Error {
    fn from(e: Error) -> Self {
        crate::wit_pack::Error {
            message: e.to_string(),
            verbose: format!("{e:?}"),
            causes: e.chain().map(|e| e.to_string()).collect(),
        }
    }
}

impl From<crate::wit_pack::Package> for upstream::Package {
    fn from(pkg: crate::wit_pack::Package) -> Self {
        let crate::wit_pack::Package {
            metadata,
            libraries,
        } = pkg;
        let metadata = metadata.0.borrow();
        upstream::Package {
            metadata: upstream::Metadata::clone(&metadata),
            libraries: libraries
                .into_iter()
                .map(|lib| upstream::Library {
                    module: lib.module.0.clone(),
                    interface: lib.interface.0.clone(),
                })
                .collect(),
        }
    }
}
