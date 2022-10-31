# Records

## Intro

- A promise statement
  - We're going to introduce record types, often called "structs" in other
    languages
  - Lets you pass data from the guest to the host
- A preview of what's to come

## Body

Defining our WIT file:

```rust
// records.wai

record person {
    name: string,
    age: u32,
}

/// Describe a person.
describe: func(person: person) -> string
```

Create the guest and publish to WAPM:

```rs
use crate::records::Person;

impl crate::records::Records for Records {
    fn describe(person: Person) -> String {
        let Person { name, age } = person;

        match age {
            1 => format!("{name} is 1 year old"),
            _ => format!("{name} is {age} years old"),
        }
    }
}
```

Use the bindings:
- Create a JavaScript project
- `wapm install --yarn tutorial/records`
- Write a script

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
