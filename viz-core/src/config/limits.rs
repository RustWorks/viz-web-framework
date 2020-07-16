use serde::{Deserialize, Serialize};

use crate::PAYLOAD_LIMIT;

#[derive(Debug, Deserialize, Serialize)]
pub struct Limits {
    #[serde(default = "form_limit")]
    pub form: usize,
    #[serde(default = "json_limit")]
    pub json: usize,
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
