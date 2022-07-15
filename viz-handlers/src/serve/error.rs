use viz_core::{Body, IntoResponse, Response, StatusCode, ThisError};

/// Static file serving Error
#[derive(ThisError, Debug)]
pub enum Error {
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

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        (
            match self {
                Error::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
                Error::InvalidPath => StatusCode::BAD_REQUEST,
                Error::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
                Error::RangeUnsatisfied(_) => StatusCode::RANGE_NOT_SATISFIABLE,
                Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            self.to_string(),
        )
            .into_response()
    }
}

impl From<Error> for viz_core::Error {
    fn from(e: Error) -> Self {
        e.into_error()
    }
}
