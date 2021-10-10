use viz::middleware::Recover;
use viz::prelude::{bail, get, http::StatusCode, router, with, Error, Response, Result, Server};

/// index
async fn index() -> &'static str {
    "Hello World!"
}

/// 404
async fn not_found() -> impl Into<Response> {
    StatusCode::NOT_FOUND
}

/// catch error
async fn error_handler() -> Result<Response> {
    bail!("Catch error")
}

/// recover panic
async fn panic_handler() -> impl Into<Response> {
    panic!("something is wrong")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut app = viz::new();

    app.routes(
        router()
            .at("/", get(index))
            .at("/error", get(error_handler))
            .at("/panic", with(Recover::default()).get(panic_handler))
            .at("*", get(not_found)),
    );

    Server::bind(&"127.0.0.1:3000".parse()?).serve(app.into_service()).await.map_err(Error::new)
}
