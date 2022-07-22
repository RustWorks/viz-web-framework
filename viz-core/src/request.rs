//! The Request Extension

use std::mem::replace;

use crate::{async_trait, header, types::PayloadError, Body, Bytes, FromRequest, Request, Result};

#[cfg(feature = "limits")]
use crate::types::Limits;
#[cfg(feature = "limits")]
use http_body::{LengthLimitError, Limited};

#[cfg(any(feature = "form", feature = "json", feature = "multipart"))]
use crate::types::Payload;

#[cfg(feature = "form")]
use crate::types::Form;

#[cfg(feature = "json")]
use crate::types::Json;

#[cfg(feature = "multipart")]
use crate::types::{Multipart, MultipartLimits};

#[cfg(feature = "cookie")]
use crate::types::{Cookie, Cookies, CookiesError};

#[cfg(feature = "session")]
use crate::types::Session;

#[cfg(feature = "params")]
use crate::types::{Params, ParamsError, PathDeserializer};

#[async_trait]
pub trait RequestExt {
    fn path(&self) -> &str;

    fn query_string(&self) -> Option<&str>;

    fn header<K, T>(&self, key: K) -> Option<T>
    where
        K: header::AsHeaderName,
        T: std::str::FromStr;

    fn content_length(&self) -> Option<u64>;

    fn content_type(&self) -> Option<mime::Mime>;

    async fn extract<T>(&mut self) -> Result<T, T::Error>
    where
        T: FromRequest;

    #[cfg(feature = "query")]
    fn query<T>(&self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned;

    /// Reads bytes
    async fn read(&mut self) -> Result<Bytes, PayloadError>;

    #[cfg(feature = "limits")]
    /// Reads bytes with a limit by name.
    async fn read_with(&mut self, name: &str, max: u64) -> Result<Bytes, PayloadError>;

    /// Reads the request body, and returns with a raw binary data buffer.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/arrayBuffer>
    async fn bytes(&mut self) -> Result<Bytes, PayloadError>;

    /// Reads the request body, and returns with a String, it is always decoded using UTF-8.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/text>
    async fn text(&mut self) -> Result<String, PayloadError>;

    #[cfg(feature = "form")]
    async fn form<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned;

    #[cfg(feature = "json")]
    /// Reads the request body, and returns with a JSON.
    ///
    /// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/Response/json>
    async fn json<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned;

    #[cfg(feature = "multipart")]
    async fn multipart(&mut self) -> Result<Multipart, PayloadError>;

    #[cfg(feature = "data")]
    fn data<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    #[cfg(feature = "data")]
    fn set_data<T>(&mut self, t: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    #[cfg(feature = "cookie")]
    fn cookies(&self) -> Result<Cookies, CookiesError>;

    #[cfg(feature = "cookie")]
    fn cookie<S>(&self, name: S) -> Option<Cookie<'_>>
    where
        S: AsRef<str>;

    #[cfg(feature = "limits")]
    fn limits(&self) -> Limits;

    #[cfg(feature = "session")]
    /// Gets session
    fn session(&self) -> &Session;

    #[cfg(feature = "params")]
    /// Gets all parameters.
    fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: serde::de::DeserializeOwned;

    #[cfg(feature = "params")]
    /// Gets single parameter by name.
    fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display;
}

#[async_trait]
impl RequestExt for Request<Body> {
    fn path(&self) -> &str {
        self.uri().path()
    }

    fn query_string(&self) -> Option<&str> {
        self.uri().query()
    }

    fn header<K, T>(&self, key: K) -> Option<T>
    where
        K: header::AsHeaderName,
        T: std::str::FromStr,
    {
        self.headers()
            .get(key)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<T>().ok())
    }

    fn content_length(&self) -> Option<u64> {
        self.header(header::CONTENT_LENGTH)
    }

    fn content_type(&self) -> Option<mime::Mime> {
        self.header(header::CONTENT_TYPE)
    }

    async fn extract<T>(&mut self) -> Result<T, T::Error>
    where
        T: FromRequest,
    {
        T::extract(self).await
    }

    #[cfg(feature = "query")]
    fn query<T>(&self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_urlencoded::from_str(self.query_string().unwrap_or_default())
            .map_err(PayloadError::UrlDecode)
    }

    /// Reads bytes
    async fn read(&mut self) -> Result<Bytes, PayloadError> {
        hyper::body::to_bytes(replace(self.body_mut(), Body::empty()))
            .await
            .map_err(|_| PayloadError::Read)
    }

    #[cfg(feature = "limits")]
    async fn read_with(&mut self, name: &str, max: u64) -> Result<Bytes, PayloadError> {
        let limit = self.limits().get(name).unwrap_or(max) as usize;
        let body = Limited::new(replace(self.body_mut(), Body::empty()), limit);
        hyper::body::to_bytes(body).await.map_err(|err| {
            if err.downcast_ref::<LengthLimitError>().is_some() {
                return PayloadError::TooLarge;
            }
            if let Ok(err) = err.downcast::<hyper::Error>() {
                return PayloadError::Hyper(*err);
            }
            PayloadError::Read
        })
    }

    async fn bytes(&mut self) -> Result<Bytes, PayloadError> {
        #[cfg(feature = "limits")]
        let bytes = self.read_with("bytes", Limits::NORMAL).await;
        #[cfg(not(feature = "limits"))]
        let bytes = self.read().await;

        bytes
    }

    async fn text(&mut self) -> Result<String, PayloadError> {
        #[cfg(feature = "limits")]
        let bytes = self.read_with("text", Limits::NORMAL).await?;
        #[cfg(not(feature = "limits"))]
        let bytes = self.read().await?;

        String::from_utf8(bytes.to_vec()).map_err(PayloadError::Utf8)
    }

    #[cfg(feature = "form")]
    async fn form<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        let _ = <Form as Payload>::check_header(self.content_type(), self.content_length(), None)?;

        #[cfg(feature = "limits")]
        let bytes = self
            .read_with(<Form as Payload>::NAME, <Form as Payload>::LIMIT)
            .await?;
        #[cfg(not(feature = "limits"))]
        let bytes = self.read().await?;

        serde_urlencoded::from_reader(bytes::Buf::reader(bytes)).map_err(PayloadError::UrlDecode)
    }

