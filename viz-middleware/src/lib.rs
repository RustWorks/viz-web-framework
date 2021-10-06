//! Viz Middleware

#![deny(missing_docs, rust_2018_idioms, unused_imports, dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Disallow warnings when running tests.
#![cfg_attr(test, deny(warnings))]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

#[cfg(feature = "logger")]
mod logger;
#[cfg(feature = "logger")]
pub use logger::Logger;

#[cfg(feature = "recover")]
mod recover;
#[cfg(feature = "recover")]
pub use recover::Recover;

#[cfg(any(feature = "request-nanoid", feature = "request-uuid"))]
mod request_id;
#[cfg(any(feature = "request-nanoid", feature = "request-uuid"))]
pub use request_id::RequestID;

#[cfg(feature = "timeout")]
mod timeout;
#[cfg(feature = "timeout")]
pub use timeout::Timeout;

#[cfg(feature = "cookies")]
mod cookies;
#[cfg(feature = "cookies")]
pub use cookies::Cookies;

#[cfg(feature = "sessions")]
pub mod sessions;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "cors")]
pub mod cors;

#[cfg(feature = "compression")]
pub mod compression;

#[cfg(feature = "jwt")]
pub mod jwt;
