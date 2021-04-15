use std::{future::Future, pin::Pin};

use viz_core::{Context, Middleware, Response, Result};

use viz_utils::tracing;

/// Recover Middleware
#[derive(Debug, Default)]
pub struct RecoverMiddleware {}

impl RecoverMiddleware {
    #[tracing::instrument(skip(cx))]
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        cx.next().await
    }
}

impl<'a> Middleware<'a, Context> for RecoverMiddleware {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}
