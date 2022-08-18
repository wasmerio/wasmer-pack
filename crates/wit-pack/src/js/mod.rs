use std::path::{Path, PathBuf};

use anyhow::Error;
use wit_bindgen_gen_core::Generator;
use wit_bindgen_gen_js::Js;
use wit_parser::Interface;

use crate::{Bindings, Files, SourceFile};

pub(crate) fn generate(bindings: &Bindings) -> Result<Files, Error> {
    let Bindings { interface, wasm } = bindings;

    let mut files = Files::new();

    files.push(
        Path::new("src")
            .join(&interface.name)
            .with_extension("wasm"),
        SourceFile::new(wasm.clone()),
    );

    generate_bindings(interface, &mut files);

    let glue = generate_glue_code(&interface.name);
    files.push(PathBuf::from("src/index.js"), glue);

    let glue = generate_package_json(bindings.package_name());
    files.push(PathBuf::from("package.json"), glue);

    Ok(files)
}

fn generate_package_json(package_name: &str) -> SourceFile {
    let package_json = serde_json::json!({
        "name": package_name,
        "version": "0.0.0",
    });

    SourceFile::new((format!("{package_json:#}")).into())
}

fn generate_bindings(interface: &Interface, files: &mut Files) {
    let imports = &[];
    let exports = std::slice::from_ref(interface);
    let mut generated = wit_bindgen_gen_core::Files::default();

    Js::new().generate_all(imports, exports, &mut generated);

    let dir = Path::new("src").join("generated");

    for (path, contents) in generated.iter() {
        let path = dir.join(path);
        files.push(path, SourceFile::new(contents.into()));
    }
}

fn generate_glue_code(_interface_name: &str) -> SourceFile {
    SourceFile::new(Vec::new())
}
