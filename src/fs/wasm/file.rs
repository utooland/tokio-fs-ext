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

use crate::console;

use super::{
    OpenOptions,
    metadata::{FileType, Metadata},
    opfs::{SyncAccessMode, open_file, opfs_err},
};

// ---------------------------------------------------------------------------
// Per-path SyncAccessHandle cache with reference counting
//
// OPFS only allows one `createSyncAccessHandle` per file at a time.  To give
// callers OS-like semantics (multiple `File` objects on the same path), we:
//
//   1. Always open handles in **Readwrite** mode (the most permissive mode).
//   2. Cache the handle and clone it to every concurrent opener.
//   3. Enforce the caller's requested read/write permission in the `File`
//      layer via `File::mode`.
//   4. Close the underlying handle only when the last user drops its guard.
// ---------------------------------------------------------------------------

thread_local! {
    static LOCKS: RefCell<FxHashMap<PathBuf, LockState>> = RefCell::new(FxHashMap::default());
    static NEXT_ID: RefCell<u64> = const { RefCell::new(0) };
}

#[derive(Default)]
struct LockState {
    /// Cached SyncAccessHandle (always opened in Readwrite mode).
    handle: Option<FileSystemSyncAccessHandle>,
    /// Number of active `File` objects sharing this handle.
    ref_count: usize,
    /// Tasks waiting for the handle to become available.
    waiters: VecDeque<Waiter>,
}

struct Waiter {
    id: u64,
    waker: Waker,
}

// -- Guard ------------------------------------------------------------------

#[derive(Debug)]
pub struct FileLockGuard {
    pub(super) path: PathBuf,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        LOCKS.with(|locks| {
            let mut locks = locks.borrow_mut();
            let Some(state) = locks.get_mut(&self.path) else {
                return;
            };

            state.ref_count -= 1;

            if state.ref_count == 0 {
                // Last user — close the cached handle.
                if let Some(h) = state.handle.take() {
                    h.close();
                } else if !state.waiters.is_empty() {
                    // Deadlock risk: the lock creator dropped without ever
                    // calling `set_lock_handle`.  Waiters will retry, but if
                    // the underlying error persists they may cycle forever.
                    console::error!(
                        "[tokio-fs-ext] potential deadlock: lock holder for {:?} dropped \
                         without setting handle, {} waiter(s) will retry",
                        self.path,
                        state.waiters.len()
                    );
                }

                if state.waiters.is_empty() {
                    locks.remove(&self.path);
                } else {
                    // Wake all waiters. The first one to poll becomes the new
                    // handle creator; the rest will wait for `set_lock_handle`.
                    let wakers: Vec<Waker> = state.waiters.drain(..).map(|w| w.waker).collect();
                    for w in wakers {
                        w.wake();
                    }
                }
            }
        });
    }
}

// -- Future -----------------------------------------------------------------

pub struct FileLockFuture {
    path: PathBuf,
    id: u64,
    /// Whether this future has inserted itself into the waiter queue.
    registered: bool,
}

impl Drop for FileLockFuture {
    fn drop(&mut self) {
        if !self.registered {
            return;
        }
        // Clean up the waiter entry if this future is cancelled.
        LOCKS.with(|locks| {
            let mut locks = locks.borrow_mut();
            let Some(state) = locks.get_mut(&self.path) else {
                return;
            };
            state.waiters.retain(|w| w.id != self.id);
            if state.ref_count == 0 && state.waiters.is_empty() && state.handle.is_none() {
                locks.remove(&self.path);
            }
        });
    }
}

