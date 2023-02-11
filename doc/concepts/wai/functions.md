# Functions

Functions are one of the most important concepts in WAI. They are what guests
and hosts use to expose functionality to each other, and have a name,
parameters, and results.

Here are some examples:

```
thunk: func()
fibonacci: func(n: u32) -> u32
sleep: async func(ms: u64)
```

Most guests will map functions to top-level functions, while most hosts will
expose functions as some sort of callable object which eventually calls into
the relevant part of the WebAssembly virtual machine.

For a more details, consult [the *Item: `func`* section][func] in the `*.wai`
format.

[func]: https://github.com/wasmerio/wai/blob/main/WAI.md#item-func
