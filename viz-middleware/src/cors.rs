//! CORS
//! https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS

use std::{
    collections::HashSet, convert::TryFrom, future::Future, hash::Hash, pin::Pin, str::FromStr,
    time::Duration,
};

use viz_core::{
    http::{
        header::{
            HeaderName, HeaderValue, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD,
            ORIGIN,
        },
        headers::{
            AccessControlAllowHeaders, AccessControlAllowMethods, AccessControlExposeHeaders,
            HeaderMap, HeaderMapExt, Origin,
        },
        Method, StatusCode,
    },
    Context, Middleware, Response, Result,
};

use viz_utils::{log, thiserror::Error as ThisError};

/// Cors Error
#[derive(ThisError, Debug)]
pub enum CorsError {
    /// Origin not allowd
    #[error("origin not allowed")]
    OriginNotAllowed,
    /// Request Method not allowed
    #[error("request-method not allowed")]
    MethodNotAllowed,
    /// Header not allowed
    #[error("header not allowed")]
    HeaderNotAllowed,
}

impl Into<Response> for CorsError {
    fn into(self) -> Response {
        (StatusCode::FORBIDDEN, format!("CORS request forbidden: {}", self.to_string())).into()
    }
}

/// CORS Middleware
#[derive(Debug)]
pub struct CorsMiddleware {
    allow_credentials: bool,
    allow_headers: HashSet<HeaderName>,
    allow_methods: HashSet<Method>,
    exposed_headers: HashSet<HeaderName>,
    allow_origins: Option<HashSet<HeaderValue>>,
    max_age: Option<u64>,
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self {
            allow_credentials: false,
            allow_headers: HashSet::new(),
            allow_methods: to_hash_set(&["GET", "POST", "HEAD", "PUT", "DELETE", "PATCH"]),
            allow_origins: Some(to_hash_set(&["*"])),
            exposed_headers: to_hash_set(&["*"]),
            max_age: Some(0),
        }
    }
}

impl CorsMiddleware {
    /// Adds a method to the existing list of allowed request methods.
    pub fn allow_method<M>(mut self, method: M) -> Self
    where
        Method: TryFrom<M>,
    {
        if let Ok(method) = TryFrom::try_from(method) {
            self.allow_methods.insert(method);
        }
        self
    }

    /// Adds multiple methods to the existing list of allowed request methods.
    pub fn allow_methods<I>(mut self, methods: I) -> Self
    where
        I: IntoIterator,
        Method: TryFrom<I::Item>,
    {
        self.allow_methods.extend(to_headers_iter(methods));
        self
    }

    /// Adds a header to the list of allowed request headers.
    pub fn allow_header<H>(mut self, header: H) -> Self
    where
        HeaderName: TryFrom<H>,
    {
        if let Ok(header) = TryFrom::try_from(header) {
            self.allow_headers.insert(header);
        }
        self
    }

    /// Adds multiple headers to the list of allowed request headers.
    pub fn allow_headers<H>(mut self, headers: H) -> Self
    where
        H: IntoIterator,
        HeaderName: TryFrom<H::Item>,
    {
        self.allow_headers.extend(to_headers_iter(headers));
        self
    }

    /// Adds a header to the list of exposed headers.
    pub fn expose_header<H>(mut self, header: H) -> Self
    where
        HeaderName: TryFrom<H>,
    {
        if let Ok(header) = TryFrom::try_from(header) {
            self.exposed_headers.insert(header);
        }
        self
    }

    /// Adds multiple headers to the list of exposed headers.
    pub fn expose_headers<I>(mut self, headers: I) -> Self
    where
        I: IntoIterator,
        HeaderName: TryFrom<I::Item>,
    {
        self.exposed_headers.extend(to_headers_iter(headers));
        self
    }

    /// Sets the `Access-Control-Max-Age` header.
    pub fn max_age(mut self, seconds: Duration) -> Self {
        self.max_age = Some(seconds.as_secs());
        self
    }

    /// Sets that *any* `Origin` header is allowed.
    pub fn allow_any_origin(mut self) -> Self {
        self.allow_origins = None;
        self
    }

    /// Add an origin to the existing list of allowed `Origin`s.
    pub fn allow_origin() {}

    /// Add multiple origins to the existing list of allowed `Origin`s.
    pub fn allow_origins<I>(mut self, origins: I) -> Self
    where
        I: IntoIterator,
        I::Item: IntoOrigin,
    {
        let iter = origins.into_iter().map(IntoOrigin::into_origin).map(|origin| {
            origin.to_string().parse().expect("Origin is always a valid HeaderValue")
        });

        self.allow_origins.get_or_insert_with(HashSet::new).extend(iter);

        self
    }

