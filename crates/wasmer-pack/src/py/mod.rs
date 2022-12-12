use std::path::Path;

use anyhow::Error;
use heck::{ToPascalCase, ToSnakeCase};
use minijinja::Environment;
use once_cell::sync::Lazy;
use wai_bindgen_gen_core::Generator;
use wai_bindgen_gen_wasmer_py::WasmerPy;

use crate::{
    types::{Interface, Package},
    Files, Metadata, Module, SourceFile,
};

static TEMPLATES: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_template(
        "bindings.__init__.py",
        include_str!("bindings.__init__.py.j2"),
    )
    .unwrap();
    env.add_template(
        "top_level.__init__.py",
        include_str!("top_level.__init__.py.j2"),
    )
    .unwrap();
    env.add_template("MANIFEST.in", include_str!("MANIFEST.in.j2"))
        .unwrap();
    env.add_template(
        "commands.__init__.py",
        include_str!("commands.__init__.py.j2"),
    )
    .unwrap();

    env
});

/// Generate Python bindings.
pub fn generate_python(package: &Package) -> Result<Files, Error> {
    let metadata = package.metadata();
    let package_name = metadata.package_name.python_name();

    let mut files = Files::new();

    let ctx = Context::for_package(package);

    if !ctx.libraries.is_empty() {
        files.insert_child_directory(
            Path::new(&package_name).join("bindings"),
            library_bindings(&ctx)?,
        );
    }

    if !ctx.commands.is_empty() {
        files.insert_child_directory(
            Path::new(&package_name).join("commands"),
            command_bindings(&ctx)?,
        );
    }

    files.insert(
        Path::new(&package_name).join("__init__.py"),
        top_level_dunder_init(package)?,
    );
    // Indicate that we use type hints
    files.insert(
        Path::new(&package_name).join("py.typed"),
        SourceFile::empty(),
    );

    files.insert(
        "pyproject.toml",
        generate_pyproject_toml(metadata, &package_name)?,
    );

    files.insert("MANIFEST.in", generate_manifest(package, &package_name)?);

    Ok(files)
}

#[derive(Debug, serde::Serialize)]
struct Context {
    commands: Vec<CommandContext>,
    libraries: Vec<LibraryContext>,
}

