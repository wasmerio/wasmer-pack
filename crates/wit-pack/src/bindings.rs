use std::path::Path;

use anyhow::{Context, Error};
use wit_parser::Interface;

use crate::Files;

/// An interface definition that has been parsed into memory.
#[derive(Debug)]
pub struct Bindings {
    pub(crate) interface: Interface,
    pub(crate) wasm: Vec<u8>,
}

impl Bindings {
    /// Parse a set of [`Bindings`] from its interface definition in memory.
    ///
    /// This will **not** attempt to parse any other files the interface
    /// definition depends on.
    pub fn from_src(name: &str, src: &str, wasm: &[u8]) -> Result<Self, Error> {
        let interface = Interface::parse(name, src).context("Unable to parse the WIT file")?;

        Ok(Bindings {
            interface,
            wasm: wasm.into(),
        })
    }

    /// Parse a set of [`Bindings`] from its interface definition and `*.wasm`
    /// files on disk, potentially recursively parsing any files they depend on.
    pub fn from_path(
        wit_file: impl AsRef<Path>,
        wasm_file: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let interface = Interface::parse_file(wit_file).context("Unable to parse the WIT file")?;

        let wasm_file = wasm_file.as_ref();
        let wasm = std::fs::read(&wasm_file)
            .with_context(|| format!("Unable to read \"{}\"", wasm_file.display()))?;

        Ok(Bindings { interface, wasm })
    }

    /// Get the generated JavaScript bindings.
    pub fn javascript(&self) -> Result<Files, Error> {
        crate::js::generate(self)
    }

    pub(crate) fn package_name(&self) -> &str {
        &self.interface.name
    }
}
