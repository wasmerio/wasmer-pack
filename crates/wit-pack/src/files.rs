use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
    path::{Path, PathBuf},
};

use anyhow::{Context, Error};

/// A set of files loaded into memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Files {
    members: BTreeMap<PathBuf, SourceFile>,
}

impl Files {
    pub fn new() -> Self {
        Files {
            members: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, path: impl Into<PathBuf>, file: SourceFile) {
        self.members.insert(path.into(), file);
    }

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

    pub fn get_mut(&mut self, path: impl AsRef<Path>) -> Option<&mut SourceFile> {
        self.members.get_mut(path.as_ref())
    }
}

impl Default for Files {
    fn default() -> Self {
        Files::new()
    }
}

impl IntoIterator for Files {
    type Item = (PathBuf, SourceFile);

    type IntoIter = <BTreeMap<PathBuf, SourceFile> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.members.into_iter()
    }
}

/// A file in memory.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct SourceFile(pub Vec<u8>);

impl SourceFile {
    pub fn new(contents: Vec<u8>) -> Self {
        SourceFile(contents)
    }

    pub fn contents(&self) -> &[u8] {
        &self.0
    }

    pub fn utf8_contents(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }
}

impl From<&str> for SourceFile {
    fn from(s: &str) -> Self {
        SourceFile::from(s.to_string())
    }
}

impl From<String> for SourceFile {
    fn from(s: String) -> Self {
        SourceFile::new(s.into_bytes())
    }
}

impl From<&[u8]> for SourceFile {
    fn from(v: &[u8]) -> Self {
        SourceFile::from(v.to_vec())
    }
}

impl From<&Vec<u8>> for SourceFile {
    fn from(v: &Vec<u8>) -> Self {
        SourceFile::from(v.clone())
    }
}

impl From<Vec<u8>> for SourceFile {
    fn from(v: Vec<u8>) -> Self {
        SourceFile::new(v)
    }
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let SourceFile(contents) = self;

        f.debug_tuple("SourceFile")
            .field(
                self.utf8_contents()
                    .as_ref()
                    .map(|c| c as &dyn Debug)
                    .unwrap_or(contents),
            )
            .finish()
    }
}
