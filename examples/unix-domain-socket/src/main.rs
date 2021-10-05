

use viz::prelude::{get, router, Error, Result, Server};

async fn hello() -> &'static str {
    "Hello World!"
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut app = viz::new();

    app.routes(router().at("/", get(hello)));

    let path = "/tmp/viz.sock";
    let _ = std::fs::remove_file(path);
    let stream =
        tokio_stream::wrappers::UnixListenerStream::new(tokio::net::UnixListener::bind(path)?);

    Server::builder(viz::hyper_accept_from_stream(stream))
        .serve(app.into_service())
        .await
        .map_err(Error::new)
}

#[cfg(windows)]
fn main() {
    println!("please run this example on unix");
}
