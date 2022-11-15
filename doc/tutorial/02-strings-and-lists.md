# Strings and Lists

Now we know how to write a WebAssembly library and add two numbers, let's work
with something slightly more interesting - strings and lists!

First, we need to create a new project to hold our code. We'll also remove the
existing code, so we start from blank slate.

```console
$ cargo new --lib tutorial-02
$ cd tutorial-02 && rm src/lib.rs
```

## The WIT File

Just like last time, our first step is to define a WIT file for our interface.

This file has two functions, the first function will create a string that greets
a person by name (i.e. _"Hello, Michael!"_)...

```wai
// strings-and-lists.wai

/// Greet a person by name.
greet: func(name: string) -> string
```

... and the other function will take a list of people's names, greeting them
all at the same time.

```wai
/// Say hello to multiple people.
greet-many: func(people: list<string>) -> string
```

## Writing Some Rust

Now we've defined our `strings-and-lists.wai` file, let's implement the crate.

The first thing we need to do is add `wai-bindgen` as a dependency.

```console
$ cargo add wai-bindgen-rust
```

We also need to tell `wai-bindgen` that we're implementing
`strings-and-lists.wai`.

```rust
// src/lib.rs

wai_bindgen_rust::export!("strings-and-lists.wai");
```

Next, we need to define a `StringsAndLists` type and implement the
`strings_and_lists::StringsAndLists` on it.

```rust
struct StringsAndLists;

impl strings_and_lists::StringsAndLists for StringsAndLists {
    fn greet(name: String) -> String {
        format!("Hello, {name}!")
    }

    fn greet_many(people: Vec<String>) -> String {
        match people.as_slice() {
            [] => "Oh, nobody's there...".to_string(),
            [person] => format!("Hello, {person}!"),
            [people @ .., last] => {
                let people = people.join(", ");
                format!("Hello, {people}, and {last}!")
            }
        }
    }
}
```

The implementation of these functions is fairly straightforward, so we don't
need to go into too much detain about it other than pointing out
`greet_many()`'s use of [_Slice Patterns_][slice-patterns].

### A Note on Ownership

While our code wasn't overly complex, there is something that needs to be
pointed out,

> Both functions use owned values for their arguments and results

This may seem odd, because it's idiomatic in normal Rust to pass strings and
lists around by reference (i.e. `&str` and `&[String]`) so the caller can
maintain ownership of the original value and doesn't need to make unnecessary
copies.

This comes back to one of the design decisions for WebAssembly, namely that a
guest (our `tutorial-02` crate) is completely sandboxed and unable to access
memory on the host.

That means when the host calls our WebAssembly function, arguments will be
passed in by

1. Allocate some memory **inside** the guest's linear memory
2. Copy the value into this newly allocated buffer
3. Hand ownership of the buffer to the guest function (i.e. our `greet()`
   method)

Luckily `wai-bindgen` will generate all the code we need for this, but it's
something to be aware of.

Another thing to keep in mind is that all return values must be owned, too.

WebAssembly doesn't have any concept of ownership and borrowing, so it'd be easy
for the host to run into use-after-free issues and dangling pointers if we were
allowed to return non-`'static` values.

## Publishing

Similar to last time, if we want to publish our package to WAPM, we'll need to
update our `Cargo.toml` file.

```toml
# Cargo.toml
[package]
...
description = "Greet one or more people"

[lib]
crate-type = ["cdylib", "rlib"]

[package.metadata.wapm]
namespace = "wasmer"
abi = "none"
bindings = { wai-bindgen = "0.1.0", exports = "strings-and-lists.wai" }
```

Now, we can publish it to WAPM.

```console
$ cargo wapm
Successfully published package `wasmer/tutorial-02@0.1.0`
```

## Using The Bindings From TypeScript

For a change, let's use our bindings from TypeScript. First, we need to create
a basic `package.json` file.

```console
$ yarn init --yes
success Saved package.json
```

We'll need to install TypeScript and `ts-node`.

```console
$ yarn add --dev ts-node typescript @types/node
```

The TypeScript compiler will need a basic `tsconfig.json` file.

```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "es2016",
    "module": "ESNext",
    "moduleResolution": "node",
    "strict": true,
    "skipLibCheck": true
  }
}
```

Now, we can use `wapm install` to add our `tutorial-02` package as a dependency.

```console
$ wapm install --yarn wasmer/tutorial-02
```

Finally, we're able to start writing some code.

```ts
// index.ts

import { bindings } from "@wasmer/tutorial-02";

async function main() {
  const strings = await bindings.strings_and_lists();
  console.log(strings.greet("World"));
  console.log(strings.greetMany(["a", "b", "c"]));
}

main();
```

If we run it using the `ts-node` loader, we'll see exactly the output we expect.

```console
$ node --loader ts-node/esm index.ts
Hello, World!
Hello, a, b, and c!
```

## Conclusion

Strings and lists are the building blocks of all meaningful applications, so
it's important to know how to use them.

Our first foray into non-primitive types has also introduced us to the
repercussions of running your code inside a fully sandboxed virtual machine -
any data received from the outside world must be copied into linear memory.

[slice-patterns]: https://adventures.michaelfbryan.com/posts/daily/slice-patterns/
