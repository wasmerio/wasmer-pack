use anyhow::Error;
use clap::Parser;
use wasmer_pack_cli::{Codegen, Language, Show};

fn main() -> Result<(), Error> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::Js(js) => js.run(Language::JavaScript),
        Cmd::Python(py) => py.run(Language::Python),
        Cmd::Show(show) => show.run(),
    }
}

#[derive(Debug, Parser)]
#[clap(version)]
enum Cmd {
    /// Generate bindings for use with NodeJS.
    Js(Codegen),
    /// Generate Python bindings.
    Python(Codegen),
    /// Show metadata for the bindings that would be generated from a Pirita
    /// file.
    Show(Show),
}
