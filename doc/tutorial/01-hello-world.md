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
- [the `wit-pack` CLI][wit-pack] for generating bindings to our package
  (`cargo install --git https://github.com/wasmerio/wit-pack wit-pack-cli`)
- [the `cargo wapm` sub-command][cargo-wapm] for publishing to WAPM
  (`cargo install cargo-wapm`)

> Note: The last two will probably be included with the Wasmer runtime in the
> future, but we'll install them manually for now.

## The WIT File

We want to start off simple for now, so let's create a library that just adds
two 32-bit integers.

The syntax for a WIT file is quite similar to Rust.

```
// hello-world.wit

/// Add two numbers
add: func(a: i32, b: i32) -> i32
```

This defines a function called `add` which takes two `i32` parameters (32-bit
signed integers) called `a` and `b`, and returns a `i32`.

You can see that normal comments start with a `//` and doc-comments use `///`.
Here, we're using `// hello-world.wit` to indicate the text should be saved to
`hello-world.wit`.

One interesting quirk of WIT is that *all* names must be written in kebab-case.
This lets `wit-bindgen` convert the name into the casing that is idiomatic for a
particular language in a particular context.

For example, if our WIT file defined a `hello-world` function, it would be
accessible as `hello_world` in Python and Rust because they use snake_case for
function names, whereas in JavaScript it would be `helloWorld`.

---

## Intro

- A promise statement:
  - Let's publish your first library to WAPM and use it
- A preview of what's to come:
  - Link to an existing "Hello World" package on WAPM
  - Show that `1+1 = 2`?

## Body

Install everything:
- Wasmer toolchain
- The `cargo wapm` tool
- Authenticate with WAPM

Define our WIT file:

```rust
// hello-world.wit

/// Add two numbers
add: func(a: i32, b: i32) -> i32
```

Create the guest:
- Use `cargo add --git https://github.com/wasmerio/wit-bindgen --branch wasmer wit-bindgen-rust`
- Define the `crate::HelloWorld` struct
- Implement the `crate::hello_world::HelloWorld` on `crate::HelloWorld`

Publish to WAPM:
- Update `Cargo.toml`
- Use `cargo wapm` to publish
- View the package on <https://wapm.io/>

Use the bindings:
- Create a Python project
- Use `wapm install --pip tutorial/hello-world` to install the package
- Write the script
- Explain why we need to explicitly instantiate the library
  - The global state for each module is wrapped up in an object
  - Lets us have nice things like sandboxing
  - Enables instantiating with different configurations (imports, WASI, etc.)
- Run it, showing that 1+1 does indeed equal 2

## Conclusion

- Reminder of how helpful the guide is
- Reiterate how important your topic is
- Call-to-action
  - Modify the `add()` function to take more arguments or use different types

## Checklist

### Inspiration 💡

- [ ] Read articles and watch videos that inspire me
- [ ] Brainstorm the topics that I want to write about in bullet points
- [ ] Reorder those bullet points to create a line of thought

### Draft ✏

- [ ] Expand those bullet points into sentences/text
- [ ] Go over the document

### Ready to Publish 🗞

- [ ] Draft 5 titles and pick one
- [ ] Revise the complete text for typos
- [ ] Preview the text
- [ ] Publish or schedule the post

[cargo-wapm]: https://lib.rs/cargo-wapm
[wit-bindgen]: https://github.com/wasmerio/wit-bindgen
[wit-pack]: https://github.com/wasmerio/wit-pack
[wit]: https://github.com/WebAssembly/component-model/blob/5754989219db51ba24def50c3ac28bb9775ead33/design/mvp/WIT.md
[published]: https://wapm.dev/Michael-F-Bryan/hello-world
