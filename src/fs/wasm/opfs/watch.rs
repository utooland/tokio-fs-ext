use std::{
    io,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, stream::FusedStream};
use js_sys::{Array, JsString};
pub use notify_types::event;
use wasm_bindgen::{
    JsCast,
    prelude::{Closure, wasm_bindgen},
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemHandle, FileSystemHandleKind};

use super::{
    super::opfs::{opfs_err, virtualize},
    CreateFileMode, OpenDirType,
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
        target: &web_sys::FileSystemFileHandle,
    ) -> js_sys::Promise;

    #[wasm_bindgen (method, structural, js_class = "FileSystemObserver", js_name = observe)]
    pub fn observe_dir(
        this: &FileSystemObserver,
        handle: &web_sys::FileSystemDirectoryHandle,
    ) -> js_sys::Promise;

    #[wasm_bindgen (method, structural, js_class = "FileSystemObserver", js_name = observe)]
    pub fn observe_dir_with_options(
        this: &FileSystemObserver,
        handle: &web_sys::FileSystemDirectoryHandle,
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

fn record_to_event(record: &FileSystemChangeRecord, base_path: &Path) -> io::Result<event::Event> {
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
    let path = base_path.join(
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

pub struct WatchStream {
    receiver: Option<tokio::sync::mpsc::UnboundedReceiver<event::Event>>,
    _observer: FileSystemObserver,
    _closure: Closure<dyn Fn(Array)>,
}

unsafe impl Send for WatchStream {}
unsafe impl Sync for WatchStream {}

impl Drop for WatchStream {
    fn drop(&mut self) {
        self._observer.disconnect();
    }
}

impl WatchStream {
    /// Consumes the `WatchStream`, returning the inner receiver.
    /// This is useful for offloading where the observer must stay in the worker.
    pub fn into_inner(mut self) -> tokio::sync::mpsc::UnboundedReceiver<event::Event> {
        self.receiver.take().expect("WatchStream already consumed")
    }
}

impl Stream for WatchStream {
    type Item = event::Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(ref mut receiver) = self.get_mut().receiver {
            receiver.poll_recv(cx)
        } else {
            Poll::Ready(None)
        }
    }
}

impl FusedStream for WatchStream {
    fn is_terminated(&self) -> bool {
        false
    }
}

pub async fn watch_dir(path: impl AsRef<Path>, recursive: bool) -> io::Result<WatchStream> {
    let base_path = virtualize(path.as_ref())?;
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    let closure = Closure::<dyn Fn(Array)>::new({
        let base_path = base_path.clone();
        move |records: Array| {
            records.iter().for_each(|record| {
                let record: FileSystemChangeRecord = record.unchecked_into();
                if let Ok(evt) = record_to_event(&record, &base_path) {
                    let _ = sender.send(evt);
                }
            });
        }
    });

    let observer = FileSystemObserver::new(closure.as_ref().unchecked_ref());
    let dir_handle = super::open_dir(&base_path, OpenDirType::NotCreate).await?;

    let options = FileSystemDirObserverOptions::new();
    options.set_recursive(recursive);
    JsFuture::from(observer.observe_dir_with_options(&dir_handle, &options))
        .await
        .map_err(opfs_err)?;

    Ok(WatchStream {
        receiver: Some(receiver),
        _observer: observer,
        _closure: closure,
    })
}

#[allow(dead_code)]
pub async fn watch_file(path: impl AsRef<Path>) -> io::Result<WatchStream> {
    let base_path = virtualize(path.as_ref())?;
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    let closure = Closure::<dyn Fn(Array)>::new({
        let base_path = base_path.clone();
        move |records: Array| {
            records.iter().for_each(|record| {
                let record: FileSystemChangeRecord = record.unchecked_into();
                if let Ok(evt) = record_to_event(&record, &base_path) {
                    let _ = sender.send(evt);
                }
            });
        }
    });

    let observer = FileSystemObserver::new(closure.as_ref().unchecked_ref());
    let file_handle = super::open_file(
        &base_path,
        CreateFileMode::NotCreate,
        super::SyncAccessMode::Readonly,
        false,
    )
    .await?
    .handle
    .clone();

    JsFuture::from(observer.observe_file(&file_handle))
        .await
        .map_err(opfs_err)?;

    Ok(WatchStream {
        receiver: Some(receiver),
        _observer: observer,
        _closure: closure,
    })
}