impl Future for FileLockFuture {
    type Output = (FileLockGuard, Option<FileSystemSyncAccessHandle>);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        LOCKS.with(|locks| {
            let mut locks = locks.borrow_mut();
            let state = locks.entry(this.path.clone()).or_default();

            if state.ref_count == 0 {
                // First opener — become the handle creator.
                state.ref_count = 1;
                state.waiters.retain(|w| w.id != this.id);
                this.registered = false;

                Poll::Ready((
                    FileLockGuard {
                        path: this.path.clone(),
                    },
                    None,
                ))
            } else if let Some(handle) = &state.handle {
                // Handle exists — share it.
                let h = handle.clone();
                state.ref_count += 1;
                state.waiters.retain(|w| w.id != this.id);
                this.registered = false;

                Poll::Ready((
                    FileLockGuard {
                        path: this.path.clone(),
                    },
                    Some(h),
                ))
            } else {
                // Handle is being created by another opener — wait.
                //
                // Deadlock risk: if the creator task is itself awaiting
                // `lock_file` for the same path (re-entrant lock), neither
                // side can make progress.  We cannot detect true re-entrancy
                // from here, but a growing waiter queue with no handle is a
                // strong signal.
                if let Some(w) = state.waiters.iter_mut().find(|w| w.id == this.id) {
                    w.waker = cx.waker().clone();
                } else {
                    if !state.waiters.is_empty() {
                        console::error!(
                            "[tokio-fs-ext] potential deadlock: {} task(s) already waiting \
                             for handle on {:?}, yet handle is still not set — possible \
                             re-entrant lock or stuck creator",
                            state.waiters.len(),
                            this.path
                        );
                    }
                    state.waiters.push_back(Waiter {
                        id: this.id,
                        waker: cx.waker().clone(),
                    });
                    this.registered = true;
                }
                Poll::Pending
            }
        })
    }
}

// -- Public API -------------------------------------------------------------

/// Acquire a shared file lock. Returns a guard and optionally the cached
/// `SyncAccessHandle`. When `None` is returned, the caller must create a
/// new handle and register it via [`set_lock_handle`].
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
        registered: false,
    }
}

/// Store a newly created `SyncAccessHandle` in the cache and wake all
/// waiters so they can share it.
pub(crate) fn set_lock_handle(path: &Path, handle: FileSystemSyncAccessHandle) {
    LOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        if let Some(state) = locks.get_mut(path) {
            state.handle = Some(handle);
            // Wake ALL waiters — they can all share the handle now.
            let wakers: Vec<Waker> = state.waiters.drain(..).map(|w| w.waker).collect();
            for w in wakers {
                w.wake();
            }
        } else {
            // Deadlock risk: the guard was already dropped (state cleaned up)
            // before we could register the handle.  The SyncAccessHandle is
            // now leaked — OPFS allows only one per file, so future openers
            // will hang forever on `createSyncAccessHandle`.
            console::error!(
                "[tokio-fs-ext] potential deadlock: set_lock_handle called for {:?} \
                 but lock state was already removed — SyncAccessHandle leaked, \
                 future opens of this file may hang",
                path
            );
            // Best-effort: close the orphaned handle to avoid OPFS-level deadlock.
            handle.close();
        }
    });
}

/// A file handle with exclusive access to the underlying OPFS file.
///
/// The file lock is automatically released when the `File` is dropped.
#[derive(Debug)]
pub struct File {
    pub(super) handle: FileSystemFileHandle,
    pub(super) sync_access_handle: FileSystemSyncAccessHandle,
    pub(super) pos: Option<u64>,
    pub(super) mode: SyncAccessMode,
    pub(super) _lock: FileLockGuard,
}

impl File {
    pub async fn create(path: impl AsRef<Path>) -> io::Result<File> {
        open_file(
            path,
            super::opfs::CreateFileMode::Create,
            SyncAccessMode::Readwrite,
            true,
        )
        .await
    }

    pub async fn create_new<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        open_file(
            &path,
            super::opfs::CreateFileMode::CreateNew,
            SyncAccessMode::Readwrite,
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
        if self.mode == SyncAccessMode::Readonly {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "file is opened in read-only mode",
            ));
        }

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
        if self.mode == SyncAccessMode::Readonly {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "file is opened in read-only mode",
            ));
        }

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
}

// NOTE: No manual Drop — closing the SyncAccessHandle is managed by
// `FileLockGuard::drop` when the last `File` on this path is dropped.
// Calling `.close()` here would invalidate the handle for all other
// `File` objects sharing it.

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
        // Only flush — the underlying SyncAccessHandle is shared across
        // multiple `File` objects and must not be closed until the last
        // FileLockGuard is dropped.
        self.flush()?;
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
