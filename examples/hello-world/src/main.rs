#![deny(warnings)]

use std::net::SocketAddr;
use viz::{get, Body, Request, Result, Router, Server, ServiceMaker};

async fn index(_: Request<Body>) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new().route("/", get(index));

    if let Err(err) = Server::bind(&addr)
        .tcp_nodelay(true)
        .serve(ServiceMaker::from(app))
        .await
    {
        println!("{}", err);
    }

    Ok(())
}
