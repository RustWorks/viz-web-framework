use crate::{Error, IntoResponse, Response, StatusCode, ThisError};

#[derive(ThisError, Debug)]
pub enum WebSocketError {
    #[error("missing `Connection` upgrade")]
    MissingConnectUpgrade,

    #[error("invalid `Connection` upgrade")]
    InvalidConnectUpgrade,

    #[error("missing `Upgrade`")]
    MissingUpgrade,

    #[error("invalid `Upgrade`")]
    InvalidUpgrade,

    #[error("missing `Sec-WebSocket-Version`")]
    MissingWebSocketVersion,

    #[error("invalid `Sec-WebSocket-Version`")]
    InvalidWebSocketVersion,

    #[error("missing `Sec-WebSocket-Key`")]
    MissingWebSocketKey,

    #[error("request upgrade required")]
    ConnectionNotUpgradable,

    #[error(transparent)]
    TungsteniteError(#[from] tokio_tungstenite::tungstenite::Error),
}

impl IntoResponse for WebSocketError {
    fn into_response(self) -> Response {
        (
            match self {
                WebSocketError::MissingConnectUpgrade
                | WebSocketError::InvalidConnectUpgrade
                | WebSocketError::MissingUpgrade
                | WebSocketError::InvalidUpgrade
                | WebSocketError::MissingWebSocketVersion
                | WebSocketError::InvalidWebSocketVersion
                | WebSocketError::MissingWebSocketKey => StatusCode::BAD_REQUEST,
                WebSocketError::ConnectionNotUpgradable => StatusCode::UPGRADE_REQUIRED,
                WebSocketError::TungsteniteError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            self.to_string(),
        )
            .into_response()
    }
}

impl From<WebSocketError> for Error {
    fn from(e: WebSocketError) -> Self {
        e.into_error()
    }
}
