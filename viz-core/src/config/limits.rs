use serde::{Deserialize, Serialize};

use crate::types::{MultipartLimits, Payload};

/// Limits Settings
#[derive(Debug, Deserialize, Serialize)]
pub struct Limits {
    /// Form Limit
    #[serde(default = "Limits::form")]
    pub form: u64,
    /// JSON Limit
    #[serde(default = "Limits::json")]
    pub json: u64,
    /// Mulitpart Limits
    #[serde(default = "Limits::multipart")]
    pub multipart: MultipartLimits,
}

impl Default for Limits {
    fn default() -> Self {
        Self { form: Self::form(), json: Self::json(), multipart: Self::multipart() }
    }
}

impl Limits {
    #[inline]
    fn form() -> u64 {
        Payload::PAYLOAD_LIMIT
    }

    #[inline]
    fn json() -> u64 {
        Payload::PAYLOAD_LIMIT * 8
    }

    #[inline]
    fn multipart() -> MultipartLimits {
        MultipartLimits::default()
    }
}
