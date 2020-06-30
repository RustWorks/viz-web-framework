use std::future::Future;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};

use hyper::service::Service as HyperService;

use viz_core::{http, Context, Error, Result};

pub struct Service {}

impl<'t, Target> HyperService<&'t Target> for Service {
    type Response = Context;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut TaskContext) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, t: &'t Target) -> Self::Future {
        let fut = async move { Ok( ) };
        Box::pin(fut)
    }
}

