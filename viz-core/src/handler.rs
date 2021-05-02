use std::{future::Future, marker::PhantomData};

use viz_utils::futures::future::BoxFuture;

use crate::{Context, Extract, Middleware, Response, Result};

pub trait HandlerBase<Args>: Clone + 'static {
    type Output: Into<Response>;
    type Future: Future<Output = Self::Output> + Send + 'static;

    fn call(&self, args: Args) -> Self::Future;
}

pub trait Handler: Send + Sync + 'static {
    fn call<'a>(&'a self, _: &'a mut Context) -> BoxFuture<'a, Result<Response>>;

    fn clone_handler(&self) -> Box<dyn Handler>;
}

impl Handler for Box<dyn Handler> {
    fn call<'a>(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Result<Response>> {
        (**self).call(cx)
    }

    fn clone_handler(&self) -> Box<dyn Handler> {
        (**self).clone_handler()
    }
}

pub struct HandlerWrapper<F, T> {
    pub(crate) f: F,
    _t: PhantomData<T>,
}

impl<F, T> HandlerWrapper<F, T> {
    pub fn new(f: F) -> Self {
        Self { f, _t: PhantomData }
    }
}

impl<F, T> Handler for HandlerWrapper<F, T>
where
    F: HandlerBase<T> + Send + Sync,
    T: Extract + Send + Sync + 'static,
    T::Error: Into<Response> + Send,
{
    #[inline]
    fn call<'a>(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Result<Response>> {
        Box::pin(async move {
            Ok(match T::extract(cx).await {
                Ok(args) => self.f.call(args).await.into(),
                Err(e) => e.into(),
            })
        })
    }

    #[inline]
    fn clone_handler(&self) -> Box<dyn Handler> {
        Box::new(Self { f: self.f.clone(), _t: PhantomData })
    }
}

impl<'a, F, T> Middleware<'a, Context> for HandlerWrapper<F, T>
where
    F: HandlerBase<T> + Send + Sync + 'static,
    T: Extract + Send + Sync + 'static,
    T::Error: Into<Response> + Send,
{
    type Output = Result<Response>;

    #[inline]
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        Handler::call(self, cx)
    }
}

pub trait HandlerCamp<'h, Args>: Clone + 'static {
    type Output: Into<Response>;
    type Future: Future<Output = Self::Output> + Send + 'h;

    fn call(&'h self, cx: &'h mut Context, args: Args) -> Self::Future;
}

pub struct HandlerSuper<F, T> {
    pub(crate) f: F,
    _t: PhantomData<T>,
}

impl<F, T> HandlerSuper<F, T> {
    pub fn new(f: F) -> Self {
        Self { f, _t: PhantomData }
    }
}

impl<F, T> Handler for HandlerSuper<F, T>
where
    F: for<'h> HandlerCamp<'h, T> + Send + Sync,
    T: Extract + Send + Sync + 'static,
    T::Error: Into<Response> + Send,
{
    #[inline]
    fn call<'a>(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Result<Response>> {
        Box::pin(async move {
            Ok(match T::extract(cx).await {
                Ok(args) => self.f.call(cx, args).await.into(),
                Err(e) => e.into(),
            })
        })
    }

    #[inline]
    fn clone_handler(&self) -> Box<dyn Handler> {
        Box::new(Self { f: self.f.clone(), _t: PhantomData })
    }
}

impl<'a, F, T> Middleware<'a, Context> for HandlerSuper<F, T>
where
    F: for<'h> HandlerCamp<'h, T> + Send + Sync + 'static,
    T: Extract + Send + Sync + 'static,
    T::Error: Into<Response> + Send,
{
    type Output = Result<Response>;

    #[inline]
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        Handler::call(self, cx)
    }
}

#[cfg(test)]
mod test {
    use futures_executor::block_on;

    use viz_utils::{anyhow::anyhow, futures::future::BoxFuture};

    use crate::*;

    #[allow(unstable_name_collisions)]
    #[test]
    fn handler() {
        #[derive(Debug, PartialEq)]
        struct Info {
            hello: String,
        }

        impl Extract for Info {
            type Error = Error;

            fn extract<'a>(_: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
                Box::pin(async { Ok(Info { hello: "world".to_owned() }) })
            }
        }

        #[derive(Debug, PartialEq)]
        struct User {
            id: usize,
        }

        impl Extract for User {
            type Error = Error;

