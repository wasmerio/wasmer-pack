# Builtin Types

All types that can be used in a `*.wai` file are intended to mappable to native
types in a general purpose programming language.

The core basic types are

- Unsigned integers (`u8`, `u16`, `u32`, `u64`)
- Signed integers (`s8`, `s16`, `s32`, `s64`)
- Floating point numbers (`float32`, `float64`)
- UTF-8 Strings (`string`)
- UTF-8 code points (`char`)
- [Void][void] or nothing (`unit`)

## Other Builtin Types

Besides the basic builtin types, there are several "generic" types built into
WAI which let users express common concepts.

### Tuples

The tuple is equivalent to a [record](./records.md) that has numeric fields.

Code generators may be able to express tuples as a first-class concept. For
example, `tuple<string, float32, float32>` would be expressed as
`(String, f32, f32)` in Rust.

### Lists

Lists are dynamically-sized sequences of the same element type. Often called
a "list", "vector", or "array", a `list<string>` would be expressed as
`Vec<String>` in Rust.

### Option

The option type is used to express a value that may or may not be present.

In Rust, an `option<T>` is expressed as [`std::option::Option<T>`][rust-option],
while other languages may choose to use `null` to represent the missing value.

It is semantically equivalent to the following variant:

```
variant option {
    none,
    some(T),
}
```

### Expected

The expected type is used to express the result of a fallible operation.

In Rust, an `expected<T, E>` is expressed as
[`std::result::Result<T, E>`][rust-result], although other languages may choose
to convert errors into exceptions.

It is semantically equivalent to the following variant:

```
variant expected {
    ok(T),
    err(E),
}
```

### Futures & Streams

The `future<T>` and `stream<T, E>` types are used to represent the result of
asynchronous operations.

[rust-option]: https://doc.rust-lang.org/std/option/enum.Option.html
[rust-result]: https://doc.rust-lang.org/std/result/enum.Result.html
[void]: https://en.wikipedia.org/wiki/Void_type
