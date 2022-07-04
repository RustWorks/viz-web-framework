use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use crate::{
    async_trait,
    handler::Transform,
    types::{Cookie, Cookies, CookiesError, Session},
    Body, Error, Handler, IntoResponse, Request, RequestExt, Response, Result, StatusCode,
};

use super::{CookieOptions, Storage, Store, PURGED, RENEWED, UNCHANGED};

pub struct Config<S, G, V> {
    inner: Arc<Store<S, G, V>>,
    cookie: Arc<CookieOptions>,
}

impl<S, G, V> Clone for Config<S, G, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            cookie: self.cookie.clone(),
        }
    }
}

impl<S, G, V> Config<S, G, V> {
    pub fn new(inner: Store<S, G, V>, cookie: CookieOptions) -> Self {
        Self {
            inner: Arc::new(inner),
            cookie: Arc::new(cookie),
        }
    }

    pub fn store(&self) -> &Store<S, G, V> {
        &self.inner
    }

    pub fn cookie(&self) -> &CookieOptions {
        &self.cookie
    }

    pub fn ttl(&self) -> Option<Duration> {
        self.cookie().max_age
    }
}

#[cfg(not(any(feature = "cookie-signed", feature = "cookie-private")))]
impl<S, G, V> Config<S, G, V> {
    pub fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.get(self.cookie.name)
    }

    pub fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        cookies.remove(self.cookie.name)
    }

    pub fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: &str) {
        cookies.add(self.cookie().into_cookie(value))
    }
}

#[cfg(all(feature = "cookie-signed", not(feature = "cookie-private")))]
impl<S, G, V> Config<S, G, V> {
    pub fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.signed_get(self.cookie.name)
    }

    pub fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        cookies.signed_remove(self.cookie.name)
    }

    pub fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: &str) {
        cookies.signed_add(self.cookie().into_cookie(value))
    }
}

#[cfg(all(feature = "cookie-private", not(feature = "cookie-signed")))]
impl<S, G, V> Config<S, G, V> {
    pub fn get_cookie<'a>(&self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.private_get(self.cookie.name)
    }

    pub fn remove_cookie<'a>(&self, cookies: &'a Cookies) {
        cookies.private_remove(self.cookie.name)
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

impl<H, S, G, V> Transform<H> for Config<S, G, V> {
    type Output = SessionMiddleware<H, S, G, V>;

    fn transform(&self, h: H) -> Self::Output {
        SessionMiddleware {
            h,
            config: self.clone(),
        }
    }
}

pub struct SessionMiddleware<H, S, G, V> {
    h: H,
    config: Config<S, G, V>,
}

impl<H, S, G, V> Clone for SessionMiddleware<H, S, G, V>
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
impl<H, O, S, G, V> Handler<Request<Body>> for SessionMiddleware<H, S, G, V>
where
    O: IntoResponse,
    H: Handler<Request<Body>, Output = Result<O>> + Clone,
    S: Storage + 'static,
    G: Fn() -> String + Send + Sync + 'static,
    V: Fn(&str) -> bool + Send + Sync + 'static,
{
    type Output = Result<Response<Body>>;

    async fn call(&self, mut req: Request<Body>) -> Self::Output {
        let cookies = req.cookies().map_err(responder_cookie_error)?;
        let cookie = self.config.get_cookie(&cookies);

        let mut session_id = cookie.map(get_cookie_value);
        let data = match &session_id {
            Some(sid) if (self.config.store().verify)(sid) => {
                self.config.store().get(sid).await.map_err(report_error)?
            }
            _ => None,
        };
        if data.is_none() && session_id.is_some() {
            session_id.take();
        }
        let session = Session::new(data.unwrap_or_default());
        req.extensions_mut().insert(session.clone());

        let resp = self.h.call(req).await.map(IntoResponse::into_response);

        let status = session.status().load(Ordering::Acquire);

        if status == UNCHANGED {
            return resp;
        }

        if status == PURGED {
            if let Some(sid) = &session_id {
                self.config
                    .store()
                    .remove(sid)
                    .await
                    .map_err(report_error)?;
                self.config.remove_cookie(&cookies);
            }

            return resp;
        }

        if status == RENEWED {
            if let Some(sid) = &session_id.take() {
                self.config
                    .store()
                    .remove(sid)
                    .await
                    .map_err(report_error)?;
            }
        }

        let sid = match session_id {
            Some(sid) => sid,
            None => {
                let sid = (self.config.store().generate)();
                self.config.set_cookie(&cookies, &sid);
                sid
            }
        };

        self.config
            .store()
            .set(
                &sid,
                session.data()?,
                &self.config.ttl().unwrap_or_else(max_age),
            )
            .await
            .map_err(report_error)?;

        resp
    }
}

fn max_age() -> Duration {
    Duration::from_secs(CookieOptions::MAX_AGE)
}

fn get_cookie_value(c: Cookie<'_>) -> String {
    c.value().to_string()
}

fn responder_cookie_error(e: CookiesError) -> Error {
    Error::Responder(e.into_response())
}

fn report_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> Error {
    Error::Report(
        Box::new(e),
        StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    )
}
