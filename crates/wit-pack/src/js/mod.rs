use std::path::Path;

use anyhow::Error;
use minijinja::Environment;
use once_cell::sync::Lazy;
use wit_bindgen_gen_core::Generator;
use wit_bindgen_gen_js::Js;
use wit_parser::Interface;

use crate::{Files, Library, Metadata, Package, SourceFile};

/// The version of `@wasmer/wasi` pulled in when using a WASI library.
const WASMER_WASI_VERSION: &str = "^1.1.2";

static TEMPLATES: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_template("library.index.js", include_str!("library.index.js.j2"))
        .unwrap();
    env.add_template("library.index.d.ts", include_str!("library.index.d.ts.j2"))
        .unwrap();

    env
});

/// Generate JavaScript bindings for a package.
pub fn generate_javascript(package: &Package) -> Result<Files, Error> {
    let mut files = Files::new();

    for lib in &package.libraries {
        files.insert_child_directory(
            Path::new("src").join(lib.interface_name()),
            library_bindings(lib)?,
        );
    }

    files.insert("src/index.js", "export default function() {}".into());
    files.insert(
        "src/index.d.ts",
        "export default function(): Promise<unknown>;".into(),
    );

    let package_json = generate_package_json(package.requires_wasi(), &package.metadata);
    files.insert("package.json", package_json);

    Ok(files)
}

fn library_bindings(library: &Library) -> Result<Files, Error> {
    let mut files = generate_bindings(&library.interface.0);
    let ctx = library.js_context();

    let index_js = TEMPLATES
        .get_template("library.index.js")
        .unwrap()
        .render(&ctx)?;
    files.insert("index.js", index_js.into());

    let typings_file = TEMPLATES
        .get_template("library.index.d.ts")
        .unwrap()
        .render(&ctx)?;
    files.insert("index.d.ts", typings_file.into());

    files.insert(
        library.module_filename(),
        SourceFile::from(&library.module.wasm),
    );

    Ok(files)
}

impl Library {
    fn js_context(&self) -> impl serde::Serialize {
        minijinja::context! {
            wasi => self.requires_wasi(),
            class_name => self.class_name(),
            interface_name => self.interface_name(),
            module_filename => self.module_filename(),
        }
    }
}

fn generate_package_json(needs_wasi: bool, metadata: &Metadata) -> SourceFile {
    let dependencies = if needs_wasi {
        serde_json::json!({
            "@wasmer/wasi": WASMER_WASI_VERSION,
        })
    } else {
        serde_json::json!({})
    };

    let package_json = serde_json::json!({
        "name": metadata.package_name.javascript_package(),
        "version": &metadata.version,
        "main": format!("src/index.js"),
        "types": format!("src/index.d.ts"),
        "type": "module",
        "dependencies": dependencies,
    });

    format!("{package_json:#}").into()
}

fn generate_bindings(interface: &Interface) -> Files {
    let imports: &[wit_parser::Interface] = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wit_bindgen_gen_core::Files::default();

    Js::new().generate_all(imports, exports, &mut generated);

    generated.into()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{Metadata, Module};

    use super::*;

    #[test]
    fn package_json() {
        let metadata = Metadata::new("wasmerio/wit-pack".parse().unwrap(), "0.0.0");

        let got = generate_package_json(false, &metadata);

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }

    #[test]
    fn package_json_wasi() {
        let metadata = Metadata::new("wasmerio/wabt".parse().unwrap(), "0.0.0");

        let got = generate_package_json(true, &metadata);

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }

    #[test]
    fn generated_files() {
        let expected: HashSet<&Path> = [
            "package.json",
            "src/index.js",
            "src/index.d.ts",
            "src/wit-pack/index.js",
            "src/wit-pack/index.d.ts",
            "src/wit-pack/intrinsics.js",
            "src/wit-pack/wit_pack_wasm.wasm",
            "src/wit-pack/wit-pack.d.ts",
            "src/wit-pack/wit-pack.js",
        ]
        .iter()
        .map(Path::new)
        .collect();
        let metadata = Metadata::new("wasmer/wit-pack".parse().unwrap(), "1.2.3");
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
        let pkg = Package {
            metadata,
            libraries: vec![Library { module, interface }],
        };

        let files = generate_javascript(&pkg).unwrap();

        insta::assert_display_snapshot!(files["package.json"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["src/index.js"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["src/index.d.ts"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["src/wit-pack/index.js"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["src/wit-pack/index.d.ts"].utf8_contents().unwrap());

        let actual_files: HashSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);
    }
}