            fn extract<'a>(_: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
                Box::pin(async {
                    // Err(anyhow!("User Error"))
                    Ok(User { id: 0 })
                })
            }
        }

        /// Helper method for extractors testing
        pub async fn extract<T: Extract>(cx: &mut Context) -> Result<T, T::Error> {
            T::extract(cx).await
        }

        block_on(async move {
            let mut cx = Context::from(http::Request::new("hello".into()));

            let r_0 = extract::<Info>(&mut cx).await.unwrap();

            assert_eq!(r_0, Info { hello: "world".to_owned() });

            let r_1 = cx.extract::<Info>().await.unwrap();

            assert_eq!(r_1, Info { hello: "world".to_owned() });

            let r = extract::<Option<Info>>(&mut cx).await.unwrap();

            assert_eq!(r, Some(Info { hello: "world".to_owned() }));

            let r0 = extract::<(Info, User)>(&mut cx).await.unwrap();
            let r1 = extract::<(User, Info)>(&mut cx).await.unwrap();
            let r2 = cx.extract::<(User, Info)>().await.unwrap();

            assert_eq!(r0.0, r1.1);
            assert_eq!(r0.1, r1.0);
            assert_eq!(r0.0, r2.1);
            assert_eq!(r1.0, r2.0);

            fn make_handler<F, Args>(handler: F) -> Box<dyn Handler>
            where
                F: HandlerBase<Args> + Send + Sync + 'static,
                Args: Extract + Send + Sync + 'static,
                Args::Error: Into<Response> + Send,
            {
                Box::new(HandlerWrapper::new(handler))
            }

            async fn a() -> Response {
                Response::new()
            }

            let h = make_handler(a);
            let mut cx = Context::from(http::Request::new("hello".into()));
            let r = h.call(&mut cx).await;
            assert!(r.is_ok());

            async fn b(i: Info, u: User) -> Result<Response> {
                assert_eq!(i, Info { hello: "world".to_owned() });
                assert_eq!(u, User { id: 0 });
                Ok(Response::new())
            }
            let h = make_handler(b);
            let mut cx = Context::from(http::Request::new("hello".into()));
            let r = h.call(&mut cx).await;
            assert!(r.is_ok());

            let c = || async { Response::new() };
            let h = make_handler(c);
            let mut cx = Context::from(http::Request::new("hello".into()));
            let r = h.call(&mut cx).await;
            assert!(r.is_ok());

            let d = || Box::pin(async { anyhow!("throws error and converts to response") });
            let h = make_handler(d);
            let mut cx = Context::from(http::Request::new("hello".into()));
            let hh = h.clone_handler();
            let r = h.call(&mut cx).await;
            let mut cx = Context::from(http::Request::new("hello".into()));
            let r0 = hh.call(&mut cx).await;
            assert_eq!(r.is_ok(), r0.is_ok());

            async fn e(u: User) -> Result<Response> {
                assert_eq!(u, User { id: 0 });
                Ok(Response::new())
            }
            let h = make_handler(e);
            let mut cx = Context::from(http::Request::new("hello".into()));
            let r = h.call(&mut cx).await;
            assert!(r.is_ok());

            impl Extract for usize {
                type Error = Error;

                fn extract<'a>(_: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
                    Box::pin(async { Ok(0) })
                }
            }

            async fn f(u: User, n: usize) -> &'static str {
                assert_eq!(u, User { id: 0 });
                assert_eq!(n, 0);
                "Hello world"
            }
            let h = make_handler(f);
            let mut cx = Context::from(http::Request::new("hello".into()));
            let r = h.call(&mut cx).await;
            assert!(r.is_ok());
            let r = Handler::call(&h.clone_handler(), &mut cx).await;
            assert!(r.is_ok());

            assert_eq!(f.call(cx.extract().await.unwrap()).await, "Hello world");
            assert_eq!(
                f.call(cx.extract().await.unwrap()).await,
                HandlerBase::call(&f, cx.extract().await.unwrap()).await
            );

            let mut cx = Context::from(http::Request::new("hello".into()));
            let r = Handler::call(&h, &mut cx).await;
            assert!(r.is_ok());
            let r = Handler::call(&h.clone_handler(), &mut cx).await;
            assert!(r.is_ok());
        });
    }

    #[test]
    fn handler_with_context() {
        block_on(async move {
            fn make_middle(
                f: impl for<'a> Middleware<'a, Context, Output = Result<Response>>,
            ) -> Box<DynMiddleware> {
                Box::new(f)
            }

            #[derive(Debug, PartialEq)]
            struct Language {
                name: String,
            }

            impl Extract for Language {
                type Error = Error;

                fn extract<'a>(_: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
                    Box::pin(async { Ok(Language { name: "rust".to_owned() }) })
                }
            }

            async fn hello(lang: Language) -> &'static str {
                assert_eq!(lang, Language { name: "rust".to_owned() });

                "Hello"
            }

            async fn world(cx: &mut Context, lang: Language) -> &'static str {
                assert_eq!(cx.method(), "GET");
                assert_eq!(lang, Language { name: "rust".to_owned() });

                "World"
            }

            let mut cx = Context::from(http::Request::new("hello".into()));

            let f: Box<dyn Handler> = Box::new(HandlerWrapper::new(hello));
            let r = f.call(&mut cx).await;
            assert!(r.is_ok());

            let f: Box<DynMiddleware> = make_middle(HandlerWrapper::new(hello));
            let r = f.call(&mut cx).await;
            assert!(r.is_ok());

            let f: Box<DynMiddleware> = make_middle(HandlerSuper::new(world));
            let r = f.call(&mut cx).await;
            assert!(r.is_ok());
        });
    }
}
