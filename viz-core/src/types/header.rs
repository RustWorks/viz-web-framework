use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use viz_utils::{anyhow::anyhow, futures::future::BoxFuture, tracing};

use crate::{
    http::headers::{self, HeaderMapExt},
    Context, Error, Extract,
};

/// Header Extractor
#[derive(Clone)]
pub struct Header<T>(pub T);

impl<T> Header<T> {
    /// Create new `Header` instance.
    pub fn new(t: T) -> Self {
        Self(t)
    }

    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Header<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Header<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Header<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Header<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T> Extract for Header<T>
where
    T: headers::Header,
{
    type Error = Error;

    #[inline]
    fn extract(cx: &mut Context) -> BoxFuture<'_, Result<Self, Self::Error>> {
        Box::pin(async move {
            cx.headers()
                .typed_try_get::<T>()
                .map_err(|e| {
                    tracing::error!("{}", e);
                    Error::new(e)
                })
                .and_then(|v| {
                    v.ok_or_else(|| {
                        let name = T::name();
                        tracing::error!("missing header {}", name);
                        anyhow!("missing header {}", name)
                    })
                })
                .map(Self)
        })
    }
}
