//! Viz Middleware

#![deny(
    missing_docs,
    rust_2018_idioms,
    unused_imports,
    dead_code
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Disallow warnings when running tests.
#![cfg_attr(test, deny(warnings))]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

#[cfg(feature = "logger")]
mod logger;
#[cfg(feature = "logger")]
pub use logger::LoggerMiddleware;

#[cfg(feature = "recover")]
mod recover;
#[cfg(feature = "recover")]
pub use recover::RecoverMiddleware;

#[cfg(feature = "request_id")]
mod request_id;
#[cfg(feature = "request_id")]
pub use request_id::RequestIDMiddleware;

#[cfg(feature = "timeout")]
mod timeout;
#[cfg(feature = "timeout")]
pub use timeout::TimeoutMiddleware;

#[cfg(feature = "cookies")]
mod cookies;
#[cfg(feature = "cookies")]
pub use cookies::CookiesMiddleware;

#[cfg(feature = "session")]
pub mod session;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "cors")]
pub mod cors;

#[cfg(feature = "compression")]
pub mod compression;
