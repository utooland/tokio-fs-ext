use bitflags::bitflags;
use futures::io::AsyncSeekExt;
use std::{io, path::Path};

use crate::fs::{
    File,
    opfs::{CreateFileMode, SyncAccessMode, open_file},
};

bitflags! {
    #[derive(Clone , Debug, Copy)]
    struct Flags: u8 {
        const READ = 1 << 0;
        const WRITE= 1 << 1;
        const APPEND = 1 << 2;
        const CREATE = 1 << 3;
        const TRUNCATE = 1 << 4;
        const CREATE_NEW = 1 << 5;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Flags::READ
    }
}

#[derive(Clone, Default, Debug, Copy)]
pub struct OpenOptions(Flags);

impl OpenOptions {
    pub fn new() -> OpenOptions {
        Default::default()
    }

    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        if read {
            self.0 |= Flags::READ;
        } else {
            self.0.remove(Flags::READ);
        }
        self
    }

    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        if write {
            self.0 |= Flags::WRITE;
        } else {
            self.0.remove(Flags::WRITE)
        }
        self
    }

    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        if append {
            self.0 |= Flags::APPEND;
        } else {
            self.0.remove(Flags::APPEND)
        }
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        if truncate {
            self.0 |= Flags::TRUNCATE;
        } else {
            self.0.remove(Flags::TRUNCATE);
        }
        self
    }

    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        if create {
            self.0 |= Flags::CREATE;
        } else {
            self.0.remove(Flags::CREATE);
        }
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        if create_new {
            self.0 |= Flags::CREATE_NEW;
        } else {
            self.0.remove(Flags::CREATE_NEW);
        }
        self
    }

    pub async fn open(&self, path: impl AsRef<Path>) -> io::Result<File> {
        let mut file = open_file(
            path,
            self.into(),
            !(self.0 & (Flags::TRUNCATE | Flags::CREATE)).is_empty(),
            self.into(),
        )
        .await?;

        if self.0.contains(Flags::APPEND) {
            file.seek(io::SeekFrom::End(0)).await?;
        }

        Ok(file)
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&OpenOptions> for CreateFileMode {
    fn from(options: &OpenOptions) -> Self {
        if options.0.contains(Flags::CREATE) {
            CreateFileMode::Create
        } else if options.0.contains(Flags::CREATE_NEW) {
            CreateFileMode::CreateNew
        } else {
            CreateFileMode::NotCreate
        }
    }
}

impl From<&OpenOptions> for SyncAccessMode {
    fn from(options: &OpenOptions) -> Self {
        if (options.0 & (Flags::APPEND | Flags::WRITE | Flags::TRUNCATE)).is_empty() {
            SyncAccessMode::Readonly
        } else {
            SyncAccessMode::Readwrite
        }
    }
}
