use std::{convert::Infallible, sync::Arc};

use super::{Form, Json, Payload};

use crate::{async_trait, Body, FromRequest, Request, RequestExt};

#[derive(Debug, Clone)]
pub struct Limits {
    inner: Arc<Vec<(&'static str, u64)>>,
}

impl Default for Limits {
    fn default() -> Self {
        Limits::new()
            .insert("text", Limits::NORMAL)
            .insert("bytes", Limits::NORMAL)
            .insert("json", <Json as Payload>::LIMIT)
            .insert("form", <Form as Payload>::LIMIT)
    }
}

impl Limits {
    pub const NORMAL: u64 = 1024 * 8;

    pub fn new() -> Self {
        Limits {
            inner: Arc::new(Vec::new()),
        }
    }

    pub fn insert(mut self, name: &'static str, limit: u64) -> Self {
        Arc::make_mut(&mut self.inner).push((name, limit));
        self
    }

    pub fn get<S>(&self, name: S) -> Option<u64>
    where
        S: AsRef<str>,
    {
        self.inner
            .binary_search_by_key(&name.as_ref(), |&(a, _)| a)
            .map(|i| self.inner[i].1)
            .ok()
    }
}

#[async_trait]
impl FromRequest for Limits {
    type Error = Infallible;

    async fn extract(req: &mut Request<Body>) -> Result<Self, Self::Error> {
        Ok(req.limits())
    }
}
