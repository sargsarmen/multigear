use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::Stream;

use crate::{
    Limits, MulterConfig, MulterError, ParseError, Selector, UnknownFieldPolicy,
    Part,
    parser::stream::{MultipartStream, StreamLimits},
    selector::{SelectorAction, SelectorEngine},
};

/// High-level multipart stream abstraction.
#[derive(Debug)]
pub struct Multipart<S> {
    inner: MultipartStream<S>,
    selector: SelectorEngine,
    limits: Limits,
    file_count: usize,
    field_count: usize,
}

impl<S> Multipart<S> {
    /// Creates a multipart stream from an already extracted boundary and a chunk source.
    pub fn new(boundary: impl Into<String>, stream: S) -> Result<Self, ParseError> {
        Ok(Self {
            inner: MultipartStream::new(boundary, stream)?,
            selector: SelectorEngine::new(Selector::any(), UnknownFieldPolicy::Ignore),
            limits: Limits::default(),
            file_count: 0,
            field_count: 0,
        })
    }

    /// Creates a multipart stream with explicit selector configuration.
    pub fn with_config(
        boundary: impl Into<String>,
        stream: S,
        config: MulterConfig,
    ) -> Result<Self, MulterError> {
        config.validate()?;
        let stream_limits = StreamLimits {
            max_file_size: config.limits.max_file_size,
            max_field_size: config.limits.max_field_size,
            max_body_size: config.limits.max_body_size,
        };
        let selector = SelectorEngine::new(config.selector, config.unknown_field_policy);
        Ok(Self {
            inner: MultipartStream::with_limits(boundary, stream, stream_limits)?,
            selector,
            limits: config.limits,
            file_count: 0,
            field_count: 0,
        })
    }
}

impl<S> Stream for Multipart<S>
where
    S: Stream<Item = Result<Bytes, MulterError>> + Unpin,
{
    type Item = Result<Part, MulterError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(parsed))) => {
                    let part = Part::from_parsed(parsed);
                    if part.file_name().is_none() {
                        match self.validate_text_part(&part) {
                            Ok(()) => return Poll::Ready(Some(Ok(part))),
                            Err(err) => return Poll::Ready(Some(Err(err))),
                        }
                    }

                    match self.selector.evaluate_file_field(part.field_name()) {
                        Ok(SelectorAction::Accept) => match self.validate_file_part(&part) {
                            Ok(()) => return Poll::Ready(Some(Ok(part))),
                            Err(err) => return Poll::Ready(Some(Err(err))),
                        },
                        Ok(SelectorAction::Ignore) => continue,
                        Err(err) => return Poll::Ready(Some(Err(err))),
                    }
                }
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl<S> Multipart<S> {
    fn validate_text_part(&mut self, part: &Part) -> Result<(), MulterError> {
        if let Some(max_field_size) = self.limits.max_field_size {
            if (part.body.len() as u64) > max_field_size {
                return Err(MulterError::FieldSizeLimitExceeded {
                    field: part.field_name().to_owned(),
                    max_field_size,
                });
            }
        }

        self.field_count += 1;
        if let Some(max_fields) = self.limits.max_fields {
            if self.field_count > max_fields {
                return Err(MulterError::FieldsLimitExceeded { max_fields });
            }
        }

        Ok(())
    }

    fn validate_file_part(&mut self, part: &Part) -> Result<(), MulterError> {
        if let Some(max_file_size) = self.limits.max_file_size {
            if (part.body.len() as u64) > max_file_size {
                return Err(MulterError::FileSizeLimitExceeded {
                    field: part.field_name().to_owned(),
                    max_file_size,
                });
            }
        }

        if !self.limits.is_mime_allowed(part.content_type()) {
            return Err(MulterError::MimeTypeNotAllowed {
                field: part.field_name().to_owned(),
                mime: part.content_type().essence_str().to_owned(),
            });
        }

        self.file_count += 1;
        if let Some(max_files) = self.limits.max_files {
            if self.file_count > max_files {
                return Err(MulterError::FilesLimitExceeded { max_files });
            }
        }

        Ok(())
    }
}
