//! Context

use std::fmt;

use crate::{http, Extract, Middlewares, Result};

/// The `Context` of an HTTP `request - response`.
pub struct Context {
    /// The request's URI
    uri: http::Uri,

    /// The request's method
    method: http::Method,

    /// The request's version
    version: http::Version,

    /// The request's headers
    headers: http::HeaderMap,

    /// The request's extensions
    extensions: http::Extensions,

    /// The request's body
    body: Option<http::Body>,

    /// The request's middleware
    middleware: Middlewares,
}

impl Context {
    /// Consumes the request to Context
    pub fn new(req: http::Request) -> Self {
        req.into()
    }

    /// Returns a reference to the associated HTTP method.
    pub fn method(&self) -> &http::Method {
        &self.method
    }

    /// Returns the associated version.
    pub fn version(&self) -> http::Version {
        self.version
    }

    /// Returns a reference to the associated URI.
    pub fn uri(&self) -> &http::Uri {
        &self.uri
    }

    /// Returns a reference to the associated path portion of the URL.
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Returns a reference to the associated query portion of the URL.
    pub fn query_str(&self) -> &str {
        if let Some(query) = self.uri().query().as_ref() {
            query
        } else {
            ""
        }
    }

    /// Returns a reference to the associated host portion of the URL.
    pub fn host(&self) -> Option<&str> {
        self.uri.host()
    }

    /// Gets content type
    pub fn mime(&self) -> Option<mime::Mime> {
        self.header(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<mime::Mime>().ok())
    }

    /// Gets content length
    pub fn len(&self) -> Option<usize> {
        self.header(http::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
    }

    /// Returns a reference to the associated header by key.
    pub fn header(&self, key: impl AsRef<str>) -> Option<&http::HeaderValue> {
        self.headers.get(key.as_ref())
    }

    /// Returns a reference to the associated header field map.
    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    /// Returns a reference to the associated extensions.
    pub fn extensions(&self) -> &http::Extensions {
        &self.extensions
    }

    /// Returns a mutable reference to the associated extensions.
    pub fn extensions_mut(&mut self) -> &mut http::Extensions {
        &mut self.extensions
    }

    /// Consumes the request, returning just the body.
    pub fn take_body(&mut self) -> Option<http::Body> {
        self.body.take()
    }

    /// Returns a reference to the associated middleware.
    pub fn middleware(&self) -> &Middlewares {
        &self.middleware
    }

    /// Returns a mutable reference to the associated middleware.
    pub fn middleware_mut(&mut self) -> &mut Middlewares {
        &mut self.middleware
    }

    /// Returns a data from the `Context` with a Extractor.
    pub async fn extract<T: Extract>(&mut self) -> Result<T, T::Error> {
        T::extract(self).await
    }

    /// Invokes the next middleware.
    pub async fn next(&mut self) -> Result {
        if let Some(m) = self.middleware.pop() {
            return m.call(self).await;
        }
        Ok(http::StatusCode::NOT_FOUND.into())
    }
}

impl From<http::Request> for Context {
    #[inline]
    fn from(req: http::Request) -> Self {
        let (::http::request::Parts { uri, method, headers, version, extensions, .. }, body) =
            req.into_parts();
        Self { uri, method, headers, version, extensions, body: Some(body), middleware: Vec::new() }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Context")
            .field("version", &self.version)
            .field("method", &self.method)
            .field("uri", &self.uri)
            .field("headers", &self.headers)
            .field("extensions", &self.extensions)
            .field("body", &self.body.is_some())
            .field("middleware", &self.middleware.len())
            .finish()
    }
}
