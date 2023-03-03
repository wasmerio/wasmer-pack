wai_bindgen_rust::import!("fs.import.wai");
wai_bindgen_rust::import!("logging.import.wai");
wai_bindgen_rust::export!("host-imports.export.wai");

struct HostImports;

impl host_imports::HostImports for HostImports {
    fn start() {
        let path = "some-file.txt";
        let text = fs::read_file(path);

        let msg = format!("Read {} bytes from {path}: {text}", text.len());
        logging::log(&msg);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
mod non_wasm_externs {
    // HACK: Normally, we would only ever compile this library to WebAssembly,
    // however when you run "cargo build --workspace" it'll try to compile for
    // the host architecture. This will cause linker errors on Windows because
    // we expect the host functions to have been provided.
    //
    // The correct solution would be to set
    // `package.forced-target = "wasm32-unknown-unknown"` in Cargo.toml, but
    // that isn't stable yet.
    // (https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#per-package-target)

    #[no_mangle]
    extern "C" fn logging_log(_: i32, _: i32) {
        std::process::abort();
    }

    #[no_mangle]
    #[export_name = "fs_read-file"]
    extern "C" fn fs_read_file(_: i32, _: i32, _: i32) {
        std::process::abort();
    }
}
