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

pub struct Metadata(RefCell<upstream::Metadata>);

impl crate::wit_pack::Metadata for Metadata {
    fn new(
        package_name: String,
        version: String,
    ) -> Result<wit_bindgen_rust::Handle<crate::Metadata>, wit_pack::Error> {
        let meta = upstream::Metadata::new(package_name.parse()?, version);

        Ok(Handle::new(Metadata(RefCell::new(meta))))
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

impl From<wit_pack::Library> for upstream::Library {
    fn from(lib: wit_pack::Library) -> Self {
        let wit_pack::Library {
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

impl From<wit_pack::Command> for upstream::Command {
    fn from(cmd: wit_pack::Command) -> Self {
        let wit_pack::Command { name, wasm } = cmd;
        upstream::Command { name, wasm }
    }
}
