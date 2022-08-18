/// The versions of [`wit_parser`] this crate is compatible with.
pub const WIT_PARSER_VERSION: &str = "^0.1.0";

#[cfg(test)]
mod tests {
    use cargo_metadata::MetadataCommand;
    use std::path::Path;

    fn wit_parser_version_from_cargo_toml() -> String {
        let manifest = MetadataCommand::new()
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .exec()
            .unwrap();
        let wit_pack = manifest
            .workspace_packages()
            .into_iter()
            .find(|pkg| pkg.name == env!("CARGO_PKG_NAME"))
            .unwrap();
        let wit_parser_dep = wit_pack
            .dependencies
            .iter()
            .find(|dep| dep.name == "wit-parser")
            .unwrap();

        wit_parser_dep.req.to_string()
    }

    /// Use the [self-modifying code][article] trick to make sure our
    /// [`WIT_PARSER_VERSION`] constant is kept in sync with the version in
    /// `Cargo.lock`.
    ///
    /// [article]: https://matklad.github.io/2022/03/26/self-modifying-code.html
    #[test]
    fn version_is_up_to_date() {
        let actual_version = wit_parser_version_from_cargo_toml();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("wit_version.rs");

        let wit_version_rs = std::fs::read_to_string(&path).unwrap();
        let variable = "WIT_PARSER_VERSION";
        let version_line = format!(r#"pub const {variable}: &str = "{actual_version}";"#);

        let const_decl_keyword = format!("pub const {variable}");
        if !wit_version_rs.contains(&const_decl_keyword) {
            panic!("{} should export a {variable} constant", file!());
        }

        let mut expected = String::new();
        for line in wit_version_rs.lines() {
            if line.contains(&const_decl_keyword) {
                expected.push_str(&version_line);
            } else {
                expected.push_str(line);
            }

            expected.push_str(if cfg!(windows) { "\r\n" } else { "\n" });
        }

        if wit_version_rs != expected {
            std::fs::write(&path, expected.as_bytes()).unwrap();
            panic!("The {variable} was out of date. Re-run the tests and commit the changes.");
        }
    }
}
