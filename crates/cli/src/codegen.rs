use std::path::PathBuf;

use anyhow::{Context, Error};
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Codegen {
    /// Where to save the generated bindings.
    #[clap(short, long)]
    out_dir: Option<PathBuf>,
    /// The Pirita file to read.
    #[clap(parse(from_os_str))]
    input: PathBuf,
}

impl Codegen {
    pub fn run(self, language: Language) -> Result<(), Error> {
        let Codegen { out_dir, input } = self;

        let pkg = crate::load_pirita_file(&input)?;

        let files = match language {
            Language::JavaScript => wasmer_pack::generate_javascript(&pkg)?,
            Language::Python => wasmer_pack::generate_python(&pkg)?,
        };

        let metadata = pkg.metadata();

        let out_dir = out_dir.unwrap_or_else(|| {
            // If no output directory was specified, let's save to something
            // like "namespace/name/"
            let pkg_name = &metadata.package_name;
            match pkg_name.namespace().as_str() {
                Some(ns) => PathBuf::from(ns).join(pkg_name.name()),
                None => PathBuf::from(pkg_name.name()),
            }
        });
        files
            .save_to_disk(&out_dir)
            .with_context(|| format!("Unable to save to \"{}\"", out_dir.display()))?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Language {
    JavaScript,
    Python,
}
