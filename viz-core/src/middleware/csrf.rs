use std::collections::HashSet;
use std::sync::Arc;

use crate::{
    async_trait,
    handler::Transform,
    header,
    headers::{HeaderName, HeaderValue},
    middleware::helper::CookieOptions,
    types::{Cookie, Cookies},
    Body, Error, FromRequest, Handler, IntoResponse, Method, Request, RequestExt, Response, Result,
    StatusCode,
};

struct Inner<S, G, V> {
    store: Store,
    ignored_methods: HashSet<Method>,
    cookie_options: CookieOptions,
    header: HeaderName,
    secret: S,
    generate: G,
    verify: V,
}

pub enum Store {
    Cookie,
    Session,
}

#[derive(Clone)]
pub struct CsrfToken(pub String);

#[async_trait]
impl FromRequest for CsrfToken {
    type Error = Error;

    async fn extract(req: &mut Request<Body>) -> Result<Self, Self::Error> {
        req.extensions()
            .get()
            .cloned()
            .ok_or_else(|| (StatusCode::FORBIDDEN, "Missing csrf token").into_error())
    }
}

pub struct Config<S, G, V>(Arc<Inner<S, G, V>>);

impl<S, G, V> Clone for Config<S, G, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S, G, V> Config<S, G, V> {
    pub const CSRF_TOKEN: &'static str = "x-csrf-token";

    pub fn new(
        store: Store,
        ignored_methods: HashSet<Method>,
        cookie_options: CookieOptions,
        secret: S,
        generate: G,
        verify: V,
    ) -> Self {
        Self(Arc::new(Inner {
            store,
            ignored_methods,
            cookie_options,
            secret,
            generate,
            verify,
            header: HeaderName::from_static(Self::CSRF_TOKEN),
        }))
    }

    pub fn cookie(&self) -> &CookieOptions {
        &self.0.cookie_options
    }

    pub fn get<'a>(&self, req: &'a Request<Body>) -> Result<Option<Vec<u8>>> {
        let inner = self.as_ref();
        match inner.store {
            Store::Cookie => match self
                .get_cookie(&req.cookies()?)
                .map(|c| c.value().to_string())
            {
                None => Ok(None),
                Some(raw_token) => match base64::decode_config(raw_token, base64::URL_SAFE) {
                    Ok(masked_token) => Ok(Some(unmask::<32>(masked_token))),
                    Err(_e) => {
                        Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid csrf token").into_error())
                    }
                },
            },
            Store::Session => req.session().get(inner.cookie_options.name),
        }
    }

    pub fn set<'a>(&self, req: &'a Request<Body>, token: Vec<u8>, secret: Vec<u8>) -> Result<()> {
        let inner = self.as_ref();
        match inner.store {
            Store::Cookie => {
                self.set_cookie(
                    &req.cookies()?,
                    &base64::encode_config(token, base64::URL_SAFE),
                );
                Ok(())
            }
            Store::Session => req.session().set(inner.cookie_options.name, secret),
        }
    }
}

#[cfg(not(any(feature = "cookie-signed", feature = "cookie-private")))]
impl<S, G, V> Config<S, G, V> {
    pub fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.get(self.cookie().name)
    }

    pub fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        cookies.remove(self.cookie().name)
    }

    pub fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: &str) {
        cookies.add(self.cookie().into_cookie(value))
    }
}

#[cfg(all(feature = "cookie-signed", not(feature = "cookie-private")))]
impl<S, G, V> Config<S, G, V> {
    pub fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.signed_get(self.cookie().name)
    }

    pub fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        cookies.signed_remove(self.cookie().name)
    }

    pub fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: &str) {
        cookies.signed_add(self.cookie().into_cookie(value))
    }
}

#[cfg(all(feature = "cookie-private", not(feature = "cookie-signed")))]
impl<S, G, V> Config<S, G, V> {
    pub fn get_cookie<'a>(&self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.private_get(self.cookie().name)
    }

    pub fn remove_cookie<'a>(&self, cookies: &'a Cookies) {
        cookies.private_remove(self.cookie().name)
    }

    pub fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: &str) {
        cookies.private_add(self.cookie().into_cookie(value))
    }
}

