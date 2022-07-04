use std::sync::{Arc, Mutex};

pub use libcookie::{Cookie, CookieJar, SameSite};

use crate::{
    async_trait, Body, FromRequest, IntoResponse, Request, RequestExt, Response, StatusCode,
    ThisError,
};

#[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
pub type CookieKey = libcookie::Key;

pub struct Cookies {
    inner: Arc<Mutex<CookieJar>>,
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    key: Option<Arc<CookieKey>>,
}

impl Clone for Cookies {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
            key: self.key.clone(),
        }
    }
}

impl Cookies {
    pub(crate) const SPLITER: char = ';';

    pub fn new(cookie_jar: CookieJar) -> Self {
        Self {
            inner: Arc::new(Mutex::new(cookie_jar)),
            #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
            key: None,
        }
    }

    pub fn jar(&self) -> &Mutex<CookieJar> {
        &self.inner
    }

    pub fn remove(&self, name: impl AsRef<str>) {
        if let Ok(mut c) = self.jar().lock() {
            c.remove(Cookie::named(name.as_ref().to_string()))
        }
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()
            .and_then(|c| c.get(name.as_ref()).cloned())
    }

    pub fn add(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.add(cookie.into_owned())
        }
    }

    pub fn add_original(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.add_original(cookie.into_owned())
        }
    }

    pub fn reset_delta(&self) {
        if let Ok(mut c) = self.jar().lock() {
            c.reset_delta()
        }
    }
}

#[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
impl Cookies {
    pub fn with_key(mut self, key: Arc<CookieKey>) -> Self {
        self.key.replace(key);
        self
    }

    pub fn key(&self) -> &CookieKey {
        self.key.as_ref().expect("the `CookieKey` is required")
    }
}

#[cfg(feature = "cookie-private")]
impl Cookies {
    pub fn private_get(&self, name: impl AsRef<str>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()
            .and_then(|c| c.private(self.key()).get(name.as_ref()))
    }

    pub fn private_add(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.private_mut(self.key()).add(cookie.into_owned())
        }
    }

    pub fn private_remove(&self, name: impl AsRef<str>) {
        if let Ok(mut c) = self.jar().lock() {
            c.private_mut(self.key())
                .remove(Cookie::named(name.as_ref().to_string()))
        }
    }

    pub fn private_add_original(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.private_mut(self.key()).add_original(cookie.into_owned())
        }
    }

    pub fn private_decrypt(&self, cookie: Cookie<'_>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()?
            .private(self.key())
            .decrypt(cookie.into_owned())
    }
}

#[cfg(feature = "cookie-signed")]
impl Cookies {
    pub fn signed_get(&self, name: impl AsRef<str>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()
            .and_then(|c| c.signed(self.key()).get(name.as_ref()))
    }

    pub fn signed_add(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.signed_mut(self.key()).add(cookie.into_owned())
        }
    }

    pub fn signed_remove(&self, name: impl AsRef<str>) {
        if let Ok(mut c) = self.jar().lock() {
            c.signed_mut(self.key())
                .remove(Cookie::named(name.as_ref().to_string()))
        }
    }

    pub fn signed_add_original(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.signed_mut(self.key()).add_original(cookie.into_owned())
        }
    }

    pub fn signed_verify(&self, cookie: Cookie<'_>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()?
            .signed(self.key())
            .verify(cookie.into_owned())
    }
}

#[derive(ThisError, Debug)]
pub enum CookiesError {
    /// Failed to read cookies
    #[error("failed to read cookies")]
    Read,
    /// Failed to parse cookies
    #[error("failed to parse cookies")]
    Parse,
}

impl IntoResponse for CookiesError {
    fn into_response(self) -> Response<Body> {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

#[async_trait]
impl<'a> FromRequest for Cookies {
    type Error = CookiesError;

    async fn extract(req: &mut Request<Body>) -> Result<Self, Self::Error> {
        req.cookies()
    }
}
