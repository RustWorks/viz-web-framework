//! Route

use viz_core::{FnExt, FromRequest, HandlerExt, IntoResponse, Method, ResponderExt, Result};

use super::Route;

macro_rules! export_internal_verb_ext {
    ($name:ident $verb:tt) => {
        #[doc = concat!(" Appends a route, handle HTTP verb `", stringify!($verb), "` with multiple parameters.")]
        pub fn $name<H, O, I>(self, handler: H) -> Self
        where
            I: FromRequest + Send + Sync + 'static,
            I::Error: IntoResponse + Send + Sync,
            H: FnExt<I, Output = Result<O>>,
            O: IntoResponse + Send + Sync + 'static,
        {
            self.on_ext(Method::$verb, handler)
        }
    };
}

macro_rules! export_verb_ext {
    ($name:ident $verb:ty) => {
        #[doc = concat!(" Appends a route, handle HTTP verb `", stringify!($verb), "` with multiple parameters.")]
        pub fn $name<H, O, I>(handler: H) -> Route
        where
            I: FromRequest + Send + Sync + 'static,
            I::Error: IntoResponse + Send + Sync,
            H: FnExt<I, Output = Result<O>>,
            O: IntoResponse + Send + Sync + 'static,
        {
            Route::new().$name(handler)
        }
    };
}

impl Route {
    /// Appends a route, with a HTTP verb and handler.
    pub fn on_ext<H, O, I>(self, method: Method, handler: H) -> Self
    where
        I: FromRequest + Send + Sync + 'static,
        I::Error: IntoResponse + Send + Sync,
        H: FnExt<I, Output = Result<O>>,
        O: IntoResponse + Send + Sync + 'static,
    {
        self.push(method, ResponderExt::new(handler).boxed())
    }

    /// Appends a route, with a HTTP verb and handler.
    pub fn any_ext<H, O, I>(self, handler: H) -> Self
    where
        I: FromRequest + Send + Sync + 'static,
        I::Error: IntoResponse + Send + Sync,
        H: FnExt<I, Output = Result<O>>,
        O: IntoResponse + Send + Sync + 'static,
    {
        [
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::OPTIONS,
            Method::CONNECT,
            Method::PATCH,
            Method::TRACE,
        ]
        .into_iter()
        .fold(self, |route, method| route.on_ext(method, handler.clone()))
    }

    repeat!(
        export_internal_verb_ext
        get_ext GET
        post_ext POST
        put_ext PUT
        delete_ext DELETE
        head_ext HEAD
        options_ext OPTIONS
        connect_ext CONNECT
        patch_ext PATCH
        trace_ext TRACE
    );
}

/// Appends a route, with a HTTP verb and multiple parameters of handler.
pub fn on_ext<H, O, I>(method: Method, handler: H) -> Route
where
    I: FromRequest + Send + Sync + 'static,
    I::Error: IntoResponse + Send + Sync,
    H: FnExt<I, Output = Result<O>>,
    O: IntoResponse + Send + Sync + 'static,
{
    Route::new().on_ext(method, handler)
}

repeat!(
    export_verb_ext
    get_ext GET
    post_ext POST
    put_ext PUT
    delete_ext DELETE
    head_ext HEAD
    options_ext OPTIONS
    connect_ext CONNECT
    patch_ext PATCH
    trace_ext TRACE
);

/// Appends a route, with multiple parameters of handler by any HTTP verbs.
pub fn any_ext<H, O, I>(handler: H) -> Route
where
    I: FromRequest + Send + Sync + 'static,
    I::Error: IntoResponse + Send + Sync,
    H: FnExt<I, Output = Result<O>>,
    O: IntoResponse + Send + Sync + 'static,
{
    Route::new().any_ext(handler)
}
