use std::{
    convert::Infallible,
    future::{ready, Future, Ready},
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
    {fmt, net::SocketAddr, sync::Arc},
};

use tower_service::Service;

use viz_core::{
    config::Config,
    http,
    types::{Params, State, StateFactory},
    Context as VizContext, Error, Result,
};
use viz_router::{Method, Router, Tree};
use viz_utils::tracing;

/// Viz Server
#[derive(Clone)]
pub struct App {
    tree: Arc<Tree>,
    config: Option<Arc<Config>>,
    state: Option<Vec<Arc<dyn StateFactory>>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Creates a server
    pub fn new() -> Self {
        Self { state: None, config: None, tree: Arc::new(Tree::new()) }
    }

    /// Sets a `State`
    pub fn state<T>(&mut self, state: T) -> &mut Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.state.get_or_insert_with(Vec::new).push(Arc::new(State::new(state)));
        self
    }

    /// Sets a `Router`
    pub fn routes(&mut self, router: Router) -> &mut Self {
        router.finish(Arc::get_mut(&mut self.tree).unwrap());
        self
    }

    /// Gets the `Config`
    pub async fn config(&mut self) -> Arc<Config> {
        tracing::info!("loading config");
        self.config.replace(Arc::new(Config::load().await.unwrap_or_default()));
        self.config.clone().unwrap()
    }

    /// Into to the Tower Service
    pub fn into_make_service(self) -> IntoMakeService<Self> {
        IntoMakeService::new(self)
    }
}

/// Serves a request and returns a response.
pub async fn serve(
    req: http::Request,
    mut addr: Option<SocketAddr>,
    tree: Arc<Tree>,
    state: Vec<Arc<dyn StateFactory>>,
    config: Arc<Config>,
) -> Result<http::Response> {
    let mut cx = VizContext::from(req);
    if addr.is_some() {
        cx.extensions_mut().insert(addr.take());
    }
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
                tree.get(&Method::Verb(http::Method::GET)).and_then(|t| t.find(path))
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

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App").finish()
    }
}

#[derive(Debug, Clone)]
pub struct AppStream {
    app: App,
    addr: Option<SocketAddr>,
}

impl AppStream {
    pub fn new(app: App, addr: Option<SocketAddr>) -> Self {
        Self { app, addr }
    }
}

impl Deref for AppStream {
    type Target = App;

    fn deref(&self) -> &App {
        &self.app
    }
}

impl Service<http::Request<http::Body>> for AppStream {
    type Response = http::Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<http::Response>> + Send>>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: http::Request) -> Self::Future {
        Box::pin(serve(
            req,
            self.addr,
            self.tree.clone(),
            self.state.clone().unwrap_or_default(),
            self.config.clone().unwrap_or_default(),
        ))
    }
}

/// Via https://docs.rs/axum/latest/axum/routing/struct.IntoMakeService.html
#[derive(Debug, Clone)]
pub struct IntoMakeService<S> {
    service: S,
}

impl<S> IntoMakeService<S> {
    fn new(service: S) -> Self {
        Self { service }
    }
}

#[cfg(feature = "tcp")]
impl Service<&hyper::server::conn::AddrStream> for IntoMakeService<App> {
    type Response = AppStream;
    type Error = Infallible;
    type Future = Ready<Result<AppStream, Infallible>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, t: &hyper::server::conn::AddrStream) -> Self::Future {
        ready(Ok(AppStream::new(self.service.clone(), Some(t.remote_addr()))))
    }
}

#[cfg(all(unix, feature = "uds"))]
impl Service<&tokio::net::UnixStream> for IntoMakeService<App> {
    type Response = AppStream;
    type Error = Infallible;
    type Future = Ready<Result<AppStream, Infallible>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _t: &tokio::net::UnixStream) -> Self::Future {
        ready(Ok(AppStream::new(self.service.clone(), None)))
    }
}
