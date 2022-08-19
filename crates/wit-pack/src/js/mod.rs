use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Error;
use heck::ToPascalCase;
use wit_bindgen_gen_core::Generator;
use wit_bindgen_gen_js::Js;
use wit_parser::Interface;

use crate::{Files, Metadata, Module, SourceFile};

/// Generate JavaScript bindings for a package.
pub fn generate_javascript(
    metadata: &Metadata,
    module: &Module,
    interface: &crate::Interface,
) -> Result<Files, Error> {
    let interface_name = &interface.0.name;

    let mut files = Files::new();

    files.push(
        Path::new("src").join(&module.name).with_extension("wasm"),
        SourceFile::new(module.wasm.clone()),
    );

    generate_bindings(&interface.0, &mut files);

    let glue_code = Path::new("src").join(interface_name).with_extension("js");
    inject_load_function(&module, interface_name, files.get_mut(&glue_code).unwrap())?;

    let typings_file = Path::new("src").join(interface_name).with_extension("d.ts");
    patch_typings_file(interface_name, files.get_mut(&typings_file).unwrap());

    let package_json = generate_package_json(&metadata.package_name, interface_name);
    files.push(PathBuf::from("package.json"), package_json);

    Ok(files)
}

fn patch_typings_file(interface_name: &str, typings_file: &mut SourceFile) {
    let class_name = interface_name.to_pascal_case();

    writeln!(&mut typings_file.0).unwrap();
    writeln!(&mut typings_file.0, "/** Load the WebAssembly module. */").unwrap();
    writeln!(
        &mut typings_file.0,
        r#"export default function(): Promise<{class_name}>;"#
    )
    .unwrap();
}

fn generate_package_json(package_name: &str, interface_name: &str) -> SourceFile {
    let package_json = serde_json::json!({
        "name": package_name,
        "version": "0.0.0",
        "main": format!("src/{interface_name}.js"),
        "types": format!("src/{interface_name}.d.ts"),
        "type": "module",
    });

    SourceFile::new((format!("{package_json:#}")).into())
}

fn generate_bindings(interface: &Interface, files: &mut Files) {
    let imports: &[wit_parser::Interface] = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wit_bindgen_gen_core::Files::default();

    Js::new().generate_all(imports, exports, &mut generated);

    let dir = Path::new("src");

    for (path, contents) in generated.iter() {
        files.push(dir.join(path), SourceFile::new(contents.into()));
    }
}

fn inject_load_function(
    module: &Module,
    interface_name: &str,
    file: &mut SourceFile,
) -> Result<(), Error> {
    if matches!(module.abi, crate::Abi::Wasi) {
        anyhow::bail!("");
    }

    let module_name = &module.name;
    let class_name = interface_name.to_pascal_case();

    writeln!(
        &mut file.0,
        r#"
import fs from "fs/promises";
import path from "path";
import * as url from "url";

export default async function() {{
    const wrapper = new {class_name}();

    const scriptDir = url.fileURLToPath(new URL('.', import.meta.url));
    const wasm = await fs.readFile(path.join(scriptDir, "{module_name}.wasm"));

    const imports = {{}};
    await wrapper.instantiate(wasm, imports);
    return wrapper;
}}
"#
    )
    .unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glue_code() {
        let mut file = SourceFile::default();
        let module = Module {
            name: "wit-pack-wasm".to_string(),
            abi: crate::Abi::None,
            wasm: Vec::new(),
        };

        inject_load_function(&module, "wit-pack", &mut file).unwrap();

        let contents = file.utf8_contents().unwrap();
        insta::assert_display_snapshot!(contents);
        assert!(contents.contains("export default async function()"));
    }

    #[test]
    fn typings() {
        let interface_name = "exports";
        let mut file = SourceFile::default();

        patch_typings_file(interface_name, &mut file);

        insta::assert_display_snapshot!(file.utf8_contents().unwrap());
        assert!(file
            .utf8_contents()
            .unwrap()
            .contains("export default function(): Promise<Exports>;"));
    }

    #[test]
    fn package_json() {
        let package_name = "wasmerio/wit-pack";

        let got = generate_package_json(package_name, "wit-pack");

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }
}
