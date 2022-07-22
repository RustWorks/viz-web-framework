use crate::{async_trait, Future};

mod after;
mod and_then;
mod around;
mod before;
mod boxed;
mod catch_error;
mod catch_unwind;
mod either;
mod fn_ext;
mod map;
mod map_err;
mod or_else;
mod responder;
mod transform;

pub use after::After;
pub use and_then::AndThen;
pub use around::{Around, Next};
pub use before::Before;
pub use boxed::BoxHandler;
pub use catch_error::CatchError;
pub use catch_unwind::CatchUnwind;
pub use either::Either;
pub use fn_ext::{FnExt, ResponderExt};
pub use map::Map;
pub use map_err::MapErr;
pub use or_else::OrElse;
pub use responder::Responder;
pub use transform::Transform;

/// Composable request handlers.
#[async_trait]
pub trait Handler<Args>: dyn_clone::DynClone + Send + Sync + 'static {
    type Output;

    #[must_use]
    async fn call(&self, args: Args) -> Self::Output;
}

impl<I, T> HandlerExt<I> for T where T: Handler<I> + ?Sized {}

#[async_trait]
impl<F, I, Fut, O> Handler<I> for F
where
    I: Send + 'static,
    F: Fn(I) -> Fut + ?Sized + Clone + Send + Sync + 'static,
    Fut: Future<Output = O> + Send,
{
    type Output = Fut::Output;

    async fn call(&self, args: I) -> Self::Output {
        (self)(args).await
    }
}

/// An extension trait for `Handler`s that provides a variety of convenient
/// combinator functions.
pub trait HandlerExt<I>: Handler<I> {
    fn boxed(self) -> BoxHandler<I, Self::Output>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    fn before<F>(self, f: F) -> Before<Self, F>
    where
        Self: Sized,
    {
        Before::new(self, f)
    }

    fn after<F>(self, f: F) -> After<Self, F>
    where
        Self: Sized,
    {
        After::new(self, f)
    }

    fn around<F>(self, f: F) -> Around<Self, F>
    where
        Self: Sized,
    {
        Around::new(self, f)
    }

    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map::new(self, f)
    }

    fn and_then<F>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
    {
        AndThen::new(self, f)
    }

    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
    {
        MapErr::new(self, f)
    }

    fn or_else<F>(self, f: F) -> OrElse<Self, F>
    where
        Self: Sized,
    {
        OrElse::new(self, f)
    }

    fn catch_error<F, R, E>(self, f: F) -> CatchError<Self, F, R, E>
    where
        Self: Sized,
    {
        CatchError::new(self, f)
    }

    fn catch_unwind<F>(self, f: F) -> CatchUnwind<Self, F>
    where
        Self: Sized,
    {
        CatchUnwind::new(self, f)
    }

    fn with<T>(self, t: T) -> T::Output
    where
        T: Transform<Self>,
        Self: Sized,
    {
        t.transform(self)
    }

    fn with_fn<F>(self, f: F) -> Self
    where
        F: Fn(Self) -> Self,
        Self: Sized,
    {
        f(self)
    }
}
