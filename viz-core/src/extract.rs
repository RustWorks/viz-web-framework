//!
//! Thanks:
//!   ntex:     https://docs.rs/ntex/0.1.14/ntex/web/trait.FromRequest.html
//!   warp:     https://github.com/seanmonstar/warp/blob/master/src/generic.rs
//!   rocket:   https://docs.rs/rocket/0.4.4/rocket/request/trait.FromRequest.html
//!   tower:    https://docs.rs/tower-web/0.3.7/tower_web/extract/trait.Extract.html

use crate::BoxFuture;
use crate::Context;
use crate::Error;
use crate::Result;

pub trait Extract: Sized {
    type Error: Into<Error>;

    fn extract<'a>(cx: &'a Context) -> BoxFuture<'a, Result<Self, Self::Error>>;
}

impl<T> Extract for Option<T>
where
    T: Extract,
{
    type Error = T::Error;

    #[inline]
    fn extract<'a>(cx: &'a Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move {
            Ok(match T::extract(cx).await {
                Ok(v) => Some(v),
                Err(e) => {
                    log::debug!("Error for Option<T> extractor: {}", e.into());
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
    fn extract<'a>(cx: &'a Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { Ok(T::extract(cx).await) })
    }
}
