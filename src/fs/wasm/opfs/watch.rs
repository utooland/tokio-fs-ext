use std::{
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use js_sys::{Array, JsString};
pub use notify_types::event;
use wasm_bindgen::{
    JsCast,
    closure::Closure,
    prelude::wasm_bindgen,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemDirectoryHandle, FileSystemHandle, FileSystemHandleKind, FileSystemSyncAccessHandle,
};

use super::{
    super::opfs::{OpfsError, virtualize},
    CreateFileMode, OpenDirType, SyncAccessMode, open_file,
};

#[wasm_bindgen]
extern "C" {
    // https://developer.mozilla.org/en-US/docs/Web/API/FileSystemObserver
    #[wasm_bindgen(extends = js_sys::Object, js_name = FileSystemObserver, typescript_type = "FileSystemObserver")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type FileSystemObserver;

    #[wasm_bindgen(constructor, js_class = "FileSystemObserver")]
    pub fn new(callback: &js_sys::Function) -> FileSystemObserver;

    #[wasm_bindgen (method, structural, js_class = "FileSystemObserver", js_name = observe)]
    pub fn observe_file(
        this: &FileSystemObserver,
        target: &FileSystemSyncAccessHandle,
    ) -> js_sys::Promise;

    #[wasm_bindgen (method, structural, js_class = "FileSystemObserver", js_name = observe)]
    pub fn observe_dir(
        this: &FileSystemObserver,
        handdle: &FileSystemSyncAccessHandle,
    ) -> js_sys::Promise;

    #[wasm_bindgen (method, structural, js_class = "FileSystemObserver", js_name = observe)]
    pub fn observe_dir_with_options(
        this: &FileSystemObserver,
        handdle: &FileSystemDirectoryHandle,
        options: &FileSystemDirObserverOptions,
    ) -> js_sys::Promise;

    #[wasm_bindgen (method, structural, js_class = "FileSystemObserver", js_name = disconnect)]
    pub fn disconnect(this: &FileSystemObserver);

}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = js_sys::Object , js_name = FileSystemObserverOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type FileSystemDirObserverOptions;
    #[wasm_bindgen(method, getter = "recursive")]
    pub fn get_recursive(this: &FileSystemDirObserverOptions) -> Option<bool>;
    #[wasm_bindgen(method, setter = "recursive")]
    pub fn set_recursive(this: &FileSystemDirObserverOptions, val: bool);
}
impl FileSystemDirObserverOptions {
    pub fn new() -> Self {
        wasm_bindgen::JsCast::unchecked_into(js_sys::Object::new())
    }
}

impl Default for FileSystemDirObserverOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to stop watching. When dropped, the watch is automatically stopped.
pub struct WatchHandle {
    observer: FileSystemObserver,
    // Keep closure alive to prevent GC
    #[allow(dead_code)]
    closure: Rc<Closure<dyn Fn(Array)>>,
}

impl WatchHandle {
    /// Stop watching and release resources
    pub fn stop(self) {
        self.observer.disconnect();
        // closure will be dropped here
    }
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object , js_name = FileSystemChangeRecord)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type FileSystemChangeRecord;

    #[wasm_bindgen (method, getter, js_class = "FileSystemChangeRecord", js_name = type)]
    pub fn r#type(this: &FileSystemChangeRecord) -> FileSystemChangeRecordType;

    #[wasm_bindgen(method, getter, structural, js_class = "FileSystemChangeRecord", js_name = relativePathComponents)]
    pub fn relative_path_components(this: &FileSystemChangeRecord) -> Array;

    #[wasm_bindgen(method, getter, structural, js_class = "FileSystemChangeRecord", js_name = changedHandle)]
    pub fn changed_handle(this: &FileSystemChangeRecord) -> Option<FileSystemHandle>;

    #[wasm_bindgen(method, getter, structural, js_class = "FileSystemChangeRecord", js_name = root)]
    pub fn root(this: &FileSystemChangeRecord) -> FileSystemHandle;

}

// https://developer.mozilla.org/en-US/docs/Web/API/FileSystemChangeRecord#type
#[wasm_bindgen]
pub enum FileSystemChangeRecordType {
    Appeared = "appeared",
    Disappeared = "disappeared",
    Errored = "errored",
    Modified = "modified",
    Moved = "moved",
    Unknown = "unknown",
}

