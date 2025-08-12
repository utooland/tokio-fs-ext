use std::{io, path::Path};

use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemDirectoryHandle, FileSystemGetDirectoryOptions};

use super::{OpfsError, options::OpenDirType, root::opfs_root, virtualize};

pub(crate) async fn open_dir(
    path: impl AsRef<Path>,
    r#type: OpenDirType,
) -> io::Result<SendWrapper<FileSystemDirectoryHandle>> {
    let virt = virtualize::virtualize(path)?;

    let components = virt
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>();

    let total_depth = components.len();

    let mut dir_handle = opfs_root().await?;

    let mut found = 0_usize;

    for c in components.iter() {
        dir_handle = get_dir_handle(
            &dir_handle,
            c,
            matches!(r#type, OpenDirType::Create | OpenDirType::CreateRecursive),
        )
        .await?;
        found += 1;
    }

    if found != total_depth {
        return Err(io::Error::from(io::ErrorKind::NotFound));
    }

    Ok(dir_handle)
}

pub(crate) async fn get_dir_handle(
    parent: &SendWrapper<FileSystemDirectoryHandle>,
    path: &str,
    create: bool,
) -> io::Result<SendWrapper<FileSystemDirectoryHandle>> {
    let options = SendWrapper::new(FileSystemGetDirectoryOptions::new());
    options.set_create(create);

    let dir_handle = SendWrapper::new(JsFuture::from(
        parent.get_directory_handle_with_options(path, &options),
    ))
    .await
    .map_err(|err| OpfsError::from(err).into_io_err())?
    .unchecked_into::<FileSystemDirectoryHandle>();
    Ok(SendWrapper::new(dir_handle))
}
