#![allow(missing_docs)]

#[cfg(feature = "axum")]
use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    http::{HeaderMap, StatusCode},
    routing::post,
};
#[cfg(feature = "axum")]
use futures::{StreamExt, stream};
#[cfg(feature = "axum")]
use rust_multer::{MemoryStorage, Multer, MulterError};

#[cfg(feature = "axum")]
async fn upload(headers: HeaderMap, body: Body) -> Result<String, (StatusCode, String)> {
    let multer = Multer::new(MemoryStorage::new());
    let content_type = rust_multer::axum::content_type_from_headers(&headers)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    let bytes: Bytes = to_bytes(body, usize::MAX)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let mut multipart = multer
        .multipart_from_content_type(
            content_type,
            stream::iter([Ok::<Bytes, MulterError>(bytes)]),
        )
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    let mut count = 0usize;
    while let Some(item) = multipart.next().await {
        item.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
        count += 1;
    }

    Ok(format!("parsed {count} multipart parts"))
}

#[cfg(feature = "axum")]
fn main() {
    let _app: Router<()> = Router::new().route("/upload", post(upload));
}

#[cfg(not(feature = "axum"))]
fn main() {
    println!("Enable the `axum` feature to run this example.");
}
