use bytes::buf::BufExt;
use serde::de::DeserializeOwned;

use viz_utils::futures::future::BoxFuture;
use viz_utils::log;
use viz_utils::serde::json;

use crate::get_length;
use crate::get_mime;
use crate::Context;
use crate::Extract;
use crate::Payload;
use crate::PayloadCheck;
use crate::PayloadError;
use crate::PAYLOAD_LIMIT;

pub struct Json<T>(pub T);

impl<T> Json<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> PayloadCheck for Json<T> {
    fn check_type(m: &mime::Mime) -> bool {
        is_json(m)
    }
}

impl<T> Extract for Json<T>
where
    T: DeserializeOwned + Send + Sync,
{
    type Error = PayloadError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move {
            let mut payload = json();

            // @TODO: read context's limits config
            // payload.set_limit(limit);

            let m = get_mime(cx);
            let l = get_length(cx);

            payload.check_header(m, l)?;

            payload.replace(
                json::from_reader(
                    payload
                        .check_real_length(cx.take_body().ok_or_else(|| PayloadError::Read)?)
                        .await?
                        .reader(),
                )
                .map(|o| Json(o))
                .map_err(|e| {
                    log::debug!("{}", e);
                    PayloadError::Parse
                })?,
            );

            Ok(payload.take())
        })
    }
}

pub fn json<T>() -> Payload<Json<T>>
where
    T: DeserializeOwned,
{
    Payload::new(PAYLOAD_LIMIT, None)
}

fn is_json(m: &mime::Mime) -> bool {
    m.type_() == mime::APPLICATION && (m.subtype() == mime::JSON || m.suffix() == Some(mime::JSON))
}
