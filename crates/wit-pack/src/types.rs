use std::{
    fmt::{self, Display, Formatter},
    path::Path,
    str::FromStr,
};

use anyhow::{Context, Error};
use heck::{ToPascalCase, ToSnakeCase};

#[derive(Debug, Clone)]
pub struct Package {
    metadata: Metadata,
    libraries: Vec<Library>,
    commands: Vec<Command>,
}

impl Package {
    /// Create a new [`Package`].
    ///
    /// # Panics
    ///
    /// This assumes all libraries have a unique [`Library::interface_name()`].
    pub fn new(metadata: Metadata, libraries: Vec<Library>, commands: Vec<Command>) -> Self {
        assert_unique_names("library", libraries.iter().map(|lib| lib.interface_name()));
        assert_unique_names("command", commands.iter().map(|cmd| cmd.name.as_str()));

        Package {
            metadata,
            libraries,
            commands,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn libraries(&self) -> impl Iterator<Item = &'_ Library> + '_ {
        self.libraries.iter()
    }

    pub fn requires_wasi(&self) -> bool {
        !self.commands.is_empty() || self.libraries.iter().any(|lib| lib.requires_wasi())
    }

    pub fn commands(&self) -> impl Iterator<Item = &'_ Command> + '_ {
        self.commands.iter()
    }
}

fn assert_unique_names<'a>(kind: &str, names: impl IntoIterator<Item = &'a str>) {
    let mut already_seen: Vec<&str> = Vec::new();

    for name in names {
        match already_seen.binary_search(&name) {
            Ok(_) => panic!("Duplicate {kind} name: {name}"),
            Err(index) => already_seen.insert(index, name),
        }
    }
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

impl Library {
    /// The name of the interface being generated.
    ///
    /// If coming from a WIT file, this will be the `wit-pack` in
    /// `wit-pack.exports.wit`.
    pub fn interface_name(&self) -> &str {
        &self.interface.0.name
    }

    /// The name of the class generated by `wit-bindgen`.
    ///
    /// For example, if you were generating bindings for `wit-pack.exports.wit`,
    /// this would be `WitPack`.
    pub fn class_name(&self) -> String {
        self.interface_name().to_pascal_case()
    }

    /// The filename of the [`Module`] this [`Library`] contains.
    ///
    /// For example, if the [`Module`] was loaded from `./path/to/wit-pack.wasm`,
    /// this would be `wit-pack.wasm`.
    pub fn module_filename(&self) -> &str {
        Path::new(&self.module.name)
            .file_name()
            .expect("We assume module names are non-empty")
            .to_str()
            .expect("The original path came from a Rust string")
    }

    pub fn requires_wasi(&self) -> bool {
        matches!(self.module.abi, Abi::Wasi)
    }
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
        let name = path
            .file_name()
            .context("Empty filename")?
            .to_string_lossy()
            .into_owned();

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

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub wasm: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
