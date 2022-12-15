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
