# Choosing Between Resources and Records

The difference between a [resource](../concepts/wai/resources.md) and a
[record](../concepts/wai/records.md) can be subtle when first starting out,
but there is a simple rule of thumb that will work 90% of the time:

> Records contain data, resources encapsulate behaviour.

## Typical Examples

A person is a good example of a record.

```
record person {
    name: string,
    age: u8,
    address: option<address>,
}

record address {
    number: u32,
    street: string,
    state: string,
    country: string,
    postcode: u32,
}
```

On the other hand, a file would be best represented using a resource.

```
resource file {
    static open: func(path: string) -> expected<file, error>
    read: func() -> expected<list<u8>, error>
    write: func(data: list<u8>) -> expected<unit, error>
}
```

## Edge Cases

While the *"Records contain data, resources encapsulate behaviour"* rule works
for most cases, you will almost certainly run into situations where something
has both data and behaviour.

This happens a lot when wrapping a "normal" library with a WAI interface so it
can be used from WebAssembly. The distinction between "object" and "data" is
more fluid in most general purpose programming languages so it can be common to
encounter something that doesn't neatly fall into the "record" or "resource"
categories.

### Workaround 1: Getters & Setters

If something would normally have publicly accessible fields **and** methods
which might modify those fields, the best solution is to make that thing a
resource with getters and setters.

For example, a virtual machine might normally expose its instruction pointer
and any global variables that are defined, while also having an `eval()` method
for evaluating code snippets.

```
resource virtual-machine {
    instruction-pointer: func() -> u32
    set-instruction-pointer: func(ip: u32)
    global-variables: func() -> list<tuple<string, value>>
    set-global-variable: func(name: string, value: value)
    eval: func(code: string) -> expected<unit, error>
}
```

This approach works particularly well when the methods will update state because
all resources are reference types, meaning any modifications made to a resource
through one handle (e.g. via a method) will be seen by all other handles to the
same resource.

One downside of this approach is that each getter or setter is implemented using
a method. When you have a large number of fields to expose, these extra methods
can become hard to maintain or make it easy to lose interesting functionality
within a wall of boilerplate.

### Workaround 2: Move Methods to Top-Level Functions

Going in the other direction, sometimes it might be better to turn methods into
top-level functions and use a record.

One example of this could be the satellite object used in a library that
predicts the motion of a satellite.

```
/// An element
record satellite {
    object-name: optional<string>,
    norad-id: u64,
    inclination: float64,
    right-ascension: float64,
    ..
}

/// Parse a satellite from its JSON representation.
satellite-from-json: func(json: string) -> expected<satellite, error>

/// Predict where a satellite will be at a particular time.
predicted-location: func(s: satellite, ts: timestamp) -> position
```

This works well when the thing being expressed is mostly data, with only a
couple pieces of assocated behaviour.

Records are passed around by value, meaning any operations that would normally
modify a field will need to return a new value with the updated field, instead.
This can be quite expensive when the record is large, because passing a record
from guest to host (or host to guest) will often mean the entire object is
serialized recursively and copied across the host-guest boundary.
