use form_data::FormData;

use viz_utils::futures::future::BoxFuture;

use crate::{
    http,
    types::{Payload, PayloadDetect, PayloadError},
    Context, Extract, Result,
};

/// Multipart Extractor
pub type Multipart<T = http::Body> = FormData<T>;

impl Extract for Multipart {
    type Error = PayloadError;

    #[inline]
    fn extract<'a>(cx: &'a mut Context) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move { cx.multipart() })
    }
}

impl PayloadDetect for Multipart {
    #[inline]
    fn detect(m: &mime::Mime) -> bool {
        m.type_() == mime::MULTIPART && m.subtype() == mime::FORM_DATA
    }
}

impl Context {
    /// Extracts Multipart from the request' body.
    pub fn multipart(&mut self) -> Result<Multipart, PayloadError> {
        let mut payload = Payload::<Multipart>::new();

        payload.set_limit(self.config().limits.multipart);

        let m = payload.check_header(self.mime(), self.len())?;

        let boundary = m.get_param(mime::BOUNDARY);

        if boundary.is_none() {
            return Err(PayloadError::Parse);
        }

        let boundary = boundary.unwrap().as_str();

        // let charset = m.get_param(mime::CHARSET).map(|c| c.to_string());

        Ok(Multipart::new(boundary, self.take_body().ok_or_else(|| PayloadError::Read)?))
    }
}
