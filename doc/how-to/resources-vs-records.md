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

On the other hand, a database connection would be best represented using a
resource.

```wai
resource database {
    static connect: func(connection_string: string) -> expected<database, error>
    query: func(sql: string) -> expected<list<record>, error>
    close: func() -> expected<unit, error>
}
```

## Key Considerations

When deciding between using a resource or a record, consider the following:

- Performance: Records require deep copying when passed between guest and host,
  which can be expensive for large or complex records. Consider using resources
  for objects with significant amounts of data or complex structures to mitigate
  performance issues.
- Immutability: Records provide a level of immutability due to their
  pass-by-value nature. If immutability is a priority, records can be a suitable
  choice. However, if you need to frequently modify an object's state, a resource
  might be more appropriate.
- Encapsulation: For objects with both data and behavior, consider whether
  separating the data and behavior into different objects—a record for data and a
  resource for behavior—adds value or complexity to your code.
- Data Sharing: If data sharing or synchronization across components or
  instances is important, resources are a better choice since they use references,
  while records are not ideal for sharing data.

## Edge Cases

While the *"Records contain data, resources encapsulate behaviour"* rule works
for most cases, you will almost certainly run into situations where something
has both data and behaviour.

This happens a lot when wrapping a "normal" library with a WAI interface so it
can be used from WebAssembly. The distinction between "object" and "data" is
more fluid in most general purpose programming languages, so it can be common to
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

```wai
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
couple of pieces of associated behaviour.

Records are passed around by value, meaning any operations that would normally
modify a field will need to return a new value with the updated field, instead.
This can be quite expensive when the record is large, because passing a record
from guest to host (or host to guest) will often mean the entire object is
serialized recursively and copied across the host-guest boundary. Consider the
trade-offs between performance and immutability when deciding whether to use
records or resources in these edge cases.
