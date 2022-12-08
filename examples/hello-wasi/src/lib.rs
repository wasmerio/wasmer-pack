wai_bindgen_rust::export!("hello-wasi.export.wai");

struct HelloWasi;

impl hello_wasi::HelloWasi for HelloWasi {
    fn print_hello_wasi() {
        println!("Hello, WASI!");
    }
}
