//!
//! Thanks:
//!   ntex:     https://docs.rs/ntex/0.1.14/ntex/web/trait.FromRequest.html
//!   rocket:   https://docs.rs/rocket/0.4.4/rocket/request/trait.FromRequest.html

use crate::Context;
use crate::Error;
use crate::Future;
use crate::Pin;
use crate::Result;

pub trait FromContext: Sized {
    type Error: Into<Error>;

    fn from_context<'a>(
        cx: &'a Context,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>>;
}

impl FromContext for Context {
    type Error = Error;

    #[inline]
    fn from_context<'a>(
        cx: &'a Context,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>> {
        Box::pin(async move { Ok(cx.clone()) })
    }
}

impl<T> FromContext for Option<T>
where
    T: FromContext,
    T::Error: Into<Error>,
{
    type Error = T::Error;

    #[inline]
    fn from_context<'a>(
        cx: &'a Context,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match T::from_context(cx).await {
                Ok(v) => Some(v),
                Err(e) => {
                    log::debug!("Error for Option<T> extractor: {}", e.into());
                    None
                }
            })
        })
    }
}

impl<T> FromContext for Result<T, T::Error>
where
    T: FromContext,
{
    type Error = T::Error;

    #[inline]
    fn from_context<'a>(
        cx: &'a Context,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>> {
        Box::pin(async move { Ok(T::from_context(cx).await) })
    }
}

macro_rules! peel {
    ($T0:ident, $($T:ident,)*) => (tuple! { $($T,)* })
}

macro_rules! tuple {
    () => (
        #[doc(hidden)]
        impl FromContext for () {
            type Error = Error;

            #[inline]
            fn from_context<'a>(
                _cx: &'a Context,
            ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>> {
                Box::pin(async { Ok(()) })
            }
        }
    );
    ( $($T:ident,)+ ) => (
        impl<$($T),+> FromContext for ($($T,)+)
        where
            $($T: FromContext + Send,)+
            $($T::Error: Into<Error> + Send,)+
        {
            type Error = Error;

            #[inline]
            fn from_context<'a>(
                cx: &'a Context,
            ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>> {
                Box::pin(async move {
                    Ok((
                        $(
                            match $T::from_context(cx).await {
                                Ok(v) => v,
                                Err(e) => return Err(e.into()),
                            },
                        )+
                    ))
                })
            }
        }
        peel! { $($T,)+ }
    )
}

tuple! { A, B, C, D, E, F, G, H, I, J, K, L, }