#[cfg(all(feature = "cookie-signed", feature = "cookie-private"))]
impl<S, G, V> Config<S, G, V> {
    pub fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        panic!("Please choose a secure option, `cookie-signed` or `cookie-private`")
    }

    pub fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        panic!("Please choose a secure option, `cookie-signed` or `cookie-private`")
    }

    pub fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: &str) {
        panic!("Please choose a secure option, `cookie-signed` or `cookie-private`")
    }
}

impl<S, G, V> AsRef<Inner<S, G, V>> for Config<S, G, V> {
    fn as_ref(&self) -> &Inner<S, G, V> {
        &self.0
    }
}

impl<H, S, G, V> Transform<H> for Config<S, G, V> {
    type Output = CsrfMiddleware<H, S, G, V>;

    fn transform(&self, h: H) -> Self::Output {
        CsrfMiddleware {
            h,
            config: self.clone(),
        }
    }
}

pub struct CsrfMiddleware<H, S, G, V> {
    h: H,
    config: Config<S, G, V>,
}

impl<H, S, G, V> Clone for CsrfMiddleware<H, S, G, V>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            h: self.h.clone(),
            config: self.config.clone(),
        }
    }
}

#[async_trait]
impl<H, O, S, G, V> Handler<Request<Body>> for CsrfMiddleware<H, S, G, V>
where
    O: IntoResponse,
    H: Handler<Request<Body>, Output = Result<O>> + Clone,
    S: Fn() -> Result<Vec<u8>> + Send + Sync + 'static,
    G: Fn(&Vec<u8>, Vec<u8>) -> Vec<u8> + Send + Sync + 'static,
    V: Fn(Vec<u8>, String) -> bool + Send + Sync + 'static,
{
    type Output = Result<Response<Body>>;

    async fn call(&self, mut req: Request<Body>) -> Self::Output {
        let mut secret = self.config.get(&req)?;
        let config = self.config.as_ref();

        if !config.ignored_methods.contains(req.method()) {
            let mut forbidden = true;
            if let Some(secret) = secret.take() {
                if let Some(raw_token) = req.header(&config.header) {
                    forbidden = !(config.verify)(secret, raw_token);
                }
            }
            if forbidden {
                return Err((StatusCode::FORBIDDEN, "Invalid csrf token").into_error());
            }
        }

        let otp = (config.secret)()?;
        let secret = (config.secret)()?;
        let token = (config.generate)(&secret, otp);
        req.extensions_mut()
            .insert(CsrfToken(String::from_utf8_lossy(&token).to_string()));
        self.config.set(&req, token, secret)?;

        self.h
            .call(req)
            .await
            .map(IntoResponse::into_response)
            .map(|mut res| {
                res.headers_mut()
                    .insert(header::VARY, HeaderValue::from_static("Cookie"));
                res
            })
    }
}

/// Gets random secret
pub fn secret() -> Result<Vec<u8>> {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_error())?;
    Ok(buf.to_vec())
}

/// Generates Token
pub fn generate(secret: &Vec<u8>, otp: Vec<u8>) -> Vec<u8> {
    mask(secret.to_vec(), otp)
}

/// Verifys Token with a secret
pub fn verify(secret: Vec<u8>, raw_token: String) -> bool {
    if let Ok(token) = base64::decode_config(raw_token, base64::URL_SAFE) {
        if token.len() == 64 {
            return secret == unmask::<32>(token);
        }
    }
    false
}

/// Retures masked token
fn mask(secret: Vec<u8>, mut otp: Vec<u8>) -> Vec<u8> {
    otp.extend::<Vec<u8>>(
        secret
            .iter()
            .enumerate()
            .map(|(i, t)| *t ^ otp[i])
            .collect(),
    );
    otp
}

/// Returens secret
fn unmask<const N: usize>(mut token: Vec<u8>) -> Vec<u8> {
    // encrypted_csrf_token
    let mut secret = token.split_off(N);
    // one_time_pad
    secret
        .iter_mut()
        .enumerate()
        .for_each(|(i, t)| *t ^= token[i]);
    secret
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn builder() {
        Config::new(
            Store::Cookie,
            [Method::GET, Method::HEAD, Method::OPTIONS, Method::TRACE].into(),
            CookieOptions::new("_csrf").max_age(Duration::from_secs(3600 * 24)),
            secret,
            generate,
            verify,
        );
    }
}
