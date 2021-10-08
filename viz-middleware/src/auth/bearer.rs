use std::{future::Future, pin::Pin};

use viz_core::{
    http::{
        header::{HeaderValue, WWW_AUTHENTICATE},
        headers::{authorization, HeaderMapExt},
        StatusCode,
    },
    Context, Middleware, Response, Result,
};

use viz_utils::tracing;

/// Bearer Auth Middleware
#[derive(Debug)]
pub struct Bearer<F>
where
    F: Fn(&str) -> bool,
{
    f: F,
}

impl<F> Bearer<F>
where
    F: Fn(&str) -> bool,
{
    const INVALID: &'static str = "invalid authorization header";

    /// Creates a `Bearer`
    pub fn new(f: F) -> Self {
        Self { f }
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let verified = cx
            .headers()
            .typed_get::<authorization::Authorization<authorization::Bearer>>()
            .map(|auth| (self.f)(auth.0.token()))
            .unwrap_or_default();

        tracing::trace!(" {:>7}", verified);

        if verified {
            return cx.next().await;
        }

        let mut res: Response = StatusCode::UNAUTHORIZED.into();
        res.headers_mut().insert(WWW_AUTHENTICATE, HeaderValue::from_static(Self::INVALID));

        Ok(res)
    }
}

impl<'a, F> Middleware<'a, Context> for Bearer<F>
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
