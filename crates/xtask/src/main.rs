use anyhow::Error;
use clap::Parser;

mod schema;
mod set_generator;

use crate::{schema::SyncSchema, set_generator::SetGenerator};

fn main() -> Result<(), Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        // Use a sane default
        std::env::set_var("RUST_LOG", "warn,xtask=info");
    }
    env_logger::init();

    match Command::parse() {
        Command::SyncSchema(s) => s.execute(),
        Command::SetGenerator(s) => s.execute(),
    }
}

#[derive(Debug, Parser)]
enum Command {
    SyncSchema(SyncSchema),
    SetGenerator(SetGenerator),
}
