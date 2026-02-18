//! Axum integration helpers.

use axum::{
    body::Bytes,
    http::{HeaderMap, header},
};
use futures::{Stream, StreamExt};

use crate::{Multer, MulterError, Multipart, ParseError, StorageEngine};

/// Axum body stream mapped into `rust-multer` chunk errors.
pub type AxumBodyStream<S> =
    futures::stream::Map<S, fn(Result<Bytes, axum::Error>) -> Result<Bytes, MulterError>>;

/// Extracts the raw `Content-Type` header from Axum request headers.
pub fn content_type_from_headers(headers: &HeaderMap) -> Result<&str, MulterError> {
    let value = headers
        .get(header::CONTENT_TYPE)
        .ok_or_else(|| ParseError::new("missing Content-Type header"))?;
    value
        .to_str()
        .map_err(|_| ParseError::new("Content-Type header must be ASCII").into())
}

/// Maps an Axum body stream into the stream shape expected by `rust-multer`.
pub fn map_body_stream<S>(stream: S) -> AxumBodyStream<S>
where
    S: Stream<Item = Result<Bytes, axum::Error>>,
{
    stream.map(axum_item_to_multer)
}

/// Creates a configured [`Multipart`] stream from Axum headers and body stream.
pub fn multipart_from_headers<S, B>(
    multer: &Multer<S>,
    headers: &HeaderMap,
    body: B,
) -> Result<Multipart<AxumBodyStream<B>>, MulterError>
where
    S: StorageEngine,
    B: Stream<Item = Result<Bytes, axum::Error>> + Unpin,
{
    let content_type = content_type_from_headers(headers)?;
    multer.multipart_from_content_type(content_type, map_body_stream(body))
}

fn axum_item_to_multer(item: Result<Bytes, axum::Error>) -> Result<Bytes, MulterError> {
    item.map_err(|err| ParseError::new(format!("axum body stream error: {err}")).into())
}
