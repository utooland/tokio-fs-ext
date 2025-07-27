use std::{
    io,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use web_sys::FileSystemSyncAccessHandle;

use crate::fs::{
    OpenOptions,
    opfs::{OpfsError, open_file},
    wasm::metadata::Metadata,
};

use super::{metadata::FileType, opfs::SyncAccessMode};

#[derive(Debug)]
pub struct File {
    pub(super) sync_access_handle: FileSystemSyncAccessHandle,
}

impl File {
    pub async fn open(path: impl AsRef<Path>) -> io::Result<File> {
        open_file(path, false, false, SyncAccessMode::Readonly).await
    }

    pub async fn create(path: impl AsRef<Path>) -> io::Result<File> {
        let mut open_options = OpenOptions::new();
        open_options.create(true);
        open_file(path, true, true, SyncAccessMode::Readonly).await
    }

    pub async fn create_new<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        if (open_file(&path, true, false, SyncAccessMode::Readonly).await).is_ok() {
            return Err(io::Error::from(io::ErrorKind::AlreadyExists));
        }
        File::create(path).await
    }

    #[must_use]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub fn size(&self) -> io::Result<usize> {
        self.sync_access_handle.get_size().map_or_else(
            |err| Err(OpfsError::from(err).into_io_err()),
            |size| Ok(size as usize),
        )
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        Ok(Metadata {
            file_type: FileType::File,
            file_size: self.size().map(|s| s as u64)?,
        })
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
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // Must ensure the vec len is equal to the file size before poll read
        unsafe { buf.assume_init(self.size()?) };
        buf.set_filled(self.size()?);

        self.sync_access_handle
            .read_with_u8_array(buf.filled_mut())
            .map_err(|err| OpfsError::from(err).into_io_err())?;

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for File {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(
            self.sync_access_handle
                .write_with_u8_array(buf)
                .map_or_else(
                    |err| Err(OpfsError::from(err).into_io_err()),
                    |size| Ok(size as usize),
                ),
        )
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(
            self.sync_access_handle
                .flush()
                .map_err(|err| OpfsError::from(err).into_io_err()),
        )
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready({
            self.sync_access_handle.close();
            Ok(())
        })
    }
}
