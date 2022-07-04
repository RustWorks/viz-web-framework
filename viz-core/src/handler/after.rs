use crate::{async_trait, Body, Handler, IntoResponse, Response, Result};

#[derive(Clone)]
pub struct After<H, F> {
    h: H,
    f: F,
}

impl<H, F> After<H, F> {
    #[inline]
    pub(crate) fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

#[async_trait]
impl<H, F, I, O> Handler<I> for After<H, F>
where
    I: Send + 'static,
    O: IntoResponse + Send,
    H: Handler<I, Output = Result<O>> + Clone,
    F: Handler<Result<Response<Body>>, Output = Result<Response<Body>>> + Clone,
{
    type Output = F::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.f
            .call(self.h.call(i).await.map(IntoResponse::into_response))
            .await
    }
}
