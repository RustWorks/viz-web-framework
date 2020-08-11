use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;

use viz_utils::{futures::future::BoxFuture, log, serde::urlencoded};

use crate::{Context, Extract, PayloadError, Result};

/// Context Extends
pub trait ContextExt {
    fn query<T>(&self) -> Result<T, PayloadError>
    where
        T: DeserializeOwned;
}

impl ContextExt for Context {
    fn query<T>(&self) -> Result<T, PayloadError>
    where
        T: DeserializeOwned,
    {
        urlencoded::from_str(self.query_str().unwrap_or_default()).map_err(|e| {
            log::debug!("{}", e);
            PayloadError::Parse
        })
    }
}

/// Query Extractor
#[derive(Debug)]
pub struct Query<T>(pub T);

impl<T> Query<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Query<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Extract for Query<T>
where
    T: DeserializeOwned,
{
    type Error = PayloadError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.query().map(|v| Query(v)) })
    }
}
