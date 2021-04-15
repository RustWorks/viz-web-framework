//! Trait implemented by types that can be extracted from Context.

use viz_utils::{futures::future::BoxFuture, tracing};

use crate::{Context, Error, Response, Result};

/// Extract
pub trait Extract: Sized {
    type Error: Into<Response> + Into<Error>;

    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>>;
}

impl<T> Extract for Option<T>
where
    T: Extract,
    T::Error: Into<Error>,
{
    type Error = T::Error;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move {
            Ok(match T::extract(cx).await {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::debug!("Error for Option<T> extractor: {}", e.into());
                    None
                }
            })
        })
    }
}

impl<T> Extract for Result<T, T::Error>
where
    T: Extract,
{
    type Error = T::Error;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { Ok(T::extract(cx).await) })
    }
}
