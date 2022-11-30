import { bindings, commands } from "wabt";
import {
    WASM_FEATURE_SIMD,
    WASM_FEATURE_BULK_MEMORY,
} from "wabt/src/bindings/wabt/wabt.js";
import {} from "jasmine";

describe("Generated WASI bindings", () => {
    it("can use the wabt library", async () => {
        const wabt = await bindings.wabt();

        const result = wabt.wat2wasm(
            "(module)",
            WASM_FEATURE_SIMD | WASM_FEATURE_BULK_MEMORY
        );

        expect(result.tag).toBe("ok");
    });

    it("can invoke the wat2wasm executable", async () => {
        const env = { args: ["wat2wasm", "--help"] };

        const { code, wasi } = await commands.wat2wasm({ wasi: env });

        expect(code).toEqual(123);

        const stdout = wasi.getStdoutString();
        expect(stdout).toContain("usage: wat2wasm [options] filename");
        expect(stdout).toContain(
            "read a file in the wasm text format, check it for errors, and"
        );
        const stderr = wasi.getStderrString();
        expect(stderr).toEqual("");
    });
});
