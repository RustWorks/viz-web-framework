use std::marker::PhantomData;

use crate::{async_trait, Handler, IntoResponse, Response, Result};

pub struct Responder<H, O>(pub(crate) H, PhantomData<O>);

impl<H, O> Responder<H, O> {
    pub fn new(h: H) -> Self {
        Self(h, PhantomData)
    }
}

impl<H, O> Clone for Responder<H, O>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

#[async_trait]
impl<H, I, O> Handler<I> for Responder<H, O>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>> + Clone,
    O: IntoResponse + Send + Sync + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, args: I) -> Self::Output {
        self.0.call(args).await.map(IntoResponse::into_response)
    }
}
