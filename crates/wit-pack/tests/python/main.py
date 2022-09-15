#!/bin/env python3

from pathlib import Path
from typing import Tuple, Union
from wit_pack.wit_pack import (
    load,
    Abi,
    Err,
    Error,
    Interface,
    Library,
    Metadata,
    Module,
    Ok,
    Package,
    T,
    WitPack,
)


project_root = Path(__file__).parents[4]


def unwrap(value: Union[Ok[T], Err[Error]]) -> T:
    if isinstance(value, Err):
        raise Exception(value.value.verbose)
    else:
        return value.value


def load_bindings(wit_pack: WitPack) -> Package:
    metadata = Metadata.new(wit_pack, "wasmer/wit-pack", "1.2.3")

    exports_wit = project_root.joinpath("crates", "wasm", "wit-pack.exports.wit")
    name = str(exports_wit)
    contents = exports_wit.read_text()
    exports = unwrap(Interface.from_wit(wit_pack, name, contents))

    wit_pack_wasm = project_root.joinpath(
        "target", "wasm32-unknown-unknown", "debug", "wit_pack_wasm.wasm"
    )
    module = Module.new(
        wit_pack, wit_pack_wasm.name, Abi.NONE, wit_pack_wasm.read_bytes()
    )
    libraries = [
        Library(exports, module),
    ]

    return Package(metadata, libraries)


def main():
    wit_pack = load()
    pkg = load_bindings(wit_pack)

    files = unwrap(wit_pack.generate_python(pkg))

    expected = {
        "pyproject.toml",
        "wit_pack/wit_pack/wit_pack_wasm.wasm",
        "wit_pack/wit_pack/__init__.py",
        "wit_pack/__init__.py",
        "wit_pack/wit_pack/bindings.py",
        "MANIFEST.in",
    }
    filenames = {f.filename for f in files}
    print("Expected", expected)
    print("Actual", filenames)

    assert filenames == expected

    # Note: wit-bindgen's glue code forces us to manually free things
    # pkg.drop()


if __name__ == "__main__":
    main()
