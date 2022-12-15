from calc import bindings

def test_2_plus_2():
    wasm = bindings.calc()

    assert wasm.add(2.0, 2.0) == 4.0
