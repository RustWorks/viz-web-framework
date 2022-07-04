use crate::{async_trait, Body, Handler, Request, Response, Result};

pub type BoxHandler<I = Request<Body>, O = Result<Response<Body>>> =
    Box<dyn Handler<I, Output = O>>;

impl Clone for BoxHandler {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

#[async_trait]
impl Handler<Request<Body>> for BoxHandler {
    type Output = Result<Response<Body>>;

    async fn call(&self, req: Request<Body>) -> Self::Output {
        self.as_ref().call(req).await
    }
}
