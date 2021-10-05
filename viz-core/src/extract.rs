use super::BoxFuture;

use crate::{Context, Response};

/// A Extractor trait.
pub trait Extract: Sized {
    /// The type of failures extracted by this Extractor.
    type Error: Into<Response>;

    /// Extract the value from Context.
    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>>;
}

impl<T> Extract for Option<T>
where
    T: Extract,
{
    type Error = T::Error;

    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
        Box::pin(async move { Ok(T::extract(cx).await.ok()) })
    }
}

impl<T> Extract for Result<T, T::Error>
where
    T: Extract,
{
    type Error = T::Error;

    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
        Box::pin(async move { Ok(T::extract(cx).await) })
    }
}
