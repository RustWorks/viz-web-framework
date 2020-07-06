#![forbid(unsafe_code)]

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

    pub use crate::middleware::*;
}
