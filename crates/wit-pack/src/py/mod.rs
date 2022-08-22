use std::path::Path;

use anyhow::Error;
use heck::ToPascalCase;
use wit_bindgen_gen_core::Generator;
use wit_bindgen_gen_wasmer_py::WasmerPy;

use crate::{Files, Interface, Metadata, Module, SourceFile};

/// Generate Python bindings.
pub fn generate_python(
    metadata: &Metadata,
    module: &Module,
    interface: &Interface,
) -> Result<Files, Error> {
    let package_name = python_package_name(&metadata.package_name);
    let interface_name = interface.0.name.as_str();

    let mut files = Files::new();

    files.push(
        Path::new(&package_name)
            .join(&module.name)
            .with_extension("wasm"),
        SourceFile::from(&module.wasm),
    );

    generate_bindings(&interface.0, &package_name, &mut files);

    let dunder_init: SourceFile = dunder_init_file(metadata, &module.name, interface_name);
    files.push(Path::new(&package_name).join("__init__.py"), dunder_init);

    let setup_py = generate_setup_py(metadata, &package_name);
    files.push(Path::new("setup.py"), setup_py);

    Ok(files)
}

fn generate_setup_py(metadata: &Metadata, package_name: &str) -> SourceFile {
    "".into()
}

fn python_package_name(raw_package_name: &str) -> String {
    raw_package_name
        .split('/')
        .last()
        .expect("Split always returns at least 1 item")
        .replace("-", "_")
}

fn dunder_init_file(metadata: &Metadata, module_name: &str, interface_name: &str) -> SourceFile {
    let Metadata {
        version,
        description,
        package_name,
    } = metadata;
    let class_name = interface_name.to_pascal_case();

    let description = description
        .clone()
        .unwrap_or_else(|| format!("Bindings to {package_name}."));

    let src = format!(
        r#"
'''
{description}
'''

__version__ = "{version}"

import wasmer as _wasmer
import pathlib as _pathlib

from .bindings import *

store = _wasmer.Store()
wasm = _pathlib.Path(__file__).parent.joinpath("{module_name}.wasm").read_bytes()
module = _wasmer.Module(store, wasm)


def load() -> {class_name}:
    imports = {{}}
    return {class_name}(store, imports, module)
"#
    );

    SourceFile::from(src)
}

fn generate_bindings(interface: &wit_parser::Interface, package_name: &str, files: &mut Files) {
    let imports = std::slice::from_ref(interface);
    let exports = &[];
    let mut generated = wit_bindgen_gen_core::Files::default();

    WasmerPy::default().generate_all(imports, exports, &mut generated);

    for (path, file) in generated.iter() {
        let path = Path::new(package_name).join(path.replace("-", "_"));
        files.push(path, SourceFile::from(file));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

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

        let files = generate_python(&metadata, &module, &interface).unwrap();

        let file_names: HashSet<&Path> = files.iter().map(|(path, _)| path).collect();
        assert_eq!(file_names, expected);
    }
}
