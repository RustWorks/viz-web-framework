use std::error::Error as StdError;

use crate::{Body, IntoResponse, Response, ThisError};

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("response")]
    Responder(Response<Body>),
    #[error("report")]
    Report(Box<dyn StdError + Send + Sync>, Response<Body>),
}

impl Error {
    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: StdError + 'static,
    {
        if let Self::Report(e, _) = self {
            return e.is::<T>();
        }
        false
    }

    #[inline]
    pub fn downcast<T>(self) -> Result<T, Self>
    where
        T: StdError + 'static,
    {
        if let Self::Report(e, r) = self {
            return match e.downcast::<T>() {
                Ok(e) => Ok(*e),
                Err(e) => Err(Self::Report(e, r)),
            };
        }
        Err(self)
    }

    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: StdError + 'static,
    {
        if let Self::Report(e, _) = self {
            return e.downcast_ref::<T>();
        }
        None
    }

    #[inline]
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: StdError + 'static,
    {
        if let Self::Report(e, _) = self {
            return e.downcast_mut::<T>();
        }
        None
    }
}

impl<E, T> From<(E, T)> for Error
where
    E: StdError + Send + Sync + 'static,
    T: IntoResponse,
{
    fn from((e, t): (E, T)) -> Self {
        Error::Report(Box::new(e), t.into_response())
    }
}
