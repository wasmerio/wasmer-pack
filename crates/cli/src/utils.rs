use std::path::Path;

use anyhow::{Context, Error};
use wasmer_pack::Package;
use webc::Container;

pub(crate) fn load(path: &Path) -> Result<Package, Error> {
    load_container(path)
        .and_then(|webc| Package::from_webc(&webc))
        .with_context(|| format!("Unable to load the package from \"{}\"", path.display()))
}

fn load_container(path: &Path) -> Result<Container, Error> {
    let err = match Container::from_disk(path) {
        Ok(c) => return Ok(c),
        Err(e) => e,
    };

    if path.is_dir() {
        // They might be using the old wapm.toml filename instead of the
        // wasmer.toml that Container::from_disk() expects.
        let wapm_toml = path.join("wapm.toml");
        if let Ok(pkg) = webc::wasmer_package::Package::from_manifest(wapm_toml) {
            return Ok(Container::from(pkg));
        }
    }

    Err(err.into())
}
