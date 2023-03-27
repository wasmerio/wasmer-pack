use std::path::Path;

use anyhow::Error;
use heck::{ToPascalCase, ToSnakeCase};
use minijinja::Environment;
use once_cell::sync::Lazy;
use wai_bindgen_gen_core::Generator;
use wai_bindgen_gen_js::Js;
use wai_parser::Interface;

use crate::{types::Command, Files, Library, Metadata, Package, SourceFile};

/// The version of `@wasmer/wasi` pulled in when using a WASI library.
///
/// Note: we need at least `1.2.2` so we get the fix for
/// [wasmer-js#310](https://github.com/wasmerio/wasmer-js/pull/310).
const WASMER_WASI_VERSION: &str = "^1.2.2";

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

    let ctx = Context::for_package(package);

    files.insert_child_directory(Path::new("src").join("bindings"), library_bindings(&ctx)?);

    for cmd in &ctx.commands {
        files.insert_child_directory(Path::new("src").join("commands"), command_bindings(cmd)?);
    }

    files.insert_child_directory("src", top_level(&ctx)?);

    let package_json = generate_package_json(package.requires_wasi(), package.metadata());
    files.insert("package.json", package_json);

    // Note: We need to wrap the generated files in an extra folder because
    // that's how "npm pack" works
    let mut f = Files::new();
    f.insert_child_directory("package", files);

    Ok(f)
}

#[derive(Debug, serde::Serialize)]
struct Context {
    libraries: Vec<LibraryContext>,
    commands: Vec<CommandContext>,
    generator: String,
    wasi: bool,
    has_wasi_libraries: bool,
}

