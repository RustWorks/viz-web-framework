mod server;

pub use server::serve;
pub use server::Server;

pub fn new() -> Server {
    Server::new()
}

pub mod prelude {
    pub use viz_core::*;
    pub use viz_router::*;
}
