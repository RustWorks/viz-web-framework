use std::{future::Future, pin::Pin};

use viz_core::{http, types::CookieContextExt, Context, Middleware, Response, Result};

use viz_utils::log;

/// Cookies Middleware
#[derive(Debug, Default)]
pub struct CookiesMiddleware {}

impl CookiesMiddleware {
    /// Creates new `CookiesMiddleware`
    pub fn new() -> Self {
        Self {}
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        log::trace!("Cookies Middleware");

        dbg!(cx.cookies()?);

        let mut res = cx.next().await?;

        let cookies = cx.cookies()?;

        dbg!(&cookies);

        let jar = cookies.read();

        for cookie in jar.delta() {
            res.headers_mut().insert(
                http::header::SET_COOKIE,
                http::header::HeaderValue::from_str(&cookie.encoded().to_string())?,
            );
        }

        Ok(res)
    }
}

impl<'a> Middleware<'a, Context> for CookiesMiddleware {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}
