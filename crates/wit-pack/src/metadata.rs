/// Information about the package being generated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Metadata {
    /// What is the package's name?
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
