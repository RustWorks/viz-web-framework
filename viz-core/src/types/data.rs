use std::{
    any::type_name,
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{async_trait, types::PayloadError, Body, FromRequest, Request, RequestExt, Result};

/// Data Extractor
pub struct Data<T: ?Sized>(pub T);

impl<T> Data<T> {
    /// Create new `Data` instance.
    #[inline]
    pub fn new(data: T) -> Self {
        Self(data)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Clone for Data<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<T> for Data<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Data<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Data<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

#[async_trait]
impl<T> FromRequest for Data<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = PayloadError;

    async fn extract(req: &mut Request<Body>) -> Result<Self, Self::Error> {
        req.data().map(Self).ok_or_else(error::<T>)
    }
}

fn error<T>() -> PayloadError {
    PayloadError::Data(type_name::<T>())
}
