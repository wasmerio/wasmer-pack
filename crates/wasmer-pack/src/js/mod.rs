use std::path::Path;

use anyhow::Error;
use heck::ToSnakeCase;
use minijinja::Environment;
use once_cell::sync::Lazy;
use wai_bindgen_gen_core::Generator;
use wai_bindgen_gen_js::Js;
use wai_parser::Interface;

use crate::{types::Command, Files, Library, Metadata, Package, SourceFile};

/// The version of `@wasmer/wasi` pulled in when using a WASI library.
const WASMER_WASI_VERSION: &str = "^1.1.2";

static TEMPLATES: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_template("bindings.index.js", include_str!("bindings.index.js.j2"))
        .unwrap();
    env.add_template(
        "bindings.index.d.ts",
        include_str!("bindings.index.d.ts.j2"),
    )
    .unwrap();
    env.add_template("command.d.ts", include_str!("command.d.ts.j2"))
        .unwrap();
    env.add_template("command.js", include_str!("command.js.j2"))
        .unwrap();
    env.add_template("top-level.index.js", include_str!("top-level.index.js.j2"))
        .unwrap();
    env.add_template(
        "top-level.index.d.ts",
        include_str!("top-level.index.d.ts.j2"),
    )
    .unwrap();

    env
});

/// Generate JavaScript bindings for a package.
pub fn generate_javascript(package: &Package) -> Result<Files, Error> {
    let mut files = Files::new();

    let libraries = package.libraries();
    if !libraries.is_empty() {
        files.insert_child_directory(
            Path::new("src").join("bindings"),
            library_bindings(libraries)?,
        );
    }

    for cmd in package.commands() {
        files.insert_child_directory(Path::new("src").join("commands"), command_bindings(cmd)?);
    }

    files.insert_child_directory("src", top_level(package.libraries(), package.commands())?);

    let package_json = generate_package_json(package.requires_wasi(), package.metadata());
    files.insert("package.json", package_json);

    // Note: We need to wrap the generated files in an extra folder because
    // that's how "npm pack" works
    let mut f = Files::new();
    f.insert_child_directory("package", files);

    Ok(f)
}

fn command_bindings(cmd: &Command) -> Result<Files, Error> {
    let mut files = Files::new();
    let module_filename = Path::new(&cmd.name).with_extension("wasm");
    let ctx = minijinja::context! {
        name => cmd.name.replace('-', "_"),
        module_filename,
    };

    files.insert(
        Path::new(&cmd.name).with_extension("js"),
        TEMPLATES
            .get_template("command.js")
            .unwrap()
            .render(&ctx)?
            .into(),
    );

    files.insert(
        Path::new(&cmd.name).with_extension("d.ts"),
        TEMPLATES
            .get_template("command.d.ts")
            .unwrap()
            .render(&ctx)?
            .into(),
    );
    files.insert(module_filename, SourceFile::from(&cmd.wasm));

    Ok(files)
}

fn top_level(libraries: &[Library], commands: &[Command]) -> Result<Files, Error> {
    let commands = commands
        .iter()
        .map(|cmd| {
            minijinja::context! {
                module => &cmd.name,
                ident => cmd.name.replace('-', "_"),
            }
        })
        .collect::<Vec<_>>();
    let ctx = minijinja::context! {
       commands,
       generator => crate::GENERATOR,
       libraries => !libraries.is_empty(),
    };
    let mut files = Files::new();

    files.insert(
        "index.js",
        TEMPLATES
            .get_template("top-level.index.js")
            .unwrap()
            .render(&ctx)?
            .into(),
    );

    files.insert(
        "index.d.ts",
        TEMPLATES
            .get_template("top-level.index.d.ts")
            .unwrap()
            .render(&ctx)?
            .into(),
    );

    Ok(files)
}