    #[cfg(feature = "json")]
    async fn json<T>(&mut self) -> Result<T, PayloadError>
    where
        T: serde::de::DeserializeOwned,
    {
        let _ = <Json as Payload>::check_header(self.content_type(), self.content_length(), None)?;

        #[cfg(feature = "limits")]
        let bytes = self
            .read_with(<Json as Payload>::NAME, <Json as Payload>::LIMIT)
            .await?;
        #[cfg(not(feature = "limits"))]
        let bytes = self.read().await?;

        serde_json::from_slice(&bytes).map_err(PayloadError::Json)
    }

    #[cfg(feature = "multipart")]
    async fn multipart(&mut self) -> Result<Multipart, PayloadError> {
        let m =
            <Multipart as Payload>::check_header(self.content_type(), self.content_length(), None)?;

        let boundary = m
            .get_param(mime::BOUNDARY)
            .ok_or(PayloadError::MissingBoundary)?
            .as_str();

        let body = replace(self.body_mut(), Body::empty());

        Ok(Multipart::with_limits(
            body,
            boundary,
            self.extensions()
                .get::<std::sync::Arc<MultipartLimits>>()
                .map(|ml| ml.as_ref().clone())
                .unwrap_or_default(),
        ))
    }

    #[cfg(feature = "data")]
    fn data<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.extensions().get().cloned()
    }

    #[cfg(feature = "data")]
    fn set_data<T>(&mut self, t: T) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.extensions_mut().insert(t)
    }

    #[cfg(feature = "cookie")]
    fn cookies(&self) -> Result<Cookies, CookiesError> {
        self.extensions()
            .get::<Cookies>()
            .cloned()
            .ok_or(CookiesError::Read)
    }

    #[cfg(feature = "cookie")]
    fn cookie<S>(&self, name: S) -> Option<Cookie<'_>>
    where
        S: AsRef<str>,
    {
        self.extensions().get::<Cookies>()?.get(name.as_ref())
    }

    #[cfg(feature = "limits")]
    fn limits(&self) -> Limits {
        self.extensions()
            .get::<Limits>()
            .cloned()
            .expect("Limits middleware is required")
    }

    #[cfg(feature = "session")]
    fn session(&self) -> &Session {
        self.extensions().get().expect("should get a session")
    }

    #[cfg(feature = "params")]
    /// Gets all parameters.
    fn params<T>(&self) -> Result<T, ParamsError>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.extensions().get::<Params>() {
            None => Err(ParamsError::Empty),
            Some(params) => {
                T::deserialize(PathDeserializer::new(params)).map_err(ParamsError::Parse)
            }
        }
    }

    #[cfg(feature = "params")]
    /// Gets single parameter by name.
    fn param<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        self.extensions()
            .get::<Params>()
            .ok_or(ParamsError::Empty)?
            .find(name)
    }
}
