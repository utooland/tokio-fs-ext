use wasm_bindgen::prelude::wasm_bindgen;

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
