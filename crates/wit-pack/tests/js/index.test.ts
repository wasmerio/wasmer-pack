import {bindings} from "@wasmer/wit-pack";
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
        const witPack = await bindings.wit_pack();
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
            "package/package.json",
            "package/src/bindings/index.d.ts",
            "package/src/bindings/index.js",
            "package/src/bindings/wit-pack/intrinsics.js",
            "package/src/bindings/wit-pack/wit-pack.d.ts",
            "package/src/bindings/wit-pack/wit-pack.js",
            "package/src/bindings/wit-pack/wit-pack.wasm",
            "package/src/commands/dummy_cmd.d.ts",
            "package/src/commands/dummy_cmd.js",
            "package/src/commands/dummy_cmd.wasm",
            "package/src/index.d.ts",
            "package/src/index.js",
        ]);
        const packageJsonFile = files.find(
            (f) => f.filename == "package/package.json"
        );
        if (!packageJsonFile) {
            throw new Error("No package.json found");
        }

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
