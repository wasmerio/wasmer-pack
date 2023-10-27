use anyhow::{Context, Error};
use std::path::{Path, PathBuf};

use cargo_wasmer::Pack;
use wasmer_pack_cli::{Codegen, Language};

pub fn compile_rust_to_wapm_package(
    manifest_path: &Path,
    out_dir: impl Into<PathBuf>,
) -> Result<PathBuf, Error> {
    let mut pack = Pack::default();
    pack.manifest.manifest_path = Some(manifest_path.to_path_buf());
    pack.out_dir = Some(out_dir.into());
    pack.debug = true;

    let meta = pack
        .metadata()
        .context("Unable to determine the package metadata")?;

    let packages = pack.resolve_packages(&meta);
    anyhow::ensure!(packages.len() == 1);

    let generated_package_dir =
        pack.generate_wasmer_package(packages[0], meta.target_directory.as_ref())?;

    Ok(generated_package_dir)
}

pub fn generate_bindings(
    dest: &Path,
    wapm_dir: &Path,
    lang: Language,
) -> Result<(), anyhow::Error> {
    tracing::info!(
        output_dir=%dest.display(),
        wapm_dir=%wapm_dir.display(),
        language=?lang,
        "Generating bindings",
    );
    let codegen = Codegen {
        out_dir: Some(dest.to_path_buf()),
        input: wapm_dir.to_path_buf(),
    };
    codegen.run(lang)?;
    Ok(())
}
