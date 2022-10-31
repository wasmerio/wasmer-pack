## Project Architecture

The `wasmer-pack` project is split across several crates depending on the various
ways it might be used.

The main crates are:

- [`crates/wasmer-pack`][wasmer-pack] - this is the meat and potatoes of `wasmer-pack`.
  It contains all the code for generating bindings to WebAssembly modules, plus
  templates for any glue code that will be needed along the way
- [`crates/cli`][cli] - this is a CLI tool that lets `wasmer-pack` generate
  bindings using the commands and libraries inside a [Pirita][pirita] file
- [`crates/wasm`][wasm] - this is a wrapper that makes `wasmer-pack` available as a
  WebAssembly module. The functions and data types that are exposed are defined
  in [`crates/wasm/wasmer-pack.exports.wai`][exports] (see [`WIT.md`][wit] for the
  syntax)


[cli]: https://github.com/wasmerio/wasmer-pack/tree/master/crates/cli
[exports]: https://github.com/wasmerio/wasmer-pack/tree/master/crates/wasm/wasmer-pack.exports.wai
[pirita]: https://github.com/wasmerio/pirita
[wasm]: https://github.com/wasmerio/wasmer-pack/tree/master/crates/wasm
[wasmer-pack]: https://github.com/wasmerio/wasmer-pack/tree/master/crates/wasmer-pack
[wit]: https://github.com/wasmerio/wai/blob/c04723063c7a5a7389660ca97f85ffd9bc9ef0b8/WIT.md
