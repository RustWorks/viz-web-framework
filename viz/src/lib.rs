#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

//! Viz

mod app;

pub use app::{serve, App};
pub use hyper::Server;

/// Prelude some stuff
pub mod prelude {
    pub use super::{App, Server};
    pub use types::*;
    pub use viz_core::*;
    pub use viz_router::*;
}

#[cfg(feature = "middleware")]
pub use viz_middleware as middleware;

pub use viz_utils as utils;

/// Creats a `Server`
pub fn new() -> App {
    App::new()
}
