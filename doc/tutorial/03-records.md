# Records

You're just doing great! Now, let us introduce record types, often called "structs" in other languages.

## Project Setup

Let's clear our `./src/lib.rs` and start from scratch

## Introduction

Record is a combination of data that structurally belongs together such as:

### Syntax for a record

> record-item ::= 'record' id '{' record-fields '}'
>
> record-fields ::= record-field
> | record-field ',' record-fields?
>
> record-field ::= id ':' ty

A _Record_ lets you pass data between the **guests** and the **hosts**

## The WIT File

Now let us define our WIT file for our interface.

```wai
//geometry.wai

/// A point coordinate structure with { x, y }
record point {
    x: float32,
    y: float32
}
```

> Note: As you use `cargo expand`, the generated file won't contain the `Point` Geometry 🙁

```Rust
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
#[allow(clippy::all)]
mod geometry {}
```

> This happens because we didn't use our `Point` geometry in any function/interface so it is not compiled to an underlying struct for `Rust`.

So now let's use our `Point` geometry in a function.

```wai
// geometry.wai

/// A point coordinate structure with { x, y }
record point {
    x: float32,
    y: float32
}

/// Calculate distance between two points
distance-between: func(p1: point, p2: point) -> float32
```

> `cargo expand`

```Rust
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use crate::geometry::{Circle, MultiLine, Point};
#[allow(clippy::all)]
mod geometry {
    /// A point coordinate structure with { x, y }
    #[repr(C)]
    pub struct Point {
        pub x: f32,
        pub y: f32,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Point {}
    #[automatically_derived]
    impl ::core::clone::Clone for Point {
        #[inline]
        fn clone(&self) -> Point {
            let _: ::core::clone::AssertParamIsClone<f32>;
            *self
        }
    }
    impl core::fmt::Debug for Point {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("Point").field("x", &self.x).field("y", &self.y).finish()
        }
    }
    #[export_name = "distance-between"]
    unsafe extern "C" fn __wai_bindgen_geometry_distance_between(
        arg0: f32,
        arg1: f32,
        arg2: f32,
        arg3: f32,
    ) -> f32 {
        let result = <super::Geometry as Geometry>::distance_between(
            Point { x: arg0, y: arg1 },
            Point { x: arg2, y: arg3 },
        );
        wai_bindgen_rust::rt::as_f32(result)
    }
    pub trait Geometry {
        /// Calculate distance between two points
        fn distance_between(p1: Point, p2: Point) -> f32;
    }
}
```

> Here, we see that our `Point` struct can be seen `distance_between` function is present and is exported as an external variable for use in our Rust file.
> We also see the Debug trait being implemented for the Point record.

This file has a record and a function. The record is a structure for a point in a coordinate system _(x,y)_. The function performs the distance calculation between two points as arguements.

## Writing Some Rust

Now we've defined our `geometry.wai` file, let's implement the crate.

The first thing we need to do is add `wai-bindgen` as a dependency

```console
$ cargo add wai-bindgen-rust
```

We also need to tell `wai-bindgen` that we're implementing
`geometry.wai`.

```rust
// src/lib.rs

wai_bindgen_rust::export!("geometry.wai");
```

Next, we need to define a `geometry` struct and implement the
`geometry::Geometry` Trait on it.

```rust
struct Geometry;

impl geometry::Geometry for Geometry {
    fn distance_between(p1: Point, p2: Point) -> f32 {
        let Point { x: x1, y: y1 } = p1;
        let Point { x: x2, y: y2 } = p2;

        ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }
}
```

> Note: This may seem comfusing so I've boiled it down:
>
> - `geometry` is the crate
> - `Geometry` is the struct
> - `geometry::Geometry` is the Trait that implements the function `distance_between` on `Geometry`

### Explaination

