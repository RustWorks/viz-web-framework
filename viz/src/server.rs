use std::{io, net::SocketAddr, sync::Arc};

use hyper::{
    server::{
        conn::{AddrIncoming, AddrStream},
        Server as HyperServer,
    },
    service::{make_service_fn, service_fn},
};

use viz_core::{
    config::Config,
    http,
    types::{Params, State, StateFactory},
    Context, Error, Result,
};
use viz_router::{Method, Router, Tree};
use viz_utils::{anyhow::anyhow, log};

/// Viz Server
pub struct Server {
    tree: Arc<Tree>,
    config: Option<Arc<Config>>,
    state: Option<Vec<Arc<dyn StateFactory>>>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            state: None,
            config: None,
            tree: Arc::new(Tree::new()),
        }
    }

    pub fn state<T>(&mut self, state: T) -> &mut Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.state
            .get_or_insert_with(Vec::new)
            .push(Arc::new(State::new(state)));
        self
    }

    pub fn routes(&mut self, router: Router) -> &mut Self {
        router.finish(Arc::get_mut(&mut self.tree).unwrap());
        self
    }

    pub async fn config(&mut self) -> Arc<Config> {
        log::info!("loading config");
        self.config
            .replace(Arc::new(Config::load().await.unwrap_or_default()));
        self.config.clone().unwrap()
    }

    pub async fn listen<A: ToString>(self, addr: A) -> Result {
        let addr = addr
            .to_string()
            .parse::<SocketAddr>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut incoming = AddrIncoming::bind(&addr)?;
        let addr = incoming.local_addr();
        incoming.set_nodelay(true);

        let config = self.config.unwrap_or_default();
        let state = self.state.unwrap_or_default();
        let tree = self.tree;
        let srv =
            HyperServer::builder(incoming).serve(make_service_fn(move |stream: &AddrStream| {
                let addr = stream.remote_addr();
                let config = config.clone();
                let state = state.clone();
                let tree = tree.clone();
                async move {
                    Ok::<_, Error>(service_fn(move |req| {
                        serve(req, addr, state.clone(), config.clone(), tree.clone())
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
    state: Vec<Arc<dyn StateFactory>>,
    config: Arc<Config>,
    tree: Arc<Tree>,
) -> Result<http::Response> {
    let mut cx = Context::from(req);
    cx.extensions_mut().insert(addr);
    cx.extensions_mut().insert(config);
    for t in state.iter() {
        t.create(cx.extensions_mut());
    }

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
