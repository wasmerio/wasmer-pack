use anyhow::Error;

use crate::{Files, Interface, Metadata, Module};

/// Generate Python bindings.
pub fn generate_python(
    _metadata: &Metadata,
    _module: &Module,
    _interface: &Interface,
) -> Result<Files, Error> {
    todo!()
}