impl Context {
    fn for_package(pkg: &Package) -> Self {
        let libraries: Vec<_> = pkg
            .libraries()
            .iter()
            .map(LibraryContext::for_lib)
            .collect();
        let commands: Vec<_> = pkg.commands().iter().map(CommandContext::for_cmd).collect();

        let has_wasi_libraries = libraries.iter().any(|lib| lib.wasi);

        let wasi = !commands.is_empty() || has_wasi_libraries;

        Context {
            libraries,
            commands,
            generator: crate::GENERATOR.to_string(),
            wasi,
            has_wasi_libraries,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct CommandContext {
    name: String,
    ident: String,
    module_filename: String,
    #[serde(skip)]
    wasm: Vec<u8>,
}

impl CommandContext {
    fn for_cmd(cmd: &Command) -> CommandContext {
        let module_filename = Path::new(&cmd.name).with_extension("wasm");

        CommandContext {
            name: cmd.name.clone(),
            ident: cmd.name.replace('-', "_"),
            module_filename: module_filename.display().to_string(),
            wasm: cmd.wasm.clone(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct LibraryContext {
    /// The identifier that should be used when accessing this library.
    ident: String,
    /// The filename of the WebAssembly module (e.g. `wasmer-pack.wasm`).
    module_filename: String,
    /// Does this library require WASI?
    wasi: bool,
    exports: InterfaceContext,
    imports: Vec<InterfaceContext>,
    #[serde(skip)]
    wasm: Vec<u8>,
}

impl LibraryContext {
    fn for_lib(lib: &Library) -> Self {
        let module_filename = Path::new(lib.module_filename()).with_extension("wasm");
        let interface_name = lib.interface_name();
        let ident = interface_name.to_snake_case();

        let exports = InterfaceContext {
            interface_name: lib.exports.name().to_string(),
            class_name: lib.exports.name().to_pascal_case(),
            interface: lib.exports.0.clone(),
        };
        let imports = lib
            .imports
            .iter()
            .map(|interface| InterfaceContext {
                interface_name: interface.name().to_string(),
                class_name: interface.name().to_pascal_case(),
                interface: interface.0.clone(),
            })
            .collect();

        LibraryContext {
            ident,
            module_filename: module_filename.display().to_string(),
            wasi: lib.requires_wasi(),
            exports,
            imports,
            wasm: lib.module.wasm.clone(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct InterfaceContext {
    /// The name of the interface (i.e. the `wasmer-pack` in
    /// `wasmer-pack.exports.wit`).
    interface_name: String,
    /// The name of the class generated by `wai-bindgen` (i.e. `WasmerPack`).
    class_name: String,
    #[serde(skip)]
    interface: Interface,
}

fn command_bindings(cmd: &CommandContext) -> Result<Files, Error> {
    let mut files = Files::new();
    let module_filename = Path::new(&cmd.name).with_extension("wasm");

    files.insert(
        Path::new(&cmd.name).with_extension("js"),
        TEMPLATES
            .get_template("command.js")
            .unwrap()
            .render(cmd)?
            .into(),
    );

    files.insert(
        Path::new(&cmd.name).with_extension("d.ts"),
        TEMPLATES
            .get_template("command.d.ts")
            .unwrap()
            .render(cmd)?
            .into(),
    );
    files.insert(module_filename, SourceFile::from(&cmd.wasm));

    Ok(files)
}

fn top_level(ctx: &Context) -> Result<Files, Error> {
    let mut files = Files::new();

    files.insert(
        "index.js",
        TEMPLATES
            .get_template("top-level.index.js")
            .unwrap()
            .render(ctx)?
            .into(),
    );

    files.insert(
        "index.d.ts",
        TEMPLATES
            .get_template("top-level.index.d.ts")
            .unwrap()
            .render(ctx)?
            .into(),
    );

    Ok(files)
}

fn library_bindings(ctx: &Context) -> Result<Files, Error> {
    let mut files = Files::new();

    for LibraryContext {
        module_filename,
        exports,
        imports,
        wasm,
        ..
    } in &ctx.libraries
    {
        let mut bindings = generate_bindings(exports, imports);
        bindings.insert(module_filename, wasm.into());
        files.insert_child_directory(&exports.interface_name, bindings);
    }

    let index_js = TEMPLATES
        .get_template("bindings.index.js")
        .unwrap()
        .render(ctx)?;
    files.insert("index.js", index_js.into());

    let typings_file = TEMPLATES
        .get_template("bindings.index.d.ts")
        .unwrap()
        .render(ctx)?;
    files.insert("index.d.ts", typings_file.into());

    Ok(files)
}

fn generate_package_json(needs_wasi: bool, metadata: &Metadata) -> SourceFile {
    let dependencies = if needs_wasi {
        serde_json::json!({
            "@wasmer/wasi": WASMER_WASI_VERSION,
        })
    } else {
        serde_json::json!({})
    };

    let mut package_json = serde_json::json!({
        "name": metadata.package_name.javascript_package(),
        "version": &metadata.version,
        "main": format!("src/index.js"),
        "types": format!("src/index.d.ts"),
        "type": "commonjs",
        "dependencies": dependencies,
    });
    if let Some(description) = &metadata.description {
        package_json["description"] = serde_json::Value::String(description.to_string());
    }

    format!("{package_json:#}").into()
}

fn generate_bindings(
    guest_exports: &InterfaceContext,
    guest_imports: &[InterfaceContext],
) -> Files {
    // Note: imports and exports were reported from the perspective of the
    // guest, but we're generating bindings from the perspective of the host.
    // Hence the "host_imports = guest_exports" thing.
    let host_imports: &[wai_parser::Interface] = &[guest_exports.interface.clone()];
    let host_exports: Vec<_> = guest_imports.iter().map(|i| i.interface.clone()).collect();

    let mut generated = wai_bindgen_gen_core::Files::default();

    Js::new().generate_all(host_imports, &host_exports, &mut generated);

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

    const WASMER_PACK_EXPORTS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../wasm/wasmer-pack.exports.wai"
    ));

    #[test]
    fn generated_files() {
        let expected: BTreeSet<&Path> = [
            "package/package.json",
            "package/src/bindings/index.d.ts",
            "package/src/bindings/index.js",
            "package/src/bindings/wasmer-pack/browser.d.ts",
            "package/src/bindings/wasmer-pack/browser.js",
            "package/src/bindings/wasmer-pack/intrinsics.js",
            "package/src/bindings/wasmer-pack/wasmer_pack_wasm.wasm",
            "package/src/bindings/wasmer-pack/wasmer-pack.d.ts",
            "package/src/bindings/wasmer-pack/wasmer-pack.js",
            "package/src/commands/first.d.ts",
            "package/src/commands/first.js",
            "package/src/commands/first.wasm",
            "package/src/commands/second-with-dashes.d.ts",
            "package/src/commands/second-with-dashes.js",
            "package/src/commands/second-with-dashes.wasm",
            "package/src/index.d.ts",
            "package/src/index.js",
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
        let exports =
            crate::Interface::from_wit("wasmer-pack.exports.wit", WASMER_PACK_EXPORTS).unwrap();
        let commands = vec![
            Command::new("first", []),
            Command::new("second-with-dashes", []),
        ];
        let browser =
            crate::Interface::from_wit("browser.wit", "greet: func(who: string) -> string")
                .unwrap();
        let libraries = vec![Library {
            module,
            exports,
            imports: vec![browser],
        }];
        let pkg = Package::new(metadata, libraries, commands);

        let files = generate_javascript(&pkg).unwrap();

        let actual_files: BTreeSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);

        let mut settings = Settings::clone_current();
        settings.add_filter(
            r"Generated by wasmer-pack v\d+\.\d+\.\d+(-\w+(\.\d+)?)?",
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
            insta::assert_display_snapshot!(files["package/src/bindings/wasmer-pack/browser.d.ts"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["package/src/bindings/wasmer-pack/browser.js"]
                .utf8_contents()
                .unwrap());
        });
    }
}
