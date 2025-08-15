use std::{io, path::Path};

use super::{create_dir, create_dir_all};

#[derive(Debug, Default)]
pub struct DirBuilder {
    recursive: bool,
}

impl DirBuilder {
    pub fn new() -> Self {
        DirBuilder::default()
    }

    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    pub async fn create(&self, path: impl AsRef<Path>) -> io::Result<()> {
        if self.recursive {
            create_dir_all(path).await?;
        } else {
            create_dir(path).await?;
        }
        Ok(())
    }
}
