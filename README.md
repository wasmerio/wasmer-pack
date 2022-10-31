# Wasmer Pack

[![Continuous integration](https://github.com/wasmerio/wasmer_pack/workflows/Continuous%20Integration/badge.svg?branch=master)](https://github.com/wasmerio/wasmer_pack/actions)

([API Docs][api-docs] | [User Docs][user-docs])

Import your WebAssembly code just like any other dependency.

## Getting Started

The easiest way to get started by installing with the `wasmer-pack` CLI.

```console
$ cargo install --git https://github.com/wasmerio/wasmer_pack
$ wasmer-pack --version
wasmer-pack-cli 0.2.3
```

The `wasmer-pack` command accepts input in the form of [Pirita][pirita]
containers, so let's download a container we can generate bindings for.

```console
$ curl -O https://registry-cdn.wapm.dev/packages/wasmer/tutorial-01/tutorial-01-0.1.0.webc
```

Now we've got everything we need to generate Python bindings to the `wasmer-pack`
package.

```console
$ wasmer-pack python tutorial-01-0.1.0.webc --out-dir py
$ tree ./py
./py
├── MANIFEST.in
├── pyproject.toml
└── tutorial_01
    ├── bindings
    │   ├── hello_world
    │   │   ├── bindings.py
    │   │   ├── __init__.py
    │   │   └── tutorial-01.wasm
    │   └── __init__.py
    ├── __init__.py
    └── py.typed

3 directories, 8 files
```

We can generate JavaScript bindings with a similar command

```console
$ wasmer-pack js tutorial-01-0.1.0.webc --out-dir js
$ tree ./js
./js
└── package
    ├── package.json
    └── src
        ├── bindings
        │   ├── hello-world
        │   │   ├── hello-world.d.ts
        │   │   ├── hello-world.js
        │   │   ├── intrinsics.js
        │   │   └── tutorial-01.wasm
        │   ├── index.d.ts
        │   └── index.js
        ├── index.d.ts
        └── index.js

4 directories, 9 files
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
[pirita]: https://github.com/wasmerio/pirita
[tutorial]: https://wasmerio.github.io/wasmer-pack/user-docs/tutorial/01-hello-world.html
