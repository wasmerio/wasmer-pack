use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Error};
use wapm_targz_to_pirita::{generate_webc_file, webc::v1::DirOrFile, TransformManifestFunctions};
use wasmer_pack::Package;

pub(crate) fn load_from_disk(path: &Path) -> Result<Package, Error> {
    let raw_webc: Vec<u8> = if path.is_dir() {
        webc_from_dir(path)?
    } else if path.extension() == Some("webc".as_ref()) {
        std::fs::read(path).with_context(|| format!("Unable to read \"{}\"", path.display()))?
    } else {
        let tarball = std::fs::read(path)
            .with_context(|| format!("Unable to read \"{}\"", path.display()))?;
        webc_from_tarball(tarball)?
    };

    Package::from_webc(&raw_webc)
}

fn webc_from_dir(path: &Path) -> Result<Vec<u8>, Error> {
    if !path.join("wapm.toml").exists() {
        anyhow::bail!(
            "The \"{}\" directory doesn't contain a \"wapm.tom\" file",
            path.display()
        );
    }

    let mut files: BTreeMap<DirOrFile, Vec<u8>> = BTreeMap::new();

    fn read_dir(
        files: &mut BTreeMap<DirOrFile, Vec<u8>>,
        dir: &Path,
        base_dir: &Path,
    ) -> Result<(), Error> {
        let entries = dir
            .read_dir()
            .with_context(|| format!("Unable to read the contents of \"{}\"", dir.display()))?;

        for entry in entries {
            let path = entry?.path();
            let relative_path = path
                .strip_prefix(base_dir)
                .expect("The filename is always prefixed by base_dir")
                .to_path_buf();

            if path.is_dir() {
                read_dir(&mut *files, &path, base_dir)?;
                files.insert(DirOrFile::Dir(relative_path), Vec::new());
            } else {
                let bytes = std::fs::read(&path)
                    .with_context(|| format!("Unable to read \"{}\"", path.display()))?;
                files.insert(DirOrFile::File(relative_path), bytes);
            }
        }

        Ok(())
    }

    read_dir(&mut files, path, path).context("Unable to read the directory into memory")?;

    let functions = wapm_targz_to_pirita::TransformManifestFunctions::default();
    let tarball = generate_webc_file(files, path, &functions)
        .context("Unable to convert the files to a tarball")?;

    Ok(tarball)
}

pub(crate) fn webc_from_tarball(tarball: Vec<u8>) -> Result<Vec<u8>, Error> {
    let files =
        wapm_targz_to_pirita::unpack_tar_gz(tarball).context("Unable to unpack the tarball")?;

    wapm_targz_to_pirita::generate_webc_file(
        files,
        Path::new("."),
        &TransformManifestFunctions::default(),
    )
}
