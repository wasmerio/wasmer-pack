//! A code generator that lets you treat WebAssembly modules like native
//! dependencies.
//!
//! # Basic Usage
//!
//! If you don't want to use the `wit-pack` CLI to generate bindings for your
//! language, you can always use this crate directly.
//!
//! ```rust,no_run
//! use wit_pack::{Abi, Module, Interface, Metadata, Library, Package};
//!
//! // First, we need to give the package some metadata
//! let package_name = "username/my-package".parse()?;
//! let metadata = Metadata::new(package_name, "1.2.3");
//!
//! // Then we'll load the libraries from disk (this example only uses one)
//! let module = Module::from_path("./module.wasm", Abi::None)?;
//! let interface = Interface::from_path("./exports.wit")?;
//! let libraries = vec![Library { module, interface }];
//! let commands = Vec::new();
//!
//! // finally, we've got all the information we need
//! let pkg = Package ::new(metadata, libraries, commands);
//!
//! // Now we can generate the bindings for our language
//! let js = wit_pack::generate_javascript(&pkg)?;
//!
//! // And finally, save them to disk
//! js.save_to_disk("./out")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

mod files;
mod js;
mod py;
mod types;
mod wit_version;

pub use crate::{
    files::{Files, SourceFile},
    js::generate_javascript,
    py::generate_python,
    types::{Abi, Command, Interface, Library, Metadata, Module, Package, PackageName},
    wit_version::WIT_PARSER_VERSION,
};
