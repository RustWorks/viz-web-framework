use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;

use viz::prelude::{route, router, Error, Result, Server};

async fn hello() -> &'static str {
    "Hello World!"
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = viz::new();

    app.routes(router().at("/", route().get(hello)));

    let path = "/tmp/viz.sock";
    let _ = std::fs::remove_file(path);
    let stream = UnixListenerStream::new(UnixListener::bind(path)?);

    Server::builder(viz::hyper_accept_from_stream(stream))
        .serve(app.into_make_service())
        .await
        .map_err(Error::new)
}
