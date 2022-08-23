/// Information about the package being generated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Metadata {
    /// What is the package's name?
    ///
    /// # Language Requirements
    ///
    /// Depending on the target language, the package name may have different
    /// constraints.
    ///
    /// For example, all packages being published to NPM should follow their
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
    ///
    /// Python package names should follow [PEP 8](https://peps.python.org/pep-0008/#package-and-module-names):
    ///
    /// > Modules should have short, all-lowercase names. Underscores can be
    /// > used in the module name if it improves readability. Python packages
    /// > should also have short, all-lowercase names, although the use of
    /// > underscores is discouraged.
    pub package_name: String,
    /// A semver-compliant version number.
    pub version: String,
    /// Extended information about the package.
    pub description: Option<String>,
}

impl Metadata {
    /// Create a new [`Metadata`] object with all required fields.
    pub fn new(package_name: impl Into<String>, version: impl Into<String>) -> Self {
        Metadata {
            package_name: package_name.into(),
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
