//!
//! Thanks:
//!   ntex:     https://docs.rs/ntex/0.1.14/ntex/web/trait.Handler.html
//!   warp:     https://github.com/seanmonstar/warp/blob/master/src/generic.rs

use std::marker::PhantomData;

use crate::Context;
use crate::FromContext;
use crate::Future;
use crate::Pin;
use crate::Response;
use crate::Result;

pub trait HandlerBase<Args>: Clone + 'static {
    type Output: Into<Response>;
    type Future: Future<Output = Self::Output> + Send + 'static;

    fn call(&self, args: Args) -> Self::Future;
}

pub trait Handler: Send + 'static {
    fn call(&self, _: Context) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>>;

    fn clone_handler(&self) -> Box<dyn Handler>;
}

pub struct HandlerWrapper<F, T>
where
    F: HandlerBase<T>,
    T: FromContext,
    T::Error: Into<Response>,
{
    h: F,
    _t: PhantomData<T>,
}

impl<F, T> HandlerWrapper<F, T>
where
    F: HandlerBase<T>,
    T: FromContext,
    T::Error: Into<Response>,
{
    pub fn new(h: F) -> Self {
        HandlerWrapper { h, _t: PhantomData }
    }
}

impl<F, T> Handler for HandlerWrapper<F, T>
where
    F: HandlerBase<T> + Send + Sync,
    T: FromContext + Send + 'static,
    T::Error: Into<Response> + Send,
{
    #[inline]
    fn call(&self, cx: Context) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        let h = self.h.clone();
        Box::pin(async move {
            Ok(match T::from_context(&cx).await {
                Ok(args) => h.call(args).await.into(),
                Err(e) => e.into(),
            })
        })
    }

    #[inline]
    fn clone_handler(&self) -> Box<dyn Handler> {
        Box::new(HandlerWrapper {
            h: self.h.clone(),
            _t: PhantomData,
        })
    }
}

macro_rules! peel {
    ($T0:ident, $($T:ident,)*) => (tuple! { $($T,)* })
}

macro_rules! tuple {
    () => {
        impl<F, R> HandlerBase<()> for F
        where
            F: Fn() -> R + Clone + 'static,
            R: Future + Send + 'static,
            R::Output: Into<Response>,
        {
            type Output = R::Output;
            type Future = R;

            fn call(&self, _: ()) -> R {
                (self)()
            }
        }
    };
    ( $($T:ident,)+ ) => (
        impl<Func, $($T,)+ R> HandlerBase<($($T,)+)> for Func
        where
            Func: Fn($($T,)+) -> R + Clone + 'static,
            R: Future + Send + 'static,
            R::Output: Into<Response>,
        {
            type Output = R::Output;
            type Future = R;

            #[inline]
            fn call(&self, args: ($($T,)+)) -> R {
                #[allow(non_snake_case)]
                let ($($T,)+) = args;
                (self)($($T,)+)
            }
        }

        peel! { $($T,)+ }
    )
}

tuple! { A, B, C, D, E, F, G, H, I, J, K, L, }

#[cfg(test)]
mod test {
    use crate::*;
    use anyhow::anyhow;
    use futures::executor::block_on;

    #[test]
    fn handler() {
        #[derive(Debug, PartialEq)]
        struct Info {
            hello: String,
        }

        impl FromContext for Info {
            type Error = Error;

            fn from_context<'a>(
                _: &'a Context,
            ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>> {
                Box::pin(async {
                    Ok(Info {
                        hello: "world".to_owned(),
                    })
                })
            }
        }

        #[derive(Debug, PartialEq)]
        struct User {
            id: usize,
        }

        impl FromContext for User {
            type Error = Error;

            fn from_context<'a>(
                _: &'a Context,
            ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>> {
                Box::pin(async {
                    Ok(User { id: 0 })
                    // Err(anyhow!("User Error"))
                })
            }
        }

        /// Helper method for extractors testing
        pub async fn from_context<T: FromContext>(cx: &Context) -> Result<T, T::Error> {
            T::from_context(cx).await
        }

        block_on(async move {
            let cx = Context::new();

            let r = from_context::<Option<Info>>(&cx).await.unwrap();

            assert_eq!(
                r,
                Some(Info {
                    hello: "world".to_owned(),
                })
            );

            let r0 = from_context::<(Info, User)>(&cx).await.unwrap();
            let r1 = from_context::<(User, Info)>(&cx).await.unwrap();

            assert_eq!(r0.0, r1.1);
            assert_eq!(r0.1, r1.0);

            fn make_handler<F, Args>(handler: F) -> Box<dyn Handler>
            where
                F: HandlerBase<Args> + Send + Sync + 'static,
                Args: FromContext + Send + 'static,
                Args::Error: Into<Response> + Send,
            {
                Box::new(HandlerWrapper::new(handler))
            }

            async fn a() -> Response {
                Response::new()
            }

            let h = make_handler(a);
            let cx = Context::new();
            let r = h.call(cx).await;
            assert!(r.is_ok());

            async fn b(i: Info, u: User) -> Result<Response> {
                assert_eq!(
                    i,
                    Info {
                        hello: "world".to_owned(),
                    }
                );
                assert_eq!(u, User { id: 0 });
                Ok(Response::new())
            }
            let h = make_handler(b);
            let cx = Context::new();
            let r = h.call(cx).await;
            assert!(r.is_ok());

            let c = || async { Response::new() };
            let h = make_handler(c);
            let cx = Context::new();
            let r = h.call(cx).await;
            assert!(r.is_ok());

            let d = || Box::pin(async { anyhow!("throws error and converts to response") });
            let h = make_handler(d);
            let cx = Context::new();
            let hh = h.clone_handler();
            let r = h.call(cx).await;
            let cx = Context::new();
            let r0 = hh.call(cx).await;
            assert_eq!(r.is_ok(), r0.is_ok());

            async fn e(u: User) -> Result<Response> {
                assert_eq!(u, User { id: 0 });
                Ok(Response::new())
            }
            let h = make_handler(e);
            let cx = Context::new();
            let r = h.call(cx).await;
            assert!(r.is_ok());

            impl FromContext for usize {
                type Error = Error;

                fn from_context<'a>(
                    _: &'a Context,
                ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'a>>
                {
                    Box::pin(async { Ok(0) })
                }
            }

            async fn f(u: User, n: usize) -> &'static str {
                assert_eq!(u, User { id: 0 });
                assert_eq!(n, 0);
                "Hello world"
            }
            let h = make_handler(f);
            let cx = Context::new();
            let r = h.call(cx).await;
            assert!(r.is_ok());
        });
    }
}
