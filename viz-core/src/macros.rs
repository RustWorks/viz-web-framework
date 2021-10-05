use crate::{BoxFuture, Context, Extract, Future, Handler, Response, Result};

macro_rules! peel {
    ($T0:ident, $($T:ident,)*) => (tuple! { $($T,)* })
}

macro_rules! tuple {
    () => (
        #[doc(hidden)]
        impl Extract for ()
        {
            type Error = Response;

            fn extract(_: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
                Box::pin(async { Ok(()) })
            }
        }

        #[doc(hidden)]
        impl<Func, Fut > Handler<()> for Func
        where
            Func: Fn() -> Fut + Clone + 'static,
            Fut: Future + Send + 'static,
            Fut::Output: Into<Response>,
        {
            type Output = Fut::Output;
            type Future = Fut;

            fn call(&self, _: ()) -> Self::Future {
                (self)()
            }
        }
    );
    ($($T:ident,)+) => (
        #[doc(hidden)]
        impl<$($T),+> Extract for ($($T,)+)
        where
            $($T: Extract + Send,)+
            $($T::Error: Into<Response> + Send + 'static,)+
        {
            type Error = Response;

            fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
                Box::pin(async move {
                    Ok((
                        $(
                            $T::extract(cx).await.map_err(Into::<Response>::into)?,
                        )+
                    ))
                })
            }
        }

        #[doc(hidden)]
        impl<Func, $($T,)+ Fut> Handler<($($T,)+)> for Func
        where
            Func: Fn($($T,)+) -> Fut + Clone + 'static,
            Fut: Future + Send + 'static,
            Fut::Output: Into<Response>,
        {
            type Output = Fut::Output;
            type Future = Fut;

            fn call(&self, args: ($($T,)+)) -> Self::Future {
                #[allow(non_snake_case)]
                let ($($T,)+) = args;
                (self)($($T,)+)
            }
        }

        peel! { $($T,)+ }
    )
}

tuple! { A, B, C, D, E, F, G, H, I, J, K, L, }
