use anyhow::Context;
use std::path::Path;
use tracing_subscriber::EnvFilter;
use wasmer_pack_testing::TestEnvironment;

fn initialize_logging() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "debug");
    }

    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}

#[test]
fn hello_wasi_python() {
    initialize_logging();

    let hello_wasi_dir = project_root().join("examples").join("hello-wasi");
    let cargo_toml = hello_wasi_dir.join("Cargo.toml");

    let env = TestEnvironment::for_crate(cargo_toml, env!("CARGO_TARGET_TMPDIR"))
        .context("Unable to initialize the test environment")
        .unwrap();
    env.python(hello_wasi_dir.join("test.py"))
        .context("Python test")
        .unwrap();
}

#[test]
#[ignore = "JavaScript tests aren't fully implemented yet"]
fn hello_wasi_javascript() {
    initialize_logging();

    let hello_wasi_dir = project_root().join("examples").join("hello-wasi");
    let cargo_toml = hello_wasi_dir.join("Cargo.toml");

    let env = TestEnvironment::for_crate(cargo_toml, env!("CARGO_TARGET_TMPDIR"))
        .context("Unable to initialize the test environment")
        .unwrap();
    env.javascript(hello_wasi_dir.join("hello-wasi.test.mjs"))
        .context("JavaScript test")
        .unwrap();
}

fn project_root() -> &'static Path {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_dir.ancestors().nth(1).unwrap();
    assert!(project_root.join(".git").exists());
    project_root
}
