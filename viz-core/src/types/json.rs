use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::buf::Buf;
use serde::de::DeserializeOwned;

use viz_utils::{futures::future::BoxFuture, serde::json, tracing};

use crate::{
    types::{Payload, PayloadDetect, PayloadError},
    Context, Extract,
};

/// Json Extractor
#[derive(Clone)]
pub struct Json<T = ()>(pub T);

impl<T> Json<T> {
    /// Create new `Json` instance.
    pub fn new(t: T) -> Self {
        Self(t)
    }

    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Json<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Json<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T> Extract for Json<T>
where
    T: DeserializeOwned + Send + Sync,
{
    type Error = PayloadError;

    #[inline]
    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
        Box::pin(async move { cx.json().await.map(Self) })
    }
}

impl PayloadDetect for Json {
    #[inline]
    fn detect(m: &mime::Mime) -> bool {
        m.type_() == mime::APPLICATION
            && (m.subtype() == mime::JSON || m.suffix() == Some(mime::JSON))
    }
}

impl Context {
    /// Extracts JSON Data from the request' body.
    pub async fn json<T>(&mut self) -> Result<T, PayloadError>
    where
        T: DeserializeOwned,
    {
        // @TODO: cache Payload<JSON> to extensions
        let mut payload = Payload::<Json>::new();

        payload.set_limit(self.config().limits.json);

        payload.check_header(self.mime(), self.size())?;

        json::from_slice(
            payload.check_real_length(self.take_body().ok_or(PayloadError::Read)?).await?.chunk(),
        )
        .map_err(|e| {
            tracing::error!("Json deserialize error: {}", e);
            PayloadError::Parse
        })
    }
}
