# Records

A record is an abstract data type containing a series of named fields. It has no
associated behaviour and acts as a way to group data together. In C++, this
would be referred to as a [plain old data][pod] type.

Depending on the language, records may be expressed in in different ways.

| Language   | Equivalent Construct     |
| ---------- | ------------------------ |
| Rust       | Struct                   |
| C          | Struct                   |
| Python     | [Data Class][dataclass]  |
| JavaScript | [Type alias][type-alias] |

## Syntax

A record contains a list of fields, where each field must be given a type.

```
record person {
    name: string,
    age: u32,
    has-lego-action-figure: bool,
}
```

For a more details, consult [the *Item: `record`* section][record] in the
`*.wai` format.

[dataclass]: https://peps.python.org/pep-0557/
[pod]: https://en.wikipedia.org/wiki/Passive_data_structure
[record]: https://github.com/wasmerio/wai/blob/main/WAI.md#item-record-bag-of-named-fields
[type-alias]: https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#type-aliases
