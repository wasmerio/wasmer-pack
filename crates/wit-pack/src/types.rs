use std::{
    fmt::{self, Display, Formatter},
    path::Path,
    str::FromStr,
};

use anyhow::{Context, Error};
use heck::ToSnakeCase;

#[derive(Debug, Clone)]
pub struct Package {
    pub metadata: Metadata,
    pub libraries: Vec<Library>,
}

/// The name of a package from WAPM (e.g. `wasmer/wit-pack`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageName {
    namespace: String,
    name: String,
}

impl PackageName {
    pub fn parse(raw: &str) -> Result<Self, Error> {
        raw.parse()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the NPM equivalent of this [`PackageName`].
    ///
    /// This should satisfy NPM's
    /// [naming rules](https://github.com/npm/validate-npm-package-name#naming-rules):
    ///
    /// - package name length should be greater than zero
    /// - all the characters in the package name must be lowercase i.e., no uppercase or mixed case names are allowed
    /// - package name can consist of hyphens
    /// - package name must not contain any non-url-safe characters (since name ends up being part of a URL)
    /// - package name should not start with . or _
    /// - package name should not contain any spaces
    /// - package name should not contain any of the following characters: ~)('!*
    /// - package name cannot be the same as a node.js/io.js core module nor a reserved/blacklisted name. For example, the following names are invalid:
    ///   - http
    ///   - stream
    ///   - node_modules
    ///   - favicon.ico
    /// - package name length cannot exceed 214
    pub fn javascript_package(&self) -> String {
        let PackageName { namespace, name } = self;
        format!("@{namespace}/{name}")
    }

    /// Get the PyPI equivalent of this [`PackageName`].
    ///
    /// This should satisfy the naming scheme outlined in
    /// [PEP 8](https://peps.python.org/pep-0008/#package-and-module-names):
    ///
    /// > Modules should have short, all-lowercase names. Underscores can be
    /// > used in the module name if it improves readability. Python packages
    /// > should also have short, all-lowercase names, although the use of
    /// > underscores is discouraged.
    pub fn python_name(&self) -> String {
        self.name.to_snake_case()
    }
}

impl FromStr for PackageName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (namespace, name) = s.split_once('/').context(
            "All packages must have a namespace (i.e. the \"wasmer\" in \"wasmer/wit-pack\")",
        )?;
        let namespace = parse_identifier(namespace)
            .with_context(|| format!("\"{namespace}\" is not a valid namespace"))?;
        let name = parse_identifier(name)
            .with_context(|| format!("\"{name}\" is not a valid package name"))?;

        Ok(PackageName { namespace, name })
    }
}

impl Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let PackageName { namespace, name } = self;
        write!(f, "{namespace}/{name}")
    }
}

fn parse_identifier(s: &str) -> Result<String, Error> {
    anyhow::ensure!(!s.is_empty(), "Identifiers can't be empty");
    anyhow::ensure!(
        s.starts_with(|c: char| c.is_ascii_alphabetic()),
        "Identifiers must start with an ascii letter",
    );
    anyhow::ensure!(
        s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')),
        "Identifiers can only contain '-', '_', ascii numbers, and letters"
    );

    Ok(s.to_string())
}

/// Information about the [`Package`] being generated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Metadata {
    /// The package's name.
    pub package_name: PackageName,
    /// A semver-compliant version number.
    pub version: String,
    /// Extended information about the package.
    pub description: Option<String>,
}

impl Metadata {
    /// Create a new [`Metadata`] object with all required fields.
    pub fn new(package_name: PackageName, version: impl Into<String>) -> Self {
        Metadata {
            package_name,
            version: version.into(),
            description: None,
        }
    }

    /// Set the [`Metadata::description`] field.
    pub fn with_description(self, description: impl Into<String>) -> Self {
        Metadata {
            description: Some(description.into()),
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub struct Library {
    pub module: Module,
    pub interface: Interface,
}

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

    #[test]
    fn sanitize_package_names() {
        let inputs = vec![
            ("package", false),
            ("namespace/package_name", true),
            ("name-space/package-name", true),
            ("n9/p21", true),
            ("name space/name", false),
            ("wasmer/package", true),
            ("@wasmer/package-name", false),
            (
                "abcdefghijklmopqrstuvwxyz_ABCDEFGHIJKLMOPQRSTUVWXYZ0123456789/abcdefghijklmopqrstuvwxyz-ABCDEFGHIJKLMOPQRSTUVWXYZ0123456789",
                true,
            ),
            ("", false),
        ];

        for (original, is_okay) in inputs {
            let got = PackageName::parse(original);
            assert_eq!(got.is_ok(), is_okay, "{original}");
        }
    }
}
