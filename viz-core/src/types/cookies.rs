use std::{
    fmt,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use viz_utils::{futures::future::BoxFuture, thiserror::Error as ThisError, tracing};

use crate::{
    http,
    types::{Cookie, CookieJar, Key},
    Context, Extract, Response, Result,
};

/// Cookies Error
#[derive(ThisError, Debug, PartialEq)]
pub enum CookiesError {
    /// Failed to read cookies
    #[error("failed to read cookies")]
    Read,
    /// Failed to parse cookies
    #[error("failed to parse cookies")]
    Parse,
}

impl Into<Response> for CookiesError {
    fn into(self) -> Response {
        (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into()
    }
}

/// Extract typed information from the request's cookies
#[derive(Clone)]
pub struct Cookies {
    inner: Arc<(Key, RwLock<CookieJar>)>,
}

impl Cookies {
    fn key(&self) -> &Key {
        &self.inner.0
    }

    fn jar(&self) -> &RwLock<CookieJar> {
        &self.inner.1
    }

    /// Reads the CookieJar
    pub fn read(&self) -> RwLockReadGuard<'_, CookieJar> {
        self.jar().read().unwrap()
    }

    /// Writes the CookieJar
    pub fn write(&self) -> RwLockWriteGuard<'_, CookieJar> {
        self.jar().write().unwrap()
    }

    /// Gets a cookie by name
    pub fn get(&self, name: &str) -> Option<Cookie<'_>> {
        self.read().get(name).cloned()
    }

    /// Adds a cookie
    pub fn add(&self, cookie: Cookie<'_>) {
        self.write().add(cookie.into_owned())
    }

    /// Gets a signed cookie by name
    pub fn get_with_singed(&self, name: &str) -> Option<Cookie<'_>> {
        self.write().signed(self.key()).get(name)
    }

    /// Adds a signed cookie
    pub fn add_with_singed(&self, cookie: Cookie<'_>) {
        self.write().signed_mut(self.key()).add(cookie.into_owned())
    }

    /// Gets a private cookie by name
    pub fn get_with_private(&self, name: &str) -> Option<Cookie<'_>> {
        self.write().private(self.key()).get(name)
    }

    /// Adds a private cookie
    pub fn add_with_private(&self, cookie: Cookie<'_>) {
        self.write().private_mut(self.key()).add(cookie.into_owned())
    }
}

impl fmt::Debug for Cookies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cookies")
            .field("key", &self.key().master())
            .field("jar", &self.jar())
            .finish()
    }
}

impl From<(Key, CookieJar)> for Cookies {
    fn from(kc: (Key, CookieJar)) -> Self {
        Cookies { inner: Arc::new((kc.0, RwLock::new(kc.1))) }
    }
}

impl Extract for Cookies {
    type Error = CookiesError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.cookies() })
    }
}

impl Context {
    /// Gets cookies.
    pub fn cookies(&mut self) -> Result<Cookies, CookiesError> {
        if let Some(cookies) = self.extensions().get::<Cookies>().cloned() {
            return Ok(cookies);
        }

        let mut jar = CookieJar::new();

        if let Some(raw_cookie) = self.header(http::header::COOKIE) {
            for pair in raw_cookie
                .to_str()
                .map_err(|e| {
                    tracing::debug!("failed to extract cookies: {}", e);
                    CookiesError::Read
                })?
                .split(';')
            {
                jar.add_original(Cookie::parse_encoded(pair.trim().to_string()).map_err(|e| {
                    tracing::debug!("failed to parse cookies: {}", e);
                    CookiesError::Parse
                })?)
            }
        }

        let cookies = Cookies::from((Key::from(self.config().cookies.secret_key.as_bytes()), jar));

        self.extensions_mut().insert::<Cookies>(cookies.clone());

        Ok(cookies)
    }

    /// Gets single cookie by name.
    pub fn cookie(&self, name: &str) -> Option<Cookie<'_>> {
        self.extensions().get::<Cookies>()?.get(name)
    }
}
