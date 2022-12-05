# Generated Package Naming

| Metadata | Value    |
| -------- | -------- |
| Status   | proposed |

## Context and Problem Statement

Currently, generated packages will derive their name from the WAPM package name,
so `wasmer/wasmer-pack-cli` gets turned into `wasmer_pack_cli` for Python and
`@wasmer/wasmer-pack-cli` for JavaScript.

Eventually, we'd like WAPM to automatically publish these packages to PyPI or
NPM, so we need to come up with names that are unique.

## Decision Drivers

1. Negligible chance of collisions
2. Generated package names are similar to the package on WAPM

## Considered Options

1. Put all packages under a `@wasmer-package` organisation and use `__` for
   delimiters
2. Host our own private registries

## Decision Outcome

TODO: make a decision

## Pros and Cons of the Options

### Option 1

- Good, because it's practically guaranteed to not have collisions
- Good, because we can publish to PyPI/NPM and be used by other packages
- Good because there is an obvious way to transform a package name back and forth
- Bad, because the names become very verbose and unwieldy
  - `wasmer/wasmer-pack` becomes `wasmer_package__wasmer__wasmer_pack` on Python
    and `@wasmer-package/wasmer__wasmer-pack` on JavaScript

### Option 2

- Good, because we don't have to worry about colliding with existing packages
- Good, because we get complete control over the registry
- Bad, because it's more infrastructure to manage (operations costs, expertise,
  ,
  etc.)
- Bad, because most package managers don't let you publish packages that depend
  on something from another registry

## Links

- Original ticket - [wasmerio/wasmer-pack#100](https://github.com/wasmerio/wasmer-pack/issues/100)
