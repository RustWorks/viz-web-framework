use std::marker::PhantomData;

use bytes::{Buf, Bytes, BytesMut};

use viz_utils::{
    futures::stream::{Stream, StreamExt},
    thiserror::Error as ThisError,
    tracing,
};

use crate::{http, Response, Result};

/// Payload Error
#[derive(ThisError, Debug, PartialEq)]
pub enum PayloadError {
    /// 400
    #[error("failed to read payload")]
    Read,

    /// 400
    #[error("failed to parse payload")]
    Parse,

    /// 411
    #[error("content-length is required")]
    LengthRequired,

    /// 413
    #[error("payload is too large")]
    TooLarge,

    /// 415
    #[error("unsupported media type")]
    UnsupportedMediaType,
}

impl Into<Response> for PayloadError {
    fn into(self) -> Response {
        (
            match self {
                Self::Read | Self::Parse => http::StatusCode::BAD_REQUEST,
                Self::LengthRequired => http::StatusCode::LENGTH_REQUIRED,
                Self::TooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                Self::UnsupportedMediaType => http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            },
            self.to_string(),
        )
            .into()
    }
}

/// Payload Content Type Detection
pub trait PayloadDetect {
    /// Detects the request's content-type
    fn detect(_: &mime::Mime) -> bool {
        true
    }
}

/// Payload Body
#[derive(Debug)]
pub struct Payload<T = ()> {
    /// A limit size
    limit: Option<usize>,
    _t: PhantomData<T>,
}

impl Payload {
    /// 1 MB
    pub const PAYLOAD_LIMIT: usize = 1024 * 1024;
}

impl<T> Payload<T>
where
    T: PayloadDetect,
{
    /// Creates new Payload instance with T.
    pub fn new() -> Self {
        Self { limit: None, _t: PhantomData }
    }

    /// Sets the limit.
    pub fn set_limit(&mut self, limit: usize) -> &mut Self {
        self.limit.replace(limit);
        self
    }

    /// Gets the limit.
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(Payload::PAYLOAD_LIMIT)
    }

    /// Checks the content-type and content-length of request.
    pub fn check_header(
        &self,
        m: Option<mime::Mime>,
        l: Option<usize>,
    ) -> Result<mime::Mime, PayloadError> {
        let m = m.ok_or_else(|| PayloadError::UnsupportedMediaType)?;

        if !T::detect(&m) {
            return Err(PayloadError::UnsupportedMediaType);
        }

        if l.is_none() {
            return Err(PayloadError::LengthRequired);
        }

        if l.unwrap() > self.limit() {
            return Err(PayloadError::TooLarge);
        }

        Ok(m)
    }

    /// Checks the real length of request body.
    pub async fn check_real_length<S>(&self, mut stream: S) -> Result<impl Buf, PayloadError>
    where
        S: Stream<Item = Result<Bytes, hyper::Error>> + Unpin,
    {
        let mut body = BytesMut::with_capacity(8192);
        let limit = self.limit();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| {
                tracing::debug!("{}", e);
                PayloadError::Read
            })?;
            if (body.len() + chunk.len()) > limit {
                return Err(PayloadError::TooLarge);
            } else {
                body.extend_from_slice(&chunk);
            }
        }

        Ok(body.freeze())
    }
}
