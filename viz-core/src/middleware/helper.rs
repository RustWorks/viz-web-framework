use std::time::Duration;

use crate::types::{Cookie, Cookies, SameSite};

/// Cookie's Options
#[derive(Debug)]
pub struct CookieOptions {
    /// Cookie's name, `viz.sid` by defaults
    pub name: &'static str,
    /// Cookie's path, `/` by defaults
    pub path: &'static str,
    /// Cookie's secure, `true` by defaults
    pub secure: bool,
    /// Cookie's http_only, `true` by defaults
    pub http_only: bool,
    /// Cookie's maximum age, `24H` by defaults
    pub max_age: Option<Duration>,
    /// Cookie's domain
    pub domain: Option<&'static str>,
    /// Cookie's same_site, `Lax` by defaults
    pub same_site: Option<SameSite>,
}

impl CookieOptions {
    pub const MAX_AGE: u64 = 3600 * 24;

    /// Creates new `CookieOptions`
    pub fn new(name: &'static str) -> Self {
        Self::default().name(name)
    }

    /// Creates new `CookieOptions` with `name`
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Creates new `CookieOptions` with `max_age`
    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.max_age.replace(max_age);
        self
    }

    /// Creates new `CookieOptions` with `domain`
    pub fn domain(mut self, domain: &'static str) -> Self {
        self.domain.replace(domain);
        self
    }

    /// Creates new `CookieOptions` with `path`
    pub fn path(mut self, path: &'static str) -> Self {
        self.path = path;
        self
    }

    /// Creates new `CookieOptions` with `secure`
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Creates new `CookieOptions` with `http_only`
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Creates new `CookieOptions` with `same_site`
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site.replace(same_site);
        self
    }

    pub fn into_cookie(&self, value: impl Into<String>) -> Cookie<'_> {
        let mut cookie = Cookie::new(self.name, value.into());

        cookie.set_path(self.path);
        cookie.set_secure(self.secure);
        cookie.set_http_only(self.http_only);
        cookie.set_same_site(self.same_site);

        if let Some(domain) = self.domain {
            cookie.set_domain(domain);
        }
        if let Some(max_age) = self.max_age {
            cookie.set_max_age(
                libcookie::time::Duration::try_from(max_age)
                    .expect("cant convert std Duration into time::Duration"),
            );
        }

        cookie
    }
}

impl Default for CookieOptions {
    fn default() -> Self {
        Self {
            domain: None,
            secure: true,
            http_only: true,
            path: "/".into(),
            name: "viz.sid".into(),
            same_site: Some(SameSite::Lax),
            max_age: Some(Duration::from_secs(Self::MAX_AGE)),
        }
    }
}

#[cfg(not(any(feature = "cookie-signed", feature = "cookie-private")))]
pub trait Cookieable {
    fn cookie(&self) -> &CookieOptions;

    fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.get(self.cookie().name)
    }

    fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        cookies.remove(self.cookie().name)
    }

    fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: impl Into<String>) {
        cookies.add(self.cookie().into_cookie(value))
    }
}

#[cfg(all(feature = "cookie-signed", not(feature = "cookie-private")))]
pub trait Cookieable {
    fn cookie(&self) -> &CookieOptions;

    fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.signed_get(self.cookie().name)
    }

    fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        cookies.signed_remove(self.cookie().name)
    }

    fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: impl Into<String>) {
        cookies.signed_add(self.cookie().into_cookie(value))
    }
}

#[cfg(all(feature = "cookie-private", not(feature = "cookie-signed")))]
pub trait Cookieable {
    fn cookie(&self) -> &CookieOptions;

    fn get_cookie<'a>(&self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.private_get(self.cookie().name)
    }

    fn remove_cookie<'a>(&self, cookies: &'a Cookies) {
        cookies.private_remove(self.cookie().name)
    }

    fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: impl Into<String>) {
        cookies.private_add(self.cookie().into_cookie(value))
    }
}

#[cfg(all(feature = "cookie-private", not(feature = "cookie-signed")))]
pub trait Cookieable {
    fn cookie(&self) -> &CookieOptions;

    fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        panic!("Please choose a secure option, `cookie-signed` or `cookie-private`")
    }

    fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        panic!("Please choose a secure option, `cookie-signed` or `cookie-private`")
    }

    fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: impl Into<String>) {
        panic!("Please choose a secure option, `cookie-signed` or `cookie-private`")
    }
}
