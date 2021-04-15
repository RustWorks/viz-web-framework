#![deny(unsafe_code)]

pub use anyhow;
#[cfg(feature = "error")]
pub use thiserror;

#[cfg(feature = "tracing")]
pub use tracing;

#[cfg(feature = "futures")]
pub use futures_util as futures;

#[cfg(feature = "serde")]
pub mod serde {
    pub use serde_json as json;
    pub use serde_urlencoded as urlencoded;
}
