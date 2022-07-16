//! Request Query Extractor

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use serde::de::DeserializeOwned;

use crate::{async_trait, types::PayloadError, FromRequest, Request, RequestExt, Result};

/// Query Extractor
pub struct Query<T = ()>(pub T);

impl<T> Query<T> {
    /// Create new `Query` instance.
    #[inline]
    pub fn new(data: T) -> Self {
        Query(data)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Clone for Query<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Query(self.0.clone())
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

impl<T> fmt::Debug for Query<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

#[async_trait]
impl<T> FromRequest for Query<T>
where
    T: DeserializeOwned,
{
    type Error = PayloadError;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.query().map(Self)
    }
}
