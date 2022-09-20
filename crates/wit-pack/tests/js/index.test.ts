import load from "@wasmer/wit-pack/src/wit-pack/index.js";
import {
    WitPack,
    Interface,
    Metadata,
    Module,
    Result,
    Error as WitPackError,
    File,
} from "@wasmer/wit-pack/src/wit-pack/wit-pack.js";
import fs from "fs/promises";
import path from "path";

describe("wit-pack bindings", () => {
    it("is self-hosting", async () => {
        const witPack = (await load()) as WitPack;
        // If we want to use wit-pack to generate some bindings for itself (how
        // meta!) we need to load the corresponding *.wasm and *.wit files.
        const projectRoot = path.resolve(".", "../../../..");
        const wit = path.join(
            projectRoot,
            "crates",
            "wasm",
            "wit-pack.exports.wit"
        );
        const witFile = await fs.readFile(wit, { encoding: "utf8" });
        const pkg = {
            metadata: Metadata.new(witPack, "wasmer/wit-pack", "0.0.0"),
            libraries: [
                {
                    interface: unwrap(
                        Interface.fromWit(witPack, path.basename(wit), witFile)
                    ),
                    module: await loadWasmModule(witPack, projectRoot),
                },
            ],
        };

        // Now we can generate the JavaScript bindings
        const result = witPack.generateJavascript(pkg);
        const files: File[] = unwrap(result);

        expect(files.map((f) => f.filename)).toEqual([
            "package.json",
            "src/index.d.ts",
            "src/index.js",
            "src/wit-pack/index.d.ts",
            "src/wit-pack/index.js",
            "src/wit-pack/intrinsics.js",
            "src/wit-pack/wit-pack.d.ts",
            "src/wit-pack/wit-pack.js",
            "src/wit-pack/wit_pack_wasm",
        ]);
        const packageJsonFile = files.find(
            (f) => f.filename == "package.json"
        )!;

        const packageJson = new TextDecoder("utf8").decode(
            packageJsonFile.contents
        );

        expect(JSON.parse(packageJson)).toEqual(
            jasmine.objectContaining({
                name: "@wasmer/wit-pack",
            })
        );
    });
});

async function loadWasmModule(witPack: WitPack, projectRoot: string) {
    const witPackWasm = path.join(
        projectRoot,
        "target",
        "wasm32-unknown-unknown",
        "debug",
        "wit_pack_wasm.wasm"
    );
    const wasm = await fs.readFile(witPackWasm);
    const module = Module.new(
        witPack,
        path.parse(witPackWasm).name,
        "none",
        wasm
    );
    return module;
}

function unwrap<T>(result: Result<T, WitPackError>): T {
    if (result.tag == "err") {
        const { verbose } = result.val;
        throw new Error(verbose);
    }

    return result.val;
}
