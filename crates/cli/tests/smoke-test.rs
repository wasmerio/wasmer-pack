use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use url::Url;

macro_rules! codegen_test {
    (
        name: $name:ident,
        url: $url:expr,
        libraries: [$($lib:literal),* $(,)?],
        commands: [$($cmd:literal),* $(,)?],
    ) => {
        #[test]
        fn $name() {
            let temp = tempfile::tempdir();
            let local_path = cached_url($url);

            generate_bindings(&local_path, temp.path());

            // TODO: check the libraries and commands were generated
        };
    };
}

codegen_test! {
    name: wabt,
    url: "https://registry-cdn.wapm.dev/packages/wasmer/wabt/wabt-1.0.33.webc",
    libraries: ["wabt"],
    commands: ["wasm-validate"],
}

codegen_test! {
    name: wit_pack_cli,
    url: "https://registry-cdn.wapm.dev/packages/wasmer/wit-pack-cli/wit-pack-cli-0.3.0-beta.webc",
    libraries: [],
    commands: ["wit-pack"],
}

codegen_test! {
    name: wit_pack,
    url: "https://registry-cdn.wapm.dev/packages/wasmer/wit-pack/wit-pack-0.3.0-beta.webc",
    libraries: ["wit-pack"],
    commands: [],
}

fn generate_bindings(webc_file: &Path, out_dir: &Path) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wit-pack"));
    cmd.arg("js")
        .arg(&local_path)
        .arg("--out-dir")
        .arg(temp.path())
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());

    let output = cmd.output().expect("Unable to ");

    if !output.status.success() {
        eprintln!("----- STDOUT -----");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("----- STDERR -----");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("Command failed: {cmd:?}");
    }
}

fn cached_url(url: &str) -> PathBuf {
    let url: Url = url.parse().unwrap();
    let filename = url.path_segments().unwrap().last().unwrap();
    let dest = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cli-fixtures")
        .join(filename);

    if !dest.exists() {
        let response = ureq::get(url.as_str()).call().unwrap();
        assert_eq!(
            response.status(),
            200,
            "{} {}",
            response.status(),
            response.status_text(),
        );
        let mut body = Vec::new();
        response.into_reader().read_to_end(&mut body).unwrap();
        std::fs::write(&dest, &body).unwrap();
    }

    dest
}
