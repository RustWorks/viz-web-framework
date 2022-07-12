use crate::{Body, Error, IntoResponse, Response, StatusCode, ThisError};

/// Static file serving Error
#[derive(Debug, ThisError)]
pub enum ServeError {
    /// Method Not Allowed
    #[error("method not allowed")]
    MethodNotAllowed,

    /// Invalid path
    #[error("invalid path")]
    InvalidPath,

    /// Precondition failed
    #[error("precondition failed")]
    PreconditionFailed,

    /// Range could not be satisfied
    #[error("range could not be satisfied")]
    RangeUnsatisfied(u64),

    /// Io error
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for ServeError {
    fn into_response(self) -> Response<Body> {
        match self {
            ServeError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            ServeError::InvalidPath => StatusCode::BAD_REQUEST,
            ServeError::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
            ServeError::RangeUnsatisfied(_) => StatusCode::RANGE_NOT_SATISFIABLE,
            ServeError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

impl From<ServeError> for Error {
    fn from(e: ServeError) -> Self {
        e.into_error()
    }
}
