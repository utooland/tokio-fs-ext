use std::{
    io::{self, SeekFrom},
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use futures::io::{AsyncRead, AsyncSeek, AsyncWrite};
use web_sys::{FileSystemReadWriteOptions, FileSystemSyncAccessHandle};

use super::{
    OpenOptions,
    metadata::{FileType, Metadata},
    opfs::{PathLockGuard, SyncAccessMode, open_file, opfs_err},
};

/// A file handle with exclusive access to the underlying OPFS file.
///
/// The file lock is automatically released when the `File` is dropped.
#[derive(Debug)]
pub struct File {
    pub(super) sync_access_handle: FileSystemSyncAccessHandle,
    pub(super) pos: Option<u64>,
    /// RAII guard that ensures exclusive access to this file path
    #[allow(dead_code)]
    pub(super) lock_guard: PathLockGuard,
}

impl File {
    pub async fn create(path: impl AsRef<Path>) -> io::Result<File> {
        open_file(
            path,
            super::opfs::CreateFileMode::Create,
            SyncAccessMode::Readonly,
            true,
        )
        .await
    }

    pub async fn create_new<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        open_file(
            &path,
            super::opfs::CreateFileMode::CreateNew,
            SyncAccessMode::Readonly,
            false,
        )
        .await
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        Ok(Metadata {
            file_type: FileType::File,
            file_size: self.size()?,
            mtime: None,
        })
    }

    pub async fn open(path: impl AsRef<Path>) -> io::Result<File> {
        open_file(
            path,
            super::opfs::CreateFileMode::NotCreate,
            SyncAccessMode::Readonly,
            false,
        )
        .await
    }

    #[must_use]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub async fn sync_all(&self) -> io::Result<()> {
        self.flush()
    }

    pub async fn sync_data(&self) -> io::Result<()> {
        self.flush()
    }

    pub fn size(&self) -> io::Result<u64> {
        self.sync_access_handle
            .get_size()
            .map_or_else(|err| Err(opfs_err(err)), |size| Ok(size as u64))
    }

    /// Truncates or extends the underlying file, updating the size of this file to become `size`.
    ///
    /// If `size` is less than the current file's size, then the file will be shrunk. If it is greater
    /// than the current file's size, then the file will be extended to `size` and have all intermediate
    /// data filled with 0s.
    ///
    /// The file's cursor is not changed. In particular, if the cursor was at the end of the file and
    /// the file was shrunk using this operation, the cursor will now be past the end.
    ///
    /// If the requested length is greater than 9007199254740991 (max safe integer in a floating-point context),
    /// this will produce an error.
    pub async fn set_len(&self, size: u64) -> io::Result<()> {
        const MAX_SAFE_INT: u64 = js_sys::Number::MAX_SAFE_INTEGER as _;
        if size > MAX_SAFE_INT {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("requested size {size} too large, max allowed is {MAX_SAFE_INT}"),
            ));
        }
        self.sync_access_handle
            .truncate_with_f64(size as _)
            .map_err(opfs_err)
    }
}

impl File {
    pub(crate) fn read_to_buf(&mut self, buf: &mut [u8]) -> io::Result<u64> {
        match self.pos {
            Some(pos) => {
                let options = FileSystemReadWriteOptions::new();
                options.set_at(pos as f64);
                let size = self
                    .sync_access_handle
                    .read_with_u8_array_and_options(buf, &options)
                    .map_err(opfs_err)? as u64;
                Ok(size)
            }
            None => {
                let size = self
                    .sync_access_handle
                    .read_with_u8_array(buf)
                    .map_err(opfs_err)? as u64;
                Ok(size)
            }
        }
    }

    pub(crate) fn write_with_buf(&mut self, buf: impl AsRef<[u8]>) -> io::Result<u64> {
        match self.pos {
            Some(pos) => {
                let options = FileSystemReadWriteOptions::new();
                options.set_at(pos as f64);
                let size = self
                    .sync_access_handle
                    .write_with_u8_array_and_options(buf.as_ref(), &options)
                    .map_err(opfs_err)? as u64;
                Ok(size)
            }
            None => {
                let size = self
                    .sync_access_handle
                    .write_with_u8_array(buf.as_ref())
                    .map_err(opfs_err)? as u64;
                Ok(size)
            }
        }
    }

    pub(super) fn flush(&self) -> io::Result<()> {
        self.sync_access_handle.flush().map_err(opfs_err)
    }

    pub(crate) fn close(&self) {
        self.sync_access_handle.close();
    }
}

impl Drop for File {
    fn drop(&mut self) {
        self.close();
    }
}

impl AsyncRead for File {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let offset = self.read_to_buf(buf)?;

        self.pos = Some(self.pos.unwrap_or_default() + offset);

        Poll::Ready(Ok(offset as usize))
    }
}

impl AsyncWrite for File {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let offset = self.write_with_buf(buf)?;

        self.pos = Some(self.pos.unwrap_or_default() + offset);

        Poll::Ready(Ok(offset as usize))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.flush()?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.close();
        Poll::Ready(Ok(()))
    }
}

impl AsyncSeek for File {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        position: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        match position {
            SeekFrom::Start(offset) => {
                self.pos = Some(offset);
            }
            SeekFrom::End(offset) => {
                self.pos = Some(
                    self.size()?
                        .checked_add_signed(offset)
                        .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
                );
            }
            SeekFrom::Current(offset) => {
                self.pos = Some(
                    self.pos
                        .unwrap_or_default()
                        .checked_add_signed(offset)
                        .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
                );
            }
        }
        Poll::Ready(Ok(self.pos.unwrap_or_default()))
    }
}
