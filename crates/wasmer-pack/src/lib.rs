//! A code generator that lets you treat WebAssembly modules like native
//! dependencies.
//!
//! # Basic Usage
//!
//! If you don't want to use the `wasmer-pack` CLI to generate bindings for your
//! language, you can always use this crate directly.
//!
//! ```rust,no_run
//! use wasmer_pack::{Abi, Module, Interface, Metadata, Library, Package};
//!
//! // First, we need to give the package some metadata
//! let package_name = "username/my-package".parse()?;
//! let metadata = Metadata::new(package_name, "1.2.3");
//!
//! // Then we'll load the libraries from disk (this example only uses one)
//! let module = Module::from_path("./module.wasm", Abi::None)?;
//! // Definitions for the functionality it exposes
//! let exports = Interface::from_path("./exports.wit")?;
//! // Functionality imported from the host
//! let imports = vec![
//!     Interface::from_path("./fs.wit")?,
//!     Interface::from_path("./logging.wit")?,
//! ];
//! let libraries = vec![Library { module, exports, imports }];
//! let commands = Vec::new();
//!
//! // finally, we've got all the information we need
//! let pkg = Package ::new(metadata, libraries, commands);
//!
//! // Now we can generate the bindings for our language
//! let js = wasmer_pack::generate_javascript(&pkg, wasmer_pack::BindingsOptions::default())?;
//!
//! // And finally, save them to disk
//! js.save_to_disk("./out")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;

mod files;
mod js;
mod pirita;
mod py;
mod types;
mod versions;

pub use crate::{
    files::{Files, SourceFile},
    js::generate_javascript,
    py::generate_python,
    types::BindingsOptions,
    types::{Abi, Command, Interface, Library, Metadata, Module, Package, PackageName},
    versions::WAI_PARSER_VERSION,
};

/// The generator name that will be mentioned at the top level of each generated
/// package.
pub const GENERATOR: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));