    fn is_origin_allowed(&self, origin: &HeaderValue) -> bool {
        if let Some(ref allowed) = self.allow_origins {
            allowed.contains(origin)
        } else {
            true
        }
    }

    fn is_header_allowed(&self, header: &str) -> bool {
        HeaderName::from_bytes(header.as_bytes())
            .map(|header| self.allow_headers.contains(&header))
            .unwrap_or(false)
    }

    fn is_method_allowed(&self, method: &HeaderValue) -> bool {
        Method::from_bytes(method.as_bytes())
            .map(|method| self.allow_methods.contains(&method))
            .unwrap_or(false)
    }

    fn append_preflight_headers(&self, origin: HeaderValue, headers: &mut HeaderMap) {
        self.append_common_headers(origin, headers);

        let allow_headers: AccessControlAllowHeaders = self.allow_headers.iter().cloned().collect();
        headers.typed_insert(allow_headers);

        let allow_methods: AccessControlAllowMethods = self.allow_methods.iter().cloned().collect();
        headers.typed_insert(allow_methods);

        if let Some(max_age) = self.max_age {
            headers.insert(ACCESS_CONTROL_MAX_AGE, max_age.into());
        }
    }

    fn append_common_headers(&self, origin: HeaderValue, headers: &mut HeaderMap) {
        headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin);

        if self.allow_credentials {
            headers.insert(ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
        }

        if !self.exposed_headers.is_empty() {
            let exposed_headers: AccessControlExposeHeaders =
                self.exposed_headers.iter().cloned().collect();
            headers.typed_insert(exposed_headers);
        }
    }

    async fn run(&self, cx: &mut Context) -> Result<Response> {
        log::trace!("CORS Middleware");

        match (cx.header(ORIGIN).cloned(), cx.method()) {
            (Some(origin), &Method::OPTIONS) => {
                if self.is_origin_allowed(&origin) {
                    return Ok(CorsError::OriginNotAllowed.into());
                }

                if let Some(method) = cx.header(ACCESS_CONTROL_REQUEST_METHOD) {
                    if !self.is_method_allowed(method) {
                        return Ok(CorsError::MethodNotAllowed.into());
                    }
                } else {
                    return Ok(CorsError::MethodNotAllowed.into());
                }

                if let Some(headers) = cx.header(ACCESS_CONTROL_REQUEST_HEADERS) {
                    let headers = headers.to_str().map_err(|_| CorsError::MethodNotAllowed)?;
                    for header in headers.split(',') {
                        if !self.is_header_allowed(header.trim()) {
                            return Err(CorsError::HeaderNotAllowed.into());
                        }
                    }
                }

                let mut res = cx.next().await?;

                self.append_preflight_headers(origin, res.headers_mut());

                Ok(res)
            }
            (Some(origin), _) => {
                if self.is_origin_allowed(&origin) {
                    return Ok(CorsError::OriginNotAllowed.into());
                }

                let mut res = cx.next().await?;

                self.append_common_headers(origin, res.headers_mut());

                Ok(res)
            }
            (None, _) => cx.next().await,
        }
    }
}

impl<'a> Middleware<'a, Context> for CorsMiddleware {
    type Output = Result<Response>;

    #[must_use]
    fn call(
        &'a self,
        cx: &'a mut Context,
    ) -> Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>> {
        Box::pin(self.run(cx))
    }
}

fn to_hash_set<T>(list: &[&str]) -> HashSet<T>
where
    T: FromStr + Hash + Eq,
{
    list.iter().map(|m| T::from_str(m).ok()).filter(|m| m.is_some()).map(|m| m.unwrap()).collect()
}

fn to_headers_iter<H, T>(headers: H) -> impl IntoIterator<Item = T>
where
    H: IntoIterator,
    T: TryFrom<H::Item>,
{
    headers
        .into_iter()
        .map(|m| TryFrom::try_from(m).ok())
        .filter(|m| m.is_some())
        .map(|m| m.unwrap())
}

/// Into to Origin
pub trait IntoOrigin {
    /// Into a Origin
    fn into_origin(self) -> Origin;
}

impl<'a> IntoOrigin for &'a str {
    fn into_origin(self) -> Origin {
        let mut parts = self.splitn(2, "://");
        let scheme = parts.next().expect("missing scheme");
        let rest = parts.next().expect("missing scheme");

        Origin::try_from_parts(scheme, rest, None).expect("invalid Origin")
    }
}
