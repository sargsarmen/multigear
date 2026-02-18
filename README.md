# rust-multer

`rust-multer` is a streaming multipart/form-data parser with selector rules, request limits, and pluggable storage engines.

## Features

- Streaming parser with structured errors
- Selector engine: `single`, `array`, `fields`, `none`, `any`
- Limits: file size, field size, file count, field count, body size, MIME allowlist
- Storage engines:
  - `MemoryStorage`
  - `DiskStorage` (with filename sanitization and strategy controls)
- Optional framework helpers:
  - `axum` feature
  - `actix` feature

## Quick Start

```rust
use bytes::Bytes;
use futures::stream;
use rust_multer::{MemoryStorage, Multer, MulterError};

#[tokio::main]
async fn main() {
    let multer = Multer::new(MemoryStorage::new());
    let body = concat!(
        "--BOUND\r\n",
        "Content-Disposition: form-data; name=\"file\"; filename=\"a.txt\"\r\n",
        "\r\n",
        "hello\r\n",
        "--BOUND--\r\n"
    );

    let output = multer
        .parse_and_store(
            "BOUND",
            stream::iter([Ok::<Bytes, MulterError>(Bytes::from_static(body.as_bytes()))]),
        )
        .await
        .expect("multipart parse");

    println!("stored files: {}", output.stored_files.len());
}
```

## Examples

- `cargo run --example custom_storage`
- `cargo run --example streaming_large_file`
- `cargo run --example field_validation`
- `cargo run --example axum_basic --features axum`
- `cargo run --example actix_basic --features actix`

## Development

```bash
cargo check --all-targets --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```
