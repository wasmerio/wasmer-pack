import load from "@wasmer/wit-pack";
import {
    Interface,
    Metadata,
    Result,
    Error as WitPackError,
    File,
    Package,
} from "@wasmer/wit-pack/src/bindings/wit-pack/wit-pack.js";
import fs from "fs/promises";
import path from "path";

describe("wit-pack bindings", () => {
    it("is self-hosting", async () => {
        const witPackPackage = load();
        const witPack = await witPackPackage.bindings.wit_pack();
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
        const pkg: Package = {
            metadata: unwrap(Metadata.new(witPack, "wasmer/wit-pack", "0.0.0")),
            libraries: [
                {
                    interface: unwrap(
                        Interface.fromWit(witPack, path.basename(wit), witFile)
                    ),
                    wasm: await loadWasmModule(projectRoot),
                    abi: "none",
                },
            ],
            commands: [{ name: "dummy_cmd", wasm: new Uint8Array() }],
        };

        // Now we can generate the JavaScript bindings
        const result = witPack.generateJavascript(pkg);
        const files: File[] = unwrap(result);

        const generatedFiles = files.map((f) => f.filename).sort();
        expect(generatedFiles).toEqual([
            "package.json",
            "src/bindings/index.d.ts",
            "src/bindings/index.js",
            "src/bindings/wit-pack/intrinsics.js",
            "src/bindings/wit-pack/wit-pack.d.ts",
            "src/bindings/wit-pack/wit-pack.js",
            "src/bindings/wit-pack/wit-pack.wasm",
            "src/commands/dummy_cmd.d.ts",
            "src/commands/dummy_cmd.js",
            "src/commands/dummy_cmd.wasm",
            "src/index.d.ts",
            "src/index.js",
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

async function loadWasmModule(projectRoot: string) {
    const witPackWasm = path.join(
        projectRoot,
        "target",
        "wasm32-unknown-unknown",
        "debug",
        "wit_pack_wasm.wasm"
    );
    return await fs.readFile(witPackWasm);
}

function unwrap<T>(result: Result<T, WitPackError>): T {
    if (result.tag == "err") {
        const { verbose } = result.val;
        throw new Error(verbose);
    }

    return result.val;
}
