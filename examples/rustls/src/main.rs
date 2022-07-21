#![deny(warnings)]

use std::{net::SocketAddr, sync::Arc};
use viz::{get, Error, Request, Result, Router, Server, ServiceMaker};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new().route("/", get(index));

    let mut incoming = viz::tls::AddrIncoming::bind(&addr).map_err(Error::normal)?;
    incoming.set_nodelay(true);

    let listener = viz::tls::Listener::new(
        incoming,
        viz::tls::rustls::Config::new()
            .cert(include_bytes!("../../tls/cert.pem").to_vec())
            .key(include_bytes!("../../tls/key.pem").to_vec())
            .build()
            .map(Arc::new)
            .map_err(Error::normal)?
            .into(),
    );

    if let Err(err) = Server::builder(listener)
        .serve(ServiceMaker::from(app))
        .await
    {
        println!("{}", err);
    }

    Ok(())
}
