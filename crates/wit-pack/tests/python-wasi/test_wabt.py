#!/bin/env python3

from pathlib import Path
from wasmer import wasi
import pytest
from wabt import Wabt
from wabt.bindings.wabt import WasmFeature, Ok


def test_two_modules_were_generated():
    wabt = Wabt()

    assert callable(wabt.bindings.wabt)
    assert callable(wabt.bindings.wabt2)


def test_generated_library():
    wabt = Wabt()
    instance = wabt.bindings.wabt()

    wasm_result = instance.wat2wasm("(module)", WasmFeature.MUTABLE_GLOBALS)
    assert isinstance(wasm_result, Ok)
    assert wasm_result.value == bytearray(
        b"\x00asm\x01\x00\x00\x00\x00\x08\x04name\x02\x01\x00\x00\t\x07linking\x02"
    )


def test_generated_commands_exist():
    wabt = Wabt()

    assert callable(wabt.commands.wasm_interp)
    assert callable(wabt.commands.wasm_strip)
    assert callable(wabt.commands.wasm_validate)
    assert callable(wabt.commands.wasm2wat)
    assert callable(wabt.commands.wast2json)
    assert callable(wabt.commands.wat2wasm)


def test_invoke_wat2wasm_executable(tmp_path: Path):
    wabt = Wabt()
    env = (
        wasi.StateBuilder("wat2wasm")
        .argument("./input.wat")
        .argument("--output=./output.wasm")
        .map_directory(".", str(tmp_path))
        .finalize()
    )
    tmp_path.joinpath("input.wat").write_text("(module)")

    exit_status = wabt.commands.wat2wasm(env)

    assert exit_status.code == 0
    generated = tmp_path.joinpath("output.wasm")
    assert generated.exists()
