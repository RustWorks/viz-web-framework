use std::str::FromStr;

use crate::http;
use crate::Extract;
use crate::Middlewares;
use crate::Params;
use crate::Response;
use crate::Result;

pub struct Context {
    method: http::Method,

    uri: http::Uri,

    version: http::Version,

    headers: http::HeaderMap,

    extensions: http::Extensions,

    body: Option<http::Body>,

    middleware: Middlewares,
}

impl Context {
    pub fn new(req: http::Request) -> Self {
        req.into()
    }

    pub fn method(&self) -> &http::Method {
        &self.method
    }

    pub fn version(&self) -> http::Version {
        self.version
    }

    pub fn uri(&self) -> &http::Uri {
        &self.uri
    }

    pub fn path(&self) -> &str {
        self.uri.path()
    }

    pub fn query(&self) -> Option<&str> {
        self.uri.query()
    }

    pub fn host(&self) -> Option<&str> {
        self.uri.host()
    }

    pub fn header(&self, key: impl AsRef<str>) -> Option<&http::HeaderValue> {
        self.headers.get(key.as_ref())
    }

    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    pub fn extensions(&self) -> &http::Extensions {
        &self.extensions
    }

    pub fn extensions_mut(&mut self) -> &mut http::Extensions {
        &mut self.extensions
    }

    pub fn take_body(&mut self) -> Option<http::Body> {
        self.body.take()
    }

    pub fn middleware(&self) -> &Middlewares {
        &self.middleware
    }

    pub fn middleware_mut(&mut self) -> &mut Middlewares {
        &mut self.middleware
    }

    pub fn params<T: FromStr>(&self, name: &str) -> Result<T, T::Err> {
        self.extensions
            .get::<Params>()
            .map(|ps| ps.find(name))
            .unwrap()
    }

    pub async fn extract<T: Extract>(&mut self) -> Result<T, T::Error> {
        T::extract(self).await
    }

    pub async fn next(&mut self) -> Result<Response> {
        if let Some(m) = self.middleware.pop() {
            m.call(self).await.or_else(|e| Ok(e.into()))
        } else {
            // TODO: Need a DefaultHandler
            Ok(Response::new())
        }
    }
}

impl From<http::Request> for Context {
    #[inline]
    fn from(req: http::Request) -> Self {
        let (parts, body) = req.into_parts();
        Self {
            body: Some(body),
            uri: parts.uri,
            method: parts.method,
            headers: parts.headers,
            version: parts.version,
            extensions: parts.extensions,
            middleware: Vec::new(),
        }
    }
}
