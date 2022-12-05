# Wasmer Pack

[![Continuous integration](https://github.com/wasmerio/wasmer-pack/workflows/Continuous%20Integration/badge.svg?branch=master)](https://github.com/wasmerio/wasmer-pack/actions)

([API Docs][api-docs] | [Project Docs][user-docs])

Import your WebAssembly code just like any other dependency.

## Getting Started

The easiest way to get started by installing with the `wasmer-pack` CLI.

```console
$ cargo install wasmer-pack-cli
$ wasmer-pack --version
wasmer-pack-cli 0.5.2
```

We also need the WAPM package we are generating bindings for. One option is to
create your own, but for convenience we'll use the `wasmer/wasmer-pack-cli`
package from WAPM.

```console
$ curl -sSO https://registry-cdn.wapm.io/packages/wasmer/wasmer-pack-cli/wasmer-pack-cli-0.5.2.tar.gz
$ tar -xzvf wasmer-pack-cli-0.5.2.tar.gz
$ tree .
.
├── wapm.toml
├── wasmer-pack-cli-0.5.2.tar.gz
└── wasmer-pack.wasm

0 directories, 2 files
```

Now we've got everything we need to generate Python bindings to the
`wasmer/wasmer-pack-cli` package.

```console
$ wasmer-pack python . --out-dir ./py
$ tree py
py
├── MANIFEST.in
├── pyproject.toml
└── wasmer_pack_cli
    ├── commands
    │   ├── __init__.py
    │   └── wasmer_pack.wasm
    ├── __init__.py
    └── py.typed

2 directories, 6 files
```

We can generate JavaScript bindings with a similar command

```console
$ wasmer-pack js . --out-dir ./js
$ tree ./js
./js
└── package
    ├── package.json
    └── src
        ├── commands
        │   ├── wasmer-pack.d.ts
        │   ├── wasmer-pack.js
        │   └── wasmer-pack.wasm
        ├── index.d.ts
        └── index.js

3 directories, 6 files
```

Check out [the tutorial][tutorial] for more.

## License

This project is licensed under the MIT license ([LICENSE-MIT](./LICENSE-MIT.md)
or <http://opensource.org/licenses/MIT>).

It is recommended to always use [`cargo crev`][crev] to verify the
trustworthiness of each of your dependencies, including this one.

[api-docs]: https://wasmerio.github.io/wasmer-pack/api-docs
[user-docs]: https://wasmerio.github.io/wasmer-pack/user-docs
[crev]: https://github.com/crev-dev/cargo-crev
[tutorial]: https://wasmerio.github.io/wasmer-pack/user-docs/tutorial/01-hello-world.html
