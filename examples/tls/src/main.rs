use viz::prelude::{get, router, Error, Result, Server};

async fn hello() -> &'static str {
    "Hello World!"
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut app = viz::new();

    app.routes(router().at("/", get(hello)));

    let stream = viz::tls::Listener::new(
        viz::tls::AddrIncoming::bind(&"127.0.0.1:3000".parse()?)?,
        viz::tls::Config::new()
            .cert(include_bytes!("./cert.cer").to_vec())
            .key(include_bytes!("./key.rsa").to_vec())
            .build()?,
    );

    Server::builder(stream)
    // Server::builder(viz::hyper_accept_from_stream(stream))
        .serve(app.into_service())
        .await
        .map_err(Error::new)
}
