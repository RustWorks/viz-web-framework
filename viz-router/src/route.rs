use std::{collections::HashMap, fmt, sync::Arc};

use viz_core::{
    http, Context, DynMiddleware, Endpoint, Extract, Future, Handler, Middleware, Response, Result,
    VecMiddleware,
};

use crate::Method;

macro_rules! verbs {
    ($(($name:ident, $verb:ident),)*) => {
        $(
            #[doc = concat!("Appends a route, handle HTTP verb `", stringify!($verb), "`")]
            pub fn $name<H, A>(self, h: H) -> Self
            where
                A: Extract,
                A::Error: Into<Response>,
                H: Handler<A>,
                H::Output: Into<Response>,
                H::Future: Future<Output = H::Output> + Send + 'static,
                Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
            {
                self.on(Method::Verb(http::Method::$verb), h)
            }
        )*
    }
}

macro_rules! stand_alone_verbs {
    ($(($name:ident, $verb:ident),)*) => {
        $(
            #[doc = concat!("Appends a route, handle HTTP verb `", stringify!($verb), "`")]
            pub fn $name<H, A>(h: H) -> Route
            where
                A: Extract,
                A::Error: Into<Response>,
                H: Handler<A>,
                H::Output: Into<Response>,
                H::Future: Future<Output = H::Output> + Send + 'static,
                Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
            {

                Route::on(Route::new(""), Method::Verb(http::Method::$verb), h)
            }
        )*
    }
}

/// Route
pub struct Route {
    // inherit parrent's middleware
    pub(crate) inherit: bool,
    pub(crate) path: String,
    pub(crate) name: Option<String>,
    pub(crate) middleware: Option<VecMiddleware>,
    pub(crate) handlers: HashMap<Method, Arc<DynMiddleware>>,
}

impl Route {
    /// Creates new Route Instance
    pub fn new(path: &str) -> Self {
        Self {
            handlers: HashMap::new(),
            path: path.to_owned(),
            middleware: None,
            inherit: true,
            name: None,
        }
    }

    /// Sets a path
    pub fn path(mut self, path: &str) -> Self {
        self.path.insert_str(0, path);
        self
    }

    /// Sets a name
    pub fn name(mut self, name: &str) -> Self {
        self.name.replace(name.to_owned());
        self
    }

    /// Sets a inherit
    pub fn inherit(mut self, b: bool) -> Self {
        self.inherit = b;
        self
    }

    /// Appends a middleware
    pub fn with<M>(mut self, m: M) -> Self
    where
        M: for<'m> Middleware<'m, Context, Output = Result>,
    {
        self.middleware.get_or_insert_with(Vec::new).insert(0, Arc::new(m));
        self
    }

    /// Appends a route, handle HTTP verb
    pub fn on<H, A>(mut self, method: Method, h: H) -> Self
    where
        A: Extract,
        A::Error: Into<Response>,
        H: Handler<A>,
        H::Output: Into<Response>,
        H::Future: Future<Output = H::Output> + Send + 'static,
        Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
    {
        self.handlers.insert(method, Arc::new(Endpoint::new(h)));
        self
    }

    verbs! {
        (get, GET),
        (post, POST),
        (put, PUT),
        (delete, DELETE),
        (options, OPTIONS),
        (connect, CONNECT),
        (patch, PATCH),
        (trace, TRACE),
    }

    /// Appends a route, handle all HTTP verbs
    pub fn all<H, A>(self, h: H) -> Self
    where
        A: Extract,
        A::Error: Into<Response>,
        H: Handler<A>,
        H::Output: Into<Response>,
        H::Future: Future<Output = H::Output> + Send + 'static,
        Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
    {
        self.on(Method::All, h)
    }

    /// Appends a route, only handle HTTP verbs
    pub fn only<H, A, const S: usize>(mut self, methods: [Method; S], h: H) -> Self
    where
        A: Extract,
        A::Error: Into<Response>,
        H: Handler<A>,
        H::Output: Into<Response>,
        H::Future: Future<Output = H::Output> + Send + 'static,
        Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
    {
        methods.iter().cloned().for_each(|method| {
            self.handlers.insert(method, Arc::new(Endpoint::new(h.clone())));
        });
        self
    }

    /// Appends a route, except handle verbs
    pub fn except<H, A, const S: usize>(mut self, methods: [Method; S], h: H) -> Self
    where
        A: Extract,
        A::Error: Into<Response>,
        H: Handler<A>,
        H::Output: Into<Response>,
        H::Future: Future<Output = H::Output> + Send + 'static,
        Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
    {
        let mut verbs = vec![
            Method::Verb(http::Method::GET),
            Method::Verb(http::Method::POST),
            Method::Verb(http::Method::PUT),
            Method::Verb(http::Method::DELETE),
            Method::Verb(http::Method::OPTIONS),
            Method::Verb(http::Method::CONNECT),
            Method::Verb(http::Method::PATCH),
            Method::Verb(http::Method::TRACE),
        ];

        verbs.dedup_by_key(|m| methods.contains(m));

        verbs.iter().cloned().for_each(|method| {
            self.handlers.insert(method, Arc::new(Endpoint::new(h.clone())));
        });

        self
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handlers
            .keys()
            .fold(
                f.debug_struct("Route")
                    .field("path", &self.path)
                    .field("name", &self.name.as_ref().map_or_else(String::new, |v| v.to_owned()))
                    .field("inherit", &self.inherit)
                    .field("middle", &self.middleware.as_ref().map_or_else(|| 0, |v| v.len())),
                |acc, x| acc.field("verb", &x),
            )
            .finish()
    }
}

/// Creates new Route
pub fn route(path: &str) -> Route {
    Route::new(path)
}

/// Appends a middleware
pub fn with<M>(m: M) -> Route
where
    M: for<'m> Middleware<'m, Context, Output = Result>,
{
    Route::with(Route::new(""), m)
}

stand_alone_verbs! {
    (get, GET),
    (post, POST),
    (put, PUT),
    (delete, DELETE),
    (options, OPTIONS),
    (connect, CONNECT),
    (patch, PATCH),
    (trace, TRACE),
}

/// Appends a route, handle all HTTP verbs
pub fn all<H, A>(h: H) -> Route
where
    A: Extract,
    A::Error: Into<Response>,
    H: Handler<A>,
    H::Output: Into<Response>,
    H::Future: Future<Output = H::Output> + Send + 'static,
    Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
{
    Route::all(Route::new(""), h)
}

/// Appends a route, only handle HTTP verbs
pub fn only<H, A, const S: usize>(methods: [Method; S], h: H) -> Route
where
    A: Extract,
    A::Error: Into<Response>,
    H: Handler<A>,
    H::Output: Into<Response>,
    H::Future: Future<Output = H::Output> + Send + 'static,
    Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
{
    Route::only(Route::new(""), methods, h)
}

/// Appends a route, except handle verbs
pub fn except<H, A, const S: usize>(methods: [Method; S], h: H) -> Route
where
    A: Extract,
    A::Error: Into<Response>,
    H: Handler<A>,
    H::Output: Into<Response>,
    H::Future: Future<Output = H::Output> + Send + 'static,
    Endpoint<H, A>: for<'m> Middleware<'m, Context, Output = Result>,
{
    Route::except(Route::new(""), methods, h)
}
