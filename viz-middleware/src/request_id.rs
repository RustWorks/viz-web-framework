use std::{future::Future, pin::Pin};

use viz_core::{http, Context, Error, Middleware, Response, Result};

const HEADER: &str = "x-request-id";

fn generate_id() -> Result<String> {
    Ok(nano_id::base64(21))
}

/// RequestID Middleware
pub struct RequestIDMiddleware {
    /// Header Name is must be lower-case.
    header: &'static str,
    /// Generates request id
    generator: Box<dyn Send + Sync + 'static + Fn() -> Result<String>>,
}

impl RequestIDMiddleware {
    /// Creates new `RequestIDMiddleware`
    pub fn new<F>(header: &'static str, generator: F) -> Self
    where
        F: Send + Sync + 'static + Fn() -> Result<String>,
    {
        Self { header, generator: Box::new(generator) }
    }
}

impl Default for RequestIDMiddleware {
    fn default() -> Self {
        Self::new(HEADER, generate_id)
    }
}

impl RequestIDMiddleware {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let mut res: Response = cx.next().await.into();

        res.headers_mut().insert(
            http::header::HeaderName::from_static(self.header),
            match cx.header_value(&self.header).cloned() {
                Some(id) => id,
                None => (self.generator)()
                    .and_then(|id| http::HeaderValue::from_str(&id).map_err(Error::new))?,
            },
        );

        Ok(res)
    }
}

impl<'a> Middleware<'a, Context> for RequestIDMiddleware {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}
