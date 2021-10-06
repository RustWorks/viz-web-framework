use std::{future::Future, pin::Pin};

use viz_core::{http, Context, Error, Middleware, Response, Result};
use viz_utils::tracing;

const HEADER: &str = "x-request-id";

#[cfg(all(feature = "request-nanoid", not(feature = "request-uuid")))]
fn generate_id() -> Result<String> {
    Ok(nano_id::base64::<21>())
}
#[cfg(all(feature = "request-uuid", not(feature = "request-nanoid")))]
fn generate_id() -> Result<String> {
    Ok(uuid::v4())
}

/// RequestID Middleware
pub struct RequestID<F = fn() -> Result<String>> {
    /// Header Name is must be lower-case.
    header: &'static str,
    /// Generates request id
    generator: F,
}

impl Default for RequestID {
    fn default() -> Self {
        Self::new(HEADER, generate_id)
    }
}

impl<F> RequestID<F>
where
    F: Fn() -> Result<String>,
{
    /// Creates new `RequestID` Middleware.
    pub fn new(header: &'static str, generator: F) -> Self {
        Self { header, generator }
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let mut res: Response = cx.next().await.into();

        let id = match cx.header_value(&self.header).cloned() {
            Some(id) => id,
            None => (self.generator)()
                .and_then(|id| http::HeaderValue::from_str(&id).map_err(Error::new))?,
        };

        tracing::trace!(" {:>7?}", id);

        res.headers_mut().insert(http::header::HeaderName::from_static(self.header), id);

        Ok(res)
    }
}

impl<'a, F> Middleware<'a, Context> for RequestID<F>
where
    F: Sync + Send + 'static + Fn() -> Result<String>,
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
