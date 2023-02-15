# Enums, Flags, Variants, and Unions

the concept of "this value can be X or Y or Z" can be expressed in several ways
depending on the context.

## Enum

The `enum` keyword can be used to introduce a C-style enum. This is a type of
named constant where each constant has its own value.

```wai
enum ordering {
    less-than,
    equal-to,
    greater-than,
}
```

For a more details, consult [the *Item: `enum`* section][enum] in the `*.wai`
format.

## Flags

The `flags` keyword can be used to introduce a bitflags variable. The easiest
way to think of this is as a "bag of bools" where each variant can be set
independently.

```wai
flags text-style {
    bold,
    italics,
    underline,
    strikethrough,
}
```

The `flags` type is a separate concept from `enum` because multiple flag
variants can be set at a time, whereas an `enum` can be only one thing at a
time. Different languages are often able to express this in a very efficient
form, typically an integer where each bit represents a different flag.

For a more details, consult [the *Item: `flags`* section][flags] in the `*.wai`
format.

## Variant

A `variant` lets you express something that is one of a set of types. This is
similar to an `enum`, except each variant may have some associated data.

```wai
variant error {
    file-not-found(string),
    parse(parse-failed),
    other,
}

record parse-failed {
    message: string,
    line-number: u32,
}
```

Variants are implemented very differently based on what is idiomatic for a
particular language.

In Rust, a `variant` is just a normal `enum`.

In TypeScript, the variant is implemented as a tagged union.

```ts
type error = { type: "file-not-found", value: string }
    | { type: "parse", value: ParseFailed }
    | { type: "other" };
```

For a more details, consult [the *Item: `variant`* section][variant] in the
`*.wai` format.

## Union

A `union` is very similar to a variant, except it drops the type tag.

```wai
union configuration {
    string,
    list<string>,
}
```

This is distinct from a `variant` because some languages may be able to
represent an `union` in a way that is more efficient or idiomatic.

For a more details, consult [the *Item: `union`* section][union] in the
`*.wai` format.


[union]: https://github.com/wasmerio/wai/blob/main/WAI.md#item-union-variant-but-with-no-case-names
[enum]: https://github.com/wasmerio/wai/blob/main/WAI.md#item-union-variant-but-with-no-case-names
[variant]: https://github.com/wasmerio/wai/blob/main/WAI.md#item-variant-one-of-a-set-of-types
[flags]: https://github.com/wasmerio/wai/blob/main/WAI.md#item-flags-bag-of-bools
