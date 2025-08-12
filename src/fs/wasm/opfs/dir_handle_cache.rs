use std::{path::PathBuf, sync::LazyLock};

use rustc_hash::FxHashMap;
use send_wrapper::SendWrapper;
use web_sys::FileSystemDirectoryHandle;

static mut DIR_HANDLE_CACHE: LazyLock<FxHashMap<PathBuf, SendWrapper<FileSystemDirectoryHandle>>> =
    LazyLock::new(FxHashMap::default);

pub(super) fn get_cached_dir_handle(
    path: &PathBuf,
) -> Option<SendWrapper<FileSystemDirectoryHandle>> {
    unsafe {
        #[allow(static_mut_refs)]
        DIR_HANDLE_CACHE.get(path).cloned()
    }
}

pub(super) fn set_cached_dir_handle(path: PathBuf, handle: SendWrapper<FileSystemDirectoryHandle>) {
    unsafe {
        #[allow(static_mut_refs)]
        DIR_HANDLE_CACHE.insert(path.clone(), handle);
    }
}

pub(super) fn remove_cached_dir_handle(path: &PathBuf, recursive: bool) {
    if recursive {
        unsafe {
            #[allow(static_mut_refs)]
            DIR_HANDLE_CACHE
                .keys()
                .filter(|k| k.starts_with(path))
                .for_each(|k| {
                    DIR_HANDLE_CACHE.remove(k);
                });
        }
    } else {
        unsafe {
            #[allow(static_mut_refs)]
            DIR_HANDLE_CACHE.remove(path)
        };
    }
}
