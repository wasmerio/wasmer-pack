#!/bin/env python3

from pathlib import Path
from typing import Tuple, Union
from wit_pack import WitPack as WitPackPackage
from wit_pack.bindings.wit_pack import (
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


def test_load_library():
    pkg = WitPackPackage()

    wit_pack = pkg.bindings.wit_pack()

    assert callable(wit_pack.generate_javascript)
    assert callable(wit_pack.generate_python)


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


def test_generate_bindings_for_wit_pack():
    pkg = WitPackPackage()

    wit_pack = pkg.bindings.wit_pack()
    pkg = load_bindings(wit_pack)

    try:
        files = unwrap(wit_pack.generate_python(pkg))

        expected = {
            "MANIFEST.in",
            "pyproject.toml",
            "wit_pack/__init__.py",
            "wit_pack/bindings/__init__.py",
            "wit_pack/bindings/wit_pack/__init__.py",
            "wit_pack/bindings/wit_pack/bindings.py",
            "wit_pack/bindings/wit_pack/wit_pack_wasm.wasm",
        }
        filenames = {f.filename for f in files}
        print("Expected", expected)
        print("Actual", filenames)

        assert filenames == expected

    finally:
        pkg.metadata.drop()
        for lib in pkg.libraries:
            lib.interface.drop()
            lib.module.drop()
