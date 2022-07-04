use crate::{async_trait, Handler, Result};

#[derive(Clone)]
pub struct Before<H, F> {
    h: H,
    f: F,
}

impl<H, F> Before<H, F> {
    #[inline]
    pub(crate) fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

#[async_trait]
impl<H, F, I, O> Handler<I> for Before<H, F>
where
    I: Send + 'static,
    F: Handler<I, Output = Result<I>> + Clone,
    H: Handler<I, Output = Result<O>> + Clone,
{
    type Output = H::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.h.call(self.f.call(i).await?).await
    }
}
