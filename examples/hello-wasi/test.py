from hello_wasi import bindings, version


def test_expected_items_are_generated():
    # Top-level items
    assert callable(bindings.hello_wasi)
    assert version == "0.0.0"

    # Our WebAssembly library
    wasm = bindings.hello_wasi()
    assert callable(wasm.print_hello_wasi)
