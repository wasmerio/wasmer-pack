use anyhow::Error;
use clap::Parser;
use wasmer_pack_cli::{Codegen, Language, Show};

fn main() -> Result<(), Error> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::JavaScript(js) => js.run(Language::JavaScript),
        Cmd::Python(py) => py.run(Language::Python),
        Cmd::Show(show) => show.run(),
    }
}

#[derive(Debug, Parser)]
#[clap(version)]
enum Cmd {
    /// Generate bindings for use with NodeJS.
    #[clap(name = "javascript", alias = "js")]
    JavaScript(Codegen),
    /// Generate Python bindings.
    #[clap(alias = "py")]
    Python(Codegen),
    /// Show metadata for the bindings that would be generated from a Pirita
    /// file.
    Show(Show),
}
