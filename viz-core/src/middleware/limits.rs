use std::sync::Arc;

use crate::{
    async_trait, handler::Transform, types, Body, Handler, IntoResponse, Request, Response, Result,
};

#[derive(Debug, Clone)]
pub struct Config {
    limits: types::Limits,
    multipart: Arc<types::MultipartLimits>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limits(mut self, limits: types::Limits) -> Self {
        self.limits = limits;
        self
    }

    pub fn multipart(mut self, limits: types::MultipartLimits) -> Self {
        *Arc::make_mut(&mut self.multipart) = limits;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            limits: types::Limits::default(),
            multipart: Arc::new(types::MultipartLimits::default()),
        }
    }
}

impl<H> Transform<H> for Config
where
    H: Clone,
{
    type Output = LimitsMiddleware<H>;

    fn transform(&self, h: H) -> Self::Output {
        LimitsMiddleware {
            h,
            config: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LimitsMiddleware<H> {
    h: H,
    config: Config,
}

#[async_trait]
impl<H, O> Handler<Request<Body>> for LimitsMiddleware<H>
where
    O: IntoResponse,
    H: Handler<Request<Body>, Output = Result<O>> + Clone,
{
    type Output = Result<Response<Body>>;

    async fn call(&self, mut req: Request<Body>) -> Self::Output {
        req.extensions_mut().insert(self.config.limits.clone());
        req.extensions_mut().insert(self.config.multipart.clone());

        self.h.call(req).await.map(IntoResponse::into_response)
    }
}
