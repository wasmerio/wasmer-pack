## Project Architecture

The `wit-pack` project is split across several crates depending on the various
ways it might be used.

The main crates are:

- [`crates/wit-pack`][wit-pack] - this is the meat and potatoes of `wit-pack`.
  It contains all the code for generating bindings to WebAssembly modules, plus
  templates for any glue code that will be needed along the way
- [`crates/cli`][cli] - this is a CLI tool that lets `wit-pack` generate
  bindings using the commands and libraries inside a [Pirita][pirita] file
- [`crates/wasm`][wasm] - this is a wrapper that makes `wit-pack` available as a
  WebAssembly module. The functions and data types that are exposed are defined
  in [`crates/wasm/wit-pack.exports.wit`][exports] (see [`WIT.md`][wit] for the
  syntax)


[cli]: https://github.com/wasmerio/wit-pack/tree/master/crates/cli
[exports]: https://github.com/wasmerio/wit-pack/tree/master/crates/wasm/wit-pack.exports.wit
[pirita]: https://github.com/wasmerio/pirita
[wasm]: https://github.com/wasmerio/wit-pack/tree/master/crates/wasm
[wit-pack]: https://github.com/wasmerio/wit-pack/tree/master/crates/wit-pack
[wit]: https://github.com/wasmerio/wit-bindgen/blob/c04723063c7a5a7389660ca97f85ffd9bc9ef0b8/WIT.md
