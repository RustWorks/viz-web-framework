use crate::{async_trait, Handler, Request, Response, Result};

/// Alias the boxed Handler.
pub type BoxHandler<I = Request, O = Result<Response>> = Box<dyn Handler<I, Output = O>>;

impl Clone for BoxHandler {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

#[async_trait]
impl Handler<Request> for BoxHandler {
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        self.as_ref().call(req).await
    }
}
