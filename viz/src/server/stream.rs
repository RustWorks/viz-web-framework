use std::{
    convert::Infallible,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use hyper::service::Service;

use crate::{
    Body, Handler, IntoResponse, Method, Params, Request, RequestExt, Response, StatusCode, Tree,
};

pub struct Stream {
    tree: Arc<Tree>,
    addr: Option<SocketAddr>,
}

impl Stream {
    pub fn new(tree: Arc<Tree>, addr: Option<SocketAddr>) -> Self {
        Self { tree, addr }
    }
}

impl Service<Request<Body>> for Stream {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(serve(req, self.tree.clone(), self.addr))
    }
}

/// Serves a request and returns a response.
pub async fn serve(
    mut req: Request<Body>,
    tree: Arc<Tree>,
    mut addr: Option<SocketAddr>,
) -> Result<Response<Body>, Infallible> {
    let method = req.method().to_owned();
    let path = req.path().to_owned();

    Ok(
        match tree.find(&method, &path).or_else(|| {
            if method == Method::HEAD {
                tree.find(&Method::GET, &path)
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
