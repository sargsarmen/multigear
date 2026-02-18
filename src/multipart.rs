use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::Stream;

use crate::{
    MulterConfig, MulterError, ParseError, Selector, UnknownFieldPolicy,
    Part,
    parser::stream::MultipartStream,
    selector::{SelectorAction, SelectorEngine},
};

/// High-level multipart stream abstraction.
#[derive(Debug)]
pub struct Multipart<S> {
    inner: MultipartStream<S>,
    selector: SelectorEngine,
}

impl<S> Multipart<S> {
    /// Creates a multipart stream from an already extracted boundary and a chunk source.
    pub fn new(boundary: impl Into<String>, stream: S) -> Result<Self, ParseError> {
        Ok(Self {
            inner: MultipartStream::new(boundary, stream)?,
            selector: SelectorEngine::new(Selector::any(), UnknownFieldPolicy::Ignore),
        })
    }

    /// Creates a multipart stream with explicit selector configuration.
    pub fn with_config(
        boundary: impl Into<String>,
        stream: S,
        config: MulterConfig,
    ) -> Result<Self, MulterError> {
        config.validate()?;
        let selector = SelectorEngine::new(config.selector, config.unknown_field_policy);
        Ok(Self {
            inner: MultipartStream::new(boundary, stream)?,
            selector,
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
                        return Poll::Ready(Some(Ok(part)));
                    }

                    match self.selector.evaluate_file_field(part.field_name()) {
                        Ok(SelectorAction::Accept) => return Poll::Ready(Some(Ok(part))),
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
