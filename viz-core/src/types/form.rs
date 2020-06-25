use bytes::buf::BufExt;
use serde::de::DeserializeOwned;

use viz_utils::futures::future::BoxFuture;
use viz_utils::log;
use viz_utils::serde::urlencoded;

use crate::get_length;
use crate::get_mime;
use crate::Context;
use crate::Extract;
use crate::Payload;
use crate::PayloadCheck;
use crate::PayloadError;
use crate::PAYLOAD_LIMIT;

pub struct Form<T>(pub T);

impl<T> Form<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Form<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> PayloadCheck for Form<T> {
    fn check_type(m: &mime::Mime) -> bool {
        is_form(m)
    }
}

impl<T> Extract for Form<T>
where
    T: DeserializeOwned + Send + Sync,
{
    type Error = PayloadError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move {
            let mut payload = form();

            // TODO: read context's limits config
            // payload.set_limit(limit);

            let m = get_mime(cx);
            let l = get_length(cx);

            payload.check_header(m, l)?;

            payload.replace(
                urlencoded::from_reader(
                    payload
                        .check_real_length(cx.take_body().ok_or_else(|| PayloadError::Read)?)
                        .await?
                        .reader(),
                )
                .map(|o| Form(o))
                .map_err(|e| {
                    log::debug!("{}", e);
                    PayloadError::Parse
                })?,
            );

            Ok(payload.take())
        })
    }
}

pub fn form<T>() -> Payload<Form<T>>
where
    T: DeserializeOwned,
{
    Payload::new(PAYLOAD_LIMIT, None)
}

fn is_form(m: &mime::Mime) -> bool {
    m.type_() == mime::APPLICATION && m.subtype() == mime::WWW_FORM_URLENCODED
}
