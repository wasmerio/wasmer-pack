import { bindings } from "@michael-f-bryan/hello-wasi";

async function main() {
    const wasm = await bindings.hello_wasi();
    wasm.printHelloWasi();
    console.log("Done!");
}

main();
