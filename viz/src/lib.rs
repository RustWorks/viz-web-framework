#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

mod server;

pub mod middleware;

pub use server::serve;
pub use server::Server;

pub fn new() -> Server {
    Server::new()
}

pub mod prelude {
    pub use viz_core::*;
    pub use viz_router::*;

    pub use crate::middleware;
}
