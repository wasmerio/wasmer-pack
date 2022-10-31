# Ecosystem Overview

The WebAssembly ecosystem has quite a few moving parts, so let's touch on some
of them briefly before diving into WIT Pack.

## WebAssembly

To quote [the WebAssembly website][website],

> WebAssembly (abbreviated Wasm) is a binary instruction format for a
> stack-based virtual machine. Wasm is designed as a portable compilation target
> for programming languages, enabling deployment on the web for client and
> server applications.

This provides a way to run code written in multiple programming languages on a
wide range of platforms - whether that is in a browser, the desktop, mobile
devices, or even embedded devices.

WebAssembly has a couple key concepts,

- **Module:** A WebAssembly binary that has been compiled into executable
  machine code
- **Instance:** A module paired with a copy of its own runtime state (the stack,
  memory, etc.) and isolated from the outside world
- **Linear Memory:** A resizeable byte buffer that an instance uses as its main
  memory ("RAM")
- **Guest:** The WebAssembly code being executed
- **Host:** An application which is executing WebAssembly code. The host is
  responsible for loading a WebAssembly guest into memory and providing it
  access to the outside world as it runs
- **Imports:** Functions and variables that the guest needs to be provided in
  order for it to run
- **Exports:** Functions and variables that the guest has made available to the
  host
- **WASI:** Short for the WebAssembly System Interface. A set of standardised
  imports that a WebAssembly guest can use to emulate a POSIX-like API, allowing
  existing programs to be ported to WebAssembly and have access to things like
  the file system, command-line arguments, and so on.

One of the biggest design goals for WebAssembly is for the code to be secure by
default because browsers will be routinely downloading and executing untrusted
WebAssembly.

This causes an asymmetry between the guest and the host, where the host has
almost complete access to the guest's state at runtime, but the guest can't
access anything from the host.

In particular, it is worth pointing out that

- The guest can't reference host memory, so any information the guest needs
  access to will need to be copied into its linear memory by the host
- The only way a guest can interact with the outside world is via functions the
  host provides
- Multiple guests running on the same host can't interact with each other unless
  the host allows it (e.g. by using the exports from one instance as imports
  for another)

## WAPM

The [*WebAssembly Package Manager*][wapm] (WAPM) is a package manager for
compiled WebAssembly packages.

Similar to other package managers, WAPM makes it easy for developers to publish
their WebAssembly packages, as well as distribute those packages or allow them
to be used by others.

A WAPM package can contain as many WASI executables and libraries as you wish.

## WAI Bindgen

The WebAssembly spec that was first released in 2017 was only a minimum viable
product. It deliberately left several features incomplete to be iterated on by
the ecosystem.

Arguably the most important functionality gap is the fact that only WebAssembly
primitives can be passed between the host guest. That means imports and exports
can only use the following data types,

- `i32` - signed 32-bit integers
- `i64` - signed 64-bit integers
- `f32` - a 32-bit float
- `f64` - a 64-bit float (often called a `double`)
- `funcref` - a reference to a WebAssembly function

You'll notice this list doesn't even include strings or boolean values!

The [`wai-bindgen`][wai-bindgen] project provides a polyfill for passing
around higher-level objects. It lets developers define their imports and exports
in a `*.wai` file, then using `wai-bindgen` to generate glue which automagically
passes things around within the constraints of WebAssembly.

We'll be getting *very* familiar with WIT files through the coming chapters, but
the full definition for the WIT file format is available [on GitHub][wit].

## WIT Pack

WIT Pack is a tool that integrates `wai-bindgen` with the WAPM ecosystem.

Each library in a package has the option of declaring `*.wai` files containing
functionality it imports and exports.

- A tool that integrates WIT Bindgen with WAPM
- You can create a package that is associated with a `*.wai` file
- When that package is published to WAPM, bindings for your favourite language
  are automatically generated


[wapm]: https://wapm.io/
[website]: https://webassembly.org/
[wai-bindgen]: https://github.com/bytecodealliance/wai-bindgen
[wit]: https://github.com/WebAssembly/component-model/blob/5754989219db51ba24def50c3ac28bb9775ead33/design/mvp/WIT.md
