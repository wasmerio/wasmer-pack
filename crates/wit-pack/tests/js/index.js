import { load } from "wit-pack";
import fs from "fs/promises";
import path from "path";

// First, we need to initialize the WebAssembly module.
const witPack = await load();

// If we want to use wit-pack to generate some bindings for itself (how meta!)
// we need to load the corresponding *.wasm and *.wit files.

const projectRoot = path.resolve(".", "../../../..");

const witPackWasm = path.join(projectRoot, "target", "wasm32-unknown-unknown", "debug", "wit_pack_wasm.wasm");
const wasm = await fs.readFile(witPackWasm);

const exportsWit = path.join(projectRoot, "crates", "wasm", "exports.wit");
const wit = await fs.readFile(exportsWit, { encoding: "utf-8" });

// Now we can load the bindings
const loadResult = witPack.loadBindings("wit-pack", wit, wasm);

// handle any parsing errors
if (loadResult.err) {
    const { message } = loadResult.err.val;
    throw new Error(message);
}

const bindings = loadResult.val;

// We want to generate JavaScript bindings
const jsResult = bindings.generateJavascript();

// ...which may fail
if (jsResult.err) {
    const { message } = jsResult.err.val;
    throw new Error(message);
}

// We now have a list of the generated files
const files = jsResult.val;
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
