#!/bin/env python3

from pathlib import Path
from typing import Union
import requests
from wasmer_pack import bindings
from wasmer_pack.bindings.wasmer_pack import (
    BindingsOptions,
    Err,
    Error,
    Ok,
    Package,
    T,
)


WASMER_PACK_WEBC_FILE = "https://cdn.wasmer.io/webcimages/371a21a5a632442570f2d0ffe0125713ab8947b8b1596708e1fcee32be8cf2b7.webc"
project_root = Path(__file__).parents[4]


def unwrap(value: Union[Ok[T], Err[Error]]) -> T:
    if isinstance(value, Err):
        raise Exception(value.value.verbose)
    else:
        return value.value


def test_generate_bindings_for_wasmer_pack():
    wasmer_pack = bindings.wasmer_pack()

    response = requests.get(WASMER_PACK_WEBC_FILE)
    response.raise_for_status()
    webc = response.content

    pkg = unwrap(Package.from_webc(wasmer_pack, webc))

    try:
        files = unwrap(pkg.generate_python(options=BindingsOptions(name=None)))

        expected = {
            'MANIFEST.in',
            'pyproject.toml',
            'wasmer_pack/__init__.py',
            'wasmer_pack/bindings/__init__.py',
            'wasmer_pack/bindings/wasmer_pack/__init__.py',
            'wasmer_pack/bindings/wasmer_pack/bindings.py',
            'wasmer_pack/bindings/wasmer_pack/wasmer-pack-wasm.wasm',
            'wasmer_pack/py.typed',
        }
        filenames = {f.filename for f in files}
        print("Expected", expected)
        print("Actual", filenames)

        assert filenames == expected

    finally:
        pkg.drop()
