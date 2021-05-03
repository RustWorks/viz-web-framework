use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use viz_utils::{anyhow::anyhow, futures::future::BoxFuture, tracing};

use crate::{http, Context, Error, Extract, Result};

/// Application state factory.
pub trait StateFactory: Send + Sync + 'static {
    /// Injects a state to Application.
    fn create(&self, extensions: &mut http::Extensions) -> bool;
}

/// Application state.
#[derive(Clone)]
pub struct State<T>(T);

impl<T> State<T> {
    /// Create new `State` instance.
    pub fn new(t: T) -> Self {
        Self(t)
    }

    /// Deconstruct to an inner value,
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for State<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&self, f)
    }
}

impl<T> StateFactory for State<T>
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

impl<T> Extract for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = Error;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.state::<Self>() })
    }
}

impl Context {
    /// Gets an app state.
    pub fn state<T>(&self) -> Result<T, Error>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.extensions()
            .get::<State<T>>()
            .cloned()
            .ok_or_else(|| {
                tracing::debug!("State extract error: {}", std::any::type_name::<T>());
                anyhow!("State is not configured")
            })
            .map(State::into_inner)
    }
}
