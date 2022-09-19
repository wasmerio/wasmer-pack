import loadWabt from "wabt/src/wabt/index.js";
import loadPackage from "wabt";
import {WASM_FEATURE_SIMD, WASM_FEATURE_BULK_MEMORY } from "wabt/src/wabt/wabt.js";
// @ts-ignore - @wasmer/wasi's package.json doesn't seem to define a typings file?
import { WASI } from "@wasmer/wasi";

describe("Generated WASI bindings", () => {
    it("can use the wabt library", async () => {
        const wabt = await loadWabt();

        const result = wabt.wat2wasm("(module)", WASM_FEATURE_SIMD | WASM_FEATURE_BULK_MEMORY);

        expect(result.tag).toBe("ok");
    });

    it("can invoke the wat2wasm executable", async () => {
        const pkg = loadPackage();
        const wasi = new WASI();

        const exitStatus = await pkg.commands.wat2wasm({wasi});

        expect(exitStatus).toMatchObject({code: 42});
    });
});
