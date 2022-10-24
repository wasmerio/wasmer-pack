# Hello, World!

Like all good tutorials, let's start with WIT Pack's equivalent of
*"Hello, World!"* - a library that adds two numbers together.

By the end, you should know how to define a simple WIT interface and implement
it in Rust. We will also publish the package to WAPM and use it from JavaScript.

You can check WAPM for the package we'll be building - it's called
[`Michael-F-Bryan/hello-world`][published].

## Installation

You will need to install several CLI tools.

- [The Rust toolchain](https://rustup.rs/) so we can compile Rust code
  (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- the `wasm32-unknown-unknown` target so Rust knows how to compile to
  WebAssembly (`rustup target add wasm32-unknown-unknown`)
- [The Wasmer runtime](https://docs.wasmer.io/ecosystem/wasmer/getting-started)
  so we can interact with WAPM (`curl https://get.wasmer.io -sSfL | sh`)
- [the `cargo wapm` sub-command][cargo-wapm] for publishing to WAPM
  (`cargo install cargo-wapm`)

Once you've installed those tools, you'll want to create a new account on
[wapm.io][wapm-io-signup] so we have somewhere to publish our code to.

Running the `wasmer login` command will let you authenticate your computer with
WAPM.

## The WIT File

We want to start off simple for now, so let's create a library that just adds
two 32-bit integers.

The syntax for a WIT file is quite similar to Rust.

```
// hello-world.wit

/// Add two numbers
add: func(a: u32, b: u32) -> u32
```

This defines a function called `add` which takes two `u32` parameters (32-bit
unsigned integers) called `a` and `b`, and returns a `u32`.

You can see that normal comments start with a `//` and doc-comments use `///`.
Here, we're using `// hello-world.wit` to indicate the text should be saved to
`hello-world.wit`.

One interesting constraint from the WIT format is that *all* names must be
written in kebab-case. This lets `wit-bindgen` convert the name into the casing
that is idiomatic for a particular language in a particular context.

For example, if our WIT file defined a `hello-world` function, it would be
accessible as `hello_world` in Python and Rust because they use snake_case for
function names, whereas in JavaScript it would be `helloWorld`.

## Writing Some Rust

Now we've got a WIT file, let's create a WebAssembly library implementing the
`hello-world.wit` interface.

First, we'll create a new Rust crate.

```console
$ cargo new --lib tutorial-01
```

You can remove all the code in `src/lib.rs` because we don't need the example
boilerplate.

```console
$ rm src/lib.rs
```

Now, we'll add `wit-bindgen` as a dependency. This will give us access to the
macros it uses for generating code.

```console
$ cd tutorial-01
$ cargo add --git https://github.com/wasmerio/wit-bindgen wit-bindgen-rust
```

Towards the top of your `src/lib.rs`, we want to tell `wit-bindgen` that this
crate *exports* our `hello-world.wit` file.

```rust
// src/lib.rs

wit_bindgen_rust::export!("hello-world.wit");
```

(note: `hello-world.wit` is relative to the crate's root - the folder
containing your `Cargo.toml` file)

Under the hood, this will generate a bunch of glue code which the WebAssembly
host will call. We can see this generated code using
[`cargo expand`][cargo-expand].

(You don't normally need to do this, but sometimes it's nice to understand what
is going on)

```console,should_fail
$ cargo expand
mod hello_world {
    #[export_name = "add"]
    unsafe extern "C" fn __wit_bindgen_hello_world_add(arg0: i32, arg1: i32) -> i32 {
        let result = <super::HelloWorld as HelloWorld>::add(arg0 as u32, arg1 as u32);
        wit_bindgen_rust::rt::as_i32(result)
    }
    pub trait HelloWorld {
        /// Add two numbers
        fn add(a: u32, b: u32) -> u32;
    }
}
```

There's a lot going on in that code, and most of it isn't relevant to you, but
there are a couple of things I'd like to point out:

1. A `hello_world` module was generated (the name comes from
   `hello-world.wit`)
2. A `HelloWorld` trait was defined with an `add()` method that matches `add()`
  from `hello-world.wit` (note: `HelloWorld` is `hello-world` in PascalCase)
3. The `__wit_bindgen_hello_world_add()` shim expects a `HelloWorld` type to
  be defined in the parent module (that's the `super::` bit), and that
  `super::HelloWorld` type must implement the `HelloWorld` trait

From assumption 3, we know that the generated code expects us to define a
`HelloWorld` type. We've only got 1 line of code at the moment, so it shouldn't
be surprising to see our code doesn't compile (yet).

```console,should_fail
$ cargo check
error[E0412]: cannot find type `HelloWorld` in module `super`
 --> src/lib.rs:1:1
  |
1 | wit_bindgen_rust::export!("hello-world.wit");
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in `super`
  |
```

We can fix that by defining a `HelloWorld` type in `lib.rs`. Adding two numbers
doesn't require any state, so we'll just use a unit struct.

```rust
pub struct HelloWorld;
```

Looking back at assumption 3, our code *still* shouldn't compile because we
haven't implemented the `HelloWorld` trait for our `HelloWorld` struct yet.

```console
$ cargo check
error[E0277]: the trait bound `HelloWorld: hello_world::HelloWorld` is not satisfied
 --> src/lib.rs:1:1
  |
1 | wit_bindgen_rust::export!("hello-world.wit");
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `hello_world::HelloWorld` is not implemented for `HelloWorld`
```

The fix is pretty trivial.

```rust
impl hello_world::HelloWorld for HelloWorld {
    fn add(a: u32, b: u32) -> u32 { a + b }
}
```

If the naming gets a bit confusing (that's a lot of variations on
`hello-world`!) try to think back to that output from `cargo expand`. The key
thing to remember is the `HelloWorld` type is defined at the root of our crate,
but the `HelloWorld` trait is inside a `hello_world` module.

Believe it or not, but we're done writing code for now. Your crate should now
compile 🙂

## Compiling To WebAssembly

At the moment, running `cargo build` will just compile our crate to a Rust
library that will work on your current machine (e.g. x86-64 Linux), so we'll
need to [*cross-compile*][cross-compile] our code to WebAssembly.

Rust makes this cross-compilation process fairly painless.

First, we need to install a version of the standard library that has already
been compiled to WebAssembly.

```console,ignore
$ rustup target add wasm32-unknown-unknown
```

We'll go into target triples a bit more when [discussing WASI][wasi], but
`wasm32-unknown-unknown` basically means we want generic 32-bit WebAssembly
where the OS is unknown (i.e. we know nothing about the underlying OS, so we
can't use it).

Next, we need to tell `rustc` that we want it to generate a `*.wasm` file.

By default, it will only generate a `rlib` (a (Rust library"), so we need to
update `Cargo.toml` so our crate's [`crate-type`][crate-type] includes a
`cdylib` (a "C-compatible dynamic library").

```toml
# Cargo.toml

[lib]
crate-type = ["cdylib", "rlib"]
```

Now, we should be able to compile our crate for `wasm32-unknown-unknown` and
see a `*.wasm` file.

```console
$ cargo build --target wasm32-unknown-unknown
$ file target/wasm32-unknown-unknown/debug/*.wasm
target/wasm32-unknown-unknown/debug/tutorial_01.wasm: WebAssembly (wasm) binary module version 0x1 (MVP)
```

The `wasmer` CLI also has an `inspect` command which can be useful for looking
at our `*.wasm` file.

```console
$ wasmer inspect target/wasm32-unknown-unknown/debug/tutorial_01.wasm
Exports:
  Functions:
    "add": [I32, I32] -> [I32]
```

You'll notice that, besides a bunch of other stuff, we're exporting an `add`
function that takes two `i32`s and returns an `i32`.

This matches the `__wit_bindgen_hello_world_add()` signature we saw earlier.

## Publishing to WAPM

Now we've got a WebAssembly binary that works, let's publish it to WAPM!

The core component in a WAPM package is the `wapm.toml` file. This acts as a
"manifest" which tells WAPM which modules are included in the package, and
important metadata like the project name, version number, and repository URL.

You can check out [the docs][publishing-docs] for a walkthrough of the full
process for packaging an arbitrary WebAssembly module.

However, while we *could* create this file ourselves, most of the information is
already available as part of our project's `Cargo.toml` file. The
[`cargo wapm`][cargo-wapm] sub-command lets us automate a lot of the fiddly
tasks like compiling the project to `wasm32-unknown-unknown`, collecting
metadata, copying binaries around, and so on.

To enable `cargo wapm`, we need to add some metadata to our `Cargo.toml`.

```toml
# Cargo.toml

[package]
...
description = "Add two numbers"
repository = "https://github.com/wasmerio/wit-pack"

[package.metadata.wapm]
namespace = "Michael-F-Bryan"  # Replace this with your WAPM username
abi = "none"
bindings = { wit-bindgen = "0.1.0", exports = "hello-world.wit" }
```

Something to note is that all packages on WAPM must have a `description` field.

Other than that, we use the [`[package.metadata]`][cargo-pkg-metadata] section
to tell `cargo wapm` a couple of things:

- which namespace we are publishing to (all WAPM packages are namespaced)
- The ABI being used (`none` corresponds to Rust's `wasm32-unknown-unknown`, and
  we'd write `wasi` if we were compiling to `wasm32-wasi`), and
- The location of our `hello-world.wit` exports, plus the version of
  `wit-bindgen` we used

Now we've updated our `Cargo.toml`, let's do a dry-run to make sure the package
builds.

```console
$ cargo wapm --dry-run
Successfully published package `Michael-F-Bryan/hello-world@0.1.0`
[INFO] Publish succeeded, but package was not published because it was run in dry-run mode
```

If we dig around the `target/wapm/` directory, we can see what `cargo wapm`
generated for us.

```console
$ tree target/wapm/tutorial-01
target/wapm/tutorial-01
├── tutorial_01.wasm
├── hello-world.wit
└── wapm.toml

0 directories, 3 files

$ cat target/wapm/tutorial-01/wapm.toml
[package]
name = "Michael-F-Bryan/tutorial-01"
version = "0.1.0"
description = "Add two numbers"
repository = "https://github.com/wasmerio/wit-pack"

[[module]]
name = "tutorial-01"
source = "tutorial_01.wasm"
abi = "none"

[module.bindings]
wit-exports = "hello-world.wit"
wit-bindgen = "0.1.0"
```

This all looks correct, so let's actually publish the package!

```console
$ cargo wapm
```

If you open up WAPM in your browser, you should see a new package has been
published. It'll look something like [`Michael-F-Bryan/tutorial-01`][pkg].

## Using the Package from Python

Let's create a Python project that uses the bindings to double-check that `1+1`
does indeed equal `2`.

First, create a new [virtual environment][venv] and activate it.

```console
$ python -m venv env
$ source env/bin/activate
```

Now we can ask the `wapm` CLI to `pip install` our `tutorial-01` package's
Python bindings.

```console
$ wapm install --pip Michael-F-Bryan/tutorial-01
...
Successfully installed tutorial-01-0.1.0 wasmer-1.1.0 wasmer-compiler-cranelift-1.1.0
```

Whenever a package is published to WAPM with the `bindings` field set, WIT Pack
will automatically generate bindings for various languages in the background.
All the `wapm` CLI is doing here is asking the WAPM backend for these bindings -
[you can run the query yourself][query] if you want.

The `tutorial_01` package exposes a `bindings` variable which we can use to
create new instances of our WebAssembly module. As you would expect, the object
we get back has our `add()` method.

```py
# main.py

from tutorial_01 import bindings

instance = bindings.hello_world()
print("1 + 1 =", instance.add(1, 1))
```

Let's run our script.

```console
$ python ./main.py
1 + 1 = 2
```

## Conclusions

Hopefully you've got a better idea for how to create a WebAssembly library and
use it from different languages, now.

To recap, the process for publishing a library to WAPM is:

1. Define a `*.wit` file with your interface
2. Create a new Rust crate and add `wit-bindgen` as a dependency
3. Implement the trait defined by `wit_bindgen_rust::export!("hello-world.wit")`
4. Add `[package.metadata.wapm]` table to your `Cargo.toml`
5. Publish to WAPM

We took a bit longer than normal to get here, but that's mainly because there
were plenty of detours to explain the "magic" that tools like `wit-bingen` and
`cargo wapm` are doing for us.  This explanation gives you a better intuition
for how the tools work, but we'll probably skip over them in the future.

Some exercises for the reader:

- If your editor has some form of intellisense or code completion, hover over
  things like `bindings.hello_world` and `instance.add` to see their signatures
- Add an `add_floats` function to `hello-world.wit` which will add 32-bit
  floating point numbers (`f32`)


[cargo-expand]: https://github.com/dtolnay/cargo-expand
[cargo-pkg-metadata]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table
[cargo-wapm]: https://lib.rs/cargo-wapm
[crate-type]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field
[cross-compile]: https://rust-lang.github.io/rustup/cross-compilation.html
[pkg]: https://wapm.dev/Michael-F-Bryan/tutorial-01
[published]: https://wapm.dev/Michael-F-Bryan/hello-world
[publishing-docs]: https://docs.wasmer.io/ecosystem/wapm/publishing-your-package
[query]: https://registry.wapm.dev/graphql?query=%7B%0A%20%20getPackageVersion(name%3A%20%22Michael-F-Bryan%2Ftutorial-01%22)%20%7B%0A%20%20%20%20version%0A%20%20%20%20bindings%20%7B%0A%20%20%20%20%20%20language%0A%20%20%20%20%20%20url%0A%20%20%20%20%7D%0A%20%20%7D%0A%7D%0A
[venv]: https://packaging.python.org/en/latest/guides/installing-using-pip-and-virtual-environments/#creating-a-virtual-environment
[wapm-io-signup]: https://wapm.io/signup
[wasi]: 07-wasi.md
[wit-bindgen]: https://github.com/wasmerio/wit-bindgen
[wit]: https://github.com/WebAssembly/component-model/blob/5754989219db51ba24def50c3ac28bb9775ead33/design/mvp/WIT.md
