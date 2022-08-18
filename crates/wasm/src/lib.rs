use std::path::Path;

use anyhow::Error;
use wit_bindgen_rust::Handle;
use wit_pack::SourceFile;

wit_bindgen_rust::export!("exports.wit");

pub struct Exports;

impl exports::Exports for Exports {
    fn load_bindings(
        name: String,
        contents: String,
        wasm: Vec<u8>,
    ) -> Result<Handle<Bindings>, exports::Error> {
        let bindings = wit_pack::Bindings::from_src(&name, &contents, &wasm)?;
        Ok(Handle::new(Bindings(bindings)))
    }
}

pub struct Bindings(wit_pack::Bindings);

impl exports::Bindings for Bindings {
    fn generate_python(&self) -> Result<Vec<exports::File>, exports::Error> {
        todo!();
    }

    fn generate_javascript(&self) -> Result<Vec<exports::File>, exports::Error> {
        let js = self.0.javascript()?;
        let files = js.iter().map(|(p, f)| file(p, f)).collect();
        Ok(files)
    }
}

fn file(p: &Path, f: &SourceFile) -> exports::File {
    exports::File {
        filename: p.display().to_string(),
        contents: f.contents().into(),
    }
}

impl From<Error> for exports::Error {
    fn from(e: Error) -> Self {
        exports::Error {
            message: e.to_string(),
            verbose: format!("{e:?}"),
            causes: e.chain().map(|e| e.to_string()).collect(),
        }
    }
}
