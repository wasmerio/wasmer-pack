use std::path::Path;

use anyhow::Error;
use heck::ToPascalCase;
use minijinja::Environment;
use once_cell::sync::Lazy;
use wit_bindgen_gen_core::Generator;
use wit_bindgen_gen_wasmer_py::WasmerPy;

use crate::{Files, Interface, Metadata, Module, SourceFile};

static TEMPLATES: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_template("__init__.py", include_str!("__init__.py.j2"))
        .unwrap();
    env.add_template("MANIFEST.in", include_str!("MANIFEST.in.j2"))
        .unwrap();

    env
});

/// Generate Python bindings.
pub fn generate_python(
    metadata: &Metadata,
    module: &Module,
    interface: &Interface,
) -> Result<Files, Error> {
    let package_name = metadata.package_name.python_name();
    let interface_name = interface.0.name.as_str();

    let mut files = Files::new();

    generate_bindings(&interface.0, &package_name, &mut files);

    files.insert(
        Path::new(&package_name)
            .join(&module.name)
            .with_extension("wasm"),
        SourceFile::from(&module.wasm),
    );

    files.insert(
        Path::new(&package_name).join("__init__.py"),
        dunder_init_file(metadata, module.abi, &module.name, interface_name)?,
    );

    files.insert(
        Path::new("pyproject.toml"),
        generate_pyproject_toml(metadata, &package_name)?,
    );

    files.insert(Path::new("MANIFEST.in"), generate_manifest()?);

    Ok(files)
}

fn generate_manifest() -> Result<SourceFile, Error> {
    let rendered = TEMPLATES.get_template("MANIFEST.in").unwrap().render(())?;

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

fn dunder_init_file(
    metadata: &Metadata,
    abi: crate::Abi,
    module_name: &str,
    interface_name: &str,
) -> Result<SourceFile, Error> {
    let Metadata {
        version,
        description,
        package_name,
    } = metadata;
    let ctx = minijinja::context! {
        interface_name,
        module_name,
        version,
        description => description
        .clone()
        .unwrap_or_else(|| format!("Bindings to {package_name}.")),
        class_name => interface_name.to_pascal_case(),
        wasi => matches!(abi, crate::Abi::Wasi),
    };

    let rendered = TEMPLATES.get_template("__init__.py").unwrap().render(ctx)?;

    Ok(rendered.into())
}

fn generate_bindings(interface: &wit_parser::Interface, package_name: &str, files: &mut Files) {
    let imports = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wit_bindgen_gen_core::Files::default();

    WasmerPy::default().generate_all(imports, exports, &mut generated);

    for (path, file) in generated.iter() {
        let path = Path::new(package_name).join(path.replace('-', "_"));
        files.insert(path, SourceFile::from(file));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn generated_files() {
        let expected: HashSet<&Path> = [
            "MANIFEST.in",
            "pyproject.toml",
            "wit_pack/__init__.py",
            "wit_pack/bindings.py",
            "wit_pack/wit_pack_wasm.wasm",
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

        let files = generate_python(&metadata, &module, &interface).unwrap();

        insta::assert_display_snapshot!(files["pyproject.toml"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["MANIFEST.in"].utf8_contents().unwrap());
        insta::assert_display_snapshot!(files["wit_pack/__init__.py"].utf8_contents().unwrap());

        let actual_files: HashSet<_> = files.iter().map(|(p, _)| p).collect();
        assert_eq!(actual_files, expected);
    }
}
