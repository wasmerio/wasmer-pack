# Contributing

## Design Goals

This project is developed under several assumptions,

- Most of the time spent using `wit-pack` will actually go into the IO before
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

Users of `wit-pack` should be able to compile the project from scratch and use
the generated bindings without needing to mess around with dependencies or
configuration.

To achieve this,

- Avoid dependencies that link to native libraries because they need a working C
  toolchain and often require installing system libraries
- Avoid conditional compilation (including feature flags) because they tend to
  introduce edge cases that are hard to test and debug
- Be opinionated - don't give the end user unnecessary flags or options unless
  it's part of the core functionality

## Release Process

The checklist:

- [ ] Create a new PR named something like *"Release v1.2.3"*
- [ ] Update [`CHANGELOG.md`][changelog] to include any user-facing changes
      since the last release (the `[Unreleased]` link at the bottom is
      particularly helpful here)
- [ ] Run [`cargo release`][cargo-release] to bump version numbers, tag commits,
      and push changes to GitHub
- [ ] Make sure the [*"Releases"*][releases] job has passed, and new binaries
      were published to WAPM
- [ ] Merge the *"Release v1.2.3"* PR in


[cargo-release]: https://github.com/crate-ci/cargo-release
[changelog]: https://github.com/wasmerio/wasmer_pack/blob/master/CHANGELOG.md
[fast-rust-builds]: https://matklad.github.io/2021/09/04/fast-rust-builds.html
[releases]: https://github.com/wasmerio/wasmer_pack/actions/workflows/releases.yml
[workflow-timer]: https://github.com/Michael-F-Bryan/workflow-timer
