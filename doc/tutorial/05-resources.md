# Exposing Resources

So far we've covered basic functions, numbers, strings, lists, records, and
variants, but there's one key aspect in programming we haven't touched on yet -
objects with behaviour and internal state!

In WIT, we use a `resource` to give the host access to an "object" that has
behaviour without exposing how that object is implemented (or even which
language it is implemented in).

To explore this concept, we'll create a basic `Calculator` resource which lets
the caller do various operations like addition and multiplication.

Like always, let's start off with a new Rust project.

```console
$ cargo new --lib tutorial-05
```

## The WIT File

The key thing to understand about a `resource` is that it only defines methods.

```wit
// resources.wit

resource calculator {
    static new: func(initial-value: float32) -> calculator
    current-value: func() -> float32
    add: func(value: float32)
    multiply: func(value: float32)
}
```

Prefixing a function with the `static` keyword will turn it into a
[`static` method][static-method]. This is useful for defining constructors or
factory functions.

Resource methods also allow the `async` modifier (i.e.
`add: async func(value: float32)`), however that will require your runtime to
support `async` functions.

## Writing Some Rust

As always, we need to add `wit-bindgen` as a dependency.

```console
$ cargo add --git https://github.com/wasmerio/wit-bindgen wit-bindgen-rust
```

We also need to ask `wit-bindgen` to generate exports for our `resources.wit`
interface.

```rust
// src/lib.rs

wit_bindgen_rust::export!("resources.wit");
```

If we run `cargo check`, we'll see that - besides the missing `Resources` type
we expected - it complains about not being able to find `Calculator`.

```console
$ cargo check
error[E0412]: cannot find type `Calculator` in module `super`
...
error[E0412]: cannot find type `Resources` in module `super`
```

We can create the `Resources` struct and implement the `resources::Resources`
trait for it. This module won't have any top-level functions, so the trait
implementation can stay empty.

```rust
pub struct Resources;

impl resources::Resources for Resources {}
```

The way the `resources::Calculator` trait was designed adds certain constraints
to our `Calculator` struct, though.

```rust
pub struct Calculator(Mutex<f32>);

impl resources::Calculator for Calculator {
    fn new(initial_value: f32) -> Handle<Calculator> {
        Handle::new(Calculator(Mutex::new(initial_value)))
    }

    fn current_value(&self) -> f32 {
        *self.0.lock().unwrap()
    }

    fn add(&self, value: f32) {
        *self.0.lock().unwrap() += value;
    }

    fn multiply(&self, value: f32) {
        *self.0.lock().unwrap() *= value;
    }
}
```

### A Note On Mutability

You'll notice that all methods in the `resources::Calculator` trait take an
immutable `&self`. This means we'll need to use [interior mutability][int-mut]
if we want to update internal state.

While this might feel a bit awkward, there is a very good reason for requiring
all state in a resource to synchronise its mutations - WebAssembly makes no
guarantees that the caller will respect the borrow checker.

## Publishing

We need to publish this package to WAPM, so let's update `Cargo.toml` with the
relevant metadata.

```toml
# Cargo.toml
[package]
...
description = "A basic calculator"

[lib]
crate-type = ["cdylib", "rlib"]

[package.metadata.wapm]
namespace = "wasmer"
abi = "none"
bindings = { wit-bindgen = "0.1.0", exports = "resources.wit" }
```

Now, we can publish it to WAPM.

```console
$ cargo wapm
Successfully published package `wasmer/tutorial-05@0.1.0`
```

## Using The Bindings From Python

Once the package has been published, create a new Python virtual environment
and import the package into Python.

```console
$ python -m venv env
$ source env/bin/activate
$ wapm install --pip wasmer/tutorial-05
```

Next, let's create our test script.

```python
# main.py

from tutorial_05 import bindings
from tutorial_05.bindings.resources import Calculator

resources = bindings.resources()
calculator = Calculator.new(resources, 3)

calculator.add(5)
calculator.multiply(2)
print("(3 + 5)*2 =", calculator.current_value())
```

Running it from the command-line should show us that `(3+5)*2 = 16`.

```console
$ python main.py
(3+5)*2 = 16
```

## Conclusion

The resource is a useful tool for exposing stateful objects that have behaviour,
and should be familiar to 


With the addition of resources, we've introduced most of the fundamental
constructs in WIT.

Exercises for the reader:
- Expand the `calculator` resource to be a fully fledged calculator
- Try to create your own Regex package using the `regex` crate

[static-method]: https://docs.python.org/3/library/functions.html#staticmethod
[int-mut]: https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
