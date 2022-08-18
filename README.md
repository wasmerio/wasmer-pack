# WIT Pack

[![Continuous integration](https://github.com/wasmerio/wit-pack/workflows/Continuous%20Integration/badge.svg?branch=master)](https://github.com/wasmerio/wit-pack/actions)

([API Docs][api-docs])

> **TODO:** Write up an interesting description.

## Getting Started

The easiest way to get started is with the `wit-pack` CLI.

```console
$ cargo install --git https://github.com/wasmerio/wit-pack
$ wit-pack --version
```

This repository contains a `wapm.toml` which is used to publish the `wit-pack`
CLI and library to WAPM.

First, we need to compile the library to WebAssembly.

```console
$ cargo build --package=wit-pack-wasm --target=wasm32-unknown-unknown
$ ls target/wasm32-unknown-unknown/debug/*.wasm
target/wasm32-unknown-unknown/debug/wit_pack_wasm.wasm
```

Now we can generate a JavaScript package to use this WebAssembly.

```console
$ wit-pack --module=wit-pack --out-dir=wit-pack-js
$ tree wit-pack-js
wit-pack-js
├── package.json
└── src
    ├── exports.wasm
    ├── generated
    │   ├── exports.d.ts
    │   ├── exports.js
    │   └── intrinsics.js
    ├── index.d.ts
    └── index.js

2 directories, 7 files
```

You can now use the package just like any other JavaScript dependency.

```ts
import { load } from "./wit-pack-js";
import fs from "fs/promises";

const wasm = await fs.readFile("foo.wasm");
const wit = await fs.readFile("interface.wit");
const result = witPack.parse("myLibrary", wit, wasm);

if (!result.ok) {
    throw new Error(result.err.message);
}

const files = result.val;
files.forEach(console.log);
```

## License

This project is licensed under the MIT license ([LICENSE-MIT](./LICENSE-MIT.md)
or <http://opensource.org/licenses/MIT>).

It is recommended to always use [`cargo crev`][crev] to verify the
trustworthiness of each of your dependencies, including this one.

[api-docs]: https://wasmerio.github.io/wit-pack
[crev]: https://github.com/crev-dev/cargo-crev
