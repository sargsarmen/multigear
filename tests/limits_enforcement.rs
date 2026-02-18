#![allow(missing_docs)]

use bytes::Bytes;
use futures::{channel::mpsc, stream};
use rust_multer::{Limits, MulterConfig, MulterError, Multipart, Selector, UnknownFieldPolicy};

#[tokio::test]
async fn enforces_max_file_size() {
    let config = config_with_limits(Limits {
        max_file_size: Some(3),
        ..Limits::default()
    });
    let body = multipart_body(&[part("upload", Some("a.bin"), Some("application/octet-stream"), "hello")]);
    let mut multipart = Multipart::with_config("BOUND", bytes_stream(body), config)
        .expect("multipart should initialize");

    let mut part = multipart
        .next_part()
        .await
        .expect("headers should parse")
        .expect("item expected");
    let err = part.bytes().await.expect_err("body should fail size limit");
    assert!(matches!(
        err,
        MulterError::FileSizeLimitExceeded {
            field,
            max_file_size: 3
        } if field == "upload"
    ));
}

#[tokio::test]
async fn enforces_max_field_size() {
    let config = config_with_limits(Limits {
        max_field_size: Some(4),
        ..Limits::default()
    });
    let body = multipart_body(&[part("note", None, None, "hello")]);
    let mut multipart = Multipart::with_config("BOUND", bytes_stream(body), config)
        .expect("multipart should initialize");

    let mut part = multipart
        .next_part()
        .await
        .expect("headers should parse")
        .expect("item expected");
    let err = part.bytes().await.expect_err("body should fail size limit");
    assert!(matches!(
        err,
        MulterError::FieldSizeLimitExceeded {
            field,
            max_field_size: 4
        } if field == "note"
    ));
}

#[tokio::test]
async fn enforces_max_files() {
    let config = config_with_limits(Limits {
        max_files: Some(1),
        ..Limits::default()
    });
    let body = multipart_body(&[
        part("a", Some("a.bin"), Some("application/octet-stream"), "one"),
        part("b", Some("b.bin"), Some("application/octet-stream"), "two"),
    ]);
    let mut multipart = Multipart::with_config("BOUND", bytes_stream(body), config)
        .expect("multipart should initialize");

    let first = multipart
        .next_part()
        .await
        .expect("first item expected")
        .expect("first file should pass");
    assert_eq!(first.field_name(), "a");

    let second = multipart.next_part().await.expect_err("second item expected");
    assert!(matches!(
        second,
        MulterError::FilesLimitExceeded { max_files: 1 }
    ));
}

#[tokio::test]
async fn enforces_max_fields() {
    let config = config_with_limits(Limits {
        max_fields: Some(1),
        ..Limits::default()
    });
    let body = multipart_body(&[
        part("first", None, None, "one"),
        part("second", None, None, "two"),
    ]);
    let mut multipart = Multipart::with_config("BOUND", bytes_stream(body), config)
        .expect("multipart should initialize");

    let first = multipart
        .next_part()
        .await
        .expect("first item expected")
        .expect("first field should pass");
    assert_eq!(first.field_name(), "first");

    let second = multipart.next_part().await.expect_err("second item expected");
    assert!(matches!(
        second,
        MulterError::FieldsLimitExceeded { max_fields: 1 }
    ));
}

#[tokio::test]
async fn enforces_max_body_size() {
    let config = config_with_limits(Limits {
        max_body_size: Some(32),
        ..Limits::default()
    });
    let body = multipart_body(&[part(
        "upload",
        Some("a.bin"),
        Some("application/octet-stream"),
        "payload that is clearly longer than thirty-two bytes",
    )]);
    let mut multipart = Multipart::with_config("BOUND", bytes_stream(body), config)
        .expect("multipart should initialize");

    let item = multipart.next_part().await.expect_err("item expected");
    assert!(matches!(
        item,
        MulterError::BodySizeLimitExceeded { max_body_size: 32 }
    ));
}

#[tokio::test]
async fn enforces_allowed_mime_types_with_wildcard() {
    let config = config_with_limits(Limits {
        allowed_mime_types: vec!["image/*".to_owned()],
        ..Limits::default()
    });
    let body = multipart_body(&[
        part("avatar", Some("a.png"), Some("image/png"), "one"),
        part("notes", Some("a.txt"), Some("text/plain"), "two"),
    ]);
    let mut multipart = Multipart::with_config("BOUND", bytes_stream(body), config)
        .expect("multipart should initialize");

    let first = multipart
        .next_part()
        .await
        .expect("first item expected")
        .expect("image file should pass");
    assert_eq!(first.field_name(), "avatar");

    let second = multipart.next_part().await.expect_err("second item expected");
    assert!(matches!(
        second,
        MulterError::MimeTypeNotAllowed { field, mime }
        if field == "notes" && mime == "text/plain"
    ));
}

#[tokio::test]
async fn fails_early_before_terminal_boundary_for_large_file_chunks() {
    let config = config_with_limits(Limits {
        max_file_size: Some(4),
        ..Limits::default()
    });
    let first_chunk = concat!(
        "--BOUND\r\n",
        "Content-Disposition: form-data; name=\"upload\"; filename=\"a.bin\"\r\n",
        "Content-Type: application/octet-stream\r\n",
        "\r\n",
        "0123456789abcdefghijklmnopqrstuvwxyz"
    );

    let (tx, rx) = mpsc::unbounded::<Result<Bytes, MulterError>>();
    tx.unbounded_send(Ok(Bytes::from_static(first_chunk.as_bytes())))
        .expect("send chunk");
    drop(tx);

    let mut multipart =
        Multipart::with_config("BOUND", rx, config).expect("multipart should initialize");
    let mut part = multipart
        .next_part()
        .await
        .expect("headers should parse")
        .expect("item expected");
    let err = part.bytes().await.expect_err("body should fail");
    assert!(matches!(
        err,
        MulterError::FileSizeLimitExceeded {
            field,
            max_file_size: 4
        } if field == "upload"
    ));
}

fn config_with_limits(limits: Limits) -> MulterConfig {
    MulterConfig {
        selector: Selector::any(),
        unknown_field_policy: UnknownFieldPolicy::Reject,
        limits,
    }
}

fn part<'a>(
    field: &'a str,
    file_name: Option<&'a str>,
    content_type: Option<&'a str>,
    body: &'a str,
) -> (&'a str, Option<&'a str>, Option<&'a str>, &'a str) {
    (field, file_name, content_type, body)
}

fn multipart_body(parts: &[(&str, Option<&str>, Option<&str>, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (field, file_name, content_type, body) in parts {
        out.extend_from_slice(b"--BOUND\r\n");
        match file_name {
            Some(file_name) => {
                let disposition = format!(
                    "Content-Disposition: form-data; name=\"{field}\"; filename=\"{file_name}\"\r\n"
                );
                out.extend_from_slice(disposition.as_bytes());
                if let Some(content_type) = content_type {
                    let header = format!("Content-Type: {content_type}\r\n");
                    out.extend_from_slice(header.as_bytes());
                }
                out.extend_from_slice(b"\r\n");
            }
            None => {
                let disposition = format!("Content-Disposition: form-data; name=\"{field}\"\r\n\r\n");
                out.extend_from_slice(disposition.as_bytes());
            }
        }
        out.extend_from_slice(body.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(b"--BOUND--\r\n");
    out
}

fn bytes_stream(body: Vec<u8>) -> impl futures::Stream<Item = Result<Bytes, MulterError>> {
    stream::iter([Ok(Bytes::from(body))])
}

