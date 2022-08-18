use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Error};

/// A set of files loaded into memory.
#[derive(Debug, Clone, PartialEq)]
pub struct Directory {
    members: BTreeMap<PathBuf, SourceFile>,
}

impl Directory {
    pub fn iter(&self) -> impl Iterator<Item = (&Path, &SourceFile)> + '_ {
        self.members.iter().map(|(k, v)| (k.as_path(), v))
    }

    pub fn save_to_disk(&self, output_dir: impl AsRef<Path>) -> Result<(), Error> {
        let output_dir = output_dir.as_ref();

        for (path, file) in self.iter() {
            let path = output_dir.join(path);

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Unable to create the \"{}\" directory", parent.display())
                })?;
            }

            std::fs::write(&path, file.contents())
                .with_context(|| format!("Unable to save to \"{}\"", path.display()))?;
        }

        Ok(())
    }
}

/// A file in memory.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceFile {
    contents: Arc<[u8]>,
}

impl SourceFile {
    pub fn contents(&self) -> &[u8] {
        &self.contents
    }

    pub fn utf8_contents(&self) -> Option<&str> {
        std::str::from_utf8(&self.contents).ok()
    }
}
