#![deny(unsafe_code)]

pub use anyhow;
pub use thiserror;

pub use log;
pub use pretty_env_logger;

pub use futures_util as futures;

pub mod serde {
    pub use ::serde_json as json;
    pub use ::serde_urlencoded as urlencoded;
}
