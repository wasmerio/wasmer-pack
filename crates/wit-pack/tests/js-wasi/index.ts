import load, { Wabt } from "wabt";

async function main() {
    let wabt = await load();
    const result = wabt.wat2wasm("(module)", 0);
    if (result.tag != "ok") {
        throw new Error("The result is not ok");
    }
}

main();
