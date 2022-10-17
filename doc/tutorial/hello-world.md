# Hello, World!

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
