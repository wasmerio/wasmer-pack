use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use wit_pack::{Abi, Interface, Metadata, Module};

#[test]
#[ignore]
fn use_javascript_bindings() {
    let Fixtures { exports, wasm } = Fixtures::load();

    let metadata = Metadata::new("@wasmer/wit-pack", env!("CARGO_PKG_VERSION"));
    let module = Module::from_path(&wasm, Abi::None).unwrap();
    let interface = Interface::from_path(&exports).unwrap();

    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("javascript");
    let _ = std::fs::remove_dir_all(&out_dir);

    let js = wit_pack::generate_javascript(&metadata, &module, &interface).unwrap();
    js.save_to_disk(&out_dir).unwrap();

    let js_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("js");

    let _ = std::fs::remove_dir_all(js_dir.join("node_modules"));

    execute("yarn", &js_dir);
    execute("yarn start", &js_dir);
}

#[test]
#[ignore]
fn use_wasi_javascript_bindings() {
    let Fixtures { exports, wasm } = Fixtures::load_wasi();

    let metadata = Metadata::new("wabt", env!("CARGO_PKG_VERSION"));
    let module = Module::from_path(&wasm, Abi::Wasi).unwrap();
    let interface = Interface::from_path(&exports).unwrap();

    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("javascript-wasi");
    let _ = std::fs::remove_dir_all(&out_dir);

    let js = wit_pack::generate_javascript(&metadata, &module, &interface).unwrap();
    js.save_to_disk(&out_dir).unwrap();

    let js_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("js-wasi");

    let _ = std::fs::remove_dir_all(js_dir.join("node_modules"));

    execute("yarn", &js_dir);
    execute("yarn start", &js_dir);
}

#[test]
#[ignore]
fn use_python_bindings() {
    let Fixtures { exports, wasm } = Fixtures::load();

    let metadata = Metadata::new("wit_pack", env!("CARGO_PKG_VERSION"));
    let module = Module::from_path(&wasm, Abi::None).unwrap();
    let interface = Interface::from_path(&exports).unwrap();

    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("python");
    let _ = std::fs::remove_dir_all(&out_dir);

    let py = wit_pack::generate_python(&metadata, &module, &interface).unwrap();
    py.save_to_disk(&out_dir).unwrap();

    let python_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("python");

    execute("pip install -r requirements.txt", &python_dir);
    execute("python3 main.py", &python_dir);
}

#[test]
fn use_wasi_python_bindings() {
    let Fixtures { exports, wasm } = Fixtures::load_wasi();

    let metadata = Metadata::new("wabt", env!("CARGO_PKG_VERSION"));
    let module = Module::from_path(&wasm, Abi::Wasi).unwrap();
    let interface = Interface::from_path(&exports).unwrap();

    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("python-wasi");
    let _ = std::fs::remove_dir_all(&out_dir);

    let py = wit_pack::generate_python(&metadata, &module, &interface).unwrap();
    py.save_to_disk(&out_dir).unwrap();

    let python_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("python-wasi");

    execute("pip install -r requirements.txt", &python_dir);
    execute("python3 main.py", &python_dir);
}

#[derive(Debug)]
struct Fixtures {
    exports: PathBuf,
    wasm: PathBuf,
}

impl Fixtures {
    fn load() -> Self {
        let project_root = project_root();

        let exports = project_root
            .join("crates")
            .join("wasm")
            .join("wit-pack.exports.wit");
        assert!(exports.exists());

        execute(
            "cargo build --target=wasm32-unknown-unknown --package=wit-pack-wasm",
            &project_root,
        );

        let wasm = project_root
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("debug")
            .join("wit_pack_wasm.wasm");

        Fixtures { exports, wasm }
    }

    // fn load_wasi() -> Self {
    //     let project_root = project_root();

    //     let exports = project_root
    //         .join("crates")
    //         .join("wasm")
    //         .join("wit-pack.exports.wit");
    //     assert!(exports.exists());

    //     execute(
    //         "RUSTFLAGS=\"-Z wasi-exec-model=reactor\" cargo +nightly build --target=wasm32-wasi --package=wit-pack-wasm",
    //         &project_root,
    //     );

    //     let wasm = project_root
    //         .join("target")
    //         .join("wasm32-wasi")
    //         .join("debug")
    //         .join("wit_pack_wasm.wasm");

    //     Fixtures { exports, wasm }
    // }

    fn load_wasi() -> Self {
        let project_root = project_root();

        let exports = project_root
            .join("crates")
            .join("wit-pack")
            .join("tests")
            .join("wabt")
            .join("wabt.exports.wit");
        assert!(exports.exists());

        let wasm = project_root
            .join("crates")
            .join("wit-pack")
            .join("tests")
            .join("wabt")
            .join("libwabt.wasm");

        Fixtures { exports, wasm }
    }
}

#[track_caller]
fn execute(command: impl AsRef<OsStr>, current_dir: impl AsRef<Path>) {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("cmd.exe");
        cmd.arg("/c");
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd
    };

    cmd.arg(command.as_ref()).current_dir(current_dir);

    assert_runs_successfully(&mut cmd);
}

#[track_caller]
fn assert_runs_successfully(cmd: &mut Command) {
    let Output {
        status,
        stdout,
        stderr,
    } = cmd
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .expect("Unable to start the process");
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
