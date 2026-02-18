#![allow(missing_docs)]

use std::path::PathBuf;

use bytes::Bytes;
use futures::{channel::mpsc, stream};
use rust_multer::{DiskStorage, FilenameStrategy, Multer, MulterError, Multipart};
use rust_multer::storage::disk::sanitize_filename;
use uuid::Uuid;

#[tokio::test]
async fn keep_strategy_sanitizes_filename_and_writes_to_disk() {
    let root = temp_root();
    let storage = DiskStorage::builder()
        .destination(&root)
        .filename(FilenameStrategy::Keep)
        .build()
        .expect("builder should succeed");
    let multer = Multer::new(storage);

    let body = multipart_body(&[("upload", "..\\..\\bad:name?.txt", "text/plain", "hello")]);
    let mut multipart =
        Multipart::new("BOUND", bytes_stream(body)).expect("multipart should initialize");
    let part = multipart
        .next_part().await.expect("part should parse").expect("part expected");

    let stored = multer.store(part).await.expect("store should succeed");
    let path = stored.path.clone().expect("disk storage should return a path");
    assert!(path.starts_with(&root));
    assert_eq!(stored.size, 5);
    assert_eq!(tokio::fs::read(&path).await.expect("read file"), b"hello");

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .expect("valid filename");
    assert!(!file_name.contains(".."));
    assert!(!file_name.contains(':'));

    cleanup(root).await;
}

#[tokio::test]
async fn random_strategy_generates_distinct_paths() {
    let root = temp_root();
    let storage = DiskStorage::builder()
        .destination(&root)
        .filename(FilenameStrategy::Random)
        .build()
        .expect("builder should succeed");
    let multer = Multer::new(storage);

    let body = multipart_body(&[
        ("a", "same.txt", "text/plain", "one"),
        ("b", "same.txt", "text/plain", "two"),
    ]);
    let mut multipart =
        Multipart::new("BOUND", bytes_stream(body)).expect("multipart should initialize");

    let first = multipart
        .next_part().await.expect("first should parse").expect("first expected");
    let first_stored = multer.store(first).await.expect("first store");
    let second = multipart
        .next_part().await.expect("second should parse").expect("second expected");
    let second_stored = multer.store(second).await.expect("second store");
    assert_ne!(first_stored.path, second_stored.path);

    cleanup(root).await;
}

#[tokio::test]
async fn custom_strategy_applies_transform() {
    let root = temp_root();
    let storage = DiskStorage::builder()
        .destination(&root)
        .custom_filename(|incoming| format!("prefix-{incoming}"))
        .build()
        .expect("builder should succeed");
    let multer = Multer::new(storage);

    let body = multipart_body(&[("doc", "report.txt", "text/plain", "payload")]);
    let mut multipart =
        Multipart::new("BOUND", bytes_stream(body)).expect("multipart should initialize");
    let part = multipart
        .next_part().await.expect("part should parse").expect("part expected");

    let stored = multer.store(part).await.expect("store should succeed");
    let file_name = stored
        .path
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|value| value.to_str())
        .expect("valid filename");
    assert!(file_name.starts_with("prefix-report"));

    cleanup(root).await;
}

#[tokio::test]
async fn disk_filter_can_reject_files_before_write() {
    let root = temp_root();
    let storage = DiskStorage::builder()
        .destination(&root)
        .filename(FilenameStrategy::Keep)
        .filter(|meta| meta.file_name.as_deref() != Some("reject.txt"))
        .build()
        .expect("builder should succeed");
    let multer = Multer::new(storage);

    let body = multipart_body(&[("upload", "reject.txt", "text/plain", "hello")]);
    let mut multipart =
        Multipart::new("BOUND", bytes_stream(body)).expect("multipart should initialize");
    let part = multipart
        .next_part().await.expect("part should parse").expect("part expected");

    let err = multer.store(part).await.expect_err("filter should reject file");
    assert!(err.to_string().contains("filter rejected"));
    assert!(!tokio::fs::try_exists(&root).await.expect("try_exists should succeed"));

    cleanup(root).await;
}

#[test]
fn sanitize_filename_rejects_traversal_and_null_bytes() {
    let traversal = sanitize_filename("../../etc/passwd");
    assert!(!traversal.contains(".."));
    assert!(!traversal.contains('/'));
    assert!(!traversal.contains('\\'));

    let nul = sanitize_filename("..\\..\\nul\0byte?.txt");
    assert!(!nul.contains('\0'));
    assert!(!nul.contains(".."));
    assert!(!nul.contains('?'));
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("rust-multer-test-{}", Uuid::new_v4()))
}

async fn cleanup(path: PathBuf) {
    let _ = tokio::fs::remove_dir_all(path).await;
}

fn multipart_body(parts: &[(&str, &str, &str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (field, file_name, content_type, body) in parts {
        out.extend_from_slice(b"--BOUND\r\n");
        let disposition =
            format!("Content-Disposition: form-data; name=\"{field}\"; filename=\"{file_name}\"\r\n");
        out.extend_from_slice(disposition.as_bytes());
        let content_type = format!("Content-Type: {content_type}\r\n\r\n");
        out.extend_from_slice(content_type.as_bytes());
        out.extend_from_slice(body.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(b"--BOUND--\r\n");
    out
}

fn bytes_stream(body: Vec<u8>) -> impl futures::Stream<Item = Result<Bytes, MulterError>> {
    stream::iter([Ok(Bytes::from(body))])
}

#[tokio::test]
async fn streams_large_file_to_disk_from_chunked_input() {
    let root = temp_root();
    let storage = DiskStorage::builder()
        .destination(&root)
        .filename(FilenameStrategy::Random)
        .build()
        .expect("builder should succeed");
    let multer = Multer::new(storage);

    let (tx, rx) = mpsc::unbounded::<Result<Bytes, MulterError>>();
    tx.unbounded_send(Ok(Bytes::from_static(
        b"--BOUND\r\nContent-Disposition: form-data; name=\"upload\"; filename=\"big.bin\"\r\n\r\n",
    )))
    .expect("send prelude");
    for _ in 0..128 {
        tx.unbounded_send(Ok(Bytes::from(vec![b'z'; 64 * 1024])))
            .expect("send payload chunk");
    }
    tx.unbounded_send(Ok(Bytes::from_static(b"\r\n--BOUND--\r\n")))
        .expect("send trailer");
    drop(tx);

    let mut multipart = Multipart::new("BOUND", rx).expect("multipart should initialize");
    let part = multipart
        .next_part()
        .await
        .expect("part should parse")
        .expect("part expected");

    let stored = multer.store(part).await.expect("store should succeed");
    assert_eq!(stored.size, 128 * 64 * 1024);

    let path = stored.path.expect("disk path should be present");
    let metadata = tokio::fs::metadata(path).await.expect("metadata should exist");
    assert_eq!(metadata.len(), 128 * 64 * 1024);

    cleanup(root).await;
}


