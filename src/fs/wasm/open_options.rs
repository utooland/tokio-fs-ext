use std::{io, path::Path};

use crate::fs::File;

pub(super) const READ: u8 = 0b0000_0001;
pub(super) const WRITE: u8 = 0b0000_0010;
pub(super) const CREATE: u8 = 0b0000_0100;
pub(super) const CREATE_NEW: u8 = 0b0000_1000;
pub(super) const TRUNCATE: u8 = 0b0001_0000;
pub(super) const APPEND: u8 = 0b0010_0000;

#[derive(Clone, Default, Debug)]
pub struct OpenOptions(pub(super) u8);

impl OpenOptions {
    pub fn new() -> OpenOptions {
        Default::default()
    }

    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        if read {
            self.0 |= READ;
        } else {
            self.0 = READ >> 1
        }
        self
    }

    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        if write {
            self.0 |= WRITE;
        } else if self.0 >= WRITE {
            self.0 = WRITE >> 1;
        }
        self
    }

    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        if append {
            self.0 |= APPEND;
        } else if self.0 >= APPEND {
            self.0 = APPEND >> 1;
        }
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        if truncate {
            self.0 |= TRUNCATE;
        } else if self.0 >= TRUNCATE {
            self.0 = TRUNCATE >> 1;
        }
        self
    }

    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        if create {
            self.0 |= CREATE;
        } else if self.0 >= CREATE {
            self.0 = CREATE >> 1;
        }
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        if create_new {
            self.0 |= CREATE_NEW;
        } else if self.0 >= CREATE_NEW {
            self.0 = CREATE_NEW >> 1;
        }
        self
    }

    pub async fn open(&self, path: impl AsRef<Path>) -> io::Result<File> {
        File::open_with_options(path, self).await
    }
}

impl OpenOptions {
    pub(super) fn readwrite(&self) -> bool {
        self.0 & WRITE > 0
    }
}
