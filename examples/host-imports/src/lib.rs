wai_bindgen_rust::import!("fs.import.wai");
wai_bindgen_rust::import!("logging.import.wai");
wai_bindgen_rust::export!("main.export.wai");

struct Main;

impl main::Main for Main {
    fn start() {
        let text = fs::read_file("some-file.txt");

        let msg = format!("Read {} bytes", text.len());
        logging::log(&msg);
    }
}
