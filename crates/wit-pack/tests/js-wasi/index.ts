import load from "wabt";
import {WASM_FEATURE_SIMD, WASM_FEATURE_BULK_MEMORY } from "wabt/src/wabt/wabt.js";

async function main() {
    let wabt = await load();

    const result = wabt.wat2wasm("(module)", WASM_FEATURE_SIMD | WASM_FEATURE_BULK_MEMORY);
    if (result.tag != "ok") {
        throw new Error("The result is not ok");
    }
}

main();
