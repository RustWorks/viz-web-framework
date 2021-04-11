use std::{future::Future, pin::Pin};

use viz_core::{
    http::{
        header::{HeaderValue, WWW_AUTHENTICATE},
        headers::{
            authorization::{Authorization, Bearer},
            HeaderMapExt,
        },
        StatusCode,
    },
    Context, Middleware, Response, Result,
};

use viz_utils::log;

/// Bearer Auth Middleware
#[derive(Debug)]
pub struct BearerMiddleware<F>
where
    F: Fn(&str) -> bool,
{
    f: F,
}

impl<F> BearerMiddleware<F>
where
    F: Fn(&str) -> bool,
{
    /// Creates a `BearerMiddleware`
    pub fn new(f: F) -> Self {
        Self { f }
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        log::trace!("Bearer Auth Middleware");

        if cx
            .headers()
            .typed_get::<Authorization<Bearer>>()
            .map(|auth| (self.f)(auth.0.token()))
            .unwrap_or_default()
        {
            return cx.next().await;
        }

        let mut res: Response = StatusCode::UNAUTHORIZED.into();
        res.headers_mut()
            .insert(WWW_AUTHENTICATE, HeaderValue::from_str("invalid authorization header")?);
        Ok(res)
    }
}

impl<'a, F> Middleware<'a, Context> for BearerMiddleware<F>
where
    F: Sync + Send + 'static + Fn(&str) -> bool,
{
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}
