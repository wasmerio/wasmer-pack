use std::path::Path;

use anyhow::{Context, Error};

use crate::Files;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Metadata {
    pub package_name: String,
    pub version: String,
    pub description: Option<String>,
}

impl Metadata {
    pub fn new(package_name: impl Into<String>, version: impl Into<String>) -> Self {
        Metadata {
            package_name: package_name.into(),
            version: version.into(),
            description: None,
        }
    }

    pub fn with_description(self, description: impl Into<String>) -> Self {
        Metadata {
            description: Some(description.into()),
            ..self
        }
    }
}

/// The interface exported by the WebAssembly module.
#[derive(Debug, Clone)]
pub struct Interface(pub(crate) wit_parser::Interface);

impl Interface {
    /// Parse an interface definition in the WIT format.
    ///
    /// This will **not** attempt to parse any other files the interface
    /// definition depends on.
    pub fn from_wit(name: &str, src: &str) -> Result<Self, Error> {
        let wit =
            wit_parser::Interface::parse(name, src).context("Unable to parse the WIT file")?;
        Ok(Interface(wit))
    }

    /// Parse an [`Interface`] from its interface definition on disk,
    /// potentially recursively parsing any files it depends on.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let wit = wit_parser::Interface::parse_file(path)
            .with_context(|| format!("Unable to parse \"{}\"", path.display()))?;
        Ok(Interface(wit))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub abi: Abi,
    pub wasm: Vec<u8>,
}

impl Module {
    pub fn from_path(path: impl AsRef<Path>, abi: Abi) -> Result<Self, Error> {
        let path = path.as_ref();
        let name = sanitized_module_name(path)?.to_string();

        let wasm = std::fs::read(path)
            .with_context(|| format!("Unable to read \"{}\"", path.display()))?;

        Ok(Module { name, abi, wasm })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Abi {
    None,
    Wasi,
}

/// An interface definition that has been parsed into memory.
#[derive(Debug)]
pub struct Bindings {
    pub metadata: Metadata,
    pub interface: Interface,
    pub module: Module,
}

impl Bindings {
    /// Get the generated JavaScript bindings.
    pub fn javascript(&self) -> Result<Files, Error> {
        crate::js::generate(self)
    }

    /// Get the generated JavaScript bindings.
    pub fn python(&self) -> Result<Files, Error> {
        todo!()
    }
}

fn sanitized_module_name(path: &Path) -> Result<&str, Error> {
    // This matches the logic used by wit-bindgen when deriving a module's name.
    // https://github.com/bytecodealliance/wit-bindgen/blob/cb871cfa1ee460b51eb1d144b175b9aab9c50aba/crates/parser/src/lib.rs#L344-L352

    let name = path
        .file_name()
        .context("wit path must end in a file name")?
        .to_str()
        .context("wit filename must be valid unicode")?;

    let first_segment = name.split(".").next().expect("Guaranteed to not be empty");

    Ok(first_segment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitized_names() {
        let inputs = vec![
            ("exports.wit", "exports"),
            ("wit-pack.exports.wit", "wit-pack"),
        ];

        for (filename, expected) in inputs {
            let got = sanitized_module_name(filename.as_ref()).unwrap();
            assert_eq!(got, expected);
        }
    }
}
