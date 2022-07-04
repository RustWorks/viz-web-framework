use viz_core::{
    handler::Transform, Body, BoxHandler, Handler, HandlerExt, IntoResponse, Method, Request,
    Response, Result,
};

use crate::Route;

#[derive(Clone, Debug, Default)]
pub struct Resource {
    name: String,
    pub(crate) routes: Vec<(String, Route)>,
}

impl Resource {
    pub fn named<S>(mut self, name: S) -> Self
    where
        S: AsRef<str>,
    {
        self.name = name.as_ref().to_owned();
        self
    }

    pub fn route<S>(mut self, path: S, route: Route) -> Self
    where
        S: AsRef<str>,
    {
        let path = path.as_ref().to_owned();
        match self
            .routes
            .iter_mut()
            .find(|(p, _)| p == &path)
            .map(|(_, r)| r)
        {
            Some(r) => *r = route.into_iter().fold(r.to_owned(), |r, (m, h)| r.on(m, h)),
            None => {
                self.routes.push((path, route));
            }
        }
        self
    }

    pub fn on<S, H, O>(mut self, path: S, method: Method, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = path.as_ref().to_owned();
        match self
            .routes
            .iter_mut()
            .find(|(p, _)| p == &path)
            .map(|(_, r)| r)
        {
            Some(r) => {
                *r = r.to_owned().on(method, handler);
            }
            None => {
                self.routes.push((path, Route::new().on(method, handler)));
            }
        }
        self
    }

    pub fn index<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        self.on("", Method::GET, handler)
    }

    pub fn new<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        self.on("new", Method::GET, handler)
    }

    pub fn create<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        self.on("", Method::POST, handler)
    }

    pub fn show<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id", self.name);
        self.on(path, Method::GET, handler)
    }

    pub fn edit<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id/edit", self.name);
        self.on(path, Method::GET, handler)
    }

    pub fn update<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id", self.name);
        self.on(path, Method::PUT, handler)
    }

    pub fn update_with_patch<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id", self.name);
        self.on(path, Method::PATCH, handler)
    }

    pub fn destroy<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request<Body>, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id", self.name);
        self.on(path, Method::DELETE, handler)
    }

    pub fn with<T>(self, t: T) -> Self
    where
        T: Transform<BoxHandler>,
        T::Output: Handler<Request<Body>, Output = Result<Response<Body>>>,
    {
        Self {
            name: self.name,
            routes: self
                .routes
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
                .collect(),
        }
    }

    pub fn map_handler<F>(self, f: F) -> Self
    where
        F: Fn(BoxHandler) -> BoxHandler,
    {
        Self {
            name: self.name,
            routes: self
                .routes
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
                .collect(),
        }
    }
}

impl IntoIterator for Resource {
    type Item = (String, Route);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.routes.into_iter()
    }
}

impl FromIterator<(String, Route)> for Resource {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, Route)>,
    {
        Self {
            name: "".to_string(),
            routes: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{get, Resource, Route};
    use viz_core::{
        async_trait, handler::Transform, Body, Handler, HandlerExt, IntoResponse, Method, Next,
        Request, Response, Result,
    };

    #[tokio::test]
    async fn resource() -> anyhow::Result<()> {
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

        async fn before(req: Request<Body>) -> Result<Request<Body>> {
            dbg!("before req");
            Ok(req)
        }

        async fn after(res: Result<Response<Body>>) -> Result<Response<Body>> {
            dbg!("after res");
            res
        }

        async fn around<H, O>((req, handler): Next<Request<Body>, H>) -> Result<Response<Body>>
        where
            H: Handler<Request<Body>, Output = Result<O>> + Clone,
            O: IntoResponse + Send + Sync + 'static,
        {
            dbg!("around before");
            let res = handler.call(req).await.map(IntoResponse::into_response);
            dbg!("around after");
            res
        }

        async fn index(req: Request<Body>) -> Result<impl IntoResponse> {
            Ok("index")
        }

        async fn any(req: Request<Body>) -> Result<&'static str> {
            Ok("any")
        }

        async fn index_posts(req: Request<Body>) -> Result<Vec<u8>> {
            Ok(b"index posts".to_vec())
        }

        async fn create_post(req: Request<Body>) -> Result<impl IntoResponse> {
            Ok("create post")
        }

        async fn new_post(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("new post".into()))
        }

        async fn show_post(req: Request<Body>) -> Result<Response<Body>> {
            dbg!("responed");
            Ok(Response::new("show post".into()))
        }

        async fn edit_post(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("edit post".into()))
        }

        async fn delete_post(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("delete post".into()))
        }

        async fn update_post(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("update post".into()))
        }

        async fn any_posts(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("any posts".into()))
        }

        async fn search_posts(req: Request<Body>) -> Result<Response<Body>> {
            Ok(Response::new("search posts".into()))
        }

        let resource = Resource::default()
            .index(index)
            .update_with_patch(any_posts);

        let resource = Resource::default()
            .named("post")
            .route("search", get(search_posts))
            .index(index_posts.before(before))
            .new(new_post)
            .create(create_post)
            .show(show_post.after(after))
            .edit(edit_post.around(around))
            .update(update_post)
            .destroy(delete_post)
            .with(Logger::new())
            .map_handler(|handler| {
                handler
                    .before(before)
                    .after(after)
                    .around(around)
                    .with(Logger::new())
                    .boxed()
            })
            .into_iter()
            .collect::<Resource>()
            .named("post");

        dbg!(std::mem::size_of_val(&resource));

        dbg!(&resource);

        let (_, h) = resource
            .routes
            .iter()
            .filter(|(p, _)| p == ":post_id")
            .nth(0)
            .and_then(|(_, r)| r.methods.iter().filter(|(m, _)| m == Method::GET).nth(0))
            .unwrap();
        let res = h.call(Request::default()).await?;
        dbg!(res);

        Ok(())
    }
}
