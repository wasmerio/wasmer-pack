use std::path::Path;

use anyhow::Error;
use heck::{ToPascalCase, ToSnakeCase};
use minijinja::Environment;
use once_cell::sync::Lazy;
use wit_bindgen_gen_core::Generator;
use wit_bindgen_gen_wasmer_py::WasmerPy;

use crate::{
    types::{Library, Package},
    Command, Files, Metadata, SourceFile,
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

    let libraries = package.libraries();
    let commands = package.commands();

    if !libraries.is_empty() {
        files.insert_child_directory(
            Path::new(&package_name).join("bindings"),
            library_bindings(libraries)?,
        );
    }

    if !commands.is_empty() {
        files.insert_child_directory(
            Path::new(&package_name).join("commands"),
            command_bindings(package.commands())?,
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

fn command_bindings(commands: &[Command]) -> Result<Files, Error> {
    let mut files = Files::new();
    let mut cmds = Vec::new();

    for cmd in commands {
        let ident = cmd.name.replace('-', "_");
        let module_filename = Path::new(&ident).with_extension("wasm");

        files.insert(&module_filename, SourceFile::from(&cmd.wasm));
        cmds.push(minijinja::context! {ident, module_filename});
    }

    let ctx = minijinja::context! {
        commands => cmds,
    };

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

fn library_bindings(libraries: &[Library]) -> Result<Files, Error> {
    let mut files = Files::new();
    let mut ctx = LibrariesContext::default();

    for lib in libraries {
        let module_filename = Path::new(lib.module_filename()).with_extension("wasm");
        let ident = lib.interface_name().to_snake_case();
        let class_name = lib.class_name();

        let mut bindings = generate_bindings(&lib.interface.0);
        bindings.insert(&module_filename, lib.module.wasm.clone().into());
        files.insert_child_directory(&ident, bindings);

        ctx.libraries.push(LibraryContext {
            ident,
            class_name,
            module_filename: module_filename.display().to_string(),
            wasi: lib.requires_wasi(),
        });
    }

    let dunder_init = TEMPLATES
        .get_template("bindings.__init__.py")
        .unwrap()
        .render(&ctx)?;
    files.insert("__init__.py", dunder_init.into());

    Ok(files)
}

#[derive(Debug, Default, serde::Serialize)]
struct LibrariesContext {
    libraries: Vec<LibraryContext>,
}

#[derive(Debug, Default, serde::Serialize)]
struct LibraryContext {
    ident: String,
    class_name: String,
    module_filename: String,
    wasi: bool,
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
    };

    let serialized = toml::to_string(&project)?;

    Ok(serialized.into())
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
struct PyProject<'a> {
    project: Project<'a>,
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

fn generate_bindings(interface: &wit_parser::Interface) -> Files {
    let imports = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wit_bindgen_gen_core::Files::default();

    WasmerPy::default().generate_all(imports, exports, &mut generated);

    let mut files = Files::from(generated);
    files.insert("__init__.py", "from .bindings import *".into());

    files
}

#[cfg(test)]
mod tests {
    use insta::Settings;

    use super::*;
    use crate::Module;
    use std::collections::BTreeSet;

    #[test]
    fn generated_files() {
        let expected: BTreeSet<&Path> = [
            "MANIFEST.in",
            "pyproject.toml",
            "wit_pack/__init__.py",
            "wit_pack/py.typed",
            "wit_pack/commands/__init__.py",
            "wit_pack/commands/first.wasm",
            "wit_pack/commands/second_with_dashes.wasm",
            "wit_pack/bindings/__init__.py",
            "wit_pack/bindings/wit_pack/__init__.py",
            "wit_pack/bindings/wit_pack/bindings.py",
            "wit_pack/bindings/wit_pack/wit_pack_wasm.wasm",
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
        let commands = vec![
            Command {
                name: "first".into(),
                wasm: Vec::new(),
            },
            Command {
                name: "second-with-dashes".into(),
                wasm: Vec::new(),
            },
        ];
        let libraries = vec![Library { interface, module }];
        let package = Package::new(metadata, libraries, commands);

        let files = generate_python(&package).unwrap();

        let actual_files: BTreeSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);

        let mut settings = Settings::clone_current();
        settings.add_filter(r"Generated by wit-pack v\d+\.\d+\.\d+", "Generated by XXX");
        settings.bind(|| {
            insta::assert_display_snapshot!(files["pyproject.toml"].utf8_contents().unwrap());
            insta::assert_display_snapshot!(files["MANIFEST.in"].utf8_contents().unwrap());
            insta::assert_display_snapshot!(files["wit_pack/__init__.py"]
                .utf8_contents()
                .unwrap()
                .replace(crate::GENERATOR, "XXX"));
            insta::assert_display_snapshot!(files["wit_pack/bindings/__init__.py"]
                .utf8_contents()
                .unwrap());
            insta::assert_display_snapshot!(files["wit_pack/commands/__init__.py"]
                .utf8_contents()
                .unwrap());
        });

        let actual_files: BTreeSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);
    }
}
