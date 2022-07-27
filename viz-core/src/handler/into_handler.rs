use crate::{FromRequest, IntoResponse, Request, Result};

use super::{FnExt, Handler, ResponderExt};

/// Trait implemented by types that can be converted to a [`Handler`].
pub trait IntoHandler<E, I = Request> {
    /// The target handler.
    type Handler: Handler<I>;

    /// Convert self to a [Handler].
    #[must_use]
    fn into_handler(self) -> Self::Handler;
}

impl<H, E, O> IntoHandler<E> for H
where
    E: FromRequest + Send + Sync + 'static,
    E::Error: IntoResponse + Send + Sync,
    H: FnExt<E, Output = Result<O>>,
    O: Send + Sync + 'static,
{
    type Handler = ResponderExt<H, E, O>;

    fn into_handler(self) -> Self::Handler {
        ResponderExt::new(self)
    }
}
