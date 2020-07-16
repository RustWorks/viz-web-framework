use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub use cookie::{Cookie, CookieJar, Key, PrivateJar, SignedJar};

use viz_utils::{futures::future::BoxFuture, log, thiserror::Error as ThisError};

use crate::{http, Context, Extract, Response, Result};

use crate::config::{ContextExt as _, Cookies as ConfigCookies};

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
                jar.add_original(Cookie::parse_encoded(pair.trim().to_string()).map_err(|e| {
                    log::debug!("failed to parse cookies: {}", e);
                    CookiesError::Parse
                })?)
            }
        }

        let cookies = Cookies::from((
            Key::derive_from(self.config().cookies.secret_key.as_bytes()),
            jar,
        ));

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

#[derive(Clone)]
pub struct Cookies {
    key: Key,
    inner: Arc<RwLock<CookieJar>>,
}

impl Cookies {
    pub fn read(&self) -> RwLockReadGuard<'_, CookieJar> {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, CookieJar> {
        self.inner.write().unwrap()
    }

    pub fn get(&self, name: &str) -> Option<Cookie<'_>> {
        self.read().get(name).cloned()
    }

    pub fn get_with_singed(&self, name: &str) -> Option<Cookie<'_>> {
        self.write().signed(&self.key).get(name)
    }

    pub fn get_with_private(&self, name: &str) -> Option<Cookie<'_>> {
        self.write().private(&self.key).get(name)
    }
}

impl From<(Key, CookieJar)> for Cookies {
    fn from(kc: (Key, CookieJar)) -> Self {
        Cookies {
            key: kc.0,
            inner: Arc::new(RwLock::new(kc.1)),
        }
    }
}

impl Extract for Cookies {
    type Error = CookiesError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.cookies() })
    }
}
