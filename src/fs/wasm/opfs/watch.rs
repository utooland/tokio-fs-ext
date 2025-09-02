use std::{
    io,
    path::{MAIN_SEPARATOR_STR, Path},
};

use js_sys::{Array, JsString};
pub use notify_types::event;
use wasm_bindgen::{
    JsCast,
    prelude::{Closure, wasm_bindgen},
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemDirectoryHandle, FileSystemHandle, FileSystemSyncAccessHandle};

use super::{super::opfs::OpfsError, CreateFileMode, OpenDirType, SyncAccessMode, open_file};

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
    pub fn changed_handle(this: &FileSystemChangeRecord) -> FileSystemHandle;

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

impl From<&FileSystemChangeRecord> for event::Event {
    fn from(record: &FileSystemChangeRecord) -> Self {
        let kind = record.changed_handle().kind();

        event::Event {
            kind: match record.r#type() {
                FileSystemChangeRecordType::Appeared => event::EventKind::Create(match kind {
                    web_sys::FileSystemHandleKind::File => event::CreateKind::File,
                    web_sys::FileSystemHandleKind::Directory => event::CreateKind::Folder,
                    _ => event::CreateKind::Any,
                }),
                FileSystemChangeRecordType::Disappeared => event::EventKind::Remove(match kind {
                    web_sys::FileSystemHandleKind::File => event::RemoveKind::File,
                    web_sys::FileSystemHandleKind::Directory => event::RemoveKind::Folder,
                    _ => event::RemoveKind::Any,
                }),
                FileSystemChangeRecordType::Modified => match kind {
                    web_sys::FileSystemHandleKind::File => {
                        event::EventKind::Modify(event::ModifyKind::Data(event::DataChange::Any))
                    }
                    web_sys::FileSystemHandleKind::Directory => event::EventKind::Modify(
                        event::ModifyKind::Metadata(event::MetadataKind::Any),
                    ),
                    _ => event::EventKind::Modify(event::ModifyKind::Any),
                },
                FileSystemChangeRecordType::Moved => {
                    event::EventKind::Modify(event::ModifyKind::Name(event::RenameMode::Any))
                }
                FileSystemChangeRecordType::Unknown => event::EventKind::Other,
                FileSystemChangeRecordType::Errored => event::EventKind::Other,
                FileSystemChangeRecordType::__Invalid => event::EventKind::Other,
            },
            paths: vec![
                format!(
                    "./{}",
                    record
                        .relative_path_components()
                        .iter()
                        .map(|p| String::from(p.unchecked_ref::<JsString>()))
                        .collect::<Vec<_>>()
                        .join(MAIN_SEPARATOR_STR)
                )
                .into(),
            ],
            ..Default::default()
        }
    }
}

pub async fn watch_dir(
    path: impl AsRef<Path>,
    recursive: bool,
    cb: impl Fn(event::Event) + Send + Sync + 'static,
) -> io::Result<()> {
    let observer = FileSystemObserver::new(
        Closure::<dyn Fn(Array)>::new(move |records: Array| {
            records.iter().for_each(|record| {
                let event = event::Event::from(record.unchecked_ref::<FileSystemChangeRecord>());
                cb(event)
            });
        })
        .into_js_value()
        .unchecked_ref(),
    );

    let dir_handle = super::open_dir(path, OpenDirType::NotCreate).await?;
    let options = FileSystemDirObserverOptions::new();
    options.set_recursive(recursive);
    JsFuture::from(observer.observe_dir_with_options(&dir_handle, &options))
        .await
        .map_err(|e| OpfsError::from(e).into_io_err())?;
    Ok(())
}

#[allow(dead_code)]
pub async fn watch_file(
    path: impl AsRef<Path>,
    cb: impl Fn(event::Event) + Send + Sync + 'static,
) -> io::Result<()> {
    let observer = FileSystemObserver::new(
        Closure::<dyn Fn(Array)>::new(move |records: Array| {
            records.iter().for_each(|record| {
                let event = event::Event::from(record.unchecked_ref::<FileSystemChangeRecord>());
                cb(event)
            });
        })
        .into_js_value()
        .unchecked_ref(),
    );

    let file_handle = open_file(
        path,
        CreateFileMode::NotCreate,
        SyncAccessMode::Readonly,
        false,
    )
    .await?
    .sync_access_handle
    .clone();

    JsFuture::from(observer.observe_file(&file_handle))
        .await
        .map_err(|e| OpfsError::from(e).into_io_err())?;
    Ok(())
}
