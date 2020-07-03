use serde::de::DeserializeOwned;

use viz_utils::{futures::future::BoxFuture, log, serde::urlencoded};

use crate::{Context, Extract, PayloadError, Result};

#[derive(Debug)]
pub struct Query<T>(pub T);

impl<T> std::ops::Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Extract for Query<T>
where
    T: DeserializeOwned + Send + Sync,
{
    type Error = PayloadError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move {
            urlencoded::from_str(cx.query().unwrap_or_default())
                .map(|o| Query(o))
                .map_err(|e| {
                    log::debug!("{}", e);
                    PayloadError::Parse
                })
        })
    }
}
