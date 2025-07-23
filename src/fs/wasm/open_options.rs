use std::{io, path::Path};

use crate::fs::File;

pub(crate) const READ: u8 = 0b0000_0001;
pub(crate) const WRITE: u8 = 0b0000_0010;
pub(crate) const CREATE: u8 = 0b0000_0100;
pub(crate) const CREATE_NEW: u8 = 0b0000_1000;
pub(crate) const TRUNCATE: u8 = 0b0001_0000;
pub(crate) const APPEND: u8 = 0b0010_0000;

#[derive(Clone, Default, Debug)]
pub struct OpenOptions(pub(crate) u8);

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions(READ)
    }

    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        if read {
            self.0 |= READ;
        }
        self
    }

    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        if write {
            self.0 |= WRITE;
        } else if self.0 > WRITE {
            self.0 -= WRITE;
        }
        self
    }

    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        if append {
            self.0 |= APPEND;
        } else if self.0 > APPEND {
            self.0 -= APPEND;
        }
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        if truncate {
            self.0 |= TRUNCATE;
        } else if self.0 > TRUNCATE {
            self.0 -= TRUNCATE;
        }
        self
    }

    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        if create {
            self.0 |= CREATE;
        } else if self.0 > CREATE {
            self.0 -= CREATE;
        }
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        if create_new {
            self.0 |= CREATE_NEW;
        } else if self.0 > CREATE_NEW {
            self.0 -= CREATE_NEW;
        }
        self
    }

    pub async fn open(&self, path: impl AsRef<Path>) -> io::Result<File> {
        File::open_with_options(path, self).await
    }
}

impl OpenOptions {
    pub(crate) fn readwrite(&self) -> bool {
        self.0 & WRITE > 0
    }
}
