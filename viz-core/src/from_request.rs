use std::convert::Infallible;

use crate::{async_trait, Body, IntoResponse, Request};

#[async_trait]
pub trait FromRequest: Sized {
    type Error: IntoResponse;

    async fn extract(req: &mut Request<Body>) -> Result<Self, Self::Error>;
}

#[async_trait]
impl<T> FromRequest for Option<T>
where
    T: FromRequest,
{
    type Error = Infallible;

    #[inline]
    async fn extract(req: &mut Request<Body>) -> Result<Self, Self::Error> {
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
    async fn extract(req: &mut Request<Body>) -> Result<Self, Self::Error> {
        Ok(T::extract(req).await)
    }
}
