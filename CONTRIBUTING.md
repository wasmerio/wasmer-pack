# Contributing

## Design Goals

This project is developed under several assumptions,

- Most of the time spent using `wasmer-pack` will actually go into the IO before
  and after using it rather than *running* the code generator (i.e. because you
  have to download large inputs from a server), so **performance is a non-goal**
  for this project
- The core library should be usable by itself, but everything else can be
  tailored to Wasmer's use cases

As such, we prefer developer experience over flexibility and raw performance.

### Goal 1: Fast Compile Times

A clean build of the entire workspace shouldn't take any longer than 30 seconds
and all CI jobs should finish within 5 minutes.

This isn't actually too difficult to achieve as long as you follow some
guidelines:

- Don't add dependencies unless you absolutely need to
- Trim out unnecessary features
- Periodically use `cargo clean && cargo build --timings` to see where compile
  time is spent
- Don't import crates that pull in half of crates.io

The rationale behind this is simple - [**a short edit-compile-test cycle is a
force multiplier**][fast-rust-builds]. If you have fast compile times then
developers can recompile and re-run the test suite after every change.

On the other hand, if CI takes 30 minutes to complete, developers will avoid
your project like the plague because getting even the most trivial changes
merged becomes a multi-hour chore.

To help this, we have [a GitHub Action][workflow-timer] which will post comments
on each PR to let you know how much your changes have affected CI times.

### Goal 2: It Just Works

Users of `wasmer-pack` should be able to compile the project from scratch and
use the generated bindings without needing to mess around with dependencies or
configuration.

To achieve this,

- Avoid dependencies that link to native libraries because they need a working C
  toolchain and often require installing system libraries
- Avoid conditional compilation (including feature flags) because they tend to
  introduce edge cases that are hard to test and debug
- Be opinionated - don't give the end user unnecessary flags or options unless
  it's part of the core functionality

## Release Process

This is the process if you ever need to cut a release:

- [ ] Make a PR which mentions the new version in its title (e.g.
      *"Release v1.2.3"* on a `releases` branch)
- [ ] Update [`CHANGELOG.md`][changelog] to include any user-facing changes
      since the last release (the `[Unreleased]` link at the bottom is
      particularly helpful here)
- [ ] Run [`cargo release`][cargo-release]. This will...
     - Promote the change log's `[Unreleased]` items to a named version
     - Bump version numbers in all `Cargo.toml` files
     - Tag the commit (e.g. `v1.2.3`)
     - Publish to crates.io, and
     - Push all commits and tags to GitHub
- [ ] Wait for the [*"Releases"*][releases] job to pass. This will...
     - Publish WebAssembly binaries to WAPM
     - Use `cargo xtask set-generator` to make the WAPM backend generate
       bindings with the new version of `wasmer-pack-cli`
- [ ] Merge the *"Release v1.2.3"* PR into the `master` branch!

## Cargo xtask

We use [the `cargo xtask` pattern][xtask] for any project automation more
complex than 1 or 2 lines of shell. This means we get access to any library
on crates.io, and having everything in pure Rust means you don't need to
manually install anything or worry about OS-specific weirdness.

Currently, there are only a couple major tasks,

- `cargo xtask set-generator` calls the mutation for setting a bindings
  generator
- `cargo xtask sync-schema` will make sure the `schema.graphql` file is in sync
  with the WAPM backend, automatically updating the file if necessary

You can run `cargo xtask --help` to find out more details.

[cargo-release]: https://github.com/crate-ci/cargo-release
[changelog]: https://github.com/wasmerio/wasmer-pack/blob/master/CHANGELOG.md
[changelog]: https://github.com/wasmerio/wasmer-pack/blob/master/crates/xtask/graphql/change_generator.graphql
[fast-rust-builds]: https://matklad.github.io/2021/09/04/fast-rust-builds.html
[releases]: https://github.com/wasmerio/wasmer-pack/actions/workflows/releases.yml
[workflow-timer]: https://github.com/Michael-F-Bryan/workflow-timer
[xtask]: https://github.com/matklad/cargo-xtask
