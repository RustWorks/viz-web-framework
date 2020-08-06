use serde::{Deserialize, Serialize};

use crate::PAYLOAD_LIMIT;

/// Limits Settings
#[derive(Debug, Deserialize, Serialize)]
pub struct Limits {
    /// Form Limit
    #[serde(default = "form_limit")]
    pub form: usize,
    /// JSON Limit
    #[serde(default = "json_limit")]
    pub json: usize,
    /// Mulitpart Limit
    #[serde(default = "multipart_limit")]
    pub multipart: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            form: form_limit(),
            json: json_limit(),
            multipart: multipart_limit(),
        }
    }
}

fn form_limit() -> usize {
    PAYLOAD_LIMIT
}

fn json_limit() -> usize {
    PAYLOAD_LIMIT * 8
}

fn multipart_limit() -> usize {
    PAYLOAD_LIMIT * 16
}
