# WebAssembly Interfaces (WAI)

The WebAssembly spec that was first released in 2017 was only a minimum viable
product and deliberately left several features incomplete to be iterated on by
the ecosystem.

Arguably the most important functionality gap is the fact that only WebAssembly
primitives can be passed between the host and guest. That means imports and
exports can only use the following data types,

- `i32` - signed 32-bit integers
- `i64` - signed 64-bit integers
- `f32` - a 32-bit float
- `f64` - a 64-bit float (often called a `double`)
- `funcref` - a reference to a WebAssembly function
- `externref` - a reference to some opaque object owned by the WebAssembly
  virtual machine

You'll notice this list doesn't even include strings or boolean values!

The WebAssembly Interfaces project (WAI for short) provides a polyfill for
passing around higher-level objects. It lets developers define their imports and
exports in a `*.wai` file, then uses `wai-bindgen` to generate glue which
automagically passes things around within the constraints of WebAssembly.

There are four main parts to WAI:

- The `*.wai` file
- The WAI Bindgen code generator
- The guest
- The host

The [Wasmer Pack](../../) project provides a convenient way to consume
WebAssembly packages which implement a WAI interface.

Some useful links:

- [The `wasmerio/wai` repository](https://github.com/wasmerio/wai)
- [The `*.wai` format](https://github.com/wasmerio/wai/blob/main/WAI.md)
- [The `wai-bindgen` CLI on crates.io](https://crates.io/crates/wai-bindgen-cli)

## The `*.wai` File

WAI uses a file (or files) with the `*.wai` extension to define the host-guest
interface for an application that uses WebAssembly.

The items in a `*.wai` file map closely to concepts shared by most programming
languages. It has [types](./types.md), interfaces
(["Resources"](./resources.md)), structs (["Records"](./records.md)),
[functions](./functions.md), [enums](./variants.md), and so on.

The precise syntax [is defined in the WAI repository][wai-format] and a parser,
[`wai-parser`][wai-parser], is available as a Rust crate.

## The Guest

In an application using WebAssembly, the "guest" is the code that has been
compiled to WebAssembly and is being loaded into a WebAssembly virtual machine.

## The Host

In an application using WebAssembly, the "host" is the code that uses a
WebAssembly virtual machine (like [`wasmer`][wasmer]) to load a guest and use
functionality it provides.

The WebAssembly spec refers to the host in some places as
[*the embedder*][embedder].

## WAI Bindgen

The WAI Bindgen code generator consumes `*.wai` files and generates glue code
that [the host](#the-host) can use for using functionality from a WebAssembly
module or [the guest](#the-guest) can use for implementing that functionality.

There are two primary ways users will interact with WAI Bindgen, the
`wai-bindgen` CLI, and the `wai-bindgen-*` family of crates.

The `wai-bindgen` CLI provides a command-line interface to the `wai-bindgen-*`
crates, and is often used for once-off investigation or integration with a
non-Rust build system.

On the other hand, the `wai-bindgen-*` crates allow users to generate bindings
programmatically and give them much more control over the generation process.

| Language   | Direction       | Code Generator                                   | Procedural Macro                        |
| ---------- | --------------- | ------------------------------------------------ | --------------------------------------- |
| Rust       | Guest           | [`wai-bindgen-gen-rust-wasm`][gen-rust-wasm]     | [`wai-bindgen-rust`][proc-rust]         |
| Rust       | Host (Wasmer)   | [`wai-bindgen-gen-wasmer`][gen-rust-wasmer]      | [`wai-bindgen-wasmer`][proc-wasmer]     |
| Rust       | Host (Wasmtime) | [`wai-bindgen-gen-wasmtime`][gen-rust-wasmtime]  | [`wai-bindgen-wasmtime`][proc-wasmtime] |
| C          | Guest           | [`wai-bindgen-gen-c`][gen-c-wasm]                |                                         |
| JavaScript | Host            | [`wai-bindgen-gen-js`][gen-js-host]              |                                         |
| Python     | Host (Wasmer)   | [`wai-bindgen-gen-wasmer-py`][gen-py-host]       |                                         |
| Python     | Host (Wasmtime) | [`wai-bindgen-gen-wasmtime-py`][gen-py-wasmtime] |                                         |

[gen-c-wasm]: https://docs.rs/wai-bindgen-gen-c/
[gen-js-host]: https://docs.rs/wai-bindgen-gen-js/
[gen-py-host]: https://docs.rs/wai-bindgen-gen-wasmer-py/
[gen-py-wasmtime]: https://docs.rs/wai-bindgen-gen-wasmtime-py/
[gen-rust-wasm]: https://docs.rs/wai-bindgen-gen-rust-wasm/
[gen-rust-wasmer]: https://docs.rs/wai-bindgen-gen-wasmer/
[gen-rust-wasmtime]: https://docs.rs/wai-bindgen-gen-wasmtime/
[proc-rust]: https://docs.rs/wai-bindgen-rust/
[proc-wasmer]: https://docs.rs/wai-bindgen-wasmer/
[proc-wasmtime]: https://docs.rs/wai-bindgen-wasmtime/

[embedder]: https://webassembly.github.io/spec/core/intro/overview.html#embedder
[idl]: https://en.wikipedia.org/wiki/Interface_description_language
[wai-format]: https://github.com/wasmerio/wai/blob/main/WAI.md
[wai-parser]: https://docs.rs/wai-parser
[wasmer]: https://docs.rs/wasmer
