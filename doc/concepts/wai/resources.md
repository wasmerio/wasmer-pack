# Resources

A resource represents an opaque object where the representation and underlying
implementation is completely hidden from the outside world. Resources may have
associated methods, static methods, or no methods at all.

Depending on the language, records may be expressed in in different ways.

| Language   | Equivalent Construct         |
| ---------- | ---------------------------- |
| Rust       | [Trait object][trait-object] |
| Python     | class                        |
| JavaScript | class                        |

Resources can only be used through a "handle" and can be owned by either the
host or the guest. Resource lifetimes are managed manually, although most
languages provide some sort of reference counting or garbage collection
mechanism.

## Syntax

The simplest resource is an opaque "token". Users can pass this value around,
but have no other way to interact with it.

```
resource file-descriptor
```

Resources can also have methods. These are functions which are associated with
the resource and are implicitly given a reference to the resource when they
are invoked.

```
resource writer {
  write: func(data: list<u8>) -> expected<unit, error>
  flush: func() -> expected<unit, error>
}
```

Additionally, resources can have `static` methods. These are often used to
implement constructors.

```
resource request {
    static new: func() -> request

    body: async func() -> list<u8>
    headers: func() -> list<string>
}
```

For a more details, consult [the *Item: `resource`* section][resource] in the
`*.wai` format.

[resource]: https://github.com/wasmerio/wai/blob/main/WAI.md#item-resource
[trait-object]: https://doc.rust-lang.org/reference/types/trait-object.html
