use std::{future::Future, pin::Pin};

use viz_core::{http, Context, Middleware, Response, Result};
use viz_utils::tracing;

/// Recover Middleware
#[derive(Debug, Default)]
pub struct Recover {}

impl Recover {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        match internal::CatchUnwind::new(cx.next()).await {
            Ok(res) => res,
            Err(err) => {
                tracing::error!(" {:>7?}", err);
                Ok(http::StatusCode::INTERNAL_SERVER_ERROR.into())
            }
        }
    }
}

impl<'a> Middleware<'a, Context> for Recover {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}

/// Thanks: <https://docs.rs/futures-util/latest/futures_util/future/struct.CatchUnwind.html>
mod internal {
    use std::{
        any::Any,
        future::Future,
        panic::{catch_unwind, AssertUnwindSafe},
        pin::Pin,
        task::{Context, Poll},
    };

    use pin_project_lite::pin_project;

    pin_project! {
        pub struct CatchUnwind<Fut> {
            #[pin]
            future: Fut,
        }
    }

    impl<Fut> CatchUnwind<Fut>
    where
        Fut: Future,
    {
        pub(super) fn new(future: Fut) -> Self {
            Self { future }
        }
    }

    impl<Fut> Future for CatchUnwind<Fut>
    where
        Fut: Future,
    {
        type Output = Result<Fut::Output, Box<dyn Any + Send>>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let f = self.project().future;
            catch_unwind(AssertUnwindSafe(|| f.poll(cx)))?.map(Ok)
        }
    }
}
