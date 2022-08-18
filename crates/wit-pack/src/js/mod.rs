use std::path::{Path, PathBuf};

use anyhow::Error;
use heck::{ToPascalCase, ToSnakeCase};
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

    let typings = generate_typings_file(&interface.name, bindings.package_name());
    files.push(PathBuf::from("src/index.d.ts"), typings);

    let package_json = generate_package_json(bindings.package_name());
    files.push(PathBuf::from("package.json"), package_json);

    Ok(files)
}

fn generate_package_json(package_name: &str) -> SourceFile {
    let package_json = serde_json::json!({
        "name": package_name,
        "version": "0.0.0",
        "lib": "src/index.js",
        "types": "src/index.d.ts",
        "type": "module",
    });

    SourceFile::new((format!("{package_json:#}")).into())
}

fn generate_bindings(interface: &Interface, files: &mut Files) {
    let imports = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wit_bindgen_gen_core::Files::default();

    Js::new().generate_all(imports, exports, &mut generated);

    let dir = Path::new("src").join("generated");

    for (path, contents) in generated.iter() {
        files.push(dir.join(path), SourceFile::new(contents.into()));
    }
}

fn generate_glue_code(interface_name: &str) -> SourceFile {
    let class_name = interface_name.to_pascal_case();
    let module_name = interface_name.to_snake_case();

    let template = r#"
import fs from "fs/promises";
import path from "path";
import * as url from 'url';

import { $CLASS_NAME } from "./generated/$MODULE_NAME.js";

export async function load() {
    const wrapper = new $CLASS_NAME();

    const scriptDir = url.fileURLToPath(new URL('.', import.meta.url));
    const wasm = await fs.readFile(path.join(scriptDir, "$MODULE_NAME.wasm"));

    const imports = {};
    await wrapper.instantiate(wasm, imports);
    return wrapper;
}
"#;

    SourceFile::new(
        template
            .replace("$CLASS_NAME", &class_name)
            .replace("$MODULE_NAME", &module_name)
            .into_bytes(),
    )
}

fn generate_typings_file(interface_name: &str, package_name: &str) -> SourceFile {
    let class_name = interface_name.to_pascal_case();
    let module_name = interface_name.to_snake_case();

    let template = r#"
import { $CLASS_NAME } from "./generated/$MODULE_NAME";

declare module "$PACKAGE_NAME" {
    /**
     * Load the WebAssembly module.
     */
    function load(): Promise<$CLASS_NAME>;
}
"#;

    SourceFile::new(
        template
            .replace("$PACKAGE_NAME", &package_name)
            .replace("$CLASS_NAME", &class_name)
            .replace("$MODULE_NAME", &module_name)
            .into_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glue_code() {
        let interface_name = "exports";

        let got = generate_glue_code(interface_name);

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }

    #[test]
    fn typings() {
        let interface_name = "exports";

        let got = generate_typings_file(interface_name, "wit-bindgen");

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }

    #[test]
    fn package_json() {
        let package_name = "wit-pack";

        let got = generate_package_json(package_name);

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }
}
