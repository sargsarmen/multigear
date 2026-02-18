<div align="center">

```
╔╦╗╦ ╦╦ ╔╦╗╦╔═╗╔═╗╔═╗╦═╗
║║║║ ║║  ║ ║║ ╦║╣ ╠═╣╠╦╝
╩ ╩╚═╝╩═╝╩ ╩╚═╝╚═╝╩ ╩╩╚═
```

**Multipart uploads. Every gear included.**

[![Crates.io](https://img.shields.io/crates/v/multigear.svg)](https://crates.io/crates/multigear)
[![Docs.rs](https://docs.rs/multigear/badge.svg)](https://docs.rs/multigear)
[![CI](https://github.com/your-org/multigear/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/multigear/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV: 1.75](https://img.shields.io/badge/rustc-1.75%2B-orange.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

</div>

---

## What is Multigear?

Multigear is a **framework-agnostic multipart/form-data upload library** for async Rust. It handles the full upload lifecycle — parsing the incoming stream, enforcing limits, validating MIME types, and writing files to their final destination — through a single fluent builder API.

It is the Rust answer to the question every web developer asks after wiring up their third file-upload endpoint from scratch: *"why do I keep writing this exact same code?"*

```toml
[dependencies]
multigear = { version = "1", features = ["axum"] }
```

```rust
use multigear::{Multer, Field, storage::disk::{DiskStorage, FilenameStrategy}};

let multer = Multer::builder()
    .fields([
        Field::file("avatar")
            .max_count(1)
            .allowed_mime_types(["image/jpeg", "image/png"]),
        Field::file("documents")
            .max_count(5)
            .allowed_mime_types(["application/pdf"]),
    ])
    .max_file_size(20 * 1024 * 1024)   // 20 MB per file
    .max_body_size(80 * 1024 * 1024)   // 80 MB total
    .storage(
        DiskStorage::builder()
            .destination("/var/uploads")
            .filename(FilenameStrategy::Random)  // UUID4 — safe by default
            .build()?
    )
    .build()?;
```

That's the entire configuration. No boilerplate, no raw boundary strings, no manual file writes.

---

## Motivation

### The problem with existing solutions

The Rust async web ecosystem has excellent low-level primitives. The `multer` crate parses multipart streams correctly and is used by Axum, Rocket, and half the async web ecosystem. But *parsing* is only the first step of an upload handler. After parsing comes the part no library handles for you:

```rust
// What you actually end up writing today, in every project, over and over:
while let Some(field) = multipart.next_field().await? {
    let name = field.name().unwrap_or("unknown").to_string();
    let file_name = field.file_name().unwrap_or("upload").to_string();
    let content_type = field.content_type().map(|m| m.to_string());

    // Is the MIME type allowed? Write that check yourself.
    // Is the file too large? Write that check yourself, while streaming.
    // Where does it go? Write the tokio::fs code yourself.
    // What's the filename? Sanitize it yourself (don't forget path traversal).
    // UUID it? Write that yourself. Track size? Yourself.

    let mut file = tokio::fs::File::create(&dest_path).await?;
    while let Some(chunk) = field.chunk().await? {
        // Check running total against limit — yourself.
        file.write_all(&chunk).await?;
    }
}
```

This code gets written by every team that builds a Rust web service. It gets written slightly wrong each time — a missing path sanitization step here, a MIME check that happens after the file is already on disk there. It is reinvented, copied from Stack Overflow, and subtly broken in production.

Node.js solved this fifteen years ago with [multer](https://github.com/expressjs/multer). Python has it. Go has it. Rust deserves something better than "parse it yourself, then figure the rest out."

### What Multigear adds

Multigear is not a parser — it is the **complete upload pipeline**:

| Layer | Existing ecosystem | Multigear |
|---|---|---|
| Parse multipart stream | `multer` crate ✅ | Built on proven primitives |
| Field / file selector API | ❌ Roll your own | `.single()` `.array()` `.fields()` `.none()` `.any()` |
| File size limits (streaming) | Partial | `max_file_size`, enforced mid-stream |
| MIME type allowlist | ❌ Roll your own | `allowed_mime_types(["image/*"])` with wildcard |
| Count limits | ❌ Roll your own | `max_files`, `max_fields` |
| Save to disk | ❌ Roll your own | `DiskStorage` — streaming, no RAM buffer |
| Buffer in memory | ❌ Roll your own | `MemoryStorage` |
| Filename sanitization | ❌ Roll your own | Built in — path traversal blocked by default |
| UUID filenames | ❌ Roll your own | `FilenameStrategy::Random` |
| Axum extractor | Third-party crates | First-class, zero boilerplate |
| Actix-Web integration | `actix-multipart` (different API) | First-class, consistent API |
| hyper 1.0 integration | ❌ Manual wiring | `MulterService` + `parse_stream()` |
| Plugin storage engines | ❌ Not possible | `StorageEngine` trait — drop in S3, GCS, etc. |

### Why a new crate instead of contributing to `multer`?

The existing `multer` crate has a deliberate design philosophy: be a pure, minimal stream parser and nothing more. That is a valid and useful design — it's why axum depends on it directly. Multigear is a different thing: a **higher-level upload framework** that happens to be built on top of the same parsing primitives. Adding storage engines, MIME validation, framework extractors, and a fluent field selector API to `multer` would contradict its design goals and bloat its dependency tree for users who only want parsing.

Multigear is the crate that sits one level above. It is to `multer` what `tower-http` is to `tower` — a batteries-included layer that completes the picture.

---

## Framework Support

Multigear works with every Rust async web framework. Framework-specific feature flags provide zero-boilerplate extractors for the two most popular ones.

### Axum

```toml
multigear = { version = "1", features = ["axum"] }
```

```rust
use axum::{extract::{Request, State}, response::Json, routing::post, Router};
use multigear::{Multer, storage::disk::DiskStorage};
use std::sync::Arc;

async fn upload(
    State(multer): State<Arc<Multer<DiskStorage>>>,
    request: Request,
) -> Json<serde_json::Value> {
    let mut multipart = multer.parse(request).await.unwrap();
    while let Some(part) = multipart.next_part().await.unwrap() {
        let saved = multer.store(part).await.unwrap();
        println!("saved: {:?}", saved.path());
    }
    Json(serde_json::json!({ "ok": true }))
}

let app = Router::new()
    .route("/upload", post(upload))
    .with_state(Arc::new(multer));
```

### Actix-Web

```toml
multigear = { version = "1", features = ["actix"] }
```

```rust
use actix_web::{web, HttpRequest, HttpResponse};
use multigear::{Multer, storage::disk::DiskStorage};

async fn upload(
    multer: web::Data<Multer<DiskStorage>>,
    req: HttpRequest,
    payload: web::Payload,
) -> HttpResponse {
    let mut multipart = multer.parse(req, payload).await.unwrap();
    while let Some(part) = multipart.next_part().await.unwrap() {
        let saved = multer.store(part).await.unwrap();
    }
    HttpResponse::Ok().finish()
}
```

### hyper 1.0

No feature flag needed for basic usage — hyper's body converts directly to the stream format Multigear already accepts:

```rust
use http_body_util::BodyExt;                // into_data_stream()
use hyper::{body::Incoming, Request};

async fn handle(req: Request<Incoming>, multer: Arc<Multer<DiskStorage>>) {
    let boundary = multigear::extract_boundary(
        req.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("")
    ).unwrap();

    // One line of glue. That's it.
    let stream = req.into_body().into_data_stream();

    let mut multipart = multer.parse_stream(stream, boundary).await.unwrap();
    // ... handle parts
}
```

For a cleaner service-level integration, enable `features = ["hyper"]` to get `MulterService` — a `hyper::service::Service` impl you can hand directly to `http1::Builder::serve_connection()`.

### Any other framework

The `parse_stream()` method is the universal entry point. If your framework can give you a `Stream<Item = Result<Bytes, E>>` and the Content-Type header, Multigear works:

```rust
// Warp, Poem, ntex, raw hyper, custom stacks — all the same
let boundary = multigear::extract_boundary(content_type)?;
let mut multipart = multer.parse_stream(body_stream, boundary).await?;
```

---

## Storage Backends

### Memory (built-in)

Files are held in `Vec<u8>`. Best for small files, testing, and serverless environments where writing to disk isn't possible.

```rust
use multigear::storage::MemoryStorage;

let multer = Multer::builder()
    .single("avatar")
    .max_file_size(5 * 1024 * 1024)
    .storage(MemoryStorage::new())
    .build()?;

// After storing, access bytes directly:
let saved = multer.store(part).await?;
let bytes: bytes::Bytes = saved.bytes();
```

### Disk (built-in)

Files stream directly from the network to disk — peak RAM usage is bounded to a small read buffer regardless of file size. A 10 GB upload won't exhaust your heap.

```rust
use multigear::storage::disk::{DiskStorage, FilenameStrategy};

let storage = DiskStorage::builder()
    .destination("/var/uploads")
    .filename(FilenameStrategy::Random)   // UUID4 — recommended for production
    // .filename(FilenameStrategy::Keep)  // Original name — apply only if you trust the client
    // .filename(FilenameStrategy::Custom(|meta| format!("{}-{}", meta.field_name, uuid())))
    .build()?;
```

Multigear **always sanitizes filenames** regardless of strategy — path separators, null bytes, and unsafe characters are stripped. `FilenameStrategy::Random` is the secure default.

### Plugin storage (S3, GCS, and beyond)

The `StorageEngine` trait is the extension point. Implement it to store files anywhere:

```rust
#[async_trait]
impl StorageEngine for MyS3Storage {
    type Output = S3Key;
    type Error  = S3Error;

    async fn store(
        &self,
        field_name: &str,
        file_name: Option<&str>,
        content_type: &str,
        stream: BoxStream<'_, Result<Bytes, MulterError>>,
    ) -> Result<S3Key, S3Error> {
        // stream directly into a PutObject multipart upload
    }
}

// Drop it straight into the builder — nothing else changes
let multer = Multer::builder()
    .fields([...])
    .storage(MyS3Storage::new(config))
    .build()?;
```

The community crate `multigear-s3` implements this for AWS S3 via `aws-sdk-s3`.

---

## Design Principles

**Streaming-first.** Every limit is enforced *during* the stream. Files that exceed `max_file_size` are rejected mid-transfer — the bytes are never fully buffered, the disk is never written. This is not an implementation detail; it is the reason the library exists.

**Fail loudly, fail early.** Configuration errors surface at `.build()` time with `ConfigError`, not at runtime during a user's upload. Invalid MIME types, conflicting field selectors, and impossible size limits are caught before your server starts accepting traffic.

**Safe by default.** Filename sanitization is always on. Path traversal attacks are blocked unconditionally. The random filename strategy requires explicit opt-out, not opt-in.

**Zero framework lock-in.** The core parsing and storage pipeline has no dependency on any web framework. Feature flags are additive — enabling `axum` does not touch your actix code, and vice versa.

---

## Feature Flags

| Flag | What it enables | Extra dependencies |
|---|---|---|
| `axum` | `MulterExtractor` implementing `axum::extract::FromRequest` | `axum = "0.7"` |
| `actix` | `parse(req, payload)` helper for Actix-Web handlers | `actix-web = "4"` |
| `hyper` | `MulterService` implementing `hyper::service::Service` | `hyper = "1"`, `hyper-util`, `http-body-util` |
| `tracing` | Structured per-part and per-request log output | `tracing = "0.1"` |
| `serde` | `Serialize`/`Deserialize` on config types | `serde = "1"` |
| `tokio-rt` *(default)* | tokio `fs` and `io-util` features for async disk I/O | — |

---

## Comparison with `multer`

`multer` is an excellent, battle-tested multipart *parser* — and Multigear is built on the same parsing foundations. They solve different problems.

| | `multer` | `multigear` |
|---|---|---|
| **Role** | Stream parser primitive | Complete upload pipeline |
| **Storage** | You write it | Built-in Memory + Disk + plugin trait |
| **MIME validation** | You write it | Built-in, wildcard-aware |
| **Filename sanitization** | You write it | Built-in, on by default |
| **Framework extractors** | You wire it | Axum, Actix, hyper — built-in |
| **Field selector API** | `allowed_fields` only | `.single()` `.array()` `.fields()` `.any()` |
| **Size of your upload handler** | 40–80 lines | 10–20 lines |

If you need a minimal parser to embed in your own framework, use `multer`. If you need file uploads to work correctly in your application without writing the same infrastructure for the third time, use Multigear.

---

## License

Licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

---

<div align="center">
<sub>Built with ⚙️ in Rust · Not affiliated with the <code>multer</code> crate or expressjs/multer</sub>
</div>

### hyper 1.0

Level 1 works without any `rust-multer` feature flag by bridging the body with `into_data_stream()`:

```rust
use http_body_util::BodyExt;
use rust_multer::{MemoryStorage, Multer};

async fn parse_hyper_body(
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<(), rust_multer::MulterError> {
    let multer = Multer::new(MemoryStorage::new());
    let content_type = req
        .headers()
        .get(hyper::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let boundary = rust_multer::extract_boundary(content_type)?;
    let stream = req.into_body().into_data_stream();
    let mut multipart = multer.parse_stream(stream, boundary).await?;
    while let Some(_part) = multipart.next_part().await? {}
    Ok(())
}
```

Level 2 uses `features = ["hyper"]` and `rust_multer::hyper::MulterService`.

## Examples

Examples live under `examples/<name>/src/main.rs` and can be run with `cargo run --example <name>`.

- `cargo run --example custom_storage`
- `cargo run --example axum_memory --features axum`
- `cargo run --example axum_disk --features axum`
- `cargo run --example axum_fields --features axum`
- `cargo run --example actix_memory --features actix`
- `cargo run --example actix_disk --features actix`
- `cargo run --example actix_fields --features actix`
- `cargo run --example hyper_raw --features hyper`
- `cargo run --example hyper_service --features hyper`

## Development

```bash
cargo check --all-targets --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```
