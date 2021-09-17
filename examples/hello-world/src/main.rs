use viz::prelude::{route, router, Error, Result, Server};

async fn hello() -> &'static str {
    "Hello World!"
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = viz::new();

    app.routes(router().at("/", route().get(hello)));

    Server::bind(&"127.0.0.1:3000".parse()?)
        .serve(app.into_make_service())
        .await
        .map_err(Error::new)
}
