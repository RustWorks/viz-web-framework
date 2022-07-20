//! A handler with extractors

use std::marker::PhantomData;

use crate::{async_trait, FromRequest, Handler, IntoResponse, Request, Response, Result};

/// A handler with extractors
#[async_trait]
pub trait FnExt<I>: Clone + Send + Sync + 'static {
    type Output;

    #[must_use]
    async fn call(&self, req: Request) -> Self::Output;

    /// Converts it into a handler
    fn to_handler<O>(self) -> ResponderExt<Self, I, O>
    where
        I: FromRequest + Send + Sync + 'static,
        I::Error: IntoResponse + Send + Sync,
        O: IntoResponse + Send + Sync + 'static,
    {
        ResponderExt::new(self)
    }
}

/// Responder with extractor
pub struct ResponderExt<H, I = (), O = ()>(H, PhantomData<I>, PhantomData<O>);

impl<H, I, O> Clone for ResponderExt<H, I, O>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData, PhantomData)
    }
}

impl<H, I, O> ResponderExt<H, I, O> {
    pub fn new(h: H) -> Self {
        Self(h, PhantomData, PhantomData)
    }
}

#[async_trait]
impl<H, I, O> Handler<Request> for ResponderExt<H, I, O>
where
    I: FromRequest + Send + Sync + 'static,
    I::Error: IntoResponse + Send + Sync,
    H: FnExt<I, Output = Result<O>>,
    O: IntoResponse + Send + Sync + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        self.0
            .call(req)
            .await
            .map(IntoResponse::into_response)
            .map_err(IntoResponse::into_error)
    }
}
