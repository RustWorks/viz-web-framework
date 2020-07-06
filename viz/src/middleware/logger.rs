use std::{future::Future, pin::Pin, time::Instant};

use viz_utils::log;

use viz_core::{http, Context, Middleware, Response, Result};

pub struct LoggerMiddleware;

impl LoggerMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LoggerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerMiddleware {
    async fn log(&self, cx: &mut Context) -> Result<Response> {
        let start = Instant::now();
        let method = cx.method().to_string();
        let path = cx.uri().path().to_owned();

        log::info!("> {:>7} {}", method, path);

        match cx.next().await {
            Ok(res) => {
                let status = res.status();

                if status == http::StatusCode::INTERNAL_SERVER_ERROR {
                    log::error!(
                        "< {:>7} {} {} {:?}",
                        method,
                        path,
                        status.as_u16(),
                        start.elapsed(),
                    );
                } else {
                    log::info!(
                        "< {:>7} {} {} {:?}",
                        method,
                        path,
                        status.as_u16(),
                        start.elapsed(),
                    );
                }

                Ok(res)
            }
            Err(err) => {
                log::error!("< {:>7} {} {} {:?}", method, path, err, start.elapsed(),);
                Err(err)
            }
        }
    }
}

impl<'a> Middleware<'a, Context> for LoggerMiddleware {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.log(cx))
    }
}
