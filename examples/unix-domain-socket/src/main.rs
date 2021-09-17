use viz::prelude::{route, router, Error, Result, Server};

async fn hello() -> &'static str {
    "Hello World!"
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<()> {
    let mut app = viz::new();

    app.routes(router().at("/", route().get(hello)));

    let path = "/tmp/viz.sock";
    let _ = std::fs::remove_file(path);
    let stream =
        tokio_stream::wrappers::UnixListenerStream::new(tokio::net::UnixListener::bind(path)?);

    Server::builder(viz::hyper_accept_from_stream(stream))
        .serve(app.into_make_service())
        .await
        .map_err(Error::new)
}

#[cfg(windows)]
#[tokio::main]
async fn main() -> Result<()> {
    let mut app = viz::new();

    app.routes(router().at("/", route().get(hello)));

    Server::bind(&"127.0.0.1:3000".parse()?)
        .serve(app.into_make_service())
        .await
        .map_err(Error::new)
}
