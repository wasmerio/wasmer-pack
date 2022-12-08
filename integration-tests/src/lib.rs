#[cfg(test)]
mod tests {
    use std::path::Path;
    use wasmer_pack_testing::TestEnvironment;

    #[test]
    fn hello_wasi() {
        let hello_wasi_dir = project_root().join("examples").join("hello-wasi");
        let cargo_toml = hello_wasi_dir.join("Cargo.toml");

        let env = TestEnvironment::for_crate(cargo_toml).unwrap();
        env.python(hello_wasi_dir.join("test.py")).unwrap();
    }

    fn project_root() -> &'static Path {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let project_root = crate_dir.ancestors().nth(1).unwrap();
        assert!(project_root.join(".git").exists());
        project_root
    }
}
