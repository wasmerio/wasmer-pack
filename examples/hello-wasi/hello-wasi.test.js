// import { bindings } from "@michael-f-bryan/hello-wasi";
const { bindings } = require("@michael-f-bryan/hello-wasi");

test("Basic Hello test", async () => {
  const wasm = await bindings.hello_wasi();
  wasm.printHelloWasi();
  expect(1 + 2).toBe(3);
});
