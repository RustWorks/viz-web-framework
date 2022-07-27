#![deny(warnings)]

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use viz::{
    get, middleware::limits, types::Form, IntoHandler, Request, Response, ResponseExt, Result,
    Router, Server, ServiceMaker,
};

#[derive(Deserialize, Serialize)]
struct Post {
    title: String,
    content: String,
}

// HTML form for creating a post
async fn new(_: Request) -> Result<Response> {
    Ok(Response::html(include_str!("../index.html")))
}

// create a post
async fn create(Form(post): Form<Post>) -> Result<Response> {
    Ok(Response::json(post)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new()
        .route("/", get(new).post(create.into_handler()))
        // limit body size
        .with(limits::Config::default());

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
