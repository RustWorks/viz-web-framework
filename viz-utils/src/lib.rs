#![deny(unsafe_code)]

pub use anyhow;
pub use thiserror;

pub use log;
pub use pretty_env_logger;

pub mod futures {
    pub use ::futures_util::ready;
    pub use ::futures_util::future;
    pub use ::futures_util::io;
    pub use ::futures_util::stream;
}

pub mod serde {
    pub use ::serde_json as json;
    pub use ::serde_urlencoded as urlencoded;
}
