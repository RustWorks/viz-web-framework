//! Compression
//! Compression middleware for Viz that will compress the response using gzip,
//! deflate and brotli compression depending on the [Accept-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header.
//!

use std::{
    future::Future,
    io::{Error, ErrorKind},
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

pub use async_compression::{
    tokio::bufread::{BrotliEncoder, DeflateEncoder, GzipEncoder},
    Level,
};
use bytes::Bytes;
use pin_project::pin_project;
use tokio_util::io::{ReaderStream, StreamReader};

use viz_core::{
    http::{
        self,
        header::{HeaderValue, CONTENT_ENCODING, CONTENT_LENGTH},
        Body,
    },
    Context, Middleware, Response, Result,
};

use viz_utils::futures::stream::Stream;

/// Compression Response Body
#[derive(Debug)]
pub struct Compression<Algo> {
    level: Level,
    algo: PhantomData<Algo>,
}

impl<Algo> Compression<Algo> {
    pub fn new() -> Self {
        Self {
            level: Level::Default,
            algo: PhantomData::default(),
        }
    }

    pub fn with_quality(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

/// A wrapper around any type that implements [`Stream`](futures::Stream) to be
/// compatible with async_compression's Stream based encoders
#[pin_project]
#[derive(Debug)]
pub struct CompressableBody<S, E>
where
    E: std::error::Error,
    S: Stream<Item = Result<Bytes, E>>,
{
    #[pin]
    body: S,
}

impl<S, E> Stream for CompressableBody<S, E>
where
    E: std::error::Error,
    S: Stream<Item = Result<Bytes, E>>,
{
    type Item = std::io::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let pin = self.project();
        // TODO: Use `.map_err()` (https://github.com/rust-lang/rust/issues/63514) once it is stabilized
        S::poll_next(pin.body, cx)
            .map(|e| e.map(|res| res.map_err(|_| Error::from(ErrorKind::InvalidData))))
    }
}

impl From<Body> for CompressableBody<Body, hyper::Error> {
    fn from(body: Body) -> Self {
        CompressableBody { body }
    }
}

impl<R> Compression<BrotliEncoder<R>> {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let res: http::Response = cx.next().await?.into();
        let (mut parts, body) = res.into_parts();

        parts
            .headers
            .append(CONTENT_ENCODING, HeaderValue::from_static("br"));
        parts.headers.remove(CONTENT_LENGTH);

        Ok(http::Response::from_parts(
            parts,
            Body::wrap_stream(ReaderStream::new(BrotliEncoder::with_quality(
                StreamReader::new(Into::<CompressableBody<Body, hyper::Error>>::into(body)),
                self.level,
            ))),
        )
        .into())
    }
}

impl<'a, R> Middleware<'a, Context> for Compression<BrotliEncoder<R>>
where
    R: Sync + Send + 'static,
{
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}

impl<R> Compression<DeflateEncoder<R>> {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let res: http::Response = cx.next().await?.into();
        let (mut parts, body) = res.into_parts();

        parts
            .headers
            .append(CONTENT_ENCODING, HeaderValue::from_static("deflate"));
        parts.headers.remove(CONTENT_LENGTH);

        Ok(http::Response::from_parts(
            parts,
            Body::wrap_stream(ReaderStream::new(DeflateEncoder::with_quality(
                StreamReader::new(Into::<CompressableBody<Body, hyper::Error>>::into(body)),
                self.level,
            ))),
        )
        .into())
    }
}

impl<'a, R> Middleware<'a, Context> for Compression<DeflateEncoder<R>>
where
    R: Sync + Send + 'static,
{
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}

impl<R> Compression<GzipEncoder<R>> {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let res: http::Response = cx.next().await?.into();
        let (mut parts, body) = res.into_parts();

        parts
            .headers
            .append(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        parts.headers.remove(CONTENT_LENGTH);

        Ok(http::Response::from_parts(
            parts,
            Body::wrap_stream(ReaderStream::new(GzipEncoder::with_quality(
                StreamReader::new(Into::<CompressableBody<Body, hyper::Error>>::into(body)),
                self.level,
            ))),
        )
        .into())
    }
}

impl<'a, R> Middleware<'a, Context> for Compression<GzipEncoder<R>>
where
    R: Sync + Send + 'static,
{
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}

/// compresses the Body of Response using brotli
pub fn brotli(
) -> Compression<BrotliEncoder<StreamReader<CompressableBody<Body, hyper::Error>, Bytes>>> {
    Compression::new()
}

/// compresses the Body of Response using gzip
pub fn gzip() -> Compression<GzipEncoder<StreamReader<CompressableBody<Body, hyper::Error>, Bytes>>>
{
    Compression::new()
}

/// compresses the Body of Response using deflate
pub fn deflate(
) -> Compression<DeflateEncoder<StreamReader<CompressableBody<Body, hyper::Error>, Bytes>>> {
    Compression::new()
}
