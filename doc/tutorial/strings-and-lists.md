# Strings and Lists

## Intro

- A promise statement:
  - We're going to start using
- A preview of what's to come:
  - Show console output with `Hello, $name!`

## Body

Defining our WIT file:

```rust
// greetings.wit

/// Say hello to multiple people.
greet: func(people: list<string>) -> string
```

Create the guest:
- Pretty much the same as hello world

```rust
impl crate::greetings::Greetings for Greetings {
    fn greet(people: Vec<String>) -> String {
        match people.as_slice() {
            [] => "Oh, nobody's there...".to_string(),
            [person] => format!("Hello, {person}!"),
            [people @ .., last] => {
                let people = people.join(", ");
                format!("Hello, {people}, and {last}!")
            }
        }
    }
}
```

Publish to WAPM:
- Similar to before
- Probably shortened to 1 paragraph, a link, and some example output

Use the bindings:
- Create a Python project
- Use `wapm install --pip tutorial/strings-and-lists` to install the package
- Write the script
- Run it using "names" from `sys.argv`

## Conclusion

- Reminder of how helpful the guide is
- Reiterate how important your topic is
  - Strings and lists are the what applications are made of
- Call-to-action
  - Read [`WIT.md`](https://github.com/wasmerio/wit-bindgen/blob/wasmer/WIT.md)
    to learn more about the syntax

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
