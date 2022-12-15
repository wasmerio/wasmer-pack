use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Error};
use wasmer_pack_testing::Language;

fn main() -> Result<(), Error> {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = crate_root.join("Cargo.toml");
    let project_root = crate_root.ancestors().nth(2).unwrap();
    assert!(project_root.join(".git").exists());
    let target_dir = project_root.join("target");

    let wapm_package =
        wasmer_pack_testing::compile_rust_to_wapm_package(&manifest_path, &target_dir)?;

    let generated_bindings = crate_root.join("generated_bindings");

    if generated_bindings.exists() {
        std::fs::remove_dir_all(&generated_bindings)
            .context("Unable to delete the old generated bindings")?;
    }

    wasmer_pack_testing::generate_bindings(&generated_bindings, &wapm_package, Language::Python)?;

    let mut cmd = Command::new("poetry");
    let status = cmd
        .arg("run")
        .arg("pytest")
        .arg("--verbose")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Unable to run poetry. Is it installed?")?;
    anyhow::ensure!(status.success(), "The tests finished unsuccessfully.");

    Ok(())
}
