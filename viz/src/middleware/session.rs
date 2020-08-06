use std::{future::Future, pin::Pin};

use viz_utils::log;

use viz_core::{http, Context, Middleware, Response, Result};

use sessions::{Session, SessionStatus, Storable};

pub struct SessionMiddleware<Store> {
    store: Store,
    from: SessionFrom,
    name: String,
    domain: String,
    secure: bool,
}

pub enum SessionFrom {
    Cookie,
    Query,
    Header,
}

impl<Store: Storable> SessionMiddleware<Store> {
    async fn run(&self, cx: &mut Context) -> Result<Response> {
        cx.next().await
    }
}

impl<'a, Store: Storable> Middleware<'a, Context> for SessionMiddleware<Store> {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}

pub trait ContextExt {
    fn session(&self) -> &Session;
}

impl ContextExt for Context {
    fn session(&self) -> &Session {
        self.extensions().get::<Session>().unwrap()
    }
}