impl TryFrom<&FileSystemChangeRecord> for event::Event {
    type Error = io::Error;
    fn try_from(record: &FileSystemChangeRecord) -> Result<Self, Self::Error> {
        let kind = record.changed_handle().map(|h| h.kind());

        let kind = match record.r#type() {
            FileSystemChangeRecordType::Appeared => event::EventKind::Create(match kind {
                Some(FileSystemHandleKind::File) => event::CreateKind::File,
                Some(FileSystemHandleKind::Directory) => event::CreateKind::Folder,
                _ => event::CreateKind::Any,
            }),
            FileSystemChangeRecordType::Disappeared => event::EventKind::Remove(match kind {
                Some(FileSystemHandleKind::File) => event::RemoveKind::File,
                Some(FileSystemHandleKind::Directory) => event::RemoveKind::Folder,
                _ => event::RemoveKind::Any,
            }),
            FileSystemChangeRecordType::Modified => match kind {
                Some(FileSystemHandleKind::File) => {
                    event::EventKind::Modify(event::ModifyKind::Data(event::DataChange::Any))
                }
                Some(FileSystemHandleKind::Directory) => {
                    event::EventKind::Modify(event::ModifyKind::Metadata(event::MetadataKind::Any))
                }
                _ => event::EventKind::Modify(event::ModifyKind::Any),
            },
            FileSystemChangeRecordType::Moved => {
                event::EventKind::Modify(event::ModifyKind::Name(event::RenameMode::Any))
            }
            FileSystemChangeRecordType::Unknown => event::EventKind::Other,
            FileSystemChangeRecordType::Errored => event::EventKind::Other,
            FileSystemChangeRecordType::__Invalid => event::EventKind::Other,
        };
        let path = virtualize(format!("/{}", record.root().name()))?.join(
            record
                .relative_path_components()
                .iter()
                .map(|p| String::from(p.unchecked_ref::<JsString>()))
                .collect::<PathBuf>(),
        );
        Ok(event::Event {
            kind,
            paths: vec![path],
            ..Default::default()
        })
    }
}

/// Watch a directory for changes.
///
/// Returns a `WatchHandle` that must be kept alive for the watch to remain active.
/// When the handle is dropped, the watch is automatically stopped.
///
/// # Arguments
/// * `path` - The directory path to watch
/// * `recursive` - Whether to watch subdirectories recursively
/// * `cb` - Callback function called for each file system event
///
/// # Example
/// ```ignore
/// let handle = watch_dir("/my/dir", true, |event| {
///     println!("Event: {:?}", event);
/// }).await?;
///
/// // Keep handle alive...
/// // handle.stop(); // or let it drop
/// ```
pub async fn watch_dir(
    path: impl AsRef<Path>,
    recursive: bool,
    cb: impl Fn(event::Event) + 'static,
) -> io::Result<WatchHandle> {
    let closure = Rc::new(Closure::<dyn Fn(Array)>::new(move |records: Array| {
        records.iter().for_each(|record| {
            let record: FileSystemChangeRecord = record.unchecked_into();
            match event::Event::try_from(&record) {
                Ok(evt) => cb(evt),
                Err(_err) => {
                    #[cfg(feature = "opfs_tracing")]
                    tracing::error!("failed to parse event from record: {_err:?}");
                }
            }
        });
    }));

    let observer = FileSystemObserver::new(closure.as_ref().as_ref().unchecked_ref());

    let dir_handle = super::open_dir(path, OpenDirType::NotCreate).await?;
    let options = FileSystemDirObserverOptions::new();
    options.set_recursive(recursive);
    JsFuture::from(observer.observe_dir_with_options(&dir_handle, &options))
        .await
        .map_err(|e| OpfsError::from(e).into_io_err())?;

    Ok(WatchHandle { observer, closure })
}

/// Watch a file for changes.
///
/// **Note**: This function opens the file with a `SyncAccessHandle`, which locks
/// the file and prevents other operations. Consider using `watch_dir` on the
/// parent directory instead if you need to perform other operations on the file.
///
/// Returns a `WatchHandle` that must be kept alive for the watch to remain active.
#[allow(dead_code)]
pub async fn watch_file(
    path: impl AsRef<Path>,
    cb: impl Fn(event::Event) + 'static,
) -> io::Result<WatchHandle> {
    let closure = Rc::new(Closure::<dyn Fn(Array)>::new(move |records: Array| {
        records.iter().for_each(|record| {
            let record: FileSystemChangeRecord = record.unchecked_into();
            match event::Event::try_from(&record) {
                Ok(evt) => cb(evt),
                Err(_err) => {
                    #[cfg(feature = "opfs_tracing")]
                    tracing::error!("failed to parse event from record: {_err:?}");
                }
            }
        });
    }));

    let observer = FileSystemObserver::new(closure.as_ref().as_ref().unchecked_ref());

    let file = open_file(
        path,
        CreateFileMode::NotCreate,
        SyncAccessMode::Readonly,
        false,
    )
    .await?;

    JsFuture::from(observer.observe_file(&file.sync_access_handle))
        .await
        .map_err(|e| OpfsError::from(e).into_io_err())?;

    // Note: file handle remains open while watching
    // This is necessary for the observer to work, but locks the file
    std::mem::forget(file);

    Ok(WatchHandle { observer, closure })
}
