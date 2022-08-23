use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use anyhow::{Context, Error};

/// Information about the package being generated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Metadata {
    /// What is the package's name?
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageName {
    namespace: String,
    name: String,
}

impl PackageName {
    pub fn namespace(&self) -> &str {
        &self.name
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn python_package(&self) -> String {
        self.name.replace("-", "_")
    }

    pub fn javascript_package(&self) -> String {
        let PackageName { namespace, name } = self;
        format!("@{namespace}/{name}")
    }
}

impl FromStr for PackageName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split("/");
        let namespace = words.next().context("Missing the namespace")?;
        let name = words.next().context("Missing the package name")?;

        anyhow::ensure!(
            namespace
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "The namespace may only contain letters, numbers, and '-' or '_'"
        );
        anyhow::ensure!(
            name.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "The name may only contain letters, numbers, and '-' or '_'"
        );

        Ok(PackageName {
            namespace: namespace.to_string(),
            name: name.to_string(),
        })
    }
}

impl Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let PackageName { namespace, name } = self;
        write!(f, "{namespace}/{name}")
    }
}
