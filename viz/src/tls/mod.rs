#![allow(clippy::module_name_repetitions)]

mod listener;

pub use listener::Listener;

/// `native_tls`
#[cfg(feature = "native-tls")]
pub mod native_tls;

/// `rustls`
#[cfg(feature = "rustls")]
pub mod rustls;
