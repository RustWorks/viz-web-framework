//! Session

use std::{convert::TryInto, future::Future, pin::Pin, sync::Arc, time::Duration};

use viz_core::{
    types::{Cookie, State},
    Context, Middleware, Response, Result,
};

use viz_utils::tracing;

pub use sessions::*;

/// Session Middleware
#[derive(Debug)]
pub struct Sessions<S: Storage> {
    /// Session's `Config`
    config: Arc<Config<S>>,
}

impl<S: Storage> Sessions<S> {
    /// Create a new session middleware
    pub fn new(config: Config<S>) -> Self {
        Self { config: Arc::new(config) }
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        let id = cx
            .cookie(&self.config.cookie.name)
            .map(|c| c.value().to_string())
            .filter(|v| self.config.verify(v))
            .unwrap_or_else(|| self.config.generate());

        let session = Session::new(&id, 0, self.config.clone());

        if let Some(d) = self.config.get(&id).await? {
            session.set_data(d)?;
        }

        cx.extensions_mut().insert(State::new(session));

        let res = cx.next().await?;

        let session = cx.session::<S>();

        tracing::trace!(" {:?}", session);

        let session_status = session.status();
        if session_status > 0 {
            let CookieOptions { name, max_age, domain, path, secure, http_only, same_site } =
                self.config.cookie();

            let mut cookie = Cookie::new(name, session.id()?);

            if let Some(domain) = domain {
                cookie.set_domain(domain);
            }

            cookie.set_max_age(Some(
                if session_status == 3 { Duration::new(0, 0) } else { *max_age }.try_into()?,
            ));

            cookie.set_path(path);
            cookie.set_secure(*secure);
            cookie.set_http_only(*http_only);
            cookie.set_same_site(*same_site);

            cx.cookies()?.add(cookie);
        }

        Ok(res)
    }
}

impl<'a, S: Storage> Middleware<'a, Context> for Sessions<S> {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}

/// Session Ext for Context
pub trait ContextExt {
    /// Gets a session
    fn session<S: Storage>(&self) -> &Session<S>;
}

impl ContextExt for Context {
    fn session<S: Storage>(&self) -> &Session<S> {
        self.extensions().get::<State<Session<S>>>().unwrap()
    }
}
