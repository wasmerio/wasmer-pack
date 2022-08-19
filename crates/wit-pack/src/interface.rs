use std::path::Path;

use anyhow::{Context, Error};

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
