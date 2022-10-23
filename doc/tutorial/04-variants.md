# Variants

## Intro

- A promise statement
  - We're going to introduce variants
  - Similar to Rust's enum or TypeScript's tagged union
- A preview of what's to come

## Body

Brief recap of how sum types work, drawing parallels to similar constructs in
existing languages.

Point out two special variants, `option<T>` and `expected<T, E>`
- Draw parallels with Rust
- Show examples of what `wit-bindgen` generates for Python and JavaScript

Define the WIT file:

```rust
// calculator.wit

variant instruction {
    add(i32),
    subtract(i32),
    multiply(i32),
    divide(i32),
}

record out-of-bounds-error {
    value: i32,
    min: i32,
    max: i32,
}

variant error {
    divide-by-zero,
    out-of-bounds(out-of-bounds-error),
}

/// Execute a sequence of instructions on a number.
///
/// This is a basic calculator!
calculate: func(start: i32, instructions: list<intruction>) -> expected<i32, error>
```

Implement the guest:
- Handle divide by zero
- Hard-code a limit of `(-10, 50)` for the value
- Publish to WAPM (links, etc.)

Use the bindings:
- Create a Python project
  - Show how you might handle errors using `isinstance()`
  - Show the version with Python's [Structural Pattern Matching](https://peps.python.org/pep-0636/)
- Create a JavaScript project
  - Show how you handle errors in TypeScript (switch-case, etc.)

## Conclusion

- Reminder of how helpful the guide is
- Reiterate how important your topic is
- Call-to-action
  - Add different error cases
  - Develop your own abstractions for working with errors

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

