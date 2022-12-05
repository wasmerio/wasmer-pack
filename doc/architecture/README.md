# Project Architecture

The `wasmer-pack` project is split across several crates depending on the various
ways it might be used.

The main crates are:

- [`crates/wasmer-pack`][wasmer-pack] - this is the meat and potatoes of `wasmer-pack`.
  It contains all the code for generating bindings to WebAssembly modules, plus
  templates for any glue code that will be needed along the way
- [`crates/cli`][cli] - this is a CLI tool that lets `wasmer-pack` generate
  bindings using the commands and libraries inside a [Pirita][pirita] file
- [`crates/wasm`][wasm] - this is a wrapper that makes `wasmer-pack` available as a
  WebAssembly module. The functions and data types that are exposed are defined
  in [`crates/wasm/wasmer-pack.exports.wai`][exports] (see [`WIT.md`][wit] for the
  syntax)

## Architecture Decision Records

An [architectural decision record][adr] (ADR) is a document that describes a choice the
team makes about a significant aspect of the software architecture they’re
planning to build. Each ADR describes the architectural decision, its context,
and its consequences.

The goal is to get knowledge about a decision out of a developer's head so it
doesn't get lost to time.

ADRs aren't big documents - if you are writing more than a couple paragraphs,
you are probably doing it wrong!

<details>
<summary>(click to see the template)</summary>

```md
# (short title of solved problem and solution)

| Metadata | Value                                                                               |
| -------- | ----------------------------------------------------------------------------------- |
| Status   | *proposed, rejected, accepted, deprecated, superseded by [ADR-123](123-example.md)* |

## Context and Problem Statement

*(Describe the context and problem statement, e.g., in free form using two to three sentences. You may want to articulate the problem in form of a question.)*

## Decision Drivers <!-- optional -->

1. *(driver 1, e.g., a force, facing concern, …)*
2. … <!-- numbers of drivers can vary -->

## Considered Options

1. option 1
2. option 2
4. … <!-- numbers of options can vary -->

## Decision Outcome

Chosen option: "option 1", because (justification. e.g., only option, which meets k.o. criterion decision driver | which resolves force force | … | comes out best).

### Positive Consequences <!-- optional -->

- (e.g., improvement of quality attribute satisfaction, follow-up decisions required, …)
- …

### Negative Consequences <!-- optional -->

- (e.g., compromising quality attribute, follow-up decisions required, …)
- …

## Pros and Cons of the Options <!-- optional -->

### option 1

*(example | description | pointer to more information | …)* <!-- optional -->

- Good, because X
- Good, because Y
- Bad, because Z
- … <!-- numbers of pros and cons can vary -->

## Links <!-- optional -->

- []()
```
</details>

[adr]: https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions
[cli]: https://github.com/wasmerio/wasmer-pack/tree/master/crates/cli
[exports]: https://github.com/wasmerio/wasmer-pack/tree/master/crates/wasm/wasmer-pack.exports.wai
[pirita]: https://github.com/wasmerio/pirita
[wasm]: https://github.com/wasmerio/wasmer-pack/tree/master/crates/wasm
[wasmer-pack]: https://github.com/wasmerio/wasmer-pack/tree/master/crates/wasmer-pack
[wit]: https://github.com/wasmerio/wai/blob/c04723063c7a5a7389660ca97f85ffd9bc9ef0b8/WIT.md
