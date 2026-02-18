/// Multipart boundary parsing helpers.
pub mod boundary;
/// Multipart part header parsing helpers.
pub mod headers;
/// Streaming multipart parser state machine.
pub mod stream;

pub use boundary::extract_multipart_boundary;
pub use headers::{
    parse_content_disposition, parse_part_content_type, parse_part_headers, ContentDisposition,
    ParsedPartHeaders,
};
pub use stream::MultipartStream;

/// Low-level multipart parser entry type.
#[derive(Debug, Clone, Default)]
pub struct Parser;
