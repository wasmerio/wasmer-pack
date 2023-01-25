#!/bin/env python3

from pathlib import Path
from typing import Union
import urllib.request
from wasmer_pack import bindings
from wasmer_pack.bindings.wasmer_pack import (
    Err,
    Error,
    Ok,
    Package,
    T,
)


WIT_PACK_TARBALL = "https://registry-cdn.wapm.dev/packages/wasmer/wit-pack/wit-pack-0.3.0-beta.tar.gz"
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


def test_generate_bindings_for_wasmer_pack():
    wasmer_pack = bindings.wasmer_pack()

    webc = []
    with urllib.request.urlopen(WIT_PACK_TARBALL) as f:
        webc.extend(f.read())

    pkg = unwrap(Package.from_webc(wasmer_pack, webc))

    try:
        files = unwrap(pkg.generate_python(wasmer_pack))

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
        pkg.drop()
