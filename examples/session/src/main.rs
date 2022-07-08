#![deny(warnings)]

use std::net::SocketAddr;
use std::time::Duration;

use sessions::MemoryStorage;

use viz::{
    get,
    middleware::{
        cookie,
        helper::CookieOptions,
        limits,
        session::{self, Store},
        csrf
    },
    Body, Request, RequestExt, Result, Router, Server, ServiceMaker,
    Method,
};

async fn index(req: Request<Body>) -> Result<&'static str> {
    req.session().set(
        "counter",
        req.session().get::<u64>("counter")?.unwrap_or_default() + 1,
    )?;
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("listening on {}", addr);

    let app = Router::new()
        .route("/", get(index).with(limits::Config::new()))
        .with(csrf::Config::new(
            csrf::Store::Cookie,
            [Method::GET, Method::HEAD, Method::OPTIONS, Method::TRACE].into(),
            CookieOptions::new("_csrf").max_age(Duration::from_secs(3600 * 24)),
            csrf::secret,
            csrf::generate,
            csrf::verify,
        ))
        .with(session::Config::new(
            Store::new(MemoryStorage::new(), nano_id::base64::<32>, |sid: &str| {
                sid.len() == 32
            }),
            CookieOptions::default(),
        ))
        .with(cookie::Config::new());

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
