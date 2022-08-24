use std::{path::Path, str::FromStr};

use anyhow::{Context, Error};

/// A WebAssembly module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    /// A name used to refer to this module (e.g. `wit_pack_wasm`).
    pub name: String,
    /// The ABI used by the module.
    pub abi: Abi,
    /// The WebAssembly code, itself.
    pub wasm: Vec<u8>,
}

impl Module {
    /// Load a [`Module`] from a file on disk.
    ///
    /// # Note
    ///
    /// The [`Module::from_path()`] constructor explicitly **doesn't** perform
    /// any validation on the module's file. It is up to the caller to ensure
    /// they pass in the correct [`Abi`].
    pub fn from_path(path: impl AsRef<Path>, abi: Abi) -> Result<Self, Error> {
        let path = path.as_ref();
        let name = sanitized_module_name(path)?.to_string();

        let wasm = std::fs::read(path)
            .with_context(|| format!("Unable to read \"{}\"", path.display()))?;

        Ok(Module { name, abi, wasm })
    }
}

/// The [*Application Binary Interface*][abi] used by a [`Module`].
///
/// [abi]: https://www.webassembly.guide/webassembly-guide/webassembly/wasm-abis
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Abi {
    None,
    Wasi,
}

impl FromStr for Abi {
    type Err = Error;

    fn from_str(s: &str) -> Result<Abi, Error> {
        match s {
            "none" => Ok(Abi::None),
            "wasi" => Ok(Abi::Wasi),
            _ => Err(Error::msg("Expected either \"none\" or \"wasi\"")),
        }
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

    let first_segment = name.split('.').next().expect("Guaranteed to not be empty");

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
