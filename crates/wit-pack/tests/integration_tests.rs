use std::{
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use wit_pack::Bindings;

#[test]
#[ignore = "Requires injecting WASI imports"]
fn use_javascript_bindings() {
    let Fixtures { exports, wasm } = Fixtures::load();
    let bindings = Bindings::from_disk(&exports, &wasm).unwrap();

    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("javascript");
    let _ = std::fs::remove_dir_all(&out_dir);

    let js = bindings.javascript().unwrap();
    js.save_to_disk(&out_dir).unwrap();

    let js_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("index.js");

    execute(Command::new("node").arg(&js_file));
}

#[derive(Debug)]
struct Fixtures {
    exports: PathBuf,
    wasm: PathBuf,
}

impl Fixtures {
    fn load() -> Self {
        let project_root = project_root();

        let exports = project_root.join("crates").join("wasm").join("exports.wit");
        assert!(exports.exists());

        execute(
            Command::new(env!("CARGO"))
                .arg("build")
                .arg("--target=wasm32-wasi")
                .arg("--package=wit-pack-wasm")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        );

        let wasm = project_root
            .join("target")
            .join("wasm32-wasi")
            .join("debug")
            .join("wit_pack_wasm.wasm");

        Fixtures { exports, wasm }
    }
}

fn execute(cmd: &mut Command) {
    let Output {
        status,
        stdout,
        stderr,
    } = cmd.output().unwrap();

    if !status.success() {
        let stdout = String::from_utf8_lossy(&stdout);
        if !stdout.is_empty() {
            println!("----- Stdout -----");
            println!("{stdout}");
        }
        let stderr = String::from_utf8_lossy(&stderr);
        if !stderr.is_empty() {
            println!("----- Stderr -----");
            println!("{stderr}");
        }
        panic!("Command failed: {cmd:?}");
    }
}

fn project_root() -> PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.ancestors().nth(2).unwrap();
    assert!(root.join(".git").exists());
    root.to_path_buf()
}
