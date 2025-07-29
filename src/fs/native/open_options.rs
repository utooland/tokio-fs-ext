use std::{io, path::Path};

use crate::fs::File;

pub struct OpenOptions(tokio::fs::OpenOptions);

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions(tokio::fs::OpenOptions::new())
    }

    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.0.read(read);
        self
    }

    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.0.write(write);
        self
    }

    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.0.append(append);
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        self.0.truncate(truncate);
        self
    }

    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.0.create(create);
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.0.create_new(create_new);
        self
    }

    pub async fn open(&self, path: impl AsRef<Path>) -> io::Result<File> {
        let inner = self.0.open(path).await?;
        Ok(File {
            inner,
            seek_pos: None,
        })
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}
