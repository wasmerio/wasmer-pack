import load from "wabt/src/wabt/index.js";
import loadPackage from "wabt";
import {WASM_FEATURE_SIMD, WASM_FEATURE_BULK_MEMORY } from "wabt/src/wabt/wabt.js";
// @ts-ignore
import { WASI } from "@wasmer/wasi";

async function main() {
    const wabt = await load();

    const result = wabt.wat2wasm("(module)", WASM_FEATURE_SIMD | WASM_FEATURE_BULK_MEMORY);
    if (result.tag != "ok") {
        throw new Error("The result is not ok");
    }

    const pkg = loadPackage();
    const wasi = new WASI({});
    // FIXME: Figure out why WABT binaries aren't valid WASI executables
    // const {code} = await pkg.commands.wasm2wat({wasi});
    // if (code != 42) {
    //     throw new Error(`Expected the command to finish with exit code 42, found ${code}`);
    // }
}

main();
