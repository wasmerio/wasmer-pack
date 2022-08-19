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
//! let metadata = Metadata::new("my-package", "1.2.3");
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

mod module;
mod files;
mod interface;
mod js;
mod metadata;
mod py;
mod wit_version;

pub use crate::{
    module::{Abi, Module},
    files::{Files, SourceFile},
    interface::Interface,
    js::generate_javascript,
    metadata::Metadata,
    py::generate_python,
    wit_version::WIT_PARSER_VERSION,
};
