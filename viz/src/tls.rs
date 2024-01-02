mod listener;

pub use listener::TlsListener;

/// `native_tls`
#[cfg(feature = "native-tls")]
pub mod native_tls;

/// `rustls`
#[cfg(feature = "rustls")]
pub mod rustls;
