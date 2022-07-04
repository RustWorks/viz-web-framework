use viz_core::{
    handler::Transform, Body, BoxHandler, Handler, HandlerExt, Next, Request, Response, Result,
};

use crate::{Resource, Route};

#[derive(Clone, Debug)]
pub struct Router {
    pub(crate) routes: Option<Vec<(String, Route)>>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: None }
    }

    fn push<S>(routes: &mut Vec<(String, Route)>, path: S, route: Route)
    where
        S: AsRef<str>,
    {
        let path = path.as_ref();
        match routes
            .iter_mut()
            .find_map(|(p, r)| if p == path { Some(r) } else { None })
        {
            Some(r) => *r = route,
            None => routes.push((path.to_string(), route)),
        }
    }

    pub fn route<S>(mut self, path: S, route: Route) -> Self
    where
        S: AsRef<str>,
    {
        Self::push(
            self.routes.get_or_insert_with(Vec::new),
            path.as_ref().trim_start_matches('/'),
            route,
        );
        self
    }

    pub fn resource<S>(self, path: S, resource: Resource) -> Self
    where
        S: AsRef<str>,
    {
        let mut path = path.as_ref().to_string();
        if !path.ends_with('/') {
            path.push('/');
        }

        resource
            .routes
            .into_iter()
            .fold(self, |router, (mut sp, route)| {
                let is_empty = sp.is_empty();
                sp = path.clone() + &sp;
                if is_empty {
                    sp = sp.trim_end_matches('/').to_string();
                }
                router.route(sp, route)
            })
    }

    pub fn nest<S>(self, path: S, router: Self) -> Self
    where
        S: AsRef<str>,
    {
        let mut path = path.as_ref().to_string();
        if !path.ends_with('/') {
            path.push('/');
        }

        match router.routes {
            Some(routes) => routes.into_iter().fold(self, |router, (mut sp, route)| {
                let is_empty = sp.is_empty();
                sp = path.clone() + &sp;
                if is_empty {
                    sp = sp.trim_end_matches('/').to_string();
                }
                router.route(sp, route)
            }),
            None => self,
        }
    }

    pub fn with<T>(self, t: T) -> Self
    where
        T: Transform<BoxHandler>,
        T::Output: Handler<Request<Body>, Output = Result<Response<Body>>>,
    {
        Self {
            routes: self.routes.map(|routes| {
                routes
                    .into_iter()
                    .map(|(path, route)| {
                        (
                            path,
                            route
                                .into_iter()
                                .map(|(method, handler)| (method, t.transform(handler).boxed()))
                                .collect(),
                        )
                    })
                    .collect()
            }),
        }
    }

    pub fn with_handler<F>(self, f: F) -> Self
    where
        F: Handler<Next<Request<Body>, BoxHandler>, Output = Result<Response<Body>>> + Clone,
    {
        Self {
            routes: self.routes.map(|routes| {
                routes
                    .into_iter()
                    .map(|(path, route)| {
                        (
                            path,
                            route
                                .into_iter()
                                .map(|(method, handler)| {
                                    (method, handler.around(f.clone()).boxed())
                                })
                                .collect(),
                        )
                    })
                    .collect()
            }),
        }
    }

    pub fn map_handler<F>(self, f: F) -> Self
    where
        F: Fn(BoxHandler) -> BoxHandler,
    {
        Self {
            routes: self.routes.map(|routes| {
                routes
                    .into_iter()
                    .map(|(path, route)| {
                        (
                            path,
                            route
                                .into_iter()
                                .map(|(method, handler)| (method, f(handler)))
                                .collect(),
                        )
                    })
                    .collect()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{get, Resource, Route, Router, Tree};
    use viz_core::{
        async_trait, handler::Transform, Body, Handler, HandlerExt, IntoResponse, Method, Next,
        Request, Response, Result, StatusCode,
    };

    #[derive(Clone)]
    struct Logger;

    impl Logger {
        fn new() -> Self {
            Self
        }
    }

    impl<H: Clone> Transform<H> for Logger {
        type Output = LoggerHandler<H>;

        fn transform(&self, h: H) -> Self::Output {
            LoggerHandler(h.clone())
        }
    }

    #[derive(Clone)]
    struct LoggerHandler<H>(H);

    #[async_trait]
    impl<H> Handler<Request<Body>> for LoggerHandler<H>
    where
        H: Handler<Request<Body>> + Clone,
    {
        type Output = H::Output;

        async fn call(&self, req: Request<Body>) -> Self::Output {
            dbg!("before logger");
            let res = self.0.call(req).await;
            dbg!("after logger");
            res
        }
    }

    #[tokio::test]
    async fn router() -> anyhow::Result<()> {
        async fn index(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("index".into()))
        }

        async fn any(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("any".into()))
        }

        async fn not_found(req: Request<Body>) -> Result<impl IntoResponse> {
            Ok(StatusCode::NOT_FOUND)
        }

        async fn search(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("search".into()))
        }

        async fn show(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("show".into()))
        }

        async fn create(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("create".into()))
        }

        async fn update(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("update".into()))
        }

        async fn delete(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("delete".into()))
        }

        let users = Resource::default()
            .named("user")
            .index(index)
            .create(create.before(|r: Request<Body>| async { Ok(r) }))
            .show(show)
            .update(update)
            .destroy(delete)
            .with(Logger::new());

        let posts = Router::new().route("search", get(search)).resource(
            "",
            Resource::default()
                .named("post")
                .create(create)
                .show(show)
                .update(update)
                .destroy(delete)
                .with(Logger::new()),
        );

        let router = Router::new()
            .route("", get(index))
            .resource("users", users.clone())
            .nest("posts", posts.resource(":post_id/users", users))
            .route("*", Route::new().any(not_found))
            .with(Logger::new());

        dbg!(&router);

        let tree: Tree = router.into();

        Ok(())
    }
}
