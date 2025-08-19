use std::{cell::RefCell, path::PathBuf};

use rustc_hash::FxHashMap;
use web_sys::FileSystemDirectoryHandle;

thread_local! {
    static  DIR_HANDLE_CACHE: RefCell<FxHashMap<PathBuf, FileSystemDirectoryHandle>> = RefCell::new(FxHashMap::default());
}

pub(super) fn get_cached_dir_handle(path: &PathBuf) -> Option<FileSystemDirectoryHandle> {
    DIR_HANDLE_CACHE.with(|cache| cache.borrow().get(path).cloned())
}

pub(super) fn set_cached_dir_handle(path: PathBuf, handle: FileSystemDirectoryHandle) {
    DIR_HANDLE_CACHE.with(|cache| {
        cache.borrow_mut().insert(path, handle);
    })
}

pub(super) fn remove_cached_dir_handle(path: &PathBuf, recursive: bool) {
    DIR_HANDLE_CACHE.with(|cache| {
        let keys: Vec<PathBuf> = {
            cache
                .borrow()
                .keys()
                .filter(|k| k.starts_with(path))
                .cloned()
                .collect()
        };

        let mut cache = cache.borrow_mut();
        if recursive {
            keys.into_iter().for_each(|k| {
                cache.remove(&k);
            });
        } else {
            cache.remove(path);
        }
    })
}
