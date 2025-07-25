use std::{
    fs::Metadata,
    io,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use web_sys::FileSystemSyncAccessHandle;

use crate::fs::{OpenOptions, opfs::open_file};

#[derive(Debug)]
pub struct File {
    pub(super) sync_access_handle: FileSystemSyncAccessHandle,
}

impl File {
    pub async fn open(path: impl AsRef<Path>) -> io::Result<File> {
        open_file(path, false, false).await
    }

    pub async fn create(path: impl AsRef<Path>) -> io::Result<File> {
        let mut open_options = OpenOptions::new();
        open_options.create(true);
        open_file(path, true, true).await
    }

    pub async fn create_new<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        if (open_file(&path, true, false).await).is_ok() {
            return Err(io::Error::from(io::ErrorKind::AlreadyExists));
        }
        File::create(path).await
    }

    #[must_use]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        todo!()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        self.sync_access_handle
            .flush()
            .expect("Failed to flush opfs sync access handle");
        self.sync_access_handle.close();
    }
}

impl AsyncRead for File {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        todo!()
    }
}

impl AsyncWrite for File {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }
}
