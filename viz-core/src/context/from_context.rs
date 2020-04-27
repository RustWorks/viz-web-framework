//!
//! Thanks:
//!   ntex:     https://docs.rs/ntex/0.1.13/ntex/web/trait.FromRequest.html
//!   rocket:   https://docs.rs/rocket/0.4.4/rocket/request/trait.FromRequest.html

use crate::async_trait;
use crate::Context;
use crate::Error;
use crate::Result;

#[async_trait(?Send)]
pub trait FromContext: Sized {
    type Error: Into<Error>;

    async fn from_context(cx: &Context) -> Result<Self, Self::Error>;
}

#[async_trait(?Send)]
impl FromContext for Context {
    type Error = Error;

    #[inline]
    async fn from_context(cx: &Context) -> Result<Self, Self::Error> {
        Ok(cx.clone())
    }
}

#[async_trait(?Send)]
impl<T> FromContext for Option<T>
where
    T: FromContext,
    T::Error: Into<Error>,
{
    type Error = T::Error;

    #[inline]
    async fn from_context(cx: &Context) -> Result<Self, Self::Error> {
        Ok(match T::from_context(cx).await {
            Ok(v) => Some(v),
            Err(e) => {
                log::debug!("Error for Option<T> extractor: {}", e.into());
                None
            }
        })
    }
}

#[async_trait(?Send)]
impl<T> FromContext for Result<T, T::Error>
where
    T: FromContext,
{
    type Error = T::Error;

    #[inline]
    async fn from_context(cx: &Context) -> Result<Self, Self::Error> {
        Ok(T::from_context(cx).await)
    }
}

macro_rules! peel {
    ($T0:ident, $($T:ident,)*) => (tuple! { $($T,)* })
}

macro_rules! tuple {
    () => (
        #[doc(hidden)]
        #[async_trait(?Send)]
        impl FromContext for () {
            type Error = Error;

            #[inline]
            async fn from_context(_: &Context) -> Result<Self, Self::Error> {
                Ok(())
            }
        }
    );
    ( $($T:ident,)+ ) => (
        #[async_trait(?Send)]
        impl<$($T),+> FromContext for ($($T,)+)
        where
            $($T: FromContext,)+
            $($T::Error: Into<Error>,)+
        {
            type Error = Error;

            #[inline]
            async fn from_context(cx: &Context) -> Result<Self, Self::Error> {
                Ok((
                    $(
                        match $T::from_context(cx).await {
                            Ok(v) => v,
                            Err(e) => return Err(e.into()),
                        },
                    )+
                ))
            }
        }
        peel! { $($T,)+ }
    )
}

tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }
