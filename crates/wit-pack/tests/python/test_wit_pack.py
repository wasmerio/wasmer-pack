#!/bin/env python3

from pathlib import Path
from typing import Union
from wit_pack import bindings
from wit_pack.bindings.wit_pack import (
    Abi,
    Command,
    Err,
    Error,
    Interface,
    Library,
    Metadata,
    Ok,
    Package,
    T,
    WitPack,
)


project_root = Path(__file__).parents[4]


def test_load_library():
    wit_pack = bindings.wit_pack()

    assert callable(wit_pack.generate_javascript)
    assert callable(wit_pack.generate_python)


def unwrap(value: Union[Ok[T], Err[Error]]) -> T:
    if isinstance(value, Err):
        raise Exception(value.value.verbose)
    else:
        return value.value


def load_bindings(wit_pack: WitPack) -> Package:
    metadata = unwrap(Metadata.new(wit_pack, "wasmer/wit-pack", "1.2.3"))

    exports_wit = project_root.joinpath("crates", "wasm", "wit-pack.exports.wit")
    name = str(exports_wit)
    contents = exports_wit.read_text()
    exports = unwrap(Interface.from_wit(wit_pack, name, contents))

    wit_pack_wasm = project_root.joinpath(
        "target", "wasm32-unknown-unknown", "debug", "wit_pack_wasm.wasm"
    )
    libraries = [
        Library(exports, Abi.NONE, wit_pack_wasm.read_bytes()),
    ]
    # Note: we need to provide a dummy command because of a bug in wit-bindgen
    commands = [Command("dummy", b"")]

    return Package(metadata, libraries, commands)


def test_generate_bindings_for_wit_pack():
    wit_pack = bindings.wit_pack()
    pkg = load_bindings(wit_pack)

    try:
        files = unwrap(wit_pack.generate_python(pkg))

        expected = {
            "MANIFEST.in",
            "pyproject.toml",
            "wit_pack/__init__.py",
            "wit_pack/py.typed",
            "wit_pack/commands/__init__.py",
            "wit_pack/commands/dummy.wasm",
            "wit_pack/bindings/__init__.py",
            "wit_pack/bindings/wit_pack/__init__.py",
            "wit_pack/bindings/wit_pack/bindings.py",
            "wit_pack/bindings/wit_pack/wit-pack.wasm",
        }
        filenames = {f.filename for f in files}
        print("Expected", expected)
        print("Actual", filenames)

        assert filenames == expected

    finally:
        pkg.metadata.drop()
        for lib in pkg.libraries:
            lib.interface.drop()
