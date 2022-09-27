use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use url::Url;

macro_rules! codegen_test {
    (
        $( #[$meta:meta] )*
        name: $name:ident,
        url: $url:expr,
        libraries: [$($lib:literal),* $(,)?],
        commands: [$($cmd:literal),* $(,)?],
    ) => {
        #[test]
        $( #[$meta] )*
        fn $name() {
            let temp = tempfile::tempdir().unwrap();
            let local_path = cached_url($url);

            let meta = metadata(&local_path);

            // Make sure we detect the correct commands and libraries
            insta::assert_display_snapshot!(format!("{meta:#}"));
            assert_contains_libraries_and_commands(&meta, &[$($lib),*], &[$($cmd),*]);

            // Make sure the binding generation doesn't fail
            generate_bindings(&local_path, temp.path());
        }
    };
}

codegen_test! {
    #[ignore]
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
    commands: ["wit-pack-wasm"],
}

fn assert_contains_libraries_and_commands(
    meta: &serde_json::Value,
    libraries: &[&str],
    commands: &[&str],
) {
    let expected_libraries: BTreeSet<&str> = libraries.iter().copied().collect();
    let actual_libraries: BTreeSet<&str> = meta["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|obj| obj["interface_name"].as_str().unwrap())
        .collect();

    assert_eq!(actual_libraries, expected_libraries);

    let expected_commands: BTreeSet<&str> = commands.iter().copied().collect();
    let actual_commands: BTreeSet<&str> = meta["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|obj| obj["name"].as_str().unwrap())
        .collect();

    assert_eq!(actual_commands, expected_commands);
}

fn metadata(webc_file: &Path) -> serde_json::Value {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wit-pack"));
    cmd.arg("show")
        .arg("--format=json")
        .arg(&webc_file)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());

    let output = cmd.output().expect("Unable to invoke wit-pack");

    if !output.status.success() {
        eprintln!("----- STDOUT -----");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout).trim());
        eprintln!("----- STDERR -----");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr).trim());
        panic!("Command failed: {cmd:?}");
    }

    serde_json::from_slice(&output.stdout).expect("Unable to deserialize the metadata")
}

fn generate_bindings(webc_file: &Path, out_dir: &Path) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wit-pack"));
    cmd.arg("js")
        .arg(&webc_file)
        .arg("--out-dir")
        .arg(out_dir)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());

    let output = cmd.output().expect("Unable to invoke wit-pack");

    if !output.status.success() {
        eprintln!("----- STDOUT -----");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout).trim());
        eprintln!("----- STDERR -----");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr).trim());
        panic!("Command failed: {cmd:?}");
    }
}

fn cached_url(url: &str) -> PathBuf {
    let url: Url = url.parse().unwrap();
    let filename = url.path_segments().unwrap().last().unwrap();
    let fixtures_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-fixtures");
    let dest = fixtures_dir.join(filename);

    std::fs::create_dir_all(&fixtures_dir).unwrap();

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
