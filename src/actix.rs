//! Actix integration helpers.

use actix_web::{
    HttpRequest,
    error::PayloadError,
    http::header,
    web::{self, Bytes},
};
use futures::{Stream, StreamExt};

use crate::{Multer, MulterError, Multipart, ParseError, StorageEngine};

/// Actix body stream mapped into `rust-multer` chunk errors.
pub type ActixBodyStream<S> =
    futures::stream::Map<S, fn(Result<Bytes, PayloadError>) -> Result<Bytes, MulterError>>;

/// Extracts the raw `Content-Type` header from an Actix request.
pub fn content_type_from_request(request: &HttpRequest) -> Result<&str, MulterError> {
    let value = request
        .headers()
        .get(header::CONTENT_TYPE)
        .ok_or_else(|| ParseError::new("missing Content-Type header"))?;
    value
        .to_str()
        .map_err(|_| ParseError::new("Content-Type header must be ASCII").into())
}

/// Maps an Actix payload stream into the stream shape expected by `rust-multer`.
pub fn map_payload_stream<S>(stream: S) -> ActixBodyStream<S>
where
    S: Stream<Item = Result<Bytes, PayloadError>>,
{
    stream.map(actix_item_to_multer)
}

/// Creates a configured [`Multipart`] stream from an Actix request and payload stream.
pub fn multipart_from_request<S>(
    multer: &Multer<S>,
    request: &HttpRequest,
    payload: web::Payload,
) -> Result<Multipart<ActixBodyStream<web::Payload>>, MulterError>
where
    S: StorageEngine,
{
    let content_type = content_type_from_request(request)?;
    multer.multipart_from_content_type(content_type, map_payload_stream(payload))
}

fn actix_item_to_multer(item: Result<Bytes, PayloadError>) -> Result<Bytes, MulterError> {
    item.map_err(|err| ParseError::new(format!("actix body stream error: {err}")).into())
}
