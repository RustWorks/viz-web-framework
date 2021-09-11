use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::buf::Buf;
use serde::de::DeserializeOwned;

use viz_utils::{futures::future::BoxFuture, serde::urlencoded, tracing};

use crate::{
    types::{Payload, PayloadDetect, PayloadError},
    Context, Extract,
};

/// Form Extractor
#[derive(Clone)]
pub struct Form<T = ()>(pub T);

impl<T> Form<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Form<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Form<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T> Extract for Form<T>
where
    T: DeserializeOwned + Send + Sync,
{
    type Error = PayloadError;

    #[inline]
    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
        Box::pin(async move { cx.form().await.map(Form) })
    }
}

impl PayloadDetect for Form {
    #[inline]
    fn detect(m: &mime::Mime) -> bool {
        m.type_() == mime::APPLICATION && m.subtype() == mime::WWW_FORM_URLENCODED
    }
}

impl Context {
    /// Extracts Form Data from the request' body.
    pub async fn form<T>(&mut self) -> Result<T, PayloadError>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let mut payload = Payload::<Form>::new();

        payload.set_limit(self.config().limits.form);

        payload.check_header(self.mime(), self.size())?;

        urlencoded::from_reader(
            payload
                .check_real_length(self.take_body().ok_or(PayloadError::Read)?)
                .await?
                .reader(),
        )
        .map_err(|e| {
            tracing::debug!("{}", e);
            PayloadError::Parse
        })
    }
}
