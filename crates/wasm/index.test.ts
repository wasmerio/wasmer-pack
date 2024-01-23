import {bindings} from "@wasmer/wasmer-pack";
import {
    Result,
    Error as WasmerPackError,
    File,
    Package,
    BindingsOptions,
} from "@wasmer/wasmer-pack/src/bindings/wasmer-pack/wasmer-pack.js";

const WASMER_PACK_WEBC_FILE = "https://cdn.wasmer.io/webcimages/371a21a5a632442570f2d0ffe0125713ab8947b8b1596708e1fcee32be8cf2b7.webc";


describe("wasmer-pack bindings", () => {
    it("is self-hosting", async () => {
        const webc = await fetch(WASMER_PACK_WEBC_FILE)
            .then(r => {
                if (!r.ok) {
                    throw new Error(`${r.status}: ${r.statusText}`);
                }
                return r.arrayBuffer();
            })
            .then(blob => new Uint8Array(blob));
        const wasmerPack = await bindings.wasmer_pack();

        // Now we can generate the JavaScript bindings
        const pkg = unwrap(Package.fromWebc(wasmerPack, webc));
        const files: File[] = unwrap(pkg.generateJavascript());

        const generatedFiles = files.map((f) => f.filename).sort();
        expect(generatedFiles).toEqual([
            "package/package.json",
            "package/src/bindings/index.d.ts",
            "package/src/bindings/index.js",
            "package/src/bindings/wasmer-pack/intrinsics.js",
            "package/src/bindings/wasmer-pack/wasmer-pack.d.ts",
            "package/src/bindings/wasmer-pack/wasmer-pack.js",
            "package/src/bindings/wasmer-pack/wasmer-pack.wasm",
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
                name: "@wasmer/wasmer-pack",
            })
        );
    });
});

function unwrap<T>(result: Result<T, WasmerPackError>): T {
    if (result.tag == "err") {
        const { verbose } = result.val;
        throw new Error(verbose);
    }

    return result.val;
}
