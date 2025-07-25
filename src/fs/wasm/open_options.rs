use bitflags::bitflags;
use std::{io, path::Path};

use crate::fs::{File, opfs::open_file};

bitflags! {
    #[derive(Clone, Default, Debug, Copy)]
    struct Flags: u8 {
        const READ = 0b0000_0001;
        const WRITE= 0b0000_0010;
        const APPEND = 0b0000_0100;
        const CREATE = 0b0000_1000;
        const TRUNCATE = 0b0001_0000;
        const CREATE_NEW = 0b0010_0000;
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
        open_file(
            path,
            self.0.contains(Flags::CREATE) && !self.0.contains(Flags::CREATE_NEW),
            self.0.contains(Flags::TRUNCATE),
        )
        .await
    }
}
