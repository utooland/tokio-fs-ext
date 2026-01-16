//! Path-based file locking for OPFS concurrent access control.
//!
//! OPFS `FileSystemSyncAccessHandle` only allows one active handle per file.
//! This module provides a cooperative locking mechanism to serialize access
//! and provide meaningful errors when contention occurs.

use std::{
    cell::RefCell,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll, Waker},
};

use rustc_hash::{FxHashMap, FxHashSet};

thread_local! {
    /// Tracks which paths currently have an active SyncAccessHandle
    static LOCKED_PATHS: RefCell<FxHashSet<PathBuf>> = RefCell::new(FxHashSet::default());
    /// Wakers waiting for a path to be unlocked
    static WAITERS: RefCell<FxHashMap<PathBuf, Vec<Waker>>> = RefCell::new(FxHashMap::default());
}

/// RAII guard that releases the path lock when dropped
pub(crate) struct PathLockGuard {
    path: PathBuf,
}

impl std::fmt::Debug for PathLockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathLockGuard")
            .field("path", &self.path)
            .finish()
    }
}

impl Drop for PathLockGuard {
    fn drop(&mut self) {
        unlock_path(&self.path);
    }
}

/// Try to acquire a lock on a path without blocking.
/// Returns `Some(guard)` if successful, `None` if the path is already locked.
#[allow(dead_code)]
pub(crate) fn try_lock_path(path: &Path) -> Option<PathLockGuard> {
    let path = normalize_path(path);
    LOCKED_PATHS.with(|locked| {
        let mut locked = locked.borrow_mut();
        if locked.contains(&path) {
            None
        } else {
            locked.insert(path.clone());
            Some(PathLockGuard { path })
        }
    })
}

/// Check if a path is currently locked
#[allow(dead_code)]
pub(crate) fn is_path_locked(path: &Path) -> bool {
    let path = normalize_path(path);
    LOCKED_PATHS.with(|locked| locked.borrow().contains(&path))
}

/// Async lock acquisition - waits until the path becomes available
pub(crate) fn lock_path(path: &Path) -> PathLockFuture {
    PathLockFuture {
        path: normalize_path(path),
    }
}

fn unlock_path(path: &PathBuf) {
    LOCKED_PATHS.with(|locked| {
        locked.borrow_mut().remove(path);
    });

    // Wake up any waiters for this path
    WAITERS.with(|waiters| {
        if let Some(wakers) = waiters.borrow_mut().remove(path) {
            for waker in wakers {
                waker.wake();
            }
        }
    });
}

fn normalize_path(path: &Path) -> PathBuf {
    // Normalize the path to ensure consistent locking
    path.to_path_buf()
}

/// Future that resolves when a path lock is acquired
pub(crate) struct PathLockFuture {
    path: PathBuf,
}

impl Future for PathLockFuture {
    type Output = PathLockGuard;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let path = &self.path;

        // Try to acquire the lock
        let acquired = LOCKED_PATHS.with(|locked| {
            let mut locked = locked.borrow_mut();
            if locked.contains(path) {
                false
            } else {
                locked.insert(path.clone());
                true
            }
        });

        if acquired {
            Poll::Ready(PathLockGuard { path: path.clone() })
        } else {
            // Register waker to be notified when path is unlocked
            WAITERS.with(|waiters| {
                waiters
                    .borrow_mut()
                    .entry(path.clone())
                    .or_default()
                    .push(cx.waker().clone());
            });
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_lock_same_path_twice() {
        let path = Path::new("/test/file.txt");

        let guard1 = try_lock_path(path);
        assert!(guard1.is_some());

        // Second lock should fail
        let guard2 = try_lock_path(path);
        assert!(guard2.is_none());

        // After dropping, lock should succeed
        drop(guard1);
        let guard3 = try_lock_path(path);
        assert!(guard3.is_some());
    }

    #[test]
    fn test_different_paths_can_lock() {
        let path1 = Path::new("/test/file1.txt");
        let path2 = Path::new("/test/file2.txt");

        let guard1 = try_lock_path(path1);
        let guard2 = try_lock_path(path2);

        assert!(guard1.is_some());
        assert!(guard2.is_some());
    }
}
