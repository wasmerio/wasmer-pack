use std::path::Path;

use anyhow::Error;
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
        "library.__init__.py",
        include_str!("library.__init__.py.j2"),
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

    for library in package.libraries() {
        files.insert_child_directory(
            Path::new(&package_name).join(library.interface_name().replace('-', "_")),
            library_bindings(library)?,
        );
    }

    if !package.commands().is_empty() {
        files.insert_child_directory(
            Path::new(&package_name).join("commands"),
            command_bindings(package.commands())?,
        );
    }

    files.insert(
        Path::new(&package_name).join("__init__.py"),
        top_level_dunder_init(metadata)?,
    );

    files.insert(
        Path::new("pyproject.toml"),
        generate_pyproject_toml(metadata, &package_name)?,
    );

    files.insert(
        Path::new("MANIFEST.in"),
        generate_manifest(package, &package_name)?,
    );

    Ok(files)
}

fn command_bindings(commands: &[Command]) -> Result<Files, Error> {
    let mut files = Files::new();
    let mut cmds = Vec::new();

    for cmd in commands {
        let ident = cmd.name.replace('-', "_");
        let module_filename = format!("{}.wasm", cmd.name);

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

fn library_bindings(library: &Library) -> Result<Files, Error> {
    let mut files = generate_bindings(&library.interface.0);

    files.insert(
        library.module_filename(),
        library.module.wasm.clone().into(),
    );

    files.insert("__init__.py", library_dunder_init(library)?);

    files.insert(
        library.module_filename(),
        SourceFile::from(&library.module.wasm),
    );

    Ok(files)
}

fn library_dunder_init(library: &Library) -> Result<SourceFile, Error> {
    let ctx = minijinja::context! {
        wasi => library.requires_wasi(),
        interface_name => library.interface_name(),
        module_filename => library.module_filename(),
        class_name => library.class_name(),
    };

    let rendered = TEMPLATES
        .get_template("library.__init__.py")
        .unwrap()
        .render(ctx)?;

    Ok(rendered.into())
}

fn generate_manifest(package: &Package, package_name: &str) -> Result<SourceFile, Error> {
    let ctx = minijinja::context! {
        package_name,
        libraries => package.libraries()
            .map(|lib| lib.interface_name())
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

fn top_level_dunder_init(metadata: &Metadata) -> Result<SourceFile, Error> {
    let Metadata {
        version,
        description,
        package_name,
    } = metadata;

    let ctx = minijinja::context! {
        version,
        description,
        package_name => package_name.to_string(),
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

    generated.into()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::Module;

    use super::*;

    #[test]
    fn generated_files() {
        let expected: HashSet<&Path> = [
            "MANIFEST.in",
            "pyproject.toml",
            "wit_pack/__init__.py",
            "wit_pack/commands/__init__.py",
            "wit_pack/commands/first.wasm",
            "wit_pack/commands/second-with-dashes.wasm",
            "wit_pack/wit_pack/__init__.py",
            "wit_pack/wit_pack/bindings.py",
            "wit_pack/wit_pack/wit_pack_wasm.wasm",
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
        let package = Package::new(metadata, vec![Library { interface, module }], commands);

        let files = generate_python(&package).unwrap();

        insta::assert_display_snapshot!(files["pyproject.toml"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["MANIFEST.in"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["wit_pack/__init__.py"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["wit_pack/wit_pack/__init__.py"]
            .utf8_contents()
            .unwrap());
        insta::assert_display_snapshot!(files["wit_pack/commands/__init__.py"]
            .utf8_contents()
            .unwrap());

        let actual_files: HashSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);
    }
}
