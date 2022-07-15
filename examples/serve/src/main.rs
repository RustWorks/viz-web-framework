// #![deny(warnings)]

use std::env;
use std::net::SocketAddr;
use viz::{
    get,
    handlers::serve,
    Body, Request, Result, Router, Server, ServiceMaker,
};

async fn index(_: Request<Body>) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let dir = env::current_dir().unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route(
            "/cargo.toml",
            get(serve::File::new(dir.join("Cargo.toml"))),
        )
        .route(
            "/examples/*",
            get(serve::Files::new(dir).listing(true)),
        );

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
