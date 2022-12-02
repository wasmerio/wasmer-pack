# Wasmer Pack

The Wasmer Pack project is a code generator that takes in a WebAssembly library
and the [WAI][wai] files defining its interface, and generates packages for
using it natively in your favourite programming language.

Useful links:
- [The Wasmer Pack repository on GitHub](https://github.com/wasmerio/wasmer-pack)
- [This user guide](https://wasmerio.github.io/wasmer-pack/user-docs)
- [The WebAssembly Package Manager](https://wapm.io/)
- [The `*.wai` format](https://github.com/wasmerio/wai/blob/main/WAI.md)

## Installation

The WAPM backend automatically runs Wasmer Pack over any packages that are
published to the registry, so most users won't need to interact with it
directly.

That said, the `wasmer-pack` CLI is available on crates.io for those wanting to
run it locally (e.g. during testing).

```console
$ cargo install wasmer-pack-cli
```

The same CLI is [published to WAPM][cli-wapm] as a WASI executable, meaning
you can use `wasmer run` to automatically fetch and run the latest version.

```console
$ wasmer run wasmer/wasmer-pack-cli --dir=. -- --help
wasmer-pack-cli 0.5.2

USAGE:
    wasmer-pack.wasm <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help      Print this message or the help of the given subcommand(s)
    js        Generate bindings for use with NodeJS
    python    Generate Python bindings
    show      Show metadata for the bindings that would be generated from a Pirita file
```

> **NOTE:** the `--dir=.` flag is important! This tells the `wasmer` CLI to let
> `wasmer/wasmer-pack-cli` access the current directory.
>
> WebAssembly is sandboxed by default, so all file system access must be
> explicitly provided.

[wai]: ./explainers/ecosystem.md#wai-bindgen
[cli-wapm]: https://wapm.io/wasmer/wasmer-pack-cli
