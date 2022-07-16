use crate::{Body, Error, IntoResponse, Response, Result, StatusCode, ThisError};

#[derive(ThisError, Debug)]
pub enum PayloadError {
    /// 400
    #[error("failed to read payload")]
    Read,

    /// 400
    #[error("failed to parse payload")]
    Parse,

    /// 400
    #[error("multipart missing boundary")]
    MissingBoundary,

    /// 400
    #[error("parse utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// 400
    #[error("{0}")]
    Hyper(#[from] hyper::Error),

    /// 400
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    /// 400
    #[error("url decode: {0}")]
    UrlDecode(#[from] serde_urlencoded::de::Error),

    /// 411
    #[error("content-length is required")]
    LengthRequired,

    /// 413
    #[error("payload is too large")]
    TooLarge,

    /// 415
    #[error("unsupported media type, `{}` is required", .0.to_string())]
    UnsupportedMediaType(mime::Mime),

    /// 500
    #[error("missing data type `{0}`")]
    Data(&'static str),
}

impl IntoResponse for PayloadError {
    fn into_response(self) -> Response<Body> {
        (
            match self {
                PayloadError::Read
                | PayloadError::Parse
                | PayloadError::MissingBoundary
                | PayloadError::Utf8(_)
                | PayloadError::Json(_)
                | PayloadError::Hyper(_)
                | PayloadError::UrlDecode(_) => StatusCode::BAD_REQUEST,
                PayloadError::LengthRequired => StatusCode::LENGTH_REQUIRED,
                PayloadError::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
                PayloadError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
                PayloadError::Data(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            self.to_string(),
        )
            .into_response()
    }
}

impl From<PayloadError> for Error {
    fn from(e: PayloadError) -> Self {
        e.into_error()
    }
}

pub trait Payload {
    const NAME: &'static str = "payload";

    /// 1MB
    const LIMIT: u64 = 1024 * 1024;

    fn mime() -> mime::Mime;

    fn detect(m: &mime::Mime) -> bool;

    #[inline]
    fn limit(limit: Option<u64>) -> u64 {
        limit.unwrap_or(Self::LIMIT)
    }

    /// Checks `Content-Type` & `Content-Length`
    #[inline]
    fn check_header(
        m: Option<mime::Mime>,
        len: Option<u64>,
        limit: Option<u64>,
    ) -> Result<mime::Mime, PayloadError> {
        let m = m.ok_or_else(|| PayloadError::UnsupportedMediaType(Self::mime()))?;

        if !Self::detect(&m) {
            return Err(PayloadError::UnsupportedMediaType(Self::mime()));
        }

        if len == None {
            return Err(PayloadError::LengthRequired);
        }

        if matches!(len, Some(len) if len  > Self::limit(limit)) {
            return Err(PayloadError::TooLarge);
        }

        Ok(m)
    }
}
