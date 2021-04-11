use cookie::Key;
use serde::{Deserialize, Serialize};

/// Cookies Settings
#[derive(Debug, Deserialize, Serialize)]
pub struct Cookies {
    /// Secret Key
    pub secret_key: String,
}

impl Default for Cookies {
    fn default() -> Self {
        let k = Key::generate();
        let mut v = Vec::new();
        v.extend_from_slice(&k.signing());
        v.extend_from_slice(&k.encryption());
        Self { secret_key: String::from_utf8_lossy(&v).to_string() }
    }
}
