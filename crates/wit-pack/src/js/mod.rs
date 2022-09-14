use std::path::Path;

use anyhow::Error;
use heck::ToPascalCase;
use minijinja::Environment;
use once_cell::sync::Lazy;
use wit_bindgen_gen_core::Generator;
use wit_bindgen_gen_js::Js;
use wit_parser::Interface;

use crate::{Files, Metadata, Module, SourceFile};

/// The version of `@wasmer/wasi` pulled in when using a WASI library.
const WASMER_WASI_VERSION: &str = "^1.1.2";

static TEMPLATES: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_template("bindings.js", include_str!("bindings.js.j2"))
        .unwrap();
    env.add_template("bindings.d.ts", include_str!("bindings.d.ts.j2"))
        .unwrap();
    env.add_template("index.js", include_str!("index.js.j2"))
        .unwrap();
    env.add_template("index.d.ts", include_str!("index.d.ts.j2"))
        .unwrap();

    env
});

/// Generate JavaScript bindings for a package.
pub fn generate_javascript(
    metadata: &Metadata,
    module: &Module,
    interface: &crate::Interface,
) -> Result<Files, Error> {
    let inputs = Inputs {
        metadata,
        libraries: vec![Library { interface, module }],
        commands: Vec::new(),
    };
    _generate_javascript(&inputs)
}

fn _generate_javascript(inputs: &Inputs) -> Result<Files, Error> {
    let mut files = Files::new();

    for lib in &inputs.libraries {
        generate_library_bindings(lib, &mut files)?;
    }

    for cmd in &inputs.commands {
        generate_command_bindings(cmd, &mut files)?;
    }

    files.insert("src/index.js", generate_top_level(inputs)?);
    files.insert("src/index.d.ts", generate_top_level_typings(inputs)?);

    let package_name = inputs.metadata.package_name.javascript_package();
    let package_json =
        generate_package_json(inputs.needs_wasi(), &package_name, &inputs.metadata.version);
    files.insert("package.json", package_json);

    Ok(files)
}

fn generate_top_level_typings(inputs: &Inputs) -> Result<SourceFile, Error> {
    let rendered = TEMPLATES
        .get_template("index.d.ts")
        .unwrap()
        .render(inputs.js_context())?;

    Ok(rendered.into())
}

fn generate_top_level(inputs: &Inputs) -> Result<SourceFile, Error> {
    let ctx = minijinja::context! {
        libraries => inputs.library_context(),
    };
    let rendered = TEMPLATES.get_template("index.js").unwrap().render(ctx)?;

    Ok(rendered.into())
}

fn generate_command_bindings(_cmd: &Command<'_>, _files: &mut Files) -> Result<(), Error> {
    Ok(())
}

#[derive(Debug, Clone)]
struct Library<'a> {
    interface: &'a crate::Interface,
    module: &'a Module,
}

impl Library<'_> {
    fn js_context(&self) -> impl serde::Serialize + '_ {
        minijinja::context! {
            interface_name => &self.interface.0.name,
            module_name => &self.module.name,
            class_name => self.interface.0.name.to_pascal_case(),
            wasi => matches!(self.module.abi, crate::Abi::Wasi),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Command<'a> {
    name: &'a str,
    module: &'a Module,
}

#[derive(Debug, Clone)]
struct Inputs<'a> {
    metadata: &'a Metadata,
    libraries: Vec<Library<'a>>,
    commands: Vec<Command<'a>>,
}

impl Inputs<'_> {
    fn needs_wasi(&self) -> bool {
        self.libraries
            .iter()
            .any(|lib| matches!(lib.module.abi, crate::Abi::Wasi))
    }

    fn js_context(&self) -> impl serde::Serialize + '_ {
        minijinja::context! {
            libraries => self.library_context(),
        }
    }

    fn library_context(&self) -> Vec<impl serde::Serialize + '_> {
        self.libraries
            .iter()
            .map(|lib| lib.js_context())
            .collect::<Vec<_>>()
    }
}

fn generate_library_bindings(library: &Library<'_>, files: &mut Files) -> Result<(), Error> {
    let Library { interface, module } = library;
    let interface_name = &interface.0.name;
    let dir = Path::new("src").join(interface_name);

    for (filename, bytes) in generate_bindings(&interface.0).iter() {
        files.insert(dir.join(filename), bytes.into());
    }

    let index_js = TEMPLATES
        .get_template("bindings.js")
        .unwrap()
        .render(library.js_context())?;
    files.insert(dir.join("index.js"), index_js.into());

    let typings_file = library_typings_file(library)?;
    files.insert(dir.join("index.d.ts"), typings_file);

    files.insert(
        dir.join(&module.name).with_extension("wasm"),
        SourceFile::from(&module.wasm),
    );

    Ok(())
}

fn library_typings_file(library: &Library<'_>) -> Result<SourceFile, Error> {
    let rendered = TEMPLATES
        .get_template("bindings.d.ts")
        .unwrap()
        .render(library.js_context())?;

    Ok(rendered.into())
}

fn generate_package_json(
    needs_wasi: bool,
    package_name: &str,
    package_version: &str,
) -> SourceFile {
    let dependencies = if needs_wasi {
        serde_json::json!({
            "@wasmer/wasi": WASMER_WASI_VERSION,
        })
    } else {
        serde_json::json!({})
    };

    let package_json = serde_json::json!({
        "name": package_name,
        "version": package_version,
        "main": format!("src/index.js"),
        "types": format!("src/index.d.ts"),
        "type": "module",
        "dependencies": dependencies,
    });

    format!("{package_json:#}").into()
}

fn generate_bindings(interface: &Interface) -> wit_bindgen_gen_core::Files {
    let imports: &[wit_parser::Interface] = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wit_bindgen_gen_core::Files::default();

    Js::new().generate_all(imports, exports, &mut generated);

    generated
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn package_json() {
        let package_name = "@wasmerio/wit-pack";

        let got = generate_package_json(false, package_name, "0.0.0");

        insta::assert_display_snapshot!(got.utf8_contents().unwrap());
    }

    #[test]
    fn package_json_wasi() {
        let package_name = "@wasmerio/wabt";

        let got = generate_package_json(true, package_name, "0.0.0");

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

        let files = generate_javascript(&metadata, &module, &interface).unwrap();

        insta::assert_display_snapshot!(files["package.json"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["src/index.js"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["src/index.d.ts"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["src/wit-pack/index.js"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["src/wit-pack/index.d.ts"].utf8_contents().unwrap());

        let actual_files: HashSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);
    }
}