Here, the function `distance_between` takes two arguement of the _Point_ type. For simplicity we [destructure](https://doc.rust-lang.org/rust-by-example/flow_control/match/destructuring/destructure_structures.html) it for a clear distinction between the x1, x2 and y1,y2 as opposed to writing `p1.x` or `p1.y` everytime.

We then find the distance between the two points using the [distance formula](https://en.wikipedia.org/wiki/Euclidean_distance).

##### Note📝

> As `.wai` files only accept kebab-casing. The function `distance_between` in the `geometry.wai` will convert to the default casings for the respected language.
>
> //change here after formatting!!
> i.e: _snake_case_ for rust, _CamelCase_ for Javascript,

### Nested Records

As we saw, the use of simpler identifiers to create a `Point` record. we can further extend this functionality using records or other valid `WAI types` to specify the record arguments to create more complex and _nested records_.

> ⚠️ Recursive types are explicitly forbidden in WAI.

```wai
record tree-node {
    children: list<tree-node>
}
```

> 👆🏼, is not allowed.

Let's futher explain `Nested Records` this with an example:

> WAI file with nested records :

```wai
/// A line geometry to represent a line segment with starting and ending point
record line-segment {
    start: point,
    end: point
}

/// A structure to represent a circle with center point and a radius
record circle {
    center: point,
    radius: float32
}

/// Arbitary shape - represent a shape with n number of points using a list of points
record multi-line{
    points: list<point>,
}

/// Calculate the perimeter of a Circle using 2*π*r.
perimeter-of-circle: func(c: circle) -> float32

/// Calculate the area of a Circle using π*r*r.
area-of-circle: func(c: circle) -> float32

/// Calculate the length of the multi-line by iterating over all the points and adding the result
multi-line-length: func(l: multi-line) -> float32
```

Here we used the `point` struct that we created earlier to futher define records (i.e. `line-segment`, `circle` and `shape`).

- line segment uses points to define starting and ending of the line
- Circle uses the point record for defining a center
- An Arbitrary shape can also be represented as a list of points

If we had x,y for representing points in each of these geometries it would have no structure and code readability. Thus, we define nested records using a previously existing record.

##### Note📝

> Records can further have _type identifiers_ such as u8, u16, float32, enum, tuple, etc.

## Writing Some Rust Again

```Rust
use crate::geometry::{Circle, MultiLine, Point};

wai_bindgen_rust::export!("geometry.wai");

struct Geometry;

impl geometry::Geometry for Geometry {
    fn distance_between(p1: Point, p2: Point) -> f32 {
        let Point { x: x1, y: y1 } = p1;
        let Point { x: x2, y: y2 } = p2;

        ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }
    fn perimeter_of_circle(c: Circle) -> f32 {
        let Circle { center: _, radius } = c;
        (2.0 * 22.0 * radius as f32) / 7.0
    }
    fn area_of_circle(c: Circle) -> f32 {
        let Circle { center: _, radius } = c;
        (22.0 * (radius * radius) as f32) / 7.0
    }
    fn multi_line_length(l: MultiLine) -> f32 {
        if l.points.len() == 0 {
            return 0.0;
        }
        let mut result = 0.0;
        for i in 1..l.points.len() {
            let p1 = l.points[i - 1];
            let p2 = l.points[i];
            result += Geometry::distance_between(p1, p2);
        }
        result
    }
}
```

Here, we defined multiple functions such as:

- `perimeter_of_circle`
- `area_of_circle`
- `multi_line_length`

> All of these functions show how the nested records can be used to perform operations.

## Publishing

Similar to last time, if we want to publish our package to WAPM, we'll need to
update our `Cargo.toml` file.

```toml
# Cargo.toml
[package]
...
description = "Geometrical representations using points"

[lib]
crate-type = ["cdylib", "rlib"]

[package.metadata.wapm]
namespace = "wasmer"
abi = "none"
bindings = { wai-bindgen = "0.1.0", exports = "geometry.wai" }
```

Now, we can publish it to WAPM.

```console
$ cargo wapm
Successfully published package `wasmer/tutorial-03@0.1.0`
```

- write the installation for typescript/python and conclusion
- write about the importance of nested records and ask bryan about records with resources in them
- Justify how the records are converted to the host e.g: data class in python and typed object in typescript. Show code snippets

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
