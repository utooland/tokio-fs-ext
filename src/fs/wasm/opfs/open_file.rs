use std::{io, path::Path};

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetFileOptions,
    FileSystemSyncAccessHandle,
};

use crate::current_dir;

use super::{
    super::File,
    OpenDirType,
    error::OpfsError,
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
    let handle = get_fs_handle(path, create).await?;

    let sync_access_handle = create_sync_access_handle(&handle, mode).await?;

    if truncate {
        sync_access_handle.truncate_with_u32(0).map_or_else(
            |err| {
                sync_access_handle.close();
                Err(OpfsError::from(err).into_io_err())
            },
            |_| {
                sync_access_handle.flush().map_err(|err| {
                    sync_access_handle.close();
                    OpfsError::from(err).into_io_err()
                })
            },
        )?;
    }
    Ok(File {
        sync_access_handle,
        pos: None,
    })
}

pub(crate) async fn get_fs_handle(
    path: impl AsRef<Path>,
    create: CreateFileMode,
) -> io::Result<FileSystemFileHandle> {
    let (dir_entry, name) = resolve_parent(path.as_ref()).await?;

    match create {
        CreateFileMode::Create => get_raw_handle(&name, &dir_entry, true).await,
        CreateFileMode::CreateNew => {
            match get_raw_handle(&name, &dir_entry, false).await {
                Ok(_) => Err(io::Error::from(io::ErrorKind::AlreadyExists)),
                Err(_) => get_raw_handle(&name, &dir_entry, true).await,
            }
        }
        CreateFileMode::NotCreate => get_raw_handle(&name, &dir_entry, false).await,
    }
}

async fn resolve_parent(path: &Path) -> io::Result<(FileSystemDirectoryHandle, String)> {
    let virt = virtualize::virtualize(path)?;
    let parent = virt.parent();

    let name = match virt.file_name() {
        Some(os_str) => Ok(os_str.to_string_lossy().to_string()),
        None => Err(io::Error::from(io::ErrorKind::InvalidFilename)),
    }?;

    let dir_entry = match parent {
        Some(parent_path) => {
            open_dir(
                parent_path,
                if parent_path == current_dir()? {
                    OpenDirType::CreateRecursive
                } else {
                    OpenDirType::NotCreate
                },
            )
            .await?
        }
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
        .map_err(|err| OpfsError::from(err).into_io_err())
        .map(|v| v.unchecked_into::<FileSystemFileHandle>())
}

async fn create_sync_access_handle(
    handle: &FileSystemFileHandle,
    mode: SyncAccessMode,
) -> io::Result<FileSystemSyncAccessHandle> {
    let file_handle_js_value = JsValue::from(handle);

    let promise = Reflect::get(&file_handle_js_value, &"createSyncAccessHandle".into())
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<Function>()
        .call1(
            &file_handle_js_value,
            &CreateSyncAccessHandleOptions::from(mode).into(),
        )
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<Promise>();

    JsFuture::from(promise)
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())
        .map(|v| v.unchecked_into::<FileSystemSyncAccessHandle>())
}
