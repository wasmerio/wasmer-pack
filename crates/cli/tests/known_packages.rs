use assert_cmd::Command;
use flate2::read::GzDecoder;
use std::{
    collections::BTreeSet,
    fs::File,
    path::{Path, PathBuf},
};
use tar::Archive;
use tempfile::TempDir;

use url::Url;

const WIT_PACK_TARBALL: &str =
    "https://registry-cdn.wapm.dev/packages/wasmer/wit-pack/wit-pack-0.3.0-beta.tar.gz";

/// Download a WEBC package and make sure it would contain the expected
/// libraries and commands.
///
/// We'll also try to generate the JavaScript bindings for good measure.
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
    name: wabt,
    url: "https://registry-cdn.wapm.dev/packages/wasmer/wabt/wabt-1.0.33.webc",
    libraries: ["bindings"],
    commands: [
        "wat2wasm", "wast2json", "wasm2wat", "wasm-interp", "wasm-validate", "wasm-strip",
    ],
}

codegen_test! {
    name: wasmer_pack_cli,
    url: "https://registry-cdn.wapm.dev/packages/wasmer/wit-pack-cli/wit-pack-cli-0.3.0-beta.webc",
    libraries: [],
    commands: ["wit-pack"],
}

codegen_test! {
    name: wasmer_pack,
    url: "https://registry-cdn.wapm.dev/packages/wasmer/wit-pack/wit-pack-0.3.0-beta.webc",
    libraries: ["wit-pack"],
    commands: [],
}

codegen_test! {
    name: wasmer_pack_tarball,
    url: WIT_PACK_TARBALL,
    libraries: ["wit-pack"],
    commands: [],
}

#[test]
fn load_a_package_from_a_directory() {
    let temp = TempDir::new().unwrap();
    let tarball = cached_url(WIT_PACK_TARBALL);

    let reader = File::open(tarball).unwrap();
    let reader = GzDecoder::new(reader);
    let mut archive = Archive::new(reader);
    archive.unpack(temp.path()).unwrap();

    let meta = metadata(temp.path());

    insta::assert_display_snapshot!(format!("{meta:#}"));
    assert_contains_libraries_and_commands(&meta, &[], &["asdf"]);
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
    let result = Command::cargo_bin("wasmer-pack")
        .unwrap()
        .arg("show")
        .arg("--format=json")
        .arg(webc_file)
        .assert()
        .success();
    let output = result.get_output();
    serde_json::from_slice(&output.stdout).unwrap()
}

fn generate_bindings(webc_file: &Path, out_dir: &Path) {
    Command::cargo_bin("wasmer-pack")
        .unwrap()
        .arg("js")
        .arg(webc_file)
        .arg("--out-dir")
        .arg(out_dir)
        .assert()
        .success();
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
