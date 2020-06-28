mod server;

pub use server::Server;

pub fn new() -> Server {
    Server::new()
}

pub mod prelude {
    pub use viz_core::http;
    pub use viz_core::Context;
    pub use viz_core::Extract;
    pub use viz_core::Response;
    pub use viz_core::{into_guard, Guard};
    pub use viz_core::{Error, Result};
    pub use viz_core::{Form, Json, Multipart, Params};

    pub use viz_router::*;
}
