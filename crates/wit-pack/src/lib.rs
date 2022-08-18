//! A code generator that lets you treat WebAssembly modules like native
//! dependencies.
//!
//! # Basic Usage
//!
//! If you don't want to use the `wit-pack` CLI to generate bindings for your
//! language, you can always use this crate directly.
//!
//! ```rust,no_run
//! use wit_pack::Bindings;
//!
//! // First, load the bindings from disk
//! let bindings = Bindings::from_disk("exports.wit", "module.wasm")?;
//!
//! // Next, generate the bindings for your language
//! let js = bindings.javascript()?;
//!
//! // And finally, save them to disk
//! js.save_to_disk("./out")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

mod bindings;
mod files;
mod js;
mod wit_version;

pub use crate::{
    bindings::Bindings,
    files::{Files, SourceFile},
    wit_version::WIT_PARSER_VERSION,
};
