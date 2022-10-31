#!/bin/env python3

from pathlib import Path
from typing import Union
from wasmer_pack import bindings
from wasmer_pack.bindings.wasmer_pack import (
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
    WasmerPack,
)


project_root = Path(__file__).parents[4]


def test_load_library():
    wasmer_pack = bindings.wasmer_pack()

    assert callable(wasmer_pack.generate_javascript)
    assert callable(wasmer_pack.generate_python)


def unwrap(value: Union[Ok[T], Err[Error]]) -> T:
    if isinstance(value, Err):
        raise Exception(value.value.verbose)
    else:
        return value.value


def load_bindings(wasmer_pack: WasmerPack) -> Package:
    metadata = unwrap(Metadata.new(wasmer_pack, "wasmer/wasmer-pack", "1.2.3"))

    exports_wit = project_root.joinpath("crates", "wasm", "wasmer-pack.exports.wit")
    name = str(exports_wit)
    contents = exports_wit.read_text()
    exports = unwrap(Interface.from_wit(wasmer_pack, name, contents))

    wasmer_pack_wasm = project_root.joinpath(
        "target", "wasm32-unknown-unknown", "debug", "wasmer_pack_wasm.wasm"
    )
    libraries = [
        Library(exports, Abi.NONE, wasmer_pack_wasm.read_bytes()),
    ]
    # Note: we need to provide a dummy command because of a bug in wit-bindgen
    commands = [Command("dummy", b"")]

    return Package(metadata, libraries, commands)


def test_generate_bindings_for_wasmer_pack():
    wasmer_pack = bindings.wasmer_pack()
    pkg = load_bindings(wasmer_pack)

    try:
        files = unwrap(wasmer_pack.generate_python(pkg))

        expected = {
            "MANIFEST.in",
            "pyproject.toml",
            "wasmer_pack/__init__.py",
            "wasmer_pack/py.typed",
            "wasmer_pack/commands/__init__.py",
            "wasmer_pack/commands/dummy.wasm",
            "wasmer_pack/bindings/__init__.py",
            "wasmer_pack/bindings/wasmer_pack/__init__.py",
            "wasmer_pack/bindings/wasmer_pack/bindings.py",
            "wasmer_pack/bindings/wasmer_pack/wasmer-pack.wasm",
        }
        filenames = {f.filename for f in files}
        print("Expected", expected)
        print("Actual", filenames)

        assert filenames == expected

    finally:
        pkg.metadata.drop()
        for lib in pkg.libraries:
            lib.interface.drop()
