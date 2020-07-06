use std::{future::Future, pin::Pin};

use viz_core::{Context, Middleware, Response, Result};

pub struct RecoverMiddleware;

impl RecoverMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RecoverMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Middleware<'a, Context> for RecoverMiddleware {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(async move { Ok(cx.next().await.into()) })
    }
}
