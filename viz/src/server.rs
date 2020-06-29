use std::{convert::Infallible, io, net::SocketAddr, sync::Arc};

use hyper::{
    server::{
        conn::{AddrIncoming, AddrStream},
        Server as HyperServer,
    },
    service::{make_service_fn, service_fn},
};

use viz_core::{http, Context, Params, Result};
use viz_router::{Method, Router, Tree};
use viz_utils::{anyhow::anyhow, log};

pub struct Server {
    tree: Arc<Tree>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            tree: Arc::new(Tree::new()),
        }
    }

    pub fn routes(mut self, router: Router) -> Self {
        router.finish(Arc::get_mut(&mut self.tree).unwrap());
        self
    }

    pub async fn listen<A: ToString>(self, addr: A) -> Result {
        let addr = addr
            .to_string()
            .parse::<SocketAddr>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut incoming = AddrIncoming::bind(&addr)?;
        let addr = incoming.local_addr();
        incoming.set_nodelay(true);

        let tree = self.tree;
        let srv =
            HyperServer::builder(incoming).serve(make_service_fn(move |stream: &AddrStream| {
                let tree = tree.clone();
                let addr = stream.remote_addr();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        serve(req, addr, tree.clone())
                    }))
                }
            }));

        log::info!("listening on http://{}", addr);

        srv.await.map_err(|e| anyhow!(e))
    }
}

/// Serves a request and returns a response.
pub async fn serve(
    req: http::Request,
    addr: SocketAddr,
    tree: Arc<Tree>,
) -> Result<http::Response> {
    let mut cx = Context::from(req);
    cx.extensions_mut().insert(addr);

    let method = cx.method().to_owned();
    let path = cx.path();

    if let Some((handler, params)) = tree
        .get(&Method::Verb(method.to_owned()))
        .and_then(|t| t.find(path))
        .or_else(|| {
            if method == http::Method::HEAD {
                tree.get(&Method::Verb(http::Method::GET))
                    .and_then(|t| t.find(path))
            } else {
                None
            }
        })
        .or_else(|| tree.get(&Method::All).and_then(|t| t.find(path)))
    {
        let params: Params = params.into();
        *cx.middleware_mut() = handler.to_owned();
        cx.extensions_mut().insert(params);
    }

    Ok(cx.next().await?.into())
}
