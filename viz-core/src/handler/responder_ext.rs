use std::marker::PhantomData;

use crate::{async_trait, FnExt, FromRequest, Handler, IntoResponse, Request, Result};

/// A wrapper of the extractors handler.
#[derive(Debug)]
pub struct ResponderExt<H, E, O>(H, PhantomData<fn(E) -> O>);

impl<H, E, O> Clone for ResponderExt<H, E, O>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<H, E, O> ResponderExt<H, E, O> {
    /// Create a new `Handler` for the extractors.
    pub fn new(h: H) -> Self {
        Self(h, PhantomData)
    }
}

#[async_trait]
impl<H, E, O> Handler<Request> for ResponderExt<H, E, O>
where
    E: FromRequest + Send + Sync + 'static,
    E::Error: IntoResponse + Send + Sync,
    H: FnExt<E, Output = Result<O>>,
    O: Send + Sync + 'static,
    // O: IntoResponse + Send + Sync + 'static,
{
    type Output = H::Output;
    // type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        self.0
            .call(req)
            .await
            // .map(IntoResponse::into_response)
            .map_err(IntoResponse::into_error)
    }
}
