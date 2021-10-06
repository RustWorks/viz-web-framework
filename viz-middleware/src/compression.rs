//! Compression
//! Compression middleware for Viz that will compress the response using gzip,
//! deflate and brotli compression depending on the [Accept-Encoding](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding) header.
//!

use std::{
    io::{Error, ErrorKind},
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

pub use async_compression::Level;
use bytes::Bytes;
use pin_project_lite::pin_project;

use viz_core::{
    http::{Body, Error as HyperError},
    Result,
};

use viz_utils::futures::stream::Stream;

#[cfg(feature = "compression-brotli")]
pub use async_compression::tokio::bufread::BrotliEncoder;
#[cfg(feature = "compression-deflate")]
pub use async_compression::tokio::bufread::DeflateEncoder;
#[cfg(feature = "compression-gzip")]
pub use async_compression::tokio::bufread::GzipEncoder;
#[cfg(any(
    feature = "compression-brotli",
    feature = "compression-gzip",
    feature = "compression-deflate"
))]
use std::future::Future;
#[cfg(any(
    feature = "compression-brotli",
    feature = "compression-gzip",
    feature = "compression-deflate"
))]
use tokio_util::io::{ReaderStream, StreamReader};
#[cfg(any(
    feature = "compression-brotli",
    feature = "compression-gzip",
    feature = "compression-deflate"
))]
use viz_core::{
    http::{
        self,
        header::{HeaderValue, CONTENT_ENCODING, CONTENT_LENGTH},
    },
    Context, Middleware, Response,
};

/// Compression Response Body
#[derive(Debug)]
pub struct Compression<Algo> {
    level: Level,
    algo: PhantomData<Algo>,
}

impl<Algo> Compression<Algo> {
    /// Creates a Compression
    pub fn new() -> Self {
        Self { level: Level::Default, algo: PhantomData }
    }

    /// Creates a Compression with a quality
    pub fn with_quality(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl<Algo> Default for Compression<Algo> {
    fn default() -> Self {
        Self::new()
    }
}

pin_project! {
    /// A wrapper around any type that implements [`Stream`](futures::Stream) to be
    /// compatible with async_compression's Stream based encoders
    #[derive(Debug)]
    pub struct CompressableBody<S, E>
    where
        E: std::error::Error,
        S: Stream<Item = Result<Bytes, E>>,
    {
        #[pin]
        body: S,
    }
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

impl From<Body> for CompressableBody<Body, HyperError> {
    fn from(body: Body) -> Self {
        CompressableBody { body }
    }
}

#[cfg(feature = "compression-gzip")]
impl<R> Compression<GzipEncoder<R>> {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let res: http::Response = cx.next().await?.into();
        let (mut parts, body) = res.into_parts();

        parts.headers.append(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        parts.headers.remove(CONTENT_LENGTH);

        Ok(http::Response::from_parts(
            parts,
            Body::wrap_stream(ReaderStream::new(GzipEncoder::with_quality(
                StreamReader::new(Into::<CompressableBody<Body, HyperError>>::into(body)),
                self.level,
            ))),
        )
        .into())
    }
}

#[cfg(feature = "compression-gzip")]
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

#[cfg(feature = "compression-brotli")]
impl<R> Compression<BrotliEncoder<R>> {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let res: http::Response = cx.next().await?.into();
        let (mut parts, body) = res.into_parts();

        parts.headers.append(CONTENT_ENCODING, HeaderValue::from_static("br"));
        parts.headers.remove(CONTENT_LENGTH);

        Ok(http::Response::from_parts(
            parts,
            Body::wrap_stream(ReaderStream::new(BrotliEncoder::with_quality(
                StreamReader::new(Into::<CompressableBody<Body, HyperError>>::into(body)),
                self.level,
            ))),
        )
        .into())
    }
}

#[cfg(feature = "compression-brotli")]
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

#[cfg(feature = "compression-deflate")]
impl<R> Compression<DeflateEncoder<R>> {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let res: http::Response = cx.next().await?.into();
        let (mut parts, body) = res.into_parts();

        parts.headers.append(CONTENT_ENCODING, HeaderValue::from_static("deflate"));
        parts.headers.remove(CONTENT_LENGTH);

        Ok(http::Response::from_parts(
            parts,
            Body::wrap_stream(ReaderStream::new(DeflateEncoder::with_quality(
                StreamReader::new(Into::<CompressableBody<Body, HyperError>>::into(body)),
                self.level,
            ))),
        )
        .into())
    }
}

#[cfg(feature = "compression-deflate")]
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

#[cfg(feature = "compression-gzip")]
/// compresses the Body of Response using gzip
pub fn gzip() -> Compression<GzipEncoder<StreamReader<CompressableBody<Body, HyperError>, Bytes>>> {
    Compression::new()
}

#[cfg(feature = "compression-brotli")]
/// compresses the Body of Response using brotli
pub fn brotli()
-> Compression<BrotliEncoder<StreamReader<CompressableBody<Body, HyperError>, Bytes>>> {
    Compression::new()
}

#[cfg(feature = "compression-deflate")]
/// compresses the Body of Response using deflate
pub fn deflate()
-> Compression<DeflateEncoder<StreamReader<CompressableBody<Body, HyperError>, Bytes>>> {
    Compression::new()
}
