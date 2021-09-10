use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use serde::de::DeserializeOwned;

use viz_utils::{futures::future::BoxFuture, serde::urlencoded, tracing};

use crate::{types::PayloadError, Context, Extract, Result};

/// Extract typed information from the request's query
#[derive(Clone)]
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

impl<T: fmt::Debug> fmt::Debug for Query<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T> Extract for Query<T>
where
    T: DeserializeOwned,
{
    type Error = PayloadError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.query().map(Query) })
    }
}

impl Context {
    /// Gets query parameters from the path
    pub fn query<T>(&self) -> Result<T, PayloadError>
    where
        T: DeserializeOwned,
    {
        urlencoded::from_str(self.query_str()).map_err(|e| {
            tracing::debug!("Query deserialize error: {}", e);
            PayloadError::Parse
        })
    }
}
