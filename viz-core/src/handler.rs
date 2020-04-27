use std::marker::PhantomData;

use crate::async_trait;
use crate::Context;
use crate::FromContext;
use crate::Future;
use crate::Response;
use crate::Result;

#[async_trait(?Send)]
pub trait HandlerBase<Args>: Clone + 'static {
    type Output: Into<Response>;

    async fn call(&self, args: Args) -> Self::Output;
}

#[async_trait(?Send)]
impl<F, R> HandlerBase<()> for F
where
    F: Clone + 'static + Fn() -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, _: ()) -> Self::Output {
        (self)().await
    }
}

#[async_trait(?Send)]
pub trait Handler {
    async fn call(&self, _: Context) -> Result<Response>;

    fn clone_handler(&self) -> Box<dyn Handler>;
}
pub struct HandlerWrapper<F, T>
where
    F: HandlerBase<T>,
    F::Output: Into<Response>,
    T: FromContext,
    T::Error: Into<Response>,
{
    h: F,
    _t: PhantomData<T>,
}

impl<F, T> HandlerWrapper<F, T>
where
    F: HandlerBase<T>,
    F::Output: Into<Response>,
    T: FromContext,
    T::Error: Into<Response>,
{
    pub fn new(h: F) -> Self {
        HandlerWrapper { h, _t: PhantomData }
    }
}

#[async_trait(?Send)]
impl<F, T> Handler for HandlerWrapper<F, T>
where
    F: HandlerBase<T>,
    F::Output: Into<Response>,
    T: FromContext + 'static,
    T::Error: Into<Response>,
{
    #[inline]
    async fn call(&self, cx: Context) -> Result<Response> {
        Ok(match T::from_context(&cx).await {
            Ok(args) => self.h.call(args).await.into(),
            Err(e) => e.into(),
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

#[async_trait(?Send)]
impl<Func, T0, R> HandlerBase<(T0,)> for Func
where
    T0: 'static,
    Func: Clone + 'static + Fn(T0) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0,)) -> Self::Output {
        (self)(args.0).await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, R> HandlerBase<(T0, T1)> for Func
where
    T0: 'static,
    T1: 'static,
    Func: Clone + 'static + Fn(T0, T1) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1)) -> Self::Output {
        (self)(args.0, args.1).await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, R> HandlerBase<(T0, T1, T2)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2)) -> Self::Output {
        (self)(args.0, args.1, args.2).await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, R> HandlerBase<(T0, T1, T2, T3)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3)) -> Self::Output {
        (self)(args.0, args.1, args.2, args.3).await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, T4, R> HandlerBase<(T0, T1, T2, T3, T4)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3, T4) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3, T4)) -> Self::Output {
        (self)(args.0, args.1, args.2, args.3, args.4).await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, T4, T5, R> HandlerBase<(T0, T1, T2, T3, T4, T5)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3, T4, T5) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3, T4, T5)) -> Self::Output {
        (self)(args.0, args.1, args.2, args.3, args.4, args.5).await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, T4, T5, T6, R> HandlerBase<(T0, T1, T2, T3, T4, T5, T6)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3, T4, T5, T6) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3, T4, T5, T6)) -> Self::Output {
        (self)(args.0, args.1, args.2, args.3, args.4, args.5, args.6).await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, T4, T5, T6, T7, R> HandlerBase<(T0, T1, T2, T3, T4, T5, T6, T7)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3, T4, T5, T6, T7) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3, T4, T5, T6, T7)) -> Self::Output {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7,
        )
        .await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, T4, T5, T6, T7, T8, R> HandlerBase<(T0, T1, T2, T3, T4, T5, T6, T7, T8)>
    for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
    T8: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3, T4, T5, T6, T7, T8)) -> Self::Output {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
        )
        .await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, R>
    HandlerBase<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
    T8: 'static,
    T9: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)) -> Self::Output {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
        )
        .await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, R>
    HandlerBase<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
    T8: 'static,
    T9: 'static,
    T10: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)) -> Self::Output {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9, args.10,
        )
        .await
    }
}

#[async_trait(?Send)]
impl<Func, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, R>
    HandlerBase<(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)> for Func
where
    T0: 'static,
    T1: 'static,
    T2: 'static,
    T3: 'static,
    T4: 'static,
    T5: 'static,
    T6: 'static,
    T7: 'static,
    T8: 'static,
    T9: 'static,
    T10: 'static,
    T11: 'static,
    Func: Clone + 'static + Fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) -> R,
    R: Future + 'static,
    R::Output: Into<Response>,
{
    type Output = R::Output;

    #[inline]
    async fn call(&self, args: (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)) -> Self::Output {
        (self)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11,
        )
        .await
    }
}

// macro_rules! tuple ({$(($n:tt, $T:ident)),+} => {
//     // // Issue: https://github.com/dtolnay/async-trait/issues/46
//     // #[async_trait(?Send)]
//     // impl<Func, $($T,)+ R> HandlerBase<($($T,)+)> for Func
//     // where
//     //     $($T: 'static,)+
//     //     Func: Clone + 'static + Fn($($T,)+) -> R,
//     //     R: Future + 'static,
//     //     R::Output: Into<Response>,
//     // {
//     //     type Output = R::Output;

//     //     async fn call(&self, args: ($($T,)+)) -> Self::Output {
//     //         (self)($(args.$n,)+).await
//     //     }
//     // }
// });

// #[rustfmt::skip]
// mod m {
//     use super::*;

// tuple!((0, A));
// tuple!((0, A), (1, B));
// tuple!((0, A), (1, B), (2, C));
// tuple!((0, A), (1, B), (2, C), (3, D));
// tuple!((0, A), (1, B), (2, C), (3, D), (4, E));
// tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F));
// tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G));
// tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H));
// tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I));
// tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I), (9, J));
// }

#[cfg(test)]
mod test {
    use crate::*;
    use anyhow::anyhow;
    use async_trait::async_trait;
    use futures::executor::block_on;

    #[test]
    fn handler() {
        #[derive(Debug, PartialEq)]
        struct Info {
            hello: String,
        }

        #[async_trait(?Send)]
        impl FromContext for Info {
            type Error = Error;

            async fn from_context(_: &Context) -> Result<Info, Self::Error> {
                Ok(Info {
                    hello: "world".to_owned(),
                })
            }
        }

        #[derive(Debug, PartialEq)]
        struct User {
            id: usize,
        }

        #[async_trait(?Send)]
        impl FromContext for User {
            type Error = Error;

            async fn from_context(_: &Context) -> Result<User, Self::Error> {
                Ok(User { id: 0 })
                // Err(anyhow!("User Error"))
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
                F: HandlerBase<Args>,
                F::Output: Into<Response>,
                Args: FromContext + 'static,
                Args::Error: Into<Response>,
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

            #[async_trait(?Send)]
            impl FromContext for usize {
                type Error = Error;

                async fn from_context(_: &Context) -> Result<Self, Self::Error> {
                    Ok(0)
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
