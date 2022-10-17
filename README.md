# WIT Pack

[![Continuous integration](https://github.com/wasmerio/wit-pack/workflows/Continuous%20Integration/badge.svg?branch=master)](https://github.com/wasmerio/wit-pack/actions)

([API Docs][api-docs] | [User Docs][user-docs])

Import your WebAssembly code just like any other dependency.

## Getting Started

The easiest way to get started by installing with the `wit-pack` CLI.

```console
$ cargo install --git https://github.com/wasmerio/wit-pack
$ wit-pack --version
wit-pack-cli 0.2.3
```

The `wit-pack` command accepts input in the form of [Pirita][pirita] containers,
so let's download a container we can generate bindings for.

```console
$ curl -O https://registry-cdn.wapm.dev/packages/wasmer/wit-pack/wit-pack-0.2.3.webc
```

Now we've got everything we need to generate Python bindings to the `wit-pack`
package.

```console
$ wit-pack python wit-pack-0.2.3.webc --out-dir ./py
$ tree ./py
./py
├── MANIFEST.in
├── pyproject.toml
└── wit_pack
    ├── bindings
    │   ├── __init__.py
    │   └── wit_pack
    │       ├── bindings.py
    │       ├── __init__.py
    │       └── wit-pack-wasm
    ├── commands
    │   ├── __init__.py
    │   └── wit-pack-wasm.wasm
    ├── __init__.py
    └── py.typed

4 directories, 10 files
```

We can generate JavaScript bindings with a similar command

```console
$ wit-pack js wit-pack-0.2.3.webc --out-dir ./js
$ tree ./js
./js
├── package.json
└── src
    ├── index.d.ts
    ├── index.js
    └── wit-pack
        ├── index.d.ts
        ├── index.js
        ├── intrinsics.js
        ├── wit-pack.d.ts
        ├── wit-pack.js
        └── wit-pack-wasm

2 directories, 9 files
```

## License

This project is licensed under the MIT license ([LICENSE-MIT](./LICENSE-MIT.md)
or <http://opensource.org/licenses/MIT>).

It is recommended to always use [`cargo crev`][crev] to verify the
trustworthiness of each of your dependencies, including this one.

[api-docs]: https://wasmerio.github.io/wit-pack/api-docs
[user-docs]: https://wasmerio.github.io/wit-pack/user-docs
[crev]: https://github.com/crev-dev/cargo-crev
[pirita]: https://github.com/wasmerio/pirita
