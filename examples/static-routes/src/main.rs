#![deny(warnings)]

use hyper::service::{make_service_fn, service_fn};
use once_cell::sync::Lazy;
use std::{convert::Infallible, net::SocketAddr};
use viz::{
    get, Body, IntoResponse, Method, types::Params, Request, RequestExt, Response, Result, Router, Server,
    StatusCode, Tree,
};

/// Static Lazy Routes
static TREE: Lazy<Tree> = Lazy::new(|| {
    Router::new()
        .route("/", get(index))
        .route("/me", get(me))
        .into()
});

async fn index(_: Request<Body>) -> Result<&'static str> {
    Ok("Hello, World!")
}

async fn me(_: Request<Body>) -> Result<&'static str> {
    Ok("Hi, It's me!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(|req| serve(req, None))) });

    if let Err(err) = Server::bind(&addr).serve(make_svc).await {
        println!("{}", err);
    }

    Ok(())
}

/// Serves a request and returns a response.
pub async fn serve(
    mut req: Request<Body>,
    mut addr: Option<SocketAddr>,
) -> Result<Response<Body>, Infallible> {
    let method = req.method().to_owned();
    let path = req.path().to_owned();

    Ok(
        match TREE.find(&method, &path).or_else(|| {
            if method == Method::HEAD {
                TREE.find(&Method::GET, &path)
            } else {
                None
            }
        }) {
            Some((handler, params)) => {
                if addr.is_some() {
                    req.extensions_mut().insert(addr.take());
                }
                req.extensions_mut().insert(Into::<Params>::into(params));
                handler
                    .call(req)
                    .await
                    .unwrap_or_else(IntoResponse::into_response)
            }
            None => StatusCode::NOT_FOUND.into_response(),
        },
    )
}
