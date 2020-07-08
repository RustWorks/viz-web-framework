use std::{future::Future, pin::Pin, time::Duration};

use async_io::Timer;

use viz_utils::{
    futures::future::{select, Either, FutureExt},
    log,
};

use viz_core::{http, Context, Middleware, Response, Result};

// 0.256s
pub struct TimeoutMiddleware {
    delay: Duration,
}

impl TimeoutMiddleware {
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }
}

impl Default for TimeoutMiddleware {
    fn default() -> Self {
        Self::new(Duration::from_millis(256))
    }
}

impl TimeoutMiddleware {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let method = cx.method().to_owned();
        let path = cx.path().to_owned();

        match select(cx.next().boxed(), self.timeout(method, path).boxed()).await {
            Either::Left((x, _)) => x,
            Either::Right((y, _)) => y,
        }
    }

    async fn timeout(&self, method: http::Method, path: String) -> Result<Response> {
        Timer::new(self.delay).await;

        log::debug!("Timeout: {} {}", method, path);

        Ok(http::StatusCode::REQUEST_TIMEOUT.into())
    }
}

impl<'a> Middleware<'a, Context> for TimeoutMiddleware {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}

