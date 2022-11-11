# Records

You're just doing great! Now, let us introduce record types, often called "structs" in other languages

## Project Setup

```console
$ cargo new --lib tutorial-03
$ cd tutorial-03 && rm src/lib.rs
```

## Introduction

Record is a combination of data that structurally belongs together such as:

```console
record pair {
   x: u32,
   y: u32,
}
record person {
   name: string,
   age: u32,
   has-lego-action-figure: bool,
}
```

### Syntax for a record:

> record-item ::= 'record' id '{' record-fields '}'
>
> record-fields ::= record-field
> | record-field ',' record-fields?
>
> record-field ::= id ':' ty

A _Record_ lets you pass data from the **guest** to the **host**

## The WIT File

Now let us define our WIT file for our interface.

This file has a record and a function. The record is a structure for a point in a coordinate system _(x,y)_. The function performs the distance calculation between two points as arguements.

```wai
// coordinates.wai

/// A point coordinate structure with { x, y }
record point {
    x: float32,
    y: float32
}

/// Calculate distance between two points
distance-between: func(p1: point, p2: point) -> float32
```

## Writing Some Rust

Now we've defined our `coordinates.wai` file, let's implement the crate.

The first thing we need to do is add `wai-bindgen` as a dependency

```console
$ cargo add wai-bindgen-rust
```

We also need to tell `wai-bindgen` that we're implementing
`coordinates.wai`.

```rust
// src/lib.rs

wai_bindgen_rust::export!("coordinates.wai");
```

Next, we need to define a `coordinates` struct and implement the
`coordinates::Coordinates` on it.

```rust
struct Coordinates;

impl coordinates::Coordinates for Coordinates {
    fn distance_between(p1: Point, p2: Point) -> f32 {
        let Point { x: x1, y: y1 } = p1;
        let Point { x: x2, y: y2 } = p2;

        ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }
}
```

### Explaination

Here, the function `distance_between` takes two arguement of the _Point_ type. For simplicity we [destructure](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_structures.html) it for a clear distinction between the x1, x2 and y1,y2 as opposed to writing `p1.x` or `p1.y` everytime.

We then find the distance between the two points using the [distance formula](https://en.wikipedia.org/wiki/Euclidean_distance).

#### Note

> As `.wai` files only accept kebab-casing. The function `distance_between` in the `coordinates.wai` will convert to the default casings for the respected language.
>
> //change here after formatting!!
> i.e: _snake_case_ for rust, _CamelCase_ for Javascript,

## Publishing

Similar to last time, if we want to publish our package to WAPM, we'll need to
update our `Cargo.toml` file.

```toml
# Cargo.toml
[package]
...
description = "Find Distance between two points"

[lib]
crate-type = ["cdylib", "rlib"]

[package.metadata.wapm]
namespace = "wasmer"
abi = "none"
bindings = { wai-bindgen = "0.1.0", exports = "coordinates.wai" }
```

Now, we can publish it to WAPM.

```console
$ cargo wapm
Successfully published package `wasmer/tutorial-03@0.1.0`
```

- write the installation for typescript/python and conclusion
- write about the importance of nested records and ask bryan about records with resources in them

## Conclusion

- Reminder of how helpful the guide is
- Reiterate how important your topic is
  - Records (structs) are the building blocks for abstraction
- Call-to-action
  - Add different fields to the record or play around with the types to see
    what is generated for the guest/host

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
