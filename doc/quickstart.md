# Wasmer Pack Quickstart Guide

This guide walks you through creating, publishing, and using a simple library
with Wasmer Pack, WAI, and WAPM.

## Creating the Library

Create a new Rust project and add the wai-bindgen-rust crate as a dependency:

```console
$ cargo new --lib quickstart
$ cd quickstart
$ cargo add wai-bindgen-rust
```

Create a WAI file to define the calculator interface:

```wai
// calculator.wai

/// Add two numbers.
add: func(a: float32, b: float32) -> float32
```

Implement the interface in Rust:

```rust
// src/lib.rs

wai_bindgen_rust::export!("calculator.wai");
struct Calculator;

impl crate::calculator::Calculator for Calculator {
    fn add(a: f32, b: f32) -> f32 { a + b }
}
```

## Publishing the Library

Install wapm using [the Wasmer installer][installer]:

```console
$ curl https://get.wasmer.io -sSfL | sh
```

Next, install the [`cargo wapm`][cargo-wapm] helper:

```console
$ cargo install cargo-wapm
```

Log in to your [WAPM account][sign-up]:

```console
$ wapm login
```

Update `Cargo.toml` to configure the package for WAPM publication:

```toml
# Cargo.toml

[package.metadata.wapm]
namespace = "<YOUR_USERNAME>"
abi = "none"
bindings = { wai-version = "0.2.0", exports = "calculator.wai" }

[lib]
crate-type = ["cdylib", "rlib"]
```

Publish the package:

```console
$ cargo wapm
```

## Using the Library

### JavaScript

Create a new JavaScript project and add the calculator package:

```console
$ yarn init --yes
$ wasmer add --yarn wai/tutorial-01
```

Import and use the package in your JavaScript code:

```js
// index.js

import { bindings } from "wai/tutorial-01";

async function main() {
    const calculator = await bindings.calculator();
    console.log("2 + 2 =", calculator.add(2, 2));
}

main();
```

### Python

Add the calculator package to your Python project:

```console
$ wasmer add --pip wai/tutorial-01
```

Import and use the package in your Python code:

```py
# main.py

from tutorial_01 import bindings

calculator = bindings.calculator()
print("2+2 = ", calculator.add(2.0, 2.0))
```

## Conclusion

Congratulations, you have successfully completed the Wasmer Pack Quickstart
Guide! In this tutorial, you learned how to:

- Create a simple calculator library using Rust and WAI.
- Implement the interface using Rust and wai-bindgen-rust.
- Publish the library to WAPM.
- Use the library in JavaScript and Python projects.

Now that you have a basic understanding of how to create, publish, and use a
Wasmer Pack library, you can explore more advanced topics and features.

Here are some suggestions for further exploration:

- Learn about more advanced WAI features, such as error handling and custom
  types.
- Discover how to optimize your WebAssembly modules for performance and size.
- Explore [the Wasmer ecosystem][wapm] and learn about other Wasmer tools and
  libraries.
- Create and publish more complex libraries, experimenting with different use
  cases and applications.

For more tutorials, guides, and resources, visit [the Wasmer Pack
documentation][docs] and the [Wasmer Pack GitHub repository][repo]. You can also
join [the Wasmer community][slack] to ask questions, share your projects, and
connect with other developers.

Good luck with your future projects, and happy coding!

[cargo-wapm]: https://github.com/wasmerio/cargo-wapm
[docs]: https://wasmerio.github.io/wasmer-pack/user-docs
[installer]: https://docs.wasmer.io/ecosystem/wapm/getting-started
[repo]: https://github.com/wasmerio/wasmer-pack
[sign-up]: https://wapm.io/signup
[slack]: https://slack.wasmer.io/
[wapm]: https://wapm.io/
