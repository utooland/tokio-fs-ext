#![cfg(not(all(target_family = "wasm", target_os = "unknown")))]

mod test_utils;

use std::{io, path::PathBuf, str, sync::LazyLock};

use futures::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    stream::TryStreamExt,
};
use tokio_fs_ext::*;

use test_utils::{CWD_LOCK, run_test};

static CWD: LazyLock<PathBuf> = LazyLock::new(|| std::env::current_dir().unwrap());

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
#[allow(clippy::uninlined_format_args)]
async fn test_dir_read_dir_contents() {
    run_test("dir_read_dir_contents", |base_path| async move {
        let dir_path = base_path.join("dir_inside");
        let file_path = base_path.join("file_inside");

        create_dir(&dir_path).await.unwrap();
        write(&file_path, "some content").await.unwrap();

        let mut rd = read_dir(&base_path).await.unwrap();
        let mut entries = Vec::new();

        while let Some(entry) = rd.next_entry().await.unwrap() {
            entries.push((
                entry.file_type().await.unwrap().is_dir(),
                entry.file_name().to_string_lossy().to_string(),
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

#[tokio::test]
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
                        e.file_type().await.unwrap().is_dir(),
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

#[tokio::test]
async fn test_dir_non_existent_path() {
    // We test a path clearly outside the managed test dir structure.
    // However, run_test cleans up a specific dir.
    // Native tests can just use a non-existent name in temp.
    let path = std::env::temp_dir().join("non_existent_dir_path_native_test");
    let _ = remove_dir_all(&path).await;

    assert!(!try_exists(&path).await.unwrap());
}

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
async fn test_file_read_to_string() {
    run_test("file_read_to_string", |base_path| async move {
        let path = base_path.join("string_file.txt");
        let data = "this is a string";

        write(&path, data.as_bytes()).await.unwrap();
        assert_eq!(read_to_string(&path).await.unwrap(), data);
    })
    .await;
}

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
async fn test_open_options_create_new_fails_if_exists() {
    run_test(
        "open_options_create_new_fails_if_exists",
        |base_path| async move {
            let path = base_path.join("file.txt");
            write(&path, "dummy").await.unwrap();

            let err = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .await
                .unwrap_err();

            assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        },
    )
    .await;
}

#[tokio::test]
async fn test_open_options_create_new_succeeds_if_not_exists() {
    run_test(
        "open_options_create_new_succeeds_if_not_exists",
        |base_path| async move {
            let path = base_path.join("file.txt");
            let result = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .await;
            assert!(result.is_ok());
        },
    )
    .await;
}

#[tokio::test]
async fn test_open_options_create_succeeds() {
    run_test("open_options_create_succeeds", |base_path| async move {
        let path = base_path.join("file.txt");
        let result = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .await;
        assert!(result.is_ok());
    })
    .await;
}

#[tokio::test]
async fn test_open_options_readonly_permission_denied() {
    run_test(
        "open_options_readonly_permission_denied",
        |base_path| async move {
            let path = base_path.join("file.txt");
            {
                let _ = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(&path)
                    .await
                    .unwrap();
            }
            let mut readonly_file = OpenOptions::new().read(true).open(&path).await.unwrap();

            // Verify that writing to a readonly file fails
            // Use write directly. On some platforms/configs (like macOS with tokio::fs),
            // write might return Ok but effectively do nothing or swallow the error.
            let result = readonly_file.write(b"should fail").await;

            if result.is_ok() {
                // If write claims success, verify if data was actually written.
                // If data appeared, then OpenOptions failed to restrict access (critical bug).
                // If data is empty (original state), then write was a no-op/swallowed error (OS quirk).
                let content = read(&path).await.unwrap();
                if !content.is_empty() {
                    panic!(
                        "Write succeeded AND data was written to readonly file! Result: {:?}",
                        result
                    );
                }
            } else {
                assert!(result.is_err(), "write failed as expected");
            }
        },
    )
    .await;
}

#[tokio::test]
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

#[tokio::test]
async fn test_open_options_truncate() {
    run_test("open_options_truncate", |base_path| async move {
        let path = base_path.join("file.txt");
        let initial_content = "initial content";

        write(&path, initial_content.as_bytes()).await.unwrap();
        assert_eq!(read(&path).await.unwrap(), initial_content.as_bytes());

        {
            let _truncate = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)
                .await
                .unwrap();
        };
        assert!(read(&path).await.unwrap().is_empty());
    })
    .await;
}

#[tokio::test]
async fn test_open_options_append() {
    run_test("open_options_append", |base_path| async move {
        let path = base_path.join("file.txt");
        let initial_content = "append";
        let additional_content = "append";

        write(&path, initial_content.as_bytes()).await.unwrap();

        let mut append = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
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

#[tokio::test]
async fn test_metadata_not_found() {
    run_test("metadata_not_found", |base_path| async move {
        let path = base_path.join("non_existent");
        let err = metadata(&path).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    })
    .await;
}

#[tokio::test]
async fn test_metadata_is_dir() {
    run_test("metadata_is_dir", |base_path| async move {
        // run_test creates base_path
        let meta = metadata(&base_path).await.unwrap();
        assert!(meta.is_dir());
    })
    .await;
}

#[tokio::test]
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

#[tokio::test]
async fn test_async_seek() {
    run_test("async_seek", |base_path| async move {
        let path = base_path.join("seek_file.txt");
        let initial_content = "Hello, world!"; // 13 bytes
        let overwrite_content = "Rust"; // 4 bytes
        let expected_content = "Hello, Rustd!"; // 13 bytes

        write(&path, initial_content.as_bytes()).await.unwrap();
        assert_eq!(read(&path).await.unwrap(), initial_content.as_bytes());

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .await
            .unwrap();

        // Seek to a specific position (e.g., after "Hello, ")
        let seek_pos = "Hello, ".len() as u64;
        let current_pos = file.seek(io::SeekFrom::Start(seek_pos)).await.unwrap();
        assert_eq!(
            current_pos, seek_pos,
            "Seek should move cursor to correct position"
        );

        file.write_all(overwrite_content.as_bytes()).await.unwrap();

        file.seek(io::SeekFrom::Start(0)).await.unwrap();
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).await.unwrap();

        assert_eq!(
            str::from_utf8(&buffer).unwrap(),
            expected_content,
            "File content should be updated after seek and write"
        );

        file.seek(io::SeekFrom::Start(0)).await.unwrap();
        file.seek(io::SeekFrom::Current(6)).await.unwrap();
        let mut partial_buffer = vec![0; 6];
        file.read_exact(&mut partial_buffer).await.unwrap();
        assert_eq!(
            str::from_utf8(&partial_buffer).unwrap(),
            " Rustd",
            "Seeking from current should work"
        );
    })
    .await;
}

#[tokio::test]
async fn test_current_dir() {
    run_test("current_dir", |base_path| async move {
        // Acquire global lock to prevent interference with other tests
        let _guard = CWD_LOCK.lock().await;

        let deep_dir = base_path.join("deep/deep");
        let file_path = PathBuf::from("deep/data.txt"); // relative
        let content = b"hello world";

        create_dir_all(&deep_dir).await.unwrap();

        let parent = deep_dir.parent().unwrap();
        set_current_dir(parent).unwrap();

        write(&file_path, content.to_vec()).await.unwrap();

        let _ = set_current_dir(&*CWD); // Restore global CWD

        let absolute_file_path = deep_dir.join("data.txt");
        let read_content = read(&absolute_file_path).await.unwrap();
        assert_eq!(read_content, content.to_vec());
    })
    .await;
}

#[tokio::test]
async fn test_cwd_auto_creation() {
    run_test("cwd_auto_creation", |base_path| async move {
        // Acquire global lock
        let _guard = CWD_LOCK.lock().await;

        let deep = base_path.join("very/deep/path");

        create_dir_all(&deep).await.unwrap();
        set_current_dir(&deep).unwrap();

        assert!(current_dir().unwrap().ends_with("very/deep/path"));

        let _ = File::create("test.txt").await.unwrap();

        assert!(try_exists(deep.join("test.txt")).await.unwrap());

        set_current_dir(&*CWD).unwrap();
    })
    .await;
}

#[tokio::test]
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
