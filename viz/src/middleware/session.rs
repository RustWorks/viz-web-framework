//! Session

use std::{convert::TryInto, future::Future, pin::Pin, sync::Arc, time::Duration};

use viz_core::{
    types::Cookie,
    types::{CookieContextExt, State},
    Context, Middleware, Response, Result,
};

use viz_utils::log;

pub use sessions::*;

/// Session Middleware
#[derive(Debug)]
pub struct SessionMiddleware {
    /// Session's `Config`
    config: Arc<Config>,
}

impl SessionMiddleware {
    /// Create a new session middleware
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        log::trace!("Session Middleware");

        let id = cx
            .cookie(&self.config.cookie.name)
            .map(|c| c.value().to_string())
            .filter(|s| self.config.verify(s))
            .unwrap_or_else(|| self.config.generate());

        let session = Session::new(&id, 0, self.config.clone());

        if let Some(d) = self.config.get(&id).await? {
            session.set_data(d)?;
        }

        cx.extensions_mut().insert(State::new(session));

        let res = cx.next().await?;

        let session = cx.session();

        let session_status = session.status();
        if session_status > 0 {
            let CookieOptions {
                name,
                max_age,
                domain,
                path,
                secure,
                http_only,
                same_site,
            } = self.config.cookie();

            let mut cookie = Cookie::new(name, session.id()?);

            if let Some(domain) = domain {
                cookie.set_domain(domain);
            }

            cookie.set_max_age(Some(
                if session_status == 3 {
                    Duration::new(0, 0)
                } else {
                    *max_age
                }
                .try_into()?,
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

impl<'a> Middleware<'a, Context> for SessionMiddleware {
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
        self.extensions().get::<State<Session>>().unwrap()
    }
}
