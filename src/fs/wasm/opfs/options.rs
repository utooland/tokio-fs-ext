use wasm_bindgen::prelude::wasm_bindgen;

/// NOTE: We always use `Readwrite` mode when creating the physical `SyncAccessHandle`.
///
/// `createSyncAccessHandle` takes an exclusive lock on the file. If we opened
/// it in `read-only` mode, we wouldn't be able to open another handle for writing
/// later (even by the same origin), which breaks the `File` abstraction where
/// multiple independent handles can exist.
///
/// Therefore, we hold the physical lock in `readwrite` mode and enforce
/// permissions in the `File` wrapper.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAccessMode {
    Readonly = "read-only",
    Readwrite = "readwrite",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateFileMode {
    Create,
    CreateNew,
    NotCreate,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum OpenDirType {
    Create,
    CreateRecursive,
    NotCreate,
}

// The file mode is still experimental:
// https://developer.mozilla.org/en-US/docs/Web/API/FileSystemFileHandle/createSyncAccessHandle#options
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CreateSyncAccessHandleOptions {
    mode: SyncAccessMode,
}

impl From<SyncAccessMode> for CreateSyncAccessHandleOptions {
    fn from(mode: SyncAccessMode) -> Self {
        Self { mode }
    }
}
