# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Changelog entries will contain a link to the pull request implementing that
change, where applicable.

> **Note:** The project was renamed from `wit-pack` to `wasmer-pack` in version
> 0.5.0. Changelog entries from 0.4.2 and earlier use the old name.

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.5.0] - 2022-11-08

### Changed

- Switched from Wasmer's fork of `wit-bindgen` on GitHub to the `wai-bindgen`
  crate on crates.io
  ([#71](https://github.com/wasmerio/wasmer-pack/pull/71))

### Fixed

- Update `PackageName` validation to accept the `_` namespace and global
  packages ([#74](https://github.com/wasmerio/wasmer-pack/pull/74))
- Package bindings will no longer have naming conflicts when importing a binding
  type with the same name as one of types we generate
  ([#75](https://github.com/wasmerio/wasmer-pack/pull/75))

### 💥 Breaking Changed 💥

- The project has been renamed from `wit-pack` to `wasmer-pack`

## [0.4.2] - 2022-10-30

### Fixed

- Put all generated JavaScript inside a `package/` folder to match the logic
  used by `npm pack` when consuming tarballs
  ([#66](https://github.com/wasmerio/wasmer_pack/pull/66))
- Update `MANIFEST.in` to include `py.typed` in the package, meaning MyPy can
  now typecheck the generated bindings
  ([#66](https://github.com/wasmerio/wasmer_pack/pull/66))
- Don't assume atoms will have the same name as their commands
  ([#64](https://github.com/wasmerio/wasmer_pack/pull/64))
- Some JavaScript bindings wouldn't run because the bindings always import
  `@wasmer/wasi`, while the dependency was only added when one or more
  libraries/commands was compiled to WASI
  ([#58](https://github.com/wasmerio/wasmer_pack/pull/58))

## [0.4.1] - 2022-10-24

### Added

- User-facing documentation and a tutorial series are now available under the
  `doc/` folder ([#47](https://github.com/wasmerio/wasmer_pack/pull/47))
- Mention the `wit-pack` version in each generated package
  ([#54](https://github.com/wasmerio/wasmer_pack/pull/54))

### Fixed

- Fixed a bug where `*.wasm` files weren't being installed with the Python
  bindings from WAPM ([#52](https://github.com/wasmerio/wasmer_pack/pull/52))

## [0.4.0] - 2022-10-12

### Added

- To facilitate caching or different means of distribution, users are now able
  to provide their own pre-compiled WebAssembly module when initialising
  libraries or running commands ([#45](https://github.com/wasmerio/wasmer_pack/pull/45))

### Changed

- Removed the `LoadArgs` type from the Python bindings in favour of named
  arguments ([#45](https://github.com/wasmerio/wasmer_pack/pull/45))
- Raised the MSRV from `1.59.0` to `1.61.0` to
  [match `minijinja`](https://github.com/mitsuhiko/minijinja/blob/c5a09ebd/CHANGELOG.md#0210)
- Removed the top-level class from the generated bindings, so now you just need
  to do something like `from wit_pack import bindings, commands` to use the
  package's libraries or commands ([#40](https://github.com/wasmerio/wasmer_pack/pull/40))

### Fixed

- Make the current directory available to the CLI when run by wasmer
  ([#37](https://github.com/wasmerio/wasmer_pack/pull/37))

## [0.3.0] - 2022-09-27

### Added

- Set up CI to automatically deploy to wapm.dev whenever GitHub receives a
  tagged commit ([#24](https://github.com/wasmerio/wasmer_pack/pull/24))
- Fleshed out the repo's documentation ([#25](https://github.com/wasmerio/wasmer_pack/pull/25))
  - Populated the `CHANGELOG.md`
  - Wrote up `CONTRIBUTING.md`
  - Rewrote the `README.md` walkthrough
- Added a *"Time Reporter"* task to CI so we can keep an eye on CI times ([#25](https://github.com/wasmerio/wasmer_pack/pull/25))
- Generate wrappers for calling WASI executables from JavaScript
  ([#26](https://github.com/wasmerio/wasmer_pack/pull/26))
- Generate wrappers for calling WASI executables from Python
  ([#27](https://github.com/wasmerio/wasmer_pack/pull/27))
- Detect all available WASI executables in a Pirita file
  ([#28](https://github.com/wasmerio/wasmer_pack/pull/28))
- Add a top-level facade to the generated Python bindings so libraries and
  commands can be accessed through one common object
  ([#30](https://github.com/wasmerio/wasmer_pack/pull/30))
- Add a top-level facade to the generated JavaScript bindings so libraries and
  commands can be accessed through one common object
  ([#34](https://github.com/wasmerio/wasmer_pack/pull/34))
- Added a `wit-pack show` sub-command to show which libraries and commands would
  be generated from a Pirita file
  ([#35](https://github.com/wasmerio/wasmer_pack/pull/35))

## Fixed

- Inspect each atom's kind when discovering the commands in a Pirita file instead
  of blindly assuming everything is a command
  ([#32](https://github.com/wasmerio/wasmer_pack/issues/32))

## [0.2.3] - 2022-09-15

### Fixed

- When run as a WASI program, the `wit-pack` CLI would unconditionally fail
  to load inputs because `mmap` isn't available ([#24](https://github.com/wasmerio/wasmer_pack/pull/24))

## [0.2.2] - 2022-09-15

(no user-facing changes)

## [0.2.1] - 2022-09-15

(no user-facing changes)

## [0.2.0] - 2022-09-15

### Added

- The `wit-pack` crate now allows packages to contain multiple WebAssembly
  modules ([#22](https://github.com/wasmerio/wasmer_pack/pull/22))

### 💥 Breaking Changed 💥

- The `wit-pack` CLI now takes a Pirita file as its only input
  ([#20](https://github.com/wasmerio/wasmer_pack/pull/20))
  - This means the commandline interface has changed
    ```console
    # Instead of this
    $ wit-pack js --exports exports.wit --name hello_world --version 0.1.1 --module wit.wasm -o=wit-js --abi=none

    # you should now do this
    $ wit-pack js -o=wit-js ./hello-world.webc
    ```

## [0.1.5] - 2022-09-12

### Added

- Introduced support for WASI libraries ([#12](https://github.com/wasmerio/wasmer_pack/pull/12))

### Changed

- The `crates/wit-pack-cli` and `crates/wit-pack-wasm` crates are now published
  to WAPM under the `wasmer` namespace instead of `Michael-F-Bryan`

## [0.1.4] - 2022-08-25

(no user-facing changes)

## [0.1.3] - 2022-08-25

(no user-facing changes)

## [0.1.2] - 2022-08-24


<!-- next-url -->
[Unreleased]: https://github.com/wasmerio/wasmer_pack/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/wasmerio/wasmer_pack/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/wasmerio/wasmer_pack/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/wasmerio/wasmer_pack/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/wasmerio/wasmer_pack/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/wasmerio/wasmer_pack/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/wasmerio/wasmer_pack/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/wasmerio/wasmer_pack/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/wasmerio/wasmer_pack/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wasmerio/wasmer_pack/compare/v0.1.5...v0.2.0
[0.1.5]: https://github.com/wasmerio/wasmer_pack/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/wasmerio/wasmer_pack/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/wasmerio/wasmer_pack/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/wasmerio/wasmer_pack/compare/6f1e4ca6f...v0.1.2
