#![deny(warnings)]

use std::net::SocketAddr;
use viz::{
    get,
    middleware::{
        cookie,
        session::{self, CookieOptions, Store},
    },
    Body, Request, RequestExt, Result, Router, Server, ServiceMaker,
};
use sessions::MemoryStorage;

async fn index(req: Request<Body>) -> Result<&'static str> {
    req.session().set(
        "counter",
        req.session().get::<u64>("counter")?.unwrap_or_default() + 1,
    )?;
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new()
        .route("/", get(index))
        .with(session::Config::new(
            Store::new(MemoryStorage::new(), nano_id::base64::<32>, |sid: &str| {
                sid.len() == 32
            }),
            CookieOptions::new(),
        ))
        .with(cookie::Config::new());

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
