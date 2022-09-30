#!/bin/env python3

from pathlib import Path
from wasmer import wasi
import pytest
from wabt import bindings, commands
from wabt.bindings.wabt import WasmFeature, Ok


def test_two_modules_were_generated():
    assert callable(bindings.wabt)
    assert callable(bindings.wabt2)


def test_generated_library():
    instance = bindings.wabt()

    wasm_result = instance.wat2wasm("(module)", WasmFeature.MUTABLE_GLOBALS)
    assert isinstance(wasm_result, Ok)
    assert wasm_result.value == bytearray(
        b"\x00asm\x01\x00\x00\x00\x00\x08\x04name\x02\x01\x00\x00\t\x07linking\x02"
    )


def test_generated_commands_exist():
    assert callable(commands.wasm_interp)
    assert callable(commands.wasm_strip)
    assert callable(commands.wasm_validate)
    assert callable(commands.wasm2wat)
    assert callable(commands.wast2json)
    assert callable(commands.wat2wasm)


def test_invoke_wat2wasm_executable(tmp_path: Path):
    env = (
        wasi.StateBuilder("wat2wasm")
        .argument("./input.wat")
        .argument("--output=./output.wasm")
        .map_directory(".", str(tmp_path))
        .finalize()
    )
    tmp_path.joinpath("input.wat").write_text("(module)")

    exit_status = commands.wat2wasm(env)

    assert exit_status.code == 0
    generated = tmp_path.joinpath("output.wasm")
    assert generated.exists()
