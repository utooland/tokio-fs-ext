use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    io::{self, SeekFrom},
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::io::{AsyncRead, AsyncSeek, AsyncWrite};
use rustc_hash::FxHashMap;
use web_sys::{FileSystemFileHandle, FileSystemReadWriteOptions, FileSystemSyncAccessHandle};

use super::{
    OpenOptions,
    metadata::{FileType, Metadata},
    opfs::{SyncAccessMode, open_file, opfs_err},
};

thread_local! {
    static LOCKS: RefCell<FxHashMap<PathBuf, FileLockState>> = RefCell::new(FxHashMap::default());
    static NEXT_ID: RefCell<u64> = const { RefCell::new(0) };
}

#[derive(Default)]
struct FileLockState {
    owner: Option<u64>,
    waiters: VecDeque<(u64, Waker)>,
}

#[derive(Debug)]
pub struct FileLockGuard {
    path: PathBuf,
    id: u64,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        LOCKS.with(|locks| {
            let mut locks = locks.borrow_mut();
            if let Some(state) = locks.get_mut(&self.path) {
                state.owner = None;
                #[cfg(feature = "opfs_tracing")]
                tracing::trace!(path = %self.path.display(), id = self.id, "Released file lock");

                if let Some((_, waker)) = state.waiters.front() {
                    waker.wake_by_ref();
                } else {
                    locks.remove(&self.path);
                }
            }
        });
    }
}

pub struct FileLockFuture {
    path: PathBuf,
    id: u64,
}

impl Future for FileLockFuture {
    type Output = FileLockGuard;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let path = &self.path;
        let id = self.id;
        LOCKS.with(|locks| {
            let mut locks = locks.borrow_mut();
            let state = locks
                .entry(path.clone())
                .or_insert_with(FileLockState::default);

            if state.owner.is_none() && state.waiters.front().is_none_or(|(w_id, _)| *w_id == id) {
                // Front of the queue and lock is free (or we were the one waiting)
                state.owner = Some(id);
                state.waiters.retain(|(w_id, _)| *w_id != id);

                #[cfg(feature = "opfs_tracing")]
                tracing::trace!(path = %path.display(), id = id, "Acquired file lock");

                Poll::Ready(FileLockGuard {
                    path: path.clone(),
                    id,
                })
            } else {
                // Not our turn or locked
                if !state.waiters.iter().any(|(w_id, _)| *w_id == id) {
                    #[cfg(feature = "opfs_tracing")]
                    tracing::trace!(path = %path.display(), id = id, "File lock busy, queuing");
                    state.waiters.push_back((id, cx.waker().clone()));
                } else {
                    // Update waker if already in queue
                    if let Some(waiter) = state.waiters.iter_mut().find(|(w_id, _)| *w_id == id) {
                        waiter.1 = cx.waker().clone();
                    }
                }
                Poll::Pending
            }
        })
    }
}

pub fn lock_file(path: impl AsRef<Path>) -> FileLockFuture {
    let id = NEXT_ID.with(|next_id| {
        let mut id = next_id.borrow_mut();
        let current = *id;
        *id += 1;
        current
    });

    FileLockFuture {
        path: path.as_ref().to_path_buf(),
        id,
    }
}

/// A file handle with exclusive access to the underlying OPFS file.
///
/// The file lock is automatically released when the `File` is dropped.
#[derive(Debug)]
pub struct File {
    pub(super) handle: FileSystemFileHandle,
    pub(super) sync_access_handle: FileSystemSyncAccessHandle,
    pub(super) pos: Option<u64>,
    pub(super) _lock: FileLockGuard,
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
        let file_val = wasm_bindgen_futures::JsFuture::from(self.handle.get_file())
            .await
            .map_err(opfs_err)?;

        let mtime = js_sys::Reflect::get(&file_val, &"lastModified".into())
            .map_err(opfs_err)?
            .as_f64()
            .map(|v| v as u64);

        Ok(Metadata {
            file_type: FileType::File,
            file_size: self.size()?,
            mtime,
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
