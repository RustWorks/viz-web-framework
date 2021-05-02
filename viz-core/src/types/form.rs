use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::buf::Buf;
use serde::de::DeserializeOwned;

use viz_utils::{futures::future::BoxFuture, serde::urlencoded, tracing};

use crate::{
    config::ContextExt as _,
    types::{get_length, get_mime, Payload, PayloadCheck, PayloadError, PAYLOAD_LIMIT},
    Context, Extract,
};

/// Context Extends
pub trait ContextExt {
    fn form<'a, T>(&'a mut self) -> BoxFuture<'a, Result<T, PayloadError>>
    where
        T: DeserializeOwned + Send + Sync;
}

impl ContextExt for Context {
    fn form<'a, T>(&'a mut self) -> BoxFuture<'a, Result<T, PayloadError>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        Box::pin(async move {
            let mut payload = form::<T>();

            payload.set_limit(self.config().limits.form);

            let m = get_mime(self);
            let l = get_length(self);

            payload.check_header(m, l)?;

            // payload.replace(
            //     urlencoded::from_reader(
            //         payload
            //             .check_real_length(self.take_body().ok_or_else(|| PayloadError::Read)?)
            //             .await?
            //             .reader(),
            //     )
            //     // .map(|o| Form(o))
            //     .map_err(|e| {
            //         tracing::debug!("{}", e);
            //         PayloadError::Parse
            //     })?,
            // );

            urlencoded::from_reader(
                payload
                    .check_real_length(self.take_body().ok_or_else(|| PayloadError::Read)?)
                    .await?
                    .reader(),
            )
            // .map(|o| Form(o))
            .map_err(|e| {
                tracing::debug!("{}", e);
                PayloadError::Parse
            })

            // Ok(payload.take())
        })
    }
}

/// Form Extractor
#[derive(Clone)]
pub struct Form<T>(pub T);

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

impl<T> PayloadCheck for Form<T> {
    fn check_type(m: &mime::Mime) -> bool {
        is_form(m)
    }
}

impl<T: fmt::Debug> fmt::Debug for Form<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(&self, f)
    }
}

impl<T> Extract for Form<T>
where
    T: DeserializeOwned + Send + Sync,
{
    type Error = PayloadError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.form().await.map(Form) })
    }
}

pub fn form<T>() -> Payload<Form<T>>
where
    T: DeserializeOwned,
{
    Payload::new()
}

fn is_form(m: &mime::Mime) -> bool {
    m.type_() == mime::APPLICATION && m.subtype() == mime::WWW_FORM_URLENCODED
}
