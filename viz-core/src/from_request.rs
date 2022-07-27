//! Extracts data from the [Request] by types.

use std::convert::Infallible;

use crate::{async_trait, IntoResponse, Request};

/// An interface for extracting data from the HTTP [`Request`].
#[async_trait]
pub trait FromRequest: Sized {
    /// The type returned in the event of a conversion error.
    type Error: IntoResponse;

    /// Extracts this type from the HTTP [`Request`].
    #[must_use]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error>;
}

#[async_trait]
impl<T> FromRequest for Option<T>
where
    T: FromRequest,
{
    type Error = Infallible;

    #[inline]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(T::extract(req).await.ok())
    }
}

#[async_trait]
impl<T> FromRequest for Result<T, T::Error>
where
    T: FromRequest,
{
    type Error = Infallible;

    #[inline]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(T::extract(req).await)
    }
}
