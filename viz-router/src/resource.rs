//! Resource

use viz_core::{
    BoxHandler, Handler, HandlerExt, IntoResponse, Method, Request, Response, Result, Transform,
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
        H: Handler<Request, Output = Result<O>> + Clone,
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
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        self.on("", Method::GET, handler)
    }

    pub fn new<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        self.on("new", Method::GET, handler)
    }

    pub fn create<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        self.on("", Method::POST, handler)
    }

    pub fn show<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id", self.name);
        self.on(path, Method::GET, handler)
    }

    pub fn edit<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id/edit", self.name);
        self.on(path, Method::GET, handler)
    }

    pub fn update<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id", self.name);
        self.on(path, Method::PUT, handler)
    }

    pub fn update_with_patch<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id", self.name);
        self.on(path, Method::PATCH, handler)
    }

    pub fn destroy<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        let path = format!(":{}_id", self.name);
        self.on(path, Method::DELETE, handler)
    }

    pub fn with<T>(self, t: T) -> Self
    where
        T: Transform<BoxHandler>,
        T::Output: Handler<Request, Output = Result<Response>>,
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
    use crate::{get, Resource};
    use viz_core::{
        async_trait, Handler, HandlerExt, IntoResponse, Method, Next, Request, Response, Result,
        Transform,
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
        impl<H> Handler<Request> for LoggerHandler<H>
        where
            H: Handler<Request> + Clone,
        {
            type Output = H::Output;

            async fn call(&self, req: Request) -> Self::Output {
                let res = self.0.call(req).await;
                res
            }
        }

        async fn before(req: Request) -> Result<Request> {
            Ok(req)
        }

        async fn after(res: Result<Response>) -> Result<Response> {
            res
        }

        async fn around<H, O>((req, handler): Next<Request, H>) -> Result<Response>
        where
            H: Handler<Request, Output = Result<O>> + Clone,
            O: IntoResponse + Send + Sync + 'static,
        {
            let res = handler.call(req).await.map(IntoResponse::into_response);
            res
        }

        async fn index(_: Request) -> Result<impl IntoResponse> {
            Ok("index")
        }

        async fn any(_: Request) -> Result<&'static str> {
            Ok("any")
        }

        async fn index_posts(_: Request) -> Result<Vec<u8>> {
            Ok(b"index posts".to_vec())
        }

        async fn create_post(_: Request) -> Result<impl IntoResponse> {
            Ok("create post")
        }

        async fn new_post(_: Request) -> Result<Response> {
            Ok(Response::new("new post".into()))
        }

        async fn show_post(_: Request) -> Result<Response> {
            Ok(Response::new("show post".into()))
        }

        async fn edit_post(_: Request) -> Result<Response> {
            Ok(Response::new("edit post".into()))
        }

        async fn delete_post(_: Request) -> Result<Response> {
            Ok(Response::new("delete post".into()))
        }

        async fn update_post(_: Request) -> Result<Response> {
            Ok(Response::new("update post".into()))
        }

        async fn any_posts(_: Request) -> Result<Response> {
            Ok(Response::new("any posts".into()))
        }

        async fn search_posts(_: Request) -> Result<Response> {
            Ok(Response::new("search posts".into()))
        }

        let resource = Resource::default()
            .index(index)
            .update_with_patch(any_posts);

        assert_eq!(2, resource.into_iter().count());

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
            .update_with_patch(any)
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

        assert_eq!(5, resource.clone().into_iter().count());
        assert_eq!(
            9,
            resource
                .clone()
                .into_iter()
                .fold(0, |sum, (_, r)| sum + r.into_iter().count())
        );

        let (_, h) = resource
            .routes
            .iter()
            .filter(|(p, _)| p == ":post_id")
            .nth(0)
            .and_then(|(_, r)| r.methods.iter().filter(|(m, _)| m == Method::GET).nth(0))
            .unwrap();

        let res = h.call(Request::default()).await?;
        assert_eq!(hyper::body::to_bytes(res.into_body()).await?, "show post");

        Ok(())
    }
}
