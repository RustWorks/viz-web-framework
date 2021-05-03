use serde::{Deserialize, Serialize};

use crate::types::Payload;

/// Limits Settings
#[derive(Debug, Deserialize, Serialize)]
pub struct Limits {
    /// Form Limit
    #[serde(default = "Limits::form")]
    pub form: usize,
    /// JSON Limit
    #[serde(default = "Limits::json")]
    pub json: usize,
    /// Mulitpart Limit
    #[serde(default = "Limits::multipart")]
    pub multipart: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { form: Self::form(), json: Self::json(), multipart: Self::multipart() }
    }
}

impl Limits {
    #[inline]
    fn form() -> usize {
        Payload::PAYLOAD_LIMIT
    }

    #[inline]
    fn json() -> usize {
        Payload::PAYLOAD_LIMIT * 8
    }

    #[inline]
    fn multipart() -> usize {
        Payload::PAYLOAD_LIMIT * 16
    }
}
