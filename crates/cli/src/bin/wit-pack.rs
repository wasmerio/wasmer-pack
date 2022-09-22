use anyhow::Error;
use clap::Parser;
use wit_pack_cli::{Codegen, Language};

fn main() -> Result<(), Error> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::Js(js) => js.run(Language::JavaScript),
        Cmd::Python(py) => py.run(Language::Python),
    }
}

#[derive(Debug, Parser)]
#[clap(version)]
enum Cmd {
    /// Generate bindings for use with NodeJS.
    Js(Codegen),
    /// Generate Python bindings.
    Python(Codegen),
}
