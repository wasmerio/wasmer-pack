import {bindings} from "@wasmer/wasmer-pack";
import {
    Interface,
    Metadata,
    Result,
    Error as WasmerPackError,
    File,
    Package,
} from "@wasmer/wasmer-pack/src/bindings/wasmer-pack/wasmer-pack.js";
import fs from "fs/promises";
import path from "path";

describe("wasmer-pack bindings", () => {
    it("is self-hosting", async () => {
        const wasmerPack = await bindings.wasmer_pack();
        // If we want to use wasmer-pack to generate some bindings for itself (how
        // meta!) we need to load the corresponding *.wasm and *.wit files.
        const projectRoot = path.resolve(".", "../../../..");
        const wit = path.join(
            projectRoot,
            "crates",
            "wasm",
            "wasmer-pack.exports.wai"
        );
        const witFile = await fs.readFile(wit, { encoding: "utf8" });
        const pkg: Package = {
            metadata: unwrap(Metadata.new(wasmerPack, "wasmer/wasmer-pack", "0.0.0")),
            libraries: [
                {
                    exports: unwrap(
                        Interface.fromWit(wasmerPack, path.basename(wit), witFile)
                    ),
                    imports: [unwrap(Interface.fromWit(wasmerPack, "browser.wit", "hello-world: func()"))],
                    wasm: await loadWasmModule(projectRoot),
                    abi: "none",
                },
            ],
            commands: [{ name: "dummy_cmd", wasm: new Uint8Array() }],
        };

        // Now we can generate the JavaScript bindings
        const result = wasmerPack.generateJavascript(pkg);
        const files: File[] = unwrap(result);

        const generatedFiles = files.map((f) => f.filename).sort();
        expect(generatedFiles).toEqual([
            "package/package.json",
            "package/src/bindings/index.d.ts",
            "package/src/bindings/index.js",
            "package/src/bindings/wasmer-pack/browser.d.ts",
            "package/src/bindings/wasmer-pack/browser.js",
            "package/src/bindings/wasmer-pack/intrinsics.js",
            "package/src/bindings/wasmer-pack/wasmer-pack.d.ts",
            "package/src/bindings/wasmer-pack/wasmer-pack.js",
            "package/src/bindings/wasmer-pack/wasmer-pack.wasm",
            "package/src/commands/dummy_cmd.d.ts",
            "package/src/commands/dummy_cmd.js",
            "package/src/commands/dummy_cmd.wasm",
            "package/src/index.d.ts",
            "package/src/index.js",
        ]);
        const packageJsonFile = files.find(
            (f) => f.filename == "package/package.json"
        )!;

        const packageJson = new TextDecoder("utf8").decode(
            packageJsonFile.contents
        );

        expect(JSON.parse(packageJson)).toEqual(
            jasmine.objectContaining({
                name: "@wasmer-package/wasmer__wasmer-pack",
            })
        );
    });
});

async function loadWasmModule(projectRoot: string) {
    const wasmerPackWasm = path.join(
        projectRoot,
        "target",
        "wasm32-unknown-unknown",
        "debug",
        "wasmer_pack_wasm.wasm"
    );
    return await fs.readFile(wasmerPackWasm);
}

function unwrap<T>(result: Result<T, WasmerPackError>): T {
    if (result.tag == "err") {
        const { verbose } = result.val;
        throw new Error(verbose);
    }

    return result.val;
}
