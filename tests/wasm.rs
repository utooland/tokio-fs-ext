#![cfg(all(target_family = "wasm", target_os = "unknown"))]

use std::{io, path::PathBuf};

use futures::{
    TryStreamExt,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use tokio_fs_ext::*;
use wasm_bindgen_test::{wasm_bindgen_test_configure, *};

wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[wasm_bindgen_test]
async fn test_dir_create_and_exists() {
    let path = "/test_dir_create_and_exists";
    let _ = remove_dir_all(path).await;

    assert!(!try_exists(path).await.unwrap());

    create_dir(path).await.unwrap();
    assert!(try_exists(path).await.unwrap());

    let _ = remove_dir_all(path).await;
    assert!(!try_exists(path).await.unwrap());
}

#[wasm_bindgen_test]
async fn test_dir_create_all_nested() {
    let path = "/test_dir_create_all_nested/sub/sub_sub";
    let base_path = "/test_dir_create_all_nested";
    let _ = remove_dir_all(base_path).await;

    assert!(!try_exists(base_path).await.unwrap());
    assert!(!try_exists(path).await.unwrap());

    create_dir_all(path).await.unwrap();
    assert!(try_exists(path).await.unwrap());
    assert!(try_exists(base_path).await.unwrap());

    let _ = remove_dir_all(base_path).await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_dir_read_dir_contents() {
    let base_path = "/test_dir_read_dir_contents";
    let dir_path = format!("{}/dir_inside", base_path);
    let file_path = format!("{}/file_inside", base_path);
    let _ = remove_dir_all(base_path).await;
    create_dir_all(base_path).await.unwrap();

    create_dir(&dir_path).await.unwrap();
    write(&file_path, "some content").await.unwrap();

    let mut rd = read_dir(base_path).await.unwrap();
    let mut entries = Vec::new();

    while let Some(e) = rd.next_entry().await.unwrap() {
        entries.push((
            e.file_type().unwrap().is_dir(),
            e.file_name().to_string_lossy().to_string(),
        ));
    }

    entries.sort_by_key(|e| e.0);

    assert_eq!(
        entries,
        vec![
            (false, "file_inside".to_string()),
            (true, "dir_inside".to_string())
        ]
    );
    assert!(rd.next_entry().await.unwrap().is_none());

    let _ = remove_dir_all(base_path).await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_dir_read_dir_stream() {
    let base_path = "/test_dir_read_dir_stream";
    let dir_path = format!("{}/dir_inside", base_path);
    let file_path = format!("{}/file_inside", base_path);
    let _ = remove_dir_all(base_path).await;
    create_dir_all(base_path).await.unwrap();

    create_dir(&dir_path).await.unwrap();
    write(&file_path, "some content").await.unwrap();

    let mut entries = futures::future::join_all(
        ReadDirStream::new(read_dir(&base_path).await.unwrap())
            .try_collect::<Vec<_>>()
            .await
            .unwrap()
            .iter()
            .map(async |e| {
                (
                    e.file_type().unwrap().is_dir(),
                    e.file_name().to_string_lossy().to_string(),
                )
            }),
    )
    .await;

    entries.sort_by_key(|e| e.0);

    assert_eq!(
        entries,
        vec![
            (false, "file_inside".to_string()),
            (true, "dir_inside".to_string())
        ]
    );

    let _ = remove_dir_all(base_path).await;
}

#[wasm_bindgen_test]
async fn test_dir_non_existent_path() {
    let path = "/non_existent_dir_path";
    let _ = remove_dir_all(path).await;

    assert!(!try_exists(path).await.unwrap());
}

// --- test_file split into smaller tests ---

#[wasm_bindgen_test]
async fn test_file_create_write_read() {
    let path = "/test_file_create_write_read/file.txt";
    let data = "hello world";
    let base_dir = "/test_file_create_write_read";
    let _ = remove_dir_all(base_dir).await;
    create_dir_all(base_dir).await.unwrap();

    assert!(!try_exists(path).await.unwrap());

    write(path, data.as_bytes()).await.unwrap();
    assert!(try_exists(path).await.unwrap());

    let read_data = read(path).await.unwrap();
    assert_eq!(read_data, data.as_bytes());

    let _ = remove_dir_all(base_dir).await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_file_copy() {
    let path = "/test_file_copy/original.txt";
    let copy_path = &format!("{path}_copy");
    let data = "copy me";
    let base_dir = "/test_file_copy";
    let _ = remove_dir_all(base_dir).await;
    create_dir_all(base_dir).await.unwrap();

    write(path, data.as_bytes()).await.unwrap();
    copy(path, copy_path).await.unwrap();

    assert!(try_exists(copy_path).await.unwrap());
    assert_eq!(read(copy_path).await.unwrap(), data.as_bytes());

    let _ = remove_dir_all(base_dir).await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_file_rename() {
    let path = "/test_file_rename/old_name.txt";
    let rename_path = &format!("{path}_rename");
    let data = "rename me";
    let base_dir = "/test_file_rename";
    let _ = remove_dir_all(base_dir).await;
    create_dir_all(base_dir).await.unwrap();

    write(path, data.as_bytes()).await.unwrap();
    rename(path, rename_path).await.unwrap();

    assert!(!try_exists(path).await.unwrap());
    assert!(try_exists(rename_path).await.unwrap());
    assert_eq!(read(rename_path).await.unwrap(), data.as_bytes());

    let _ = remove_dir_all(base_dir).await;
}

#[wasm_bindgen_test]
async fn test_file_read_to_string() {
    let path = "/test_file_read_to_string/string_file.txt";
    let data = "this is a string";
    let base_dir = "/test_file_read_to_string";
    let _ = remove_dir_all(base_dir).await;
    create_dir_all(base_dir).await.unwrap();

    write(path, data.as_bytes()).await.unwrap();
    assert_eq!(read_to_string(path).await.unwrap(), data);

    let _ = remove_dir_all(base_dir).await;
}

#[wasm_bindgen_test]
async fn test_file_read_to_end_small() {
    let path = "/test_file_read_to_end/test_file_read_to_end_small.txt";
    let data = "this is for read_to_end ";
    let base_dir = "/test_file_read_to_end";
    let _ = remove_dir_all(base_dir).await;
    create_dir_all(base_dir).await.unwrap();

    write(path, data.as_bytes()).await.unwrap();
    let mut file = OpenOptions::new().read(true).open(path).await.unwrap();
    let mut buffer = vec![];

    assert!(file.read_to_end(&mut buffer).await.is_ok());
    assert_eq!(str::from_utf8(&buffer).unwrap(), data);

    let _ = remove_dir_all(base_dir).await;
}

#[wasm_bindgen_test]
async fn test_file_read_to_end_big() {
    let path = "/test_file_read_to_end/test_file_read_to_end_big.txt";
    let data = "this is for read_to_end ".repeat(10);
    let base_dir = "/test_file_read_to_end";
    let _ = remove_dir_all(base_dir).await;
    create_dir_all(base_dir).await.unwrap();

    write(path, data.as_bytes()).await.unwrap();
    let mut file = OpenOptions::new().read(true).open(path).await.unwrap();
    let mut buffer = vec![];

    assert!(file.read_to_end(&mut buffer).await.is_ok());
    assert_eq!(str::from_utf8(&buffer).unwrap(), data);

    let _ = remove_dir_all(base_dir).await;
}

#[wasm_bindgen_test]
async fn test_file_remove() {
    let path = "/test_file_remove/file_to_remove.txt";
    let base_dir = "/test_file_remove";
    let _ = remove_dir_all(base_dir).await;
    create_dir_all(base_dir).await.unwrap();

    write(path, "content").await.unwrap();
    assert!(try_exists(path).await.unwrap());

    remove_file(path).await.unwrap();
    assert!(!try_exists(path).await.unwrap());

    let _ = remove_dir_all(base_dir).await;
}

#[wasm_bindgen_test]
async fn test_open_options_create_new_fails_if_exists() {
    let path = "/test_open_options_create_new_fails_if_exists";
    let _ = remove_file(path).await;
    write(path, "dummy").await.unwrap();

    let err = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .await
        .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);

    let _ = remove_file(path).await;
}

#[wasm_bindgen_test]
async fn test_open_options_create_new_succeeds_if_not_exists() {
    let path = "/test_open_options_create_new_succeeds_if_not_exists";
    let _ = remove_file(path).await;

    let result = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .await;
    assert!(result.is_ok());

    let _ = remove_file(path).await;
}

#[wasm_bindgen_test]
async fn test_open_options_create_succeeds() {
    let path = "/test_open_options_create_succeeds";
    let _ = remove_file(path).await;

    let result = OpenOptions::new().create(true).write(true).open(path).await;
    assert!(result.is_ok());

    let _ = remove_file(path).await;
}

#[wasm_bindgen_test]
async fn test_open_options_readonly_permission_denied() {
    let path = "/test_open_options_readonly_permission_denied";

    {
        let _ = remove_file(path).await;
        let _readonly_file = OpenOptions::new().create(true).open(path).await.unwrap();
    }

    let _readonly_file = OpenOptions::new().read(true).open(path).await.unwrap();

    let err = write(path, "attempt to write").await.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);

    let _ = remove_file(path).await;
}

#[wasm_bindgen_test]
async fn test_open_options_read_write_behavior() {
    let path = "/test_open_options_read_write_behavior";
    let contents = "somedata".repeat(16);
    let _ = remove_file(path).await;

    {
        let mut rw_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)
            .await
            .unwrap();

        assert!(rw_file.write(contents.as_bytes()).await.is_ok());
        rw_file.seek(io::SeekFrom::Start(0)).await.unwrap();
        let mut data = vec![];
        assert!(rw_file.read_to_end(&mut data).await.is_ok());
        assert_eq!(data.as_slice(), contents.as_bytes());
    }

    {
        let mut rw_file = OpenOptions::new().read(true).open(path).await.unwrap();

        let mut data = vec![];
        assert!(rw_file.read_to_end(&mut data).await.is_ok());
        assert_eq!(data.as_slice(), contents.as_bytes());
    }

    let _ = remove_file(path).await;
}

#[wasm_bindgen_test]
async fn test_open_options_truncate() {
    let path = "/test_open_options_truncate";
    let initial_content = "initial content";
    let _ = remove_file(path).await;

    write(path, initial_content.as_bytes()).await.unwrap();
    assert_eq!(read(path).await.unwrap(), initial_content.as_bytes());

    {
        let _truncate = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .await
            .unwrap();
        // https://developer.mozilla.org/en-US/docs/Web/API/FileSystemFileHandle/createSyncAccessHandle#readwrite
        assert_eq!(
            OpenOptions::new()
                .read(true)
                .open(path)
                .await
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        // Async read might succeed depending on implementation, but sync open MUST fail.
    };
    assert!(read(path).await.unwrap().is_empty());

    let _ = remove_file(path).await;
}

#[wasm_bindgen_test]
async fn test_open_options_append() {
    let path = "/test_open_options_append";
    let initial_content = "append";
    let additional_content = "append";
    let _ = remove_file(path).await;

    write(path, initial_content.as_bytes()).await.unwrap();

    let mut append = OpenOptions::new()
        .append(true)
        .read(true)
        .write(true)
        .open(path)
        .await
        .unwrap();

    append
        .write_all(additional_content.as_bytes())
        .await
        .unwrap();
    append.seek(io::SeekFrom::Start(0)).await.unwrap();
    let mut data = vec![];
    append.read_to_end(&mut data).await.unwrap();
    assert_eq!(
        data.as_slice(),
        (initial_content.to_string() + additional_content).as_bytes()
    );

    let _ = remove_file(path).await;
}

// --- test_metadata split into smaller tests ---

#[wasm_bindgen_test]
async fn test_metadata_not_found() {
    let path = "/test_metadata_not_found";
    let _ = remove_dir_all(path).await;

    let err = metadata(path).await.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}

#[wasm_bindgen_test]
async fn test_metadata_is_dir() {
    let path = "/test_metadata_is_dir";
    let _ = remove_dir_all(path).await;

    create_dir(path).await.unwrap();
    let meta = metadata(path).await.unwrap();
    assert!(meta.is_dir());

    let _ = remove_dir_all(path).await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_metadata_is_file_and_len() {
    let dir_path = "/test_metadata_is_file_and_len_dir";
    let file_path = &format!("{}/file_with_content.txt", dir_path);
    let content = "some file content";
    let _ = remove_dir_all(dir_path).await;
    create_dir(dir_path).await.unwrap();

    write(file_path, content.as_bytes()).await.unwrap();
    let f_metadata = metadata(file_path).await.unwrap();

    assert!(f_metadata.is_file());
    assert_eq!(f_metadata.len(), content.len() as u64);

    let _ = remove_dir_all(dir_path).await;
}

#[wasm_bindgen_test]
async fn test_async_seek() {
    let path = "/test_async_seek/seek_file.txt";
    let initial_content = "Hello, world!"; // 13 bytes
    let overwrite_content = "Rust"; // 4 bytes
    let expected_content = "Hello, Rustd!"; // 13 bytes
    let base_dir = "/test_async_seek";
    let _ = remove_dir_all(base_dir).await;
    create_dir_all(base_dir).await.unwrap();

    write(path, initial_content.as_bytes()).await.unwrap();
    assert_eq!(read(path).await.unwrap(), initial_content.as_bytes());

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .await
        .unwrap();

    // Seek to a specific position (e.g., after "Hello, ")
    let seek_pos = "Hello, ".len() as u64;
    let current_pos = file.seek(io::SeekFrom::Start(seek_pos)).await.unwrap();
    assert_eq!(
        current_pos, seek_pos,
        "Seek should move cursor to correct position"
    );

    // Write new content from that position
    file.write_all(overwrite_content.as_bytes()).await.unwrap();

    // Seek back to the beginning to read the entire content
    file.seek(io::SeekFrom::Start(0)).await.unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).await.unwrap();

    // Verify the content
    assert_eq!(
        str::from_utf8(&buffer).unwrap(),
        expected_content,
        "File content should be updated after seek and write"
    );

    // Test seeking from current position
    file.seek(io::SeekFrom::Start(0)).await.unwrap(); // Reset to start
    file.seek(io::SeekFrom::Current(6)).await.unwrap(); // Move 6 bytes forward
    let mut partial_buffer = vec![0; 6]; // Read " Rust!"
    file.read_exact(&mut partial_buffer).await.unwrap();
    assert_eq!(
        str::from_utf8(&partial_buffer).unwrap(),
        " Rustd",
        "Seeking from current should work"
    );

    // Clean up
    let _ = remove_dir_all(base_dir).await;
}

#[wasm_bindgen_test]
async fn test_current_dir() {
    assert_eq!(current_dir().unwrap().to_string_lossy(), "/");

    let deep_dir = PathBuf::from("/test_current_dir/deep/deep");
    let file_path = PathBuf::from("./deep/data.txt");
    let content = b"hello world";

    create_dir_all(&deep_dir).await.unwrap();

    set_current_dir(deep_dir.parent().unwrap()).unwrap();

    write(&file_path, content.to_vec()).await.unwrap();

    set_current_dir("/").unwrap();

    let read_content = read(deep_dir.join("data.txt")).await.unwrap();

    assert_eq!(read_content, content.to_vec());

    let _ = remove_dir_all(deep_dir).await;
}
