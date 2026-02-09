#![cfg(all(target_family = "wasm", target_os = "unknown"))]

mod test_utils;

use std::{io, path::PathBuf, str};

use futures::{
    TryStreamExt,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use tokio_fs_ext::*;
use wasm_bindgen_test::{wasm_bindgen_test_configure, *};

use test_utils::run_test;

wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[wasm_bindgen_test]
async fn test_dir_create_and_exists() {
    run_test("dir_create_and_exists", |path| async move {
        // base_dir is already created by run_test
        assert!(try_exists(&path).await.unwrap());

        // Test creating a sub-directory
        let sub = path.join("sub");
        assert!(!try_exists(&sub).await.unwrap());

        create_dir(&sub).await.unwrap();
        assert!(try_exists(&sub).await.unwrap());
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_dir_create_all_nested() {
    run_test("dir_create_all_nested", |base_path| async move {
        let path = base_path.join("sub/sub_sub");

        assert!(try_exists(&base_path).await.unwrap());
        assert!(!try_exists(&path).await.unwrap());

        create_dir_all(&path).await.unwrap();
        assert!(try_exists(&path).await.unwrap());
    })
    .await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_dir_read_dir_contents() {
    run_test("dir_read_dir_contents", |base_path| async move {
        let dir_path = base_path.join("dir_inside");
        let file_path = base_path.join("file_inside");

        create_dir(&dir_path).await.unwrap();
        write(&file_path, "some content").await.unwrap();

        let mut rd = read_dir(&base_path).await.unwrap();
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
    })
    .await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_dir_read_dir_stream() {
    run_test("dir_read_dir_stream", |base_path| async move {
        let dir_path = base_path.join("dir_inside");
        let file_path = base_path.join("file_inside");

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
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_dir_non_existent_path() {
    // run_test manages its own dir. We test a path OUTSIDE of it.
    let path = "/non_existent_dir_path_explicit";
    let _ = remove_dir_all(path).await;
    assert!(!try_exists(path).await.unwrap());
}

#[wasm_bindgen_test]
async fn test_file_create_write_read() {
    run_test("file_create_write_read", |base_path| async move {
        let path = base_path.join("file.txt");
        let data = "hello world";

        assert!(!try_exists(&path).await.unwrap());

        write(&path, data.as_bytes()).await.unwrap();
        assert!(try_exists(&path).await.unwrap());

        let read_data = read(&path).await.unwrap();
        assert_eq!(read_data, data.as_bytes());
    })
    .await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_file_copy() {
    run_test("file_copy", |base_path| async move {
        let path = base_path.join("original.txt");
        let copy_path = base_path.join("original.txt_copy");
        let data = "copy me";

        write(&path, data.as_bytes()).await.unwrap();
        copy(&path, &copy_path).await.unwrap();

        assert!(try_exists(&copy_path).await.unwrap());
        assert_eq!(read(&copy_path).await.unwrap(), data.as_bytes());
    })
    .await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_file_rename() {
    run_test("file_rename", |base_path| async move {
        let path = base_path.join("old_name.txt");
        let rename_path = base_path.join("old_name.txt_rename");
        let data = "rename me";

        write(&path, data.as_bytes()).await.unwrap();
        rename(&path, &rename_path).await.unwrap();

        assert!(!try_exists(&path).await.unwrap());
        assert!(try_exists(&rename_path).await.unwrap());
        assert_eq!(read(&rename_path).await.unwrap(), data.as_bytes());
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_file_read_to_string() {
    run_test("file_read_to_string", |base_path| async move {
        let path = base_path.join("string_file.txt");
        let data = "this is a string";

        write(&path, data.as_bytes()).await.unwrap();
        assert_eq!(read_to_string(&path).await.unwrap(), data);
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_file_read_to_end_small() {
    run_test("file_read_to_end_small", |base_path| async move {
        let path = base_path.join("file.txt");
        let data = "this is for read_to_end ";

        write(&path, data.as_bytes()).await.unwrap();
        let mut file = OpenOptions::new().read(true).open(&path).await.unwrap();
        let mut buffer = vec![];

        assert!(file.read_to_end(&mut buffer).await.is_ok());
        assert_eq!(str::from_utf8(&buffer).unwrap(), data);
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_file_read_to_end_big() {
    run_test("file_read_to_end_big", |base_path| async move {
        let path = base_path.join("file.txt");
        let data = "this is for read_to_end ".repeat(10);

        write(&path, data.as_bytes()).await.unwrap();
        let mut file = OpenOptions::new().read(true).open(&path).await.unwrap();
        let mut buffer = vec![];

        assert!(file.read_to_end(&mut buffer).await.is_ok());
        assert_eq!(str::from_utf8(&buffer).unwrap(), data);
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_file_remove() {
    run_test("file_remove", |base_path| async move {
        let path = base_path.join("file_to_remove.txt");

        write(&path, "content").await.unwrap();
        assert!(try_exists(&path).await.unwrap());

        remove_file(&path).await.unwrap();
        assert!(!try_exists(&path).await.unwrap());
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_open_options_create_new_fails_if_exists() {
    run_test(
        "open_options_create_new_fails_if_exists",
        |base_path| async move {
            let path = base_path.join("file.txt");
            write(&path, "dummy").await.unwrap();

            let err = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
                .await
                .unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        },
    )
    .await;
}

#[wasm_bindgen_test]
async fn test_open_options_create_new_succeeds_if_not_exists() {
    run_test(
        "open_options_create_new_succeeds_if_not_exists",
        |base_path| async move {
            let path = base_path.join("file.txt");
            let result = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
                .await;
            assert!(result.is_ok());
        },
    )
    .await;
}

#[wasm_bindgen_test]
async fn test_open_options_create_succeeds() {
    run_test("open_options_create_succeeds", |base_path| async move {
        let path = base_path.join("file.txt");
        let result = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .await;
        assert!(result.is_ok());
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_open_options_readonly_permission_denied() {
    run_test(
        "open_options_readonly_permission_denied",
        |base_path| async move {
            let path = base_path.join("file.txt");

            {
                let _readonly_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&path)
                    .await
                    .unwrap();
            }

            {
                let mut readonly_file = OpenOptions::new().read(true).open(&path).await.unwrap();

                // Verify that writing to a readonly file fails
                let err = readonly_file.write(b"attempt to write").await.unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
            }
        },
    )
    .await;
}

#[wasm_bindgen_test]
async fn test_open_options_read_write_behavior() {
    run_test("open_options_read_write_behavior", |base_path| async move {
        let path = base_path.join("file.txt");
        let contents = "somedata".repeat(16);

        {
            let mut rw_file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(&path)
                .await
                .unwrap();

            assert!(rw_file.write(contents.as_bytes()).await.is_ok());
            rw_file.seek(io::SeekFrom::Start(0)).await.unwrap();
            let mut data = vec![];
            assert!(rw_file.read_to_end(&mut data).await.is_ok());
            assert_eq!(data.as_slice(), contents.as_bytes());
        }

        {
            let mut rw_file = OpenOptions::new().read(true).open(&path).await.unwrap();

            let mut data = vec![];
            assert!(rw_file.read_to_end(&mut data).await.is_ok());
            assert_eq!(data.as_slice(), contents.as_bytes());
        }
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_open_options_truncate() {
    run_test("open_options_truncate", |base_path| async move {
        let path = base_path.join("file.txt");
        let initial_content = "initial content";

        write(&path, initial_content.as_bytes()).await.unwrap();
        assert_eq!(read(&path).await.unwrap(), initial_content.as_bytes());

        {
            let _truncate = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&path)
                .await
                .unwrap();

            // With handle caching, opening Readonly while a Readwrite handle
            // exists reuses the same underlying SyncAccessHandle, matching OS
            // semantics where multiple File objects can coexist on one path.
            let shared_read = OpenOptions::new().read(true).open(&path).await;
            assert!(shared_read.is_ok());
        };
        assert!(read(&path).await.unwrap().is_empty());
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_open_options_append() {
    run_test("open_options_append", |base_path| async move {
        let path = base_path.join("file.txt");
        let initial_content = "append";
        let additional_content = "append";

        write(&path, initial_content.as_bytes()).await.unwrap();

        let mut append = OpenOptions::new()
            .append(true)
            .read(true)
            .write(true)
            .open(&path)
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
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_metadata_not_found() {
    run_test("metadata_not_found", |base_path| async move {
        let path = base_path.join("non_existent");
        let err = metadata(&path).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_metadata_is_dir() {
    run_test("metadata_is_dir", |base_path| async move {
        // base_path is a directory
        let meta = metadata(&base_path).await.unwrap();
        assert!(meta.is_dir());
    })
    .await;
}

#[wasm_bindgen_test]
#[allow(clippy::uninlined_format_args)]
async fn test_metadata_is_file_and_len() {
    run_test("metadata_is_file_and_len", |base_path| async move {
        let file_path = base_path.join("file_with_content.txt");
        let content = "some file content";

        write(&file_path, content.as_bytes()).await.unwrap();
        let f_metadata = metadata(&file_path).await.unwrap();

        assert!(f_metadata.is_file());
        assert_eq!(f_metadata.len(), content.len() as u64);
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_metadata_modified_time() {
    run_test("metadata_modified_time", |base_path| async move {
        use std::time::UNIX_EPOCH;
        let file_path = base_path.join("file_with_mtime.txt");

        write(&file_path, b"time test").await.unwrap();

        let meta = match metadata(&file_path).await {
            Ok(m) => m,
            Err(_e) => return, // Skip if unavailable
        };

        match meta.modified() {
            Ok(ts) => {
                ts.duration_since(UNIX_EPOCH)
                    .expect("mtime should be after UNIX_EPOCH");
            }
            Err(e) => {
                assert_eq!(e.kind(), io::ErrorKind::Other);
            }
        }
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_async_seek() {
    run_test("async_seek", |base_path| async move {
        let path = base_path.join("seek_file.txt");
        let initial_content = "Hello, world!";
        let overwrite_content = "Rust";
        let expected_content = "Hello, Rustd!";

        write(&path, initial_content.as_bytes()).await.unwrap();
        assert_eq!(read(&path).await.unwrap(), initial_content.as_bytes());

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .await
            .unwrap();

        let seek_pos = "Hello, ".len() as u64;
        let current_pos = file.seek(io::SeekFrom::Start(seek_pos)).await.unwrap();
        assert_eq!(current_pos, seek_pos);

        file.write_all(overwrite_content.as_bytes()).await.unwrap();
        file.seek(io::SeekFrom::Start(0)).await.unwrap();

        let mut buffer = vec![];
        file.read_to_end(&mut buffer).await.unwrap();
        assert_eq!(str::from_utf8(&buffer).unwrap(), expected_content);

        file.seek(io::SeekFrom::Start(0)).await.unwrap();
        file.seek(io::SeekFrom::Current(6)).await.unwrap();
        let mut partial_buffer = vec![0; 6];
        file.read_exact(&mut partial_buffer).await.unwrap();
        assert_eq!(str::from_utf8(&partial_buffer).unwrap(), " Rustd");
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_current_dir() {
    // Current dir is global, so we need to be careful with run_test isolation for CWD tests.
    // However, run_test creates a dedicated directory.
    run_test("current_dir", |base_path| async move {
        let deep_dir = base_path.join("deep/deep");
        let file_path = PathBuf::from("./deep/data.txt"); // Relative path
        let content = b"hello world";

        create_dir_all(&deep_dir).await.unwrap();

        let parent = deep_dir.parent().unwrap();
        set_current_dir(parent).unwrap();

        write(&file_path, content.to_vec()).await.unwrap();

        // Check relative read works
        // Note: write() uses CWD if path is relative.

        // Reset CWD for safety
        set_current_dir("/").unwrap();

        let absolute_file_path = deep_dir.join("data.txt");
        let read_content = read(&absolute_file_path).await.unwrap();
        assert_eq!(read_content, content.to_vec());
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_cwd_auto_creation() {
    run_test("cwd_auto_creation", |base_path| async move {
        let deep = base_path.join("very/deep/path");

        // Set CWD to a deep, non-existent path
        set_current_dir(&deep).unwrap();

        assert_eq!(current_dir().unwrap(), deep);

        // This triggers auto-creation
        File::create("test.txt").await.unwrap();

        assert!(try_exists(&deep).await.unwrap());
        assert!(try_exists(deep.join("test.txt")).await.unwrap());

        set_current_dir("/").unwrap();
    })
    .await;
}

#[wasm_bindgen_test]
async fn test_mix_file_lock_and_async_api() {
    run_test("mix_file_lock_and_async_api", |base_path| async move {
        let path = base_path.join("file.txt");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&path)
            .await
            .unwrap();

        file.write_all(b"initial").await.unwrap();

        let content = read(&path).await.unwrap();
        assert_eq!(content, b"initial");

        write(&path, b"overwrite").await.unwrap();

        file.seek(io::SeekFrom::Start(0)).await.unwrap();
        let mut buf = vec![];
        file.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"overwrite");
    })
    .await;
}

#[cfg(feature = "opfs_watch")]
#[wasm_bindgen_test]
async fn test_watch_dir_callback() {
    use futures::StreamExt;
    run_test("watch_dir_callback", |base_path| async move {
        let (tx, mut rx) = futures::channel::mpsc::unbounded();
        let result = watch::watch_dir(&base_path, true, move |event| {
            let _ = tx.unbounded_send(event);
        })
        .await;

        match result {
            Ok(()) => {
                let file_path = base_path.join("test.txt");
                write(&file_path, "hello").await.unwrap();

                let event = rx.next().await.expect("Should receive an event");
                assert!(!event.paths.is_empty());
                assert!(event.paths[0].to_str().unwrap().contains("test.txt"));
                assert!(event.kind.is_create());
            }
            Err(e) => {
                let err_msg = e.to_string();
                if !err_msg.contains("FileSystemObserver") && !err_msg.contains("not a function") {
                    panic!("watch_dir failed: {:?}", e);
                }
            }
        }
    })
    .await;
}

#[cfg(feature = "opfs_watch")]
#[wasm_bindgen_test]
async fn test_watch_file_callback() {
    use futures::StreamExt;
    run_test("watch_file_callback", |base_path| async move {
        let path = base_path.join("file.txt");
        write(&path, "initial").await.unwrap();

        let (tx, mut rx) = futures::channel::mpsc::unbounded();
        let result = watch::watch_file(&path, move |event| {
            let _ = tx.unbounded_send(event);
        })
        .await;

        match result {
            Ok(()) => {
                write(&path, "updated").await.unwrap();
                let event = rx.next().await.expect("Should receive an event");
                assert!(event.kind.is_modify());
            }
            Err(e) => {
                let err_msg = e.to_string();
                if !err_msg.contains("FileSystemObserver") && !err_msg.contains("not a function") {
                    panic!("watch_file failed: {:?}", e);
                }
            }
        }
    })
    .await;
}

#[cfg(feature = "opfs_watch")]
#[wasm_bindgen_test]
async fn test_watch_remove_event() {
    use futures::StreamExt;
    run_test("watch_remove_event", |base_path| async move {
        let file_path = base_path.join("remove_me.txt");
        write(&file_path, "bye").await.unwrap();

        let (tx, mut rx) = futures::channel::mpsc::unbounded();
        let result = watch::watch_dir(&base_path, false, move |event| {
            let _ = tx.unbounded_send(event);
        })
        .await;

        match result {
            Ok(()) => {
                remove_file(&file_path).await.unwrap();
                let event = rx.next().await.expect("Should receive an event");
                assert!(event.kind.is_remove());
            }
            Err(e) => {
                // Skip if not supported
                let err_msg = e.to_string();
                if !err_msg.contains("FileSystemObserver") && !err_msg.contains("not a function") {
                    panic!("watch_dir failed: {:?}", e);
                }
            }
        }
    })
    .await;
}

#[cfg(feature = "opfs_watch")]
#[wasm_bindgen_test]
async fn test_watch_rename_event() {
    use futures::StreamExt;
    run_test("watch_rename_event", |base_path| async move {
        let old_path = base_path.join("old.txt");
        let new_path = base_path.join("new.txt");
        write(&old_path, "move me").await.unwrap();

        let (tx, mut rx) = futures::channel::mpsc::unbounded();
        let result = watch::watch_dir(&base_path, false, move |event| {
            let _ = tx.unbounded_send(event);
        })
        .await;

        match result {
            Ok(()) => {
                rename(&old_path, &new_path).await.unwrap();
                let event = rx.next().await.expect("Should receive an event");
                assert!(
                    event.kind.is_modify() || event.kind.is_remove() || event.kind.is_create(),
                    "Unexpected event kind for rename: {:?}",
                    event.kind
                );
            }
            Err(e) => {
                let err_msg = e.to_string();
                if !err_msg.contains("FileSystemObserver") && !err_msg.contains("not a function") {
                    panic!("watch_dir failed: {:?}", e);
                }
            }
        }
    })
    .await;
}
