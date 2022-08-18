use anyhow::Error;
use wit_bindgen_rust::Handle;

wit_bindgen_rust::export!("exports.wit");

pub struct Exports;

impl exports::Exports for Exports {
    fn parse(
        name: String,
        contents: String,
        wasm: Vec<u8>,
    ) -> Result<Handle<Bindings>, exports::Error> {
        let bindings = wit_pack::Bindings::from_src(&name, &contents, &wasm)?;
        Ok(Handle::new(Bindings(bindings)))
    }

    fn parse_from_disk(
        wit_file_path: String,
        wasm_path: String,
    ) -> Result<Handle<Bindings>, exports::Error> {
        let bindings = wit_pack::Bindings::from_disk(&wit_file_path, &wasm_path)?;
        Ok(Handle::new(Bindings(bindings)))
    }
}

pub struct Bindings(wit_pack::Bindings);

impl exports::Bindings for Bindings {
    fn generate_python(&self, _ouput_dir: String) -> Result<(), exports::Error> {
        todo!();
    }

    fn generate_javascript(&self, output_dir: String) -> Result<(), exports::Error> {
        self.0.javascript()?.save_to_disk(&output_dir)?;
        Ok(())
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
