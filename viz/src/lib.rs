#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

//! Viz

mod server;

pub use server::serve;
pub use server::Server;

pub fn new() -> Server {
    Server::new()
}

pub mod prelude {
    pub use viz_core::*;
    pub use viz_router::*;
    pub use types::*;
}

#[cfg(any(feature = "middleware", feature = "middleware-full"))]
pub use viz_middleware as middleware;

pub use viz_utils as utils;
