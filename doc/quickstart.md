# Quickstart Guide

To get a taste of Wasmer Pack, let's build the simplest possible library - a
calculator for adding two numbers.

## Creating the Library

First, create a new Rust project and add the `wai-bindgen-rust` crate as a
dependency.

```console
$ cargo new --lib quickstart
$ cd quickstart
$ cargo add wai-bindgen-rust
```

Now, let's define a WAI file that lets us add two floating point numbers.

```wai
// calculator.wai

/// Add two numbers.
add: func(a: float32, b: float32) -> float32
```

We can use the `wai_bindgen_rust::export!()` macro to "export" this interface
(i.e. make it available to some host application).

```rust
// src/lib.rs

// generate the WAI glue code under the `calculator` module
wai_bindgen_rust::export!("calculator.wai");

// Create a type to attach our functionalty to
struct Calculator;

// Implement the trait generated by our glue code
impl crate::calculator::Calculator for Calculator {
	fn add(a: f32, b: f32) -> f32 { a + b }
}
```

## Let's Publish It!

Publishing requires installing `wapm` with [the Wasmer installer][installer] and
the [`cargo wapm`][cargo-wapm] helper installed (`cargo install cargo-wapm`). If
you haven't already, make sure to run `wapm` login to log into your WAPM account
(don't forget to [sign up][sign-up] if you haven't already).

Now we're set up, we'll need to update `Cargo.toml` so this package can be
published to WAPM.

```toml
# Cargo.toml
[package]
name = "quickstart"
version = "0.1.0"
description = "A simple calculator"

[package.metadata.wapm]
namespace = "<YOUR_USERNAME>"  # The namespace to publish it to
abi = "none" # How to compile the crate. "none" is "wasm32-unknown-unknown"
bindings = { wai-version = "0.2.0", exports = "calculator.wai" }
```

We also need to tell the Rust compiler to generate a `cdylib` ("C-compatible
dynamic library"). It's also a good idea to add the `rlib` crate type so
integration tests can import the library as a Rust dependency.

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]
```

And publish!

```console
$ cargo wapm
```

> **Note:** WAI also works with other languages, such as C or C++. If you want to
> publish packages to WAPM with it, you will need to write a `wapm.toml` by
> hand and use `wapm publish` instead of `cargo wapm`.

## Consuming WAPM Packages in your codebase

To see how packages can be used in the real world, let's add
[`wai/tutorial-01`][tutorial-01] to a JavaScript project.

First, we'll need to create a new JavaScript package and add the
`wai/tutorial-01` package as a dependency.

```console
$ yarn init --yes
$ wasmer add --yarn wai/tutorial-01
```

This runs `yarn` add under the hood. Depending on the project, you might use
the `--npm` flag to do `npm install` or `--pip` for `pip install`.

Now, let's create a script.

```js
// index.js

import { bindings } from "wai/tutorial-01";

async function main() {
	const calculator = await bindings.calculator();
	console.log("2 + 2 =", calculator.add(2, 2));
}

main();
```

Or, if you want to do it in python:

```console
$ wasmer add --pip wai/tutorial-01
```

The python code is a bit simpler because we don't need to worry about `async`.

```py
# main.py
from tutorial_01 import bindings

calculator = bindings.calculator()
print("2+2 = ", calculator.add(2.0, 2.0))
```


[cargo-wapm]: https://github.com/wasmerio/cargo-wapm
[installer]: https://docs.wasmer.io/ecosystem/wapm/getting-started
[sign-up]: https://wapm.io/signup
[tutorial-01]: https://wapm.io/wai/tutorial-01
