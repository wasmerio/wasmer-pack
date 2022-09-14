//! A code generator that lets you treat WebAssembly modules like native
//! dependencies.
//!
//! # Basic Usage
//!
//! If you don't want to use the `wit-pack` CLI to generate bindings for your
//! language, you can always use this crate directly.
//!
//! ```rust,no_run
//! use wit_pack::{Module, Interface, Metadata, Abi};
//!
//! // First, load the relevant information from disk...
//! let package_name = "username/my-package".parse()?;
//! let metadata = Metadata::new(package_name, "1.2.3");
//! let module = Module::from_path("./module.wasm", Abi::None)?;
//! let interface = Interface::from_path("./exports.wit")?;
//!
//! // Now we can generate the bindings for our language
//! let js = wit_pack::generate_javascript(&metadata, &module, &interface)?;
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
    types::{Abi, Interface, Metadata, Module},
    wit_version::WIT_PARSER_VERSION,
};
