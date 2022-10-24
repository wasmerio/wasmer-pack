# Strings and Lists

Now we know how to write a WebAssembly library and add two numbers, let's work
with something slightly more interesting - strings and lists!

First, we need to create a new project to hold our code. We'll also remove the
existing code so we start from blank slate.

```console
$ cargo new --lib tutorial-02
$ cd tutorial-02 && rm src/lib.rs
```

## The WIT File

Just like last time, our first step is to define a WIT file for our interface.

This file has two functions, the first function will create a string that greets
a person by name (i.e. *"Hello, Michael!"*)...

```wit
// strings-and-lists.wit

/// Greet a person by name.
greet: func(name: string) -> string
```

... and the other function will take a list of people's names, greeting them
all at the same time.

```wit
/// Say hello to multiple people.
greet-many: func(people: list<string>) -> string
```

## Writing Some Rust

Now we've defined our `strings-and-lists.wit` file, let's implement the crate.

The first thing we need to do is add `wit-bindgen` as a dependency.

```console
$ cargo add --git https://github.com/wasmerio/wit-bindgen wit-bindgen-rust
```

We also need to tell `wit-bindgen` that we're implementing
`strings-and-lists.wit`.

```rust
// src/lib.rs

wit_bindgen_rust::export!("strings-and-lists.wit");
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
`greet_many()`'s use of [*Slice Patterns*][slice-patterns].

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

Luckily `wit-bindgen` will generate all the code we need for this, but it's
something to be aware of.

Another thing to keep in mind is that all return values must be owned, too.

WebAssembly doesn't have any concept of ownership and borrowing, so it'd be easy
for the host to run into use-after-free issues and dangling pointers if we were
allowed to return non-`'static` values.

---

## Intro

- A promise statement:
  - We're going to start using
- A preview of what's to come:
  - Show console output with `Hello, $name!`

## Body

Defining our WIT file:

```rust
// greetings.wit

/// Say hello to multiple people.
greet: func(people: list<string>) -> string
```

Create the guest:
- Pretty much the same as hello world

```rust
impl crate::greetings::Greetings for Greetings {
    fn greet(people: Vec<String>) -> String {
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

Publish to WAPM:
- Similar to before
- Probably shortened to 1 paragraph, a link, and some example output

Use the bindings:
- Create a Python project
- Use `wapm install --pip tutorial/strings-and-lists` to install the package
- Write the script
- Run it using "names" from `sys.argv`

## Conclusion

- Reminder of how helpful the guide is
- Reiterate how important your topic is
  - Strings and lists are the what applications are made of
- Call-to-action
  - Read [`WIT.md`](https://github.com/wasmerio/wit-bindgen/blob/wasmer/WIT.md)
    to learn more about the syntax

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

[slice-patterns]: https://adventures.michaelfbryan.com/posts/daily/slice-patterns/
