//! Trait implemented by types that can be extracted from Context.
//!
//! Thanks:
//!   actix:    https://docs.rs/actix-web/3.0.0-alpha.3/actix_web/trait.FromRequest.html
//!   rocket:   https://docs.rs/rocket/0.4.4/rocket/request/trait.FromRequest.html
//!   tower:    https://docs.rs/tower-web/0.3.7/tower_web/extract/trait.Extract.html
//!   warp:     https://github.com/seanmonstar/warp/blob/master/src/generic.rs

use viz_utils::{futures::future::BoxFuture, log};

use crate::{Context, Error, Response, Result};

pub trait Extract: Sized {
    type Error: Into<Error> + Into<Response>;

    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>>;
}

impl<T> Extract for Option<T>
where
    T: Extract,
{
    type Error = T::Error;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move {
            Ok(match T::extract(cx).await {
                Ok(v) => Some(v),
                Err(e) => {
                    log::debug!("Error for Option<T> extractor: {}", Into::<Error>::into(e));
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