impl Context {
    fn for_package(pkg: &Package) -> Self {
        let commands = pkg
            .commands()
            .iter()
            .cloned()
            .map(CommandContext::from)
            .collect();

        let libraries = pkg
            .libraries()
            .iter()
            .cloned()
            .map(LibraryContext::from)
            .collect();

        Context {
            commands,
            libraries,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct LibraryContext {
    ident: String,
    class_name: String,
    module_filename: String,
    wasi: bool,
    #[serde(skip)]
    exports: Interface,
    #[serde(skip)]
    module: Module,
}

impl From<crate::Library> for LibraryContext {
    fn from(lib: crate::Library) -> Self {
        let module_filename = Path::new(lib.module_filename()).with_extension("wasm");
        let ident = lib.interface_name().to_snake_case();
        let class_name = lib.class_name();

        LibraryContext {
            ident,
            class_name,
            module_filename: module_filename.display().to_string(),
            wasi: lib.requires_wasi(),
            exports: lib.exports,
            module: lib.module,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct CommandContext {
    ident: String,
    module_filename: String,
    #[serde(skip)]
    wasm: Vec<u8>,
}

impl From<crate::Command> for CommandContext {
    fn from(cmd: crate::Command) -> CommandContext {
        let ident = cmd.name.replace('-', "_");
        let module_filename = format!("{ident}.wasm");
        CommandContext {
            ident,
            module_filename,
            wasm: cmd.wasm,
        }
    }
}

fn command_bindings(ctx: &Context) -> Result<Files, Error> {
    let mut files = Files::new();

    for cmd in &ctx.commands {
        files.insert(&cmd.module_filename, SourceFile::from(&cmd.wasm));
    }

    files.insert(
        "__init__.py",
        TEMPLATES
            .get_template("commands.__init__.py")
            .unwrap()
            .render(&ctx)?
            .into(),
    );

    Ok(files)
}

fn library_bindings(ctx: &Context) -> Result<Files, Error> {
    let mut files = Files::new();

    for lib in &ctx.libraries {
        let mut bindings = generate_bindings(&lib.exports.0);
        bindings.insert(&lib.module_filename, lib.module.wasm.clone().into());
        files.insert_child_directory(&lib.ident, bindings);
    }

    let dunder_init = TEMPLATES
        .get_template("bindings.__init__.py")
        .unwrap()
        .render(&ctx)?;
    files.insert("__init__.py", dunder_init.into());

    Ok(files)
}

fn generate_manifest(package: &Package, package_name: &str) -> Result<SourceFile, Error> {
    let ctx = minijinja::context! {
        package_name,
        libraries => package.libraries()
        .iter()
            .map(|lib| lib.interface_name())
            .collect::<Vec<_>>(),
        commands => package.commands()
            .iter()
            .map(|cmd| cmd.name.as_str())
            .collect::<Vec<_>>(),
    };
    let rendered = TEMPLATES
        .get_template("MANIFEST.in")
        .unwrap()
        .render(&ctx)?;

    Ok(rendered.into())
}

fn generate_pyproject_toml(metadata: &Metadata, package_name: &str) -> Result<SourceFile, Error> {
    let Metadata {
        version,
        description,
        ..
    } = metadata;

    let project = PyProject {
        project: Project {
            name: package_name,
            version,
            description: description.as_deref(),
            readme: None,
            keywords: Vec::new(),
            dependencies: vec!["wasmer", "wasmer_compiler_cranelift"],
        },
        build_system: BuildSystem {
            requires: &["setuptools", "setuptools-scm"],
            build_backend: "setuptools.build_meta",
        },
    };

    let serialized = toml::to_string(&project)?;

    Ok(serialized.into())
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
struct PyProject<'a> {
    project: Project<'a>,
    build_system: BuildSystem<'a>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
struct BuildSystem<'a> {
    requires: &'a [&'a str],
    build_backend: &'a str,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
struct Project<'a> {
    name: &'a str,
    version: &'a str,
    description: Option<&'a str>,
    readme: Option<&'a Path>,
    keywords: Vec<&'a str>,
    dependencies: Vec<&'a str>,
}

fn top_level_dunder_init(package: &Package) -> Result<SourceFile, Error> {
    let Metadata {
        version,
        description,
        package_name,
    } = package.metadata();

    let ctx = minijinja::context! {
        version,
        description,
        generator => crate::GENERATOR,
        package_name => package_name.to_string(),
        ident => package_name.name().to_pascal_case(),
        commands => !package.commands().is_empty(),
        libraries => !package.libraries().is_empty(),
    };

    let rendered = TEMPLATES
        .get_template("top_level.__init__.py")
        .unwrap()
        .render(ctx)?;

    Ok(rendered.into())
}

fn generate_bindings(interface: &wai_parser::Interface) -> Files {
    let imports = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wai_bindgen_gen_core::Files::default();

    WasmerPy::default().generate_all(imports, exports, &mut generated);

    let mut files = Files::from(generated);
    files.insert("__init__.py", "from .bindings import *".into());

    files
}

#[cfg(test)]
mod tests {
    use insta::Settings;

    use super::*;
    use crate::{Command, Library, Module};
    use std::collections::BTreeSet;

    const WASMER_PACK_EXPORTS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../wasm/wasmer-pack.exports.wai"
    ));

    #[test]
    fn generated_files() {
        let expected: BTreeSet<&Path> = [
            "MANIFEST.in",
            "pyproject.toml",
            "wasmer_pack/__init__.py",
            "wasmer_pack/py.typed",
            "wasmer_pack/commands/__init__.py",
            "wasmer_pack/commands/first.wasm",
            "wasmer_pack/commands/second_with_dashes.wasm",
            "wasmer_pack/bindings/__init__.py",
            "wasmer_pack/bindings/wasmer_pack/__init__.py",
            "wasmer_pack/bindings/wasmer_pack/bindings.py",
            "wasmer_pack/bindings/wasmer_pack/wasmer_pack_wasm.wasm",
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
        let package = Package::new(metadata, libraries, commands);

        let files = generate_python(&package).unwrap();

        let actual_files: BTreeSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);

        let mut settings = Settings::clone_current();
        settings.add_filter(
            r"Generated by wasmer-pack v\d+\.\d+\.\d+(-\w+(\.\d+)?)?",
            "Generated by XXX",
        );
        settings.bind(|| {
            insta::assert_display_snapshot!(files["pyproject.toml"].utf8_contents().unwrap());
            insta::assert_display_snapshot!(files["MANIFEST.in"].utf8_contents().unwrap());
            insta::assert_display_snapshot!(files["wasmer_pack/__init__.py"]
                .utf8_contents()
                .unwrap()
                .replace(crate::GENERATOR, "XXX"));
            insta::assert_display_snapshot!(files["wasmer_pack/bindings/__init__.py"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["wasmer_pack/commands/__init__.py"]
                .utf8_contents()
                .unwrap());
        });
        insta::assert_display_snapshot!(files["wasmer_pack/py.typed"].utf8_contents().unwrap());

        let actual_files: BTreeSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);
    }
}
