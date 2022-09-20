# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- Set up CI to automatically deploy to wapm.dev whenever GitHub receives a
  tagged commit ([#24](https://github.com/wasmerio/wit-pack/pull/24))
- Fleshed out the repo's documentation ([#25](https://github.com/wasmerio/wit-pack/pull/25))
  - Populated the `CHANGELOG.md`
  - Wrote up `CONTRIBUTING.md`
  - Rewrote the `README.md` walkthrough
- Added a *"Time Reporter"* task to CI so we can keep an eye on CI times ([#25](https://github.com/wasmerio/wit-pack/pull/25))
- Generate wrappers for calling WASI executables from JavaScript
  ([#26](https://github.com/wasmerio/wit-pack/pull/26)) and Python
  ([#27](https://github.com/wasmerio/wit-pack/pull/27))
- Detect all available WASI executables in a Pirita file
  ([#28](https://github.com/wasmerio/wit-pack/pull/28))

# [0.2.3] - 2022-09-15

## Fixed

- When run as a WASI program, the `wit-pack` CLI would unconditionally fail
  to load inputs because `mmap` isn't available ([#24](https://github.com/wasmerio/wit-pack/pull/24))

# [0.2.2] - 2022-09-15

(no user-facing changes)

# [0.2.1] - 2022-09-15

(no user-facing changes)

# [0.2.0] - 2022-09-15

## Added

- The `wit-pack` crate now allows packages to contain multiple WebAssembly
  modules ([#22](https://github.com/wasmerio/wit-pack/pull/22))

## Changed

- The `wit-pack` CLI now takes a Pirita file as input ([#20](https://github.com/wasmerio/wit-pack/pull/20))

# [0.1.5] - 2022-09-12

## Added

- Introduced support for WASI libraries ([#12](https://github.com/wasmerio/wit-pack/pull/12))

## Changed

- The `crates/wit-pack-cli` and `crates/wit-pack-wasm` crates are now published
  to WAPM under the `wasmer` namespace instead of `Michael-F-Bryan`

# [0.1.4] - 2022-08-25

(no user-facing changes)

# [0.1.3] - 2022-08-25

(no user-facing changes)

# [0.1.2] - 2022-08-24


<!-- next-url -->
[Unreleased]: https://github.com/wasmerio/wit-pack/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/wasmerio/wit-pack/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/wasmerio/wit-pack/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/wasmerio/wit-pack/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wasmerio/wit-pack/compare/v0.1.5...v0.2.0
[0.1.5]: https://github.com/wasmerio/wit-pack/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/wasmerio/wit-pack/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/wasmerio/wit-pack/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/wasmerio/wit-pack/compare/6f1e4ca6f...v0.1.2
