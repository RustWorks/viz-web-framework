use std::{collections::HashMap, fmt, sync::Arc};

use viz_core::{
    http, Context, DynMiddleware, Extract, Guard, HandlerBase, HandlerCamp, HandlerSuper,
    HandlerWrapper, Middleware, Middlewares, Response, Result,
};

use crate::{Method};

macro_rules! verbs {
    ($(($name:ident, $verb:ident),)*) => {
        $(
            pub fn $name<F, T>(self, handler: F) -> Self
            where
                F: HandlerBase<T> + Send + Sync + 'static,
                T: Extract + Send + Sync + 'static,
                T::Error: Into<Response> + Send,
            {
                self.on(Method::Verb(http::Method::$verb), handler)
            }
        )*
    }
}

macro_rules! verbs2 {
    ($(($name:ident, $verb:ident),)*) => {
        $(
            pub fn $name<F, T>(self, handler: F) -> Self
            where
                F: for<'h> HandlerCamp<'h, T> + Send + Sync + 'static,
                T: Extract + Send + Sync + 'static,
                T::Error: Into<Response> + Send,
            {
                self.on2(Method::Verb(http::Method::$verb), handler)
            }
        )*
    }
}

/// Route
pub struct Route {
    // inherit parrent's middleware
    pub(crate) carry: bool,
    pub(crate) path: String,
    pub(crate) name: Option<String>,
    pub(crate) guard: Option<Box<dyn Guard>>,
    pub(crate) middleware: Option<Middlewares>,
    pub(crate) handlers: HashMap<Method, Arc<DynMiddleware>>,
}

impl Route {
    /// Creates new Route Instance
    pub fn new(path: &str) -> Self {
        Self {
            handlers: HashMap::new(),
            path: path.to_owned(),
            middleware: None,
            carry: true,
            guard: None,
            name: None,
        }
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path.insert_str(0, path);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name.replace(name.to_owned());
        self
    }

    pub fn carry(mut self, b: bool) -> Self {
        self.carry = b;
        self
    }

    pub fn mid<M>(mut self, m: M) -> Self
    where
        M: for<'m> Middleware<'m, Context, Output = Result>,
    {
        self.middleware.get_or_insert_with(Vec::new).insert(0, Arc::new(m));
        self
    }

    pub fn guard<G>(mut self, g: G) -> Self
    where
        G: Into<Box<dyn Guard>>,
    {
        self.guard.replace(g.into());
        self
    }

    pub fn on<F, T>(mut self, method: Method, handler: F) -> Self
    where
        F: HandlerBase<T> + Send + Sync + 'static,
        T: Extract + Send + Sync + 'static,
        T::Error: Into<Response> + Send,
    {
        self.handlers.insert(method, Arc::new(HandlerWrapper::new(handler)));
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

    pub fn all<F, T>(self, handler: F) -> Self
    where
        F: HandlerBase<T> + Send + Sync + 'static,
        T: Extract + Send + Sync + 'static,
        T::Error: Into<Response> + Send,
    {
        self.on(Method::All, handler)
    }

    pub fn only<F, T, const S: usize>(mut self, methods: [Method; S], handler: F) -> Self
    where
        F: HandlerBase<T> + Send + Sync + 'static,
        T: Extract + Send + Sync + 'static,
        T::Error: Into<Response> + Send,
    {
        methods.iter().cloned().for_each(|method| {
            self.handlers.insert(method, Arc::new(HandlerWrapper::new(handler.clone())));
        });
        self
    }

    pub fn except<F, T, const S: usize>(mut self, methods: [Method; S], handler: F) -> Self
    where
        F: HandlerBase<T> + Send + Sync + 'static,
        T: Extract + Send + Sync + 'static,
        T::Error: Into<Response> + Send,
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
            self.handlers.insert(method, Arc::new(HandlerWrapper::new(handler.clone())));
        });

        self
    }

    pub fn on2<F, T>(mut self, method: Method, handler: F) -> Self
    where
        F: for<'h> HandlerCamp<'h, T> + Send + Sync + 'static,
        T: Extract + Send + Sync + 'static,
        T::Error: Into<Response> + Send,
    {
        self.handlers.insert(method, Arc::new(HandlerSuper::new(handler)));
        self
    }

    verbs2! {
        (get2, GET),
        (post2, POST),
        (put2, PUT),
        (delete2, DELETE),
        (options2, OPTIONS),
        (connect2, CONNECT),
        (patch2, PATCH),
        (trace2, TRACE),
    }

    pub fn all2<F, T>(self, handler: F) -> Self
    where
        F: for<'h> HandlerCamp<'h, T> + Send + Sync + 'static,
        T: Extract + Send + Sync + 'static,
        T::Error: Into<Response> + Send,
    {
        self.on2(Method::All, handler)
    }

    pub fn only2<F, T, const S: usize>(mut self, methods: [Method; S], handler: F) -> Self
    where
        F: for<'h> HandlerCamp<'h, T> + Send + Sync + 'static,
        T: Extract + Send + Sync + 'static,
        T::Error: Into<Response> + Send,
    {
        methods.iter().cloned().for_each(|method| {
            self.handlers.insert(method, Arc::new(HandlerSuper::new(handler.clone())));
        });
        self
    }

    pub fn except2<F, T, const S: usize>(mut self, methods: [Method; S], handler: F) -> Self
    where
        F: for<'h> HandlerCamp<'h, T> + Send + Sync + 'static,
        T: Extract + Send + Sync + 'static,
        T::Error: Into<Response> + Send,
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
            self.handlers.insert(method, Arc::new(HandlerSuper::new(handler.clone())));
        });

        self
    }

    pub fn all3<H>(mut self, handler: H) -> Self
    where
        H: for<'m> Middleware<'m, Context, Output = Result>,
    {
        self.handlers.insert(Method::All, Arc::new(handler));
        self
    }
}

pub fn route() -> Route {
    Route::new("")
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handlers
            .keys()
            .fold(
                f.debug_struct("Route")
                    .field("path", &self.path)
                    .field("name", &self.name.as_ref().map_or_else(String::new, |v| v.to_owned()))
                    .field("carry", &self.carry)
                    .field("middle", &self.middleware.as_ref().map_or_else(|| 0, |v| v.len()))
                    .field("guard", &self.guard.as_ref().map_or_else(|| 0, |_| 1)),
                |acc, x| acc.field("verb", &x),
            )
            .finish()
    }
}
