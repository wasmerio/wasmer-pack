#!/bin/env python3

from pathlib import Path
from wabt.wabt import load as loadWabt, WasmFeature, Ok
import wabt
from wasmer import engine, wasi, Store, Module, ImportObject, Instance


def test_two_modules_were_generated():
    import wabt.wabt as _
    import wabt.wabt2 as _


def test_generated_library():
    wabt = loadWabt()

    # wat2wasm test
    wasm_result = wabt.wat2wasm("(module)", WasmFeature.MUTABLE_GLOBALS)
    assert isinstance(wasm_result, Ok)
    assert wasm_result.value == bytearray(
        b"\x00asm\x01\x00\x00\x00\x00\x08\x04name\x02\x01\x00\x00\t\x07linking\x02"
    )


def test_generated_commands_exist():
    pkg = wabt.load()

    assert callable(pkg.commands.wasm_interp)
    assert callable(pkg.commands.wasm_strip)
    assert callable(pkg.commands.wasm_validate)
    assert callable(pkg.commands.wasm2wat)
    assert callable(pkg.commands.wast2json)
    assert callable(pkg.commands.wat2wasm)


def test_invoke_wat2wasm_executable(tmp_path: Path):
    pkg = wabt.load()
    env = (
        wasi.StateBuilder("wat2wasm")
        .argument("./input.wat")
        .argument("--output=./output.wasm")
        .map_directory(".", str(tmp_path))
        .finalize()
    )
    tmp_path.joinpath("input.wat").write_text("(module)")

    exit_status = pkg.commands.wat2wasm(env)

    assert exit_status.code == 0
    generated = tmp_path.joinpath("output.wasm")
    assert generated.exists()
