use std::{io, path::Path};

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetFileOptions,
    FileSystemSyncAccessHandle,
};

// use crate::current_dir;

use super::{
    super::{
        File,
        file::{FileLockGuard, lock_file, set_lock_handle},
    },
    OpenDirType,
    error::opfs_err,
    open_dir,
    options::{CreateFileMode, CreateSyncAccessHandleOptions, SyncAccessMode},
    root::root,
    virtualize,
};

#[cfg_attr(feature = "opfs_tracing", tracing::instrument(level = "trace", fields(path = %path.as_ref().to_string_lossy())))]
pub(crate) async fn open_file(
    path: impl AsRef<Path>,
    create: CreateFileMode,
    mode: SyncAccessMode,
    truncate: bool,
) -> io::Result<File> {
    let (handle, _lock, sync_access_handle) = get_file_and_lock(&path, create, mode).await?;

    if truncate {
        sync_access_handle.truncate_with_u32(0).map_err(opfs_err)?;
        sync_access_handle.flush().map_err(opfs_err)?;
        // On error the `_lock` guard is dropped, which decrements
        // ref_count and closes the cached handle when it reaches 0.
        // We must NOT close the SyncAccessHandle here because other
        // `File` objects may already be sharing it.
    }
    Ok(File {
        handle,
        sync_access_handle,
        pos: Some(0),
        mode,
        _lock,
    })
}

pub(crate) async fn lock_and_handle(
    path: impl AsRef<Path>,
    mode: Option<SyncAccessMode>,
    create: CreateFileMode,
) -> io::Result<(
    FileLockGuard,
    Option<FileSystemSyncAccessHandle>,
    FileSystemFileHandle,
)> {
    if matches!(create, CreateFileMode::CreateNew) {
        // Safety: `CreateNew` relies on a check-then-act sequence in `resolve_file_handle`.
        // We must hold the lock *before* checking existence to ensure atomicity within the app.
        let (lock, sync_handle) = lock_file(&path, mode).await;
        // Check-then-act sequence happens here, protected by the lock
        let file_handle = resolve_file_handle(&path, create).await?;
        Ok((lock, sync_handle, file_handle))
    } else {
        // optimistically race for performance in `Open` (NotCreate) and `Create` (Overwrite/Open)
        let ((lock, sync_handle), file_handle_res) =
            futures::join!(lock_file(&path, mode), resolve_file_handle(&path, create));
        let file_handle = file_handle_res?;
        Ok((lock, sync_handle, file_handle))
    }
}

pub(crate) async fn get_file_and_lock(
    path: impl AsRef<Path>,
    create: CreateFileMode,
    access_mode: SyncAccessMode,
) -> io::Result<(
    FileSystemFileHandle,
    FileLockGuard,
    FileSystemSyncAccessHandle,
)> {
    let (lock, sync_handle, file_handle) =
        lock_and_handle(&path, Some(access_mode), create).await?;

    let sync_access_handle = if let Some(h) = sync_handle {
        h
    } else {
        let h = create_sync_access_handle(&file_handle, access_mode).await?;
        set_lock_handle(&path, h.clone());
        h
    };

    Ok((file_handle, lock, sync_access_handle))
}

pub(crate) async fn resolve_file_handle(
    path: impl AsRef<Path>,
    create: CreateFileMode,
) -> io::Result<FileSystemFileHandle> {
    let (dir_entry, name) = resolve_parent(path).await?;

    match create {
        CreateFileMode::Create => get_raw_handle(&name, &dir_entry, true).await,
        CreateFileMode::CreateNew => match get_raw_handle(&name, &dir_entry, false).await {
            Ok(_) => Err(io::Error::from(io::ErrorKind::AlreadyExists)),
            Err(_) => get_raw_handle(&name, &dir_entry, true).await,
        },
        CreateFileMode::NotCreate => get_raw_handle(&name, &dir_entry, false).await,
    }
}

async fn resolve_parent(path: impl AsRef<Path>) -> io::Result<(FileSystemDirectoryHandle, String)> {
    let virt = virtualize::virtualize(path)?;
    let parent = virt.parent();

    let name = match virt.file_name() {
        Some(os_str) => Ok(os_str.to_string_lossy().to_string()),
        None => Err(io::Error::from(io::ErrorKind::InvalidFilename)),
    }?;

    let dir_entry = match parent {
        Some(parent_path) => open_dir(parent_path, OpenDirType::NotCreate).await?,
        None => root().await?,
    };
    Ok((dir_entry, name))
}

async fn get_raw_handle(
    name: &str,
    dir_entry: &FileSystemDirectoryHandle,
    create: bool,
) -> io::Result<FileSystemFileHandle> {
    let option = FileSystemGetFileOptions::new();
    option.set_create(create);
    JsFuture::from(dir_entry.get_file_handle_with_options(name, &option))
        .await
        .map_err(opfs_err)
        .map(|v| v.unchecked_into::<FileSystemFileHandle>())
}

async fn create_sync_access_handle(
    handle: &FileSystemFileHandle,
    mode: SyncAccessMode,
) -> io::Result<FileSystemSyncAccessHandle> {
    let file_handle_js_value = JsValue::from(handle);

    let promise = Reflect::get(&file_handle_js_value, &"createSyncAccessHandle".into())
        .map_err(opfs_err)?
        .unchecked_into::<Function>()
        .call1(
            &file_handle_js_value,
            &CreateSyncAccessHandleOptions::from(mode).into(),
        )
        .map_err(opfs_err)?
        .unchecked_into::<Promise>();

    JsFuture::from(promise)
        .await
        .map_err(opfs_err)
        .map(|v| v.unchecked_into::<FileSystemSyncAccessHandle>())
}
