import load, { WitPack, Exports, Metadata, Module, Result, Error as WitPackError, File } from "wit-pack";
import fs from "fs/promises";
import path from "path";

async function main() {
    // First, we need to initialize the WebAssembly module.
    const witPack = await load();

    // If we want to use wit-pack to generate some bindings for itself (how meta!)
    // we need to load the corresponding *.wasm and *.wit files.

    const projectRoot = path.resolve(".", "../../../..");

    const metadata = Metadata.new(witPack, "wit-pack", "0.0.0");
    const module = await loadWasmModule(witPack, projectRoot);
    const wit = path.join(projectRoot, "crates", "wasm", "wit-pack.exports.wit");
    const witFile = await fs.readFile(wit, {encoding: "utf8"});
    const exports = unwrap(Exports.fromWit(witPack, path.basename(wit), witFile));

    // Now we can generate the JavaScript bindings
    const result = witPack.generateJavascript({exports, module, metadata});
    const files: File[] = unwrap(result);

    // We should have been given a list of the generated files
    console.log(files.map(f => f.filename));

    // let's find the package.json
    const packageJsonFile = files.find(f => f.filename == "package.json");
    if (!packageJsonFile) {
        throw new Error("Unable to find package.json in the list of files");
    }

    // All files come from wit-pack as bytes, with text files being encoded as UTF8.
    const contents = new TextDecoder("utf8").decode(packageJsonFile.contents);

    const generatedPackageJson = JSON.parse(contents);

    // Make sure we generated something with the right name.
    if (generatedPackageJson.name != "wit-pack") {
        throw new Error('We should have generated a package called "wit-pack"');
    }
}

async function loadWasmModule(witPack: WitPack, projectRoot: string) {
    const witPackWasm = path.join(projectRoot, "target", "wasm32-unknown-unknown", "debug", "wit_pack_wasm.wasm");
    const wasm = await fs.readFile(witPackWasm);
    const module = Module.new(witPack, path.parse(witPackWasm).name, "none", wasm);
    return module;
}

function unwrap<T>(result: Result<T, WitPackError>): T {
    if (result.tag == "err") {
        const {verbose} = result.val;
        throw new Error(verbose);
    }

    return result.val;
}

main();
