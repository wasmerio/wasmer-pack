#!/bin/env python3

from pathlib import Path
from typing import Tuple, Union
from wabt.wabt import load, WasmFeature, Ok


def main():
    wabt = load()

    # wat2wasm test
    wasm_result = wabt.wat2wasm("(module)", WasmFeature.MUTABLE_GLOBALS)
    assert isinstance(wasm_result, Ok)
    assert wasm_result.value == bytearray(
        b"\x00asm\x01\x00\x00\x00\x00\x08\x04name\x02\x01\x00\x00\t\x07linking\x02"
    )

    # wasm2wat test
    wat_result = wabt.wasm2wat(
        bytearray(
            [
                0,
                97,
                115,
                109,
                1,
                0,
                0,
                0,
                0,
                8,
                4,
                110,
                97,
                109,
                101,
                2,
                1,
                0,
                0,
                9,
                7,
                108,
                105,
                110,
                107,
                105,
                110,
                103,
                2,
            ]
        ),
        WasmFeature.MUTABLE_GLOBALS,
    )
    assert isinstance(wat_result, Ok)
    assert wat_result.value == "(module)\n"


if __name__ == "__main__":
    main()