fn library_bindings(libraries: &[Library]) -> Result<Files, Error> {
    let mut files = Files::new();
    let mut lib_ctx = Vec::new();

    for lib in libraries {
        let module_filename = Path::new(lib.module_filename()).with_extension("wasm");
        let interface_name = lib.interface_name();
        let ident = interface_name.to_snake_case();
        let class_name = lib.class_name();

        let mut bindings = generate_bindings(&lib.interface.0);
        bindings.insert(&module_filename, lib.module.wasm.clone().into());
        files.insert_child_directory(interface_name, bindings);

        lib_ctx.push(LibraryContext {
            ident,
            interface_name: interface_name.to_string(),
            class_name,
            module_filename: module_filename.display().to_string(),
            wasi: lib.requires_wasi(),
        });
    }

    let ctx = LibrariesContext {
        libraries: lib_ctx,
        wasi: libraries.iter().any(|lib| lib.requires_wasi()),
    };

    let index_js = TEMPLATES
        .get_template("bindings.index.js")
        .unwrap()
        .render(&ctx)?;
    files.insert("index.js", index_js.into());

    let typings_file = TEMPLATES
        .get_template("bindings.index.d.ts")
        .unwrap()
        .render(&ctx)?;
    files.insert("index.d.ts", typings_file.into());

    Ok(files)
}

#[derive(Debug, Default, serde::Serialize)]
struct LibrariesContext {
    libraries: Vec<LibraryContext>,
    wasi: bool,
}

#[derive(Debug, Default, serde::Serialize)]
struct LibraryContext {
    /// The identifier that should be used when accessing this library.
    ident: String,
    /// The name of the interface (i.e. the `wasmer-pack` in
    /// `wasmer-pack.exports.wit`).
    interface_name: String,
    /// The name of the class generated by `wai-bindgen` (i.e. `WasmerPack`).
    class_name: String,
    /// The filename of the WebAssembly module (e.g. `wasmer-pack.wasm`).
    module_filename: String,
    wasi: bool,
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
    let imports: &[wai_parser::Interface] = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wai_bindgen_gen_core::Files::default();

    Js::new().generate_all(imports, exports, &mut generated);

    generated.into()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use insta::Settings;

    use crate::{Metadata, Module};

    use super::*;

    #[test]
    fn package_json() {
        let metadata = Metadata::new("wasmerio/wasmer-pack".parse().unwrap(), "0.0.0");

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
        let expected: BTreeSet<&Path> = [
            "package/package.json",
            "package/src/commands/first.d.ts",
            "package/src/commands/first.js",
            "package/src/commands/first.wasm",
            "package/src/commands/second-with-dashes.d.ts",
            "package/src/commands/second-with-dashes.js",
            "package/src/commands/second-with-dashes.wasm",
            "package/src/index.d.ts",
            "package/src/index.js",
            "package/src/bindings/index.d.ts",
            "package/src/bindings/index.js",
            "package/src/bindings/wasmer-pack/intrinsics.js",
            "package/src/bindings/wasmer-pack/wasmer_pack_wasm.wasm",
            "package/src/bindings/wasmer-pack/wasmer-pack.d.ts",
            "package/src/bindings/wasmer-pack/wasmer-pack.js",
        ]
        .iter()
        .map(Path::new)
        .collect();
        let metadata = Metadata::new("wasmer/wasmer-pack".parse().unwrap(), "1.2.3");
        let module = Module {
            name: "wasmer_pack_wasm.wasm".to_string(),
            abi: crate::Abi::None,
            wasm: Vec::new(),
        };
        let interface = crate::Interface::from_wit(
            "wasmer-pack.exports.wit",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../wasm/wasmer-pack.exports.wit"
            )),
        )
        .unwrap();
        let commands = vec![
            Command {
                name: "first".to_string(),
                wasm: Vec::new(),
            },
            Command {
                name: "second-with-dashes".to_string(),
                wasm: Vec::new(),
            },
        ];
        let libraries = vec![Library { module, interface }];
        let pkg = Package::new(metadata, libraries, commands);

        let files = generate_javascript(&pkg).unwrap();

        let actual_files: BTreeSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);

        let mut settings = Settings::clone_current();
        settings.add_filter(
            r"Generated by wasmer-pack v\d+\.\d+\.\d+",
            "Generated by XXX",
        );
        settings.bind(|| {
            insta::assert_display_snapshot!(files["package/package.json"].utf8_contents().unwrap());
            insta::assert_display_snapshot!(files["package/src/commands/first.d.ts"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["package/src/commands/first.js"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["package/src/commands/second-with-dashes.d.ts"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["package/src/commands/second-with-dashes.js"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["package/src/index.d.ts"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["package/src/index.js"].utf8_contents().unwrap());
            insta::assert_display_snapshot!(files["package/src/bindings/index.d.ts"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["package/src/bindings/index.js"]
                .utf8_contents()
                .unwrap());
        });
    }
}
