/// The versions of [`wai_parser`] this crate is compatible with.
pub const WAI_PARSER_VERSION: &str = include_str!("wai_version.txt");

#[cfg(test)]
mod tests {
    use cargo_metadata::MetadataCommand;
    use std::path::Path;

    fn wai_parser_version_from_cargo_toml() -> String {
        let manifest = MetadataCommand::new()
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .exec()
            .unwrap();
        let wasmer_pack = manifest
            .workspace_packages()
            .into_iter()
            .find(|pkg| pkg.name == env!("CARGO_PKG_NAME"))
            .unwrap();
        let wai_parser_dep = wasmer_pack
            .dependencies
            .iter()
            .find(|dep| dep.name == "wai-parser")
            .unwrap();

        wai_parser_dep.req.to_string()
    }

    /// Use the [self-modifying code][article] trick to make sure our
    /// [`wai_PARSER_VERSION`] constant is kept in sync with the version in
    /// `Cargo.lock`.
    ///
    /// [article]: https://matklad.github.io/2022/03/26/self-modifying-code.html
    #[test]
    fn version_is_up_to_date() {
        let actual_version = wai_parser_version_from_cargo_toml();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("wai_version.txt");

        let wai_version_txt = std::fs::read_to_string(&path).unwrap();

        if wai_version_txt != actual_version {
            std::fs::write(&path, actual_version).unwrap();
            panic!(
                "The WAI parser version was out of date. Re-run the tests and commit the changes."
            );
        }
    }
}
