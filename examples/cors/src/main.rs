#![deny(warnings)]

use std::net::SocketAddr;

use viz::{
    get, middleware::cors, Body, Request, Result, Router, Server, ServiceMaker,
};

async fn index(_req: Request<Body>) -> Result<&'static str> {
    Ok("Hello, World!")
}

async fn options(_req: Request<Body>) -> Result<&'static str> {
    Ok("No Content!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new()
        .route("/", get(index).options(options))
        .with(cors::Config::default());

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
