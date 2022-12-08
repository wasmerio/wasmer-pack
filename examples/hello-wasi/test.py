import hello_wasi


def test_expected_items_are_generated():
    # Top-level items
    assert callable(hello_wasi.bindings.hello_wasi)

    # Our WebAssembly library
    wasm = hello_wasi.bindings.hello_wasi()
    assert callable(wasm.print_hello_wasi)
