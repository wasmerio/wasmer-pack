use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
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
    let package_name =
        sanitize_javascript_package_name(&metadata.package_name).context("Invalid package name")?;
    let package_version = &metadata.version;

    let mut files = Files::new();

    files.push(
        Path::new("src").join(&module.name).with_extension("wasm"),
        SourceFile::from(&module.wasm),
    );

    generate_bindings(&interface.0, &mut files);

    let glue_code = Path::new("src").join(interface_name).with_extension("js");
    inject_load_function(module, interface_name, files.get_mut(&glue_code).unwrap())?;

    let typings_file = Path::new("src").join(interface_name).with_extension("d.ts");
    patch_typings_file(interface_name, files.get_mut(&typings_file).unwrap());

    let package_json =
        generate_package_json(module.abi, package_name, package_version, interface_name);
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

fn generate_package_json(
    abi: crate::Abi,
    package_name: &str,
    package_version: &str,
    interface_name: &str,
) -> SourceFile {
    let package_json = if abi == crate::Abi::Wasi {
        serde_json::json!({
            "name": package_name,
            "version": package_version,
            "main": format!("src/{interface_name}.js"),
            "types": format!("src/{interface_name}.d.ts"),
            "type": "module",
            "dependencies": {
                "@wasmer/wasi": "^1.1.2",
            },
        })
    } else {
        serde_json::json!({
            "name": package_name,
            "version": package_version,
            "main": format!("src/{interface_name}.js"),
            "types": format!("src/{interface_name}.d.ts"),
            "type": "module",
        })
    };

    format!("{package_json:#}").into()
}

/// Try to make sure the provided string can be used as a JavaScript package
/// name.
///
/// This won't catch everything, but it should provide a "good enough" first
/// approximation.
fn sanitize_javascript_package_name(name: &str) -> Result<&str, Error> {
    anyhow::ensure!(!name.is_empty(), "Package names can't be empty");

    for (i, c) in name.char_indices() {
        anyhow::ensure!(
            matches!(c, '.' | '-' | '_' | '/' | '@' | 'a'..='z' | 'A'..='Z' | '0'..='9'),
            "Invalid character, '{c}', at index {i}",
        );
    }

    let words: Vec<_> = name.split('/').collect();

    match *words.as_slice() {
        [_top_level_name] => {}
        [namespace, _name] => {
            anyhow::ensure!(
                namespace.starts_with('@'),
                "The namespace should start with a '@'"
            );
        }
        _ => anyhow::bail!("Namespaced JavaScript packages look like @namespace/package"),
    }

    Ok(name)
}

fn generate_bindings(interface: &Interface, files: &mut Files) {
    let imports: &[wit_parser::Interface] = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wit_bindgen_gen_core::Files::default();

    Js::new().generate_all(imports, exports, &mut generated);

    let dir = Path::new("src");

    for (path, contents) in generated.iter() {
        files.push(dir.join(path), contents.into());
    }
}

fn inject_load_function(
    module: &Module,
    interface_name: &str,
    file: &mut SourceFile,
) -> Result<(), Error> {
    let module_name = &module.name;
    let class_name = interface_name.to_pascal_case();

    writeln!(
        &mut file.0,
        r#"
import fs from "fs/promises";
import path from "path";
import * as url from "url";
"#
    )
    .unwrap();

    if matches!(module.abi, crate::Abi::Wasi) {
        writeln!(
            &mut file.0,
            r#"
import {{init as initWasi, WASI }} from "@wasmer/wasi";
"#
        )
        .unwrap();
    }

    writeln!(
        &mut file.0,
        r#"
export default async function() {{
    const wrapper = new {class_name}();

    const scriptDir = url.fileURLToPath(new URL('.', import.meta.url));
    const wasm = await fs.readFile(path.join(scriptDir, "{module_name}.wasm"));
"#
    )
    .unwrap();

    writeln!(
        &mut file.0,
        r#"
    let moduleToAwait;
    if (WebAssembly.compileStreaming) {{
      moduleToAwait = WebAssembly.compileStreaming(wasm);
    }}
    else {{
      moduleToAwait = WebAssembly.compile(wasm);
    }}
"#
    )
    .unwrap();

    if matches!(module.abi, crate::Abi::Wasi) {
        writeln!(
            &mut file.0,
            r#"
    const toAwait = await Promise.all([initWasi(), moduleToAwait]);
    const module = toAwait[1];
    let wasi = new WASI({{}});
    const imports = wasi.getImports(module);
"#
        )
        .unwrap();
    } else {
        writeln!(
            &mut file.0,
            r#"
    const module = await moduleToAwait;
    const imports = {{}};
"#
        )
        .unwrap();
    }

    writeln!(
        &mut file.0,
        r#"
    await wrapper.instantiate(module, imports);
"#
    )
    .unwrap();

    if matches!(module.abi, crate::Abi::Wasi) {
        writeln!(
            &mut file.0,
            r#"
    wasi.instantiate(wrapper.instance);
    "#
        )
        .unwrap();
    }

    writeln!(
        &mut file.0,
        r#"
    return wrapper;
}}
"#
    )
    .unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
        let package_name = "@wasmerio/wit-pack";

        let got = generate_package_json(crate::Abi::None, package_name, "0.0.0", "wit-pack");

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }

    #[test]
    fn package_json_wasi() {
        let package_name = "@wasmerio/wabt";

        let got = generate_package_json(crate::Abi::Wasi, package_name, "0.0.0", "wabt");

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }

    #[test]
    fn generated_files() {
        let expected: HashSet<&Path> = [
            "package.json",
            "src/wit-pack.js",
            "src/wit-pack.d.ts",
            "src/wit_pack_wasm.wasm",
            "src/intrinsics.js",
        ]
        .iter()
        .map(Path::new)
        .collect();
        let metadata = Metadata::new("wit-pack", "1.2.3");
        let module = Module {
            name: "wit_pack_wasm.wasm".to_string(),
            abi: crate::Abi::None,
            wasm: Vec::new(),
        };
        let interface = crate::Interface::from_wit(
            "wit-pack.exports.wit",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../wasm/wit-pack.exports.wit"
            )),
        )
        .unwrap();

        let files = generate_javascript(&metadata, &module, &interface).unwrap();

        let file_names: HashSet<&Path> = files.iter().map(|(path, _)| path).collect();
        assert_eq!(file_names, expected);
    }

    #[test]
    fn sanitize_js_package_names() {
        let inputs = vec![
            ("package", true),
            ("@wasmer/package", true),
            ("@wasmer/package-name", true),
            (
                "abcdefghijklmopqrstuvwxyz-ABCDEFGHIJKLMOPQRSTUVWXYZ_0123456789",
                true,
            ),
            ("wasmer/package", false),
            ("", false),
        ];

        for (original, is_okay) in inputs {
            let got = sanitize_javascript_package_name(original);
            assert_eq!(got.is_ok(), is_okay, "{original}");
        }
    }
}
