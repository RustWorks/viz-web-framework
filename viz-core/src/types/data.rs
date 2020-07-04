use std::ops::{Deref, DerefMut};

use viz_utils::{anyhow::anyhow, futures::future::BoxFuture, log};

use crate::{http, Context, Error, Extract, Result};

pub trait ContextExt {
    fn data<T>(&self) -> Result<T, Error>
    where
        T: Clone + Send + Sync + 'static;
}

impl ContextExt for Context {
    fn data<T>(&self) -> Result<T, Error>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.extensions()
            .get::<Data<T>>()
            .cloned()
            .ok_or_else(|| {
                log::debug!(
                    "Failed to construct Data extractor. \
                 Request path: {}",
                    self.path()
                );
                anyhow!("Data is not configured")
            })
            .map(|v| v.into_inner())
    }
}

#[derive(Clone, Debug)]
pub struct Data<T>(T);

impl<T> Data<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(t: T) -> Self {
        Self(t)
    }

    pub fn into_inner(self) -> T {
        self.0
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

pub trait DataFactory: Send + Sync + 'static {
    fn create(&self, extensions: &mut http::Extensions) -> bool;
}

impl<T> DataFactory for Data<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn create(&self, extensions: &mut http::Extensions) -> bool {
        if extensions.get::<Self>().is_none() {
            extensions.insert(self.clone());
            true
        } else {
            false
        }
    }
}

impl<T> Extract for Data<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = Error;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move {
            cx.extensions().get::<Data<T>>().cloned().ok_or_else(|| {
                log::debug!(
                    "Failed to construct Data extractor. \
                 Request path: {}",
                    cx.path()
                );
                anyhow!("Data is not configured")
            })
        })
    }
}
