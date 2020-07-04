use std::sync::{Arc, RwLock};

pub use cookie::{Cookie, CookieJar};

use viz_utils::{futures::future::BoxFuture, log, thiserror::Error as ThisError};

use crate::{http, Context, Extract, Response, Result};

pub trait ContextExt {
    fn cookies(&mut self) -> Result<Cookies, CookiesError>;

    fn cookie(&self, name: &str) -> Option<Cookie<'_>>;
}

impl ContextExt for Context {
    fn cookies(&mut self) -> Result<Cookies, CookiesError> {
        if let Some(cookies) = self.extensions().get::<Cookies>() {
            return Ok(cookies.clone());
        }

        let mut jar = CookieJar::new();

        if let Some(raw_cookie) = self.header(http::header::COOKIE) {
            for pair in raw_cookie
                .to_str()
                .map_err(|e| {
                    log::debug!("failed to extract cookies: {}", e);
                    CookiesError::Read
                })?
                .split(';')
            {
                jar
                    // .signed(&key)
                    .add_original(Cookie::parse_encoded(pair.trim().to_string()).map_err(|e| {
                        log::debug!("failed to parse cookies: {}", e);
                        CookiesError::Parse
                    })?);
            }
        }

        let cookies = Cookies::from(jar);

        self.extensions_mut().insert::<Cookies>(cookies.clone());

        Ok(cookies)
    }

    fn cookie(&self, name: &str) -> Option<Cookie<'_>> {
        self.extensions().get::<Cookies>()?.get(name)
    }
}

#[derive(ThisError, Debug, PartialEq)]
pub enum CookiesError {
    #[error("failed to read cookies")]
    Read,
    #[error("failed to parse cookies")]
    Parse,
}

impl Into<Response> for CookiesError {
    fn into(self) -> Response {
        (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into()
    }
}

#[derive(Clone, Debug)]
pub struct Cookies(Arc<RwLock<CookieJar>>);

impl Cookies {
    pub fn get(&self, name: &str) -> Option<Cookie<'_>> {
        self.0.read().unwrap().get(name).cloned()
    }
}

impl From<CookieJar> for Cookies {
    fn from(c: CookieJar) -> Self {
        Cookies(Arc::new(RwLock::new(c)))
    }
}

impl Extract for Cookies {
    type Error = CookiesError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.cookies() })
    }
}
