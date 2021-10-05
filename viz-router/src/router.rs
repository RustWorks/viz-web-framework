use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use viz_core::{Context, Middleware, Response, Result, VecMiddleware};
use viz_utils::tracing;

use crate::{Method, PathTree, Route};

/// Router
pub struct Router {
    // inherit parrent's middleware
    inherit: bool,
    path: String,
    name: Option<String>,
    routes: Option<Vec<Route>>,
    children: Option<Vec<Router>>,
    middleware: Option<VecMiddleware>,
}

impl Router {
    /// Creates new Router with path.
    pub fn new(path: &str) -> Self {
        Self {
            name: None,
            inherit: true,
            routes: None,
            children: None,
            middleware: None,
            path: path.to_owned(),
        }
    }

    /// Prefix path of the Router.
    pub fn path(mut self, path: &str) -> Self {
        self.path.insert_str(0, path);
        self
    }

    /// Name of the Router.
    pub fn name(mut self, name: &str) -> Self {
        self.name.replace(name.to_owned());
        self
    }

    /// Inherits parrent's middleware
    pub fn inherit(mut self, b: bool) -> Self {
        self.inherit = b;
        self
    }

    /// Appends middleware to the Router.
    pub fn with<M>(mut self, m: M) -> Self
    where
        M: for<'a> Middleware<'a, Context, Output = Result<Response>>,
    {
        self.middleware.get_or_insert_with(Vec::new).insert(0, Arc::new(m));
        self
    }

    /// Creates a scope Router with path.
    pub fn scope(mut self, path: &str, router: Router) -> Self {
        self.children.get_or_insert_with(Vec::new).push(router.path(path));
        self
    }

    /// Appends a route.
    pub fn route(mut self, route: Route) -> Self {
        self.routes.get_or_insert_with(Vec::new).push(route);
        self
    }

    /// Appends a route with path.
    pub fn at(mut self, path: &str, route: Route) -> Self {
        self.routes.get_or_insert_with(Vec::new).push(route.path(path));
        self
    }

    /// Outputs the routes to map.
    pub fn finish(mut self, tree: &mut HashMap<Method, PathTree<VecMiddleware>>) {
        let m0 = self.middleware.take().unwrap_or_default();
        let h0 = !m0.is_empty();

        if let Some(routes) = self.routes.take() {
            for mut route in routes {
                let m1 = route.middleware.take().unwrap_or_default();
                let inherit = route.inherit;
                let path = join_paths(&self.path, &route.path);

                for (method, handler) in route.handlers {
                    tracing::info!("{:>6}:{}", method.as_str(), &path);

                    let mut m = vec![handler];

                    if !m1.is_empty() {
                        m.extend_from_slice(&m1);
                    }

                    if h0 && inherit {
                        m.extend_from_slice(&m0);
                    }

                    tree.entry(method).or_insert_with(PathTree::new).insert(&path, m);
                }
            }
        }

        if let Some(children) = self.children.take() {
            for mut child in children {
                let path = join_paths(&self.path, &child.path);
                child.path = path;
                if h0 && child.inherit {
                    child.middleware.get_or_insert_with(Vec::new).extend_from_slice(&m0);
                }
                child.finish(tree);
            }
        }
    }
}

/// Creates new Router with empty path
pub fn router() -> Router {
    Router::new("")
}

pub(crate) fn join_paths(a: &str, b: &str) -> String {
    if b.is_empty() {
        return a.to_owned();
    }
    a.trim_end_matches('/').to_owned() + "/" + b.trim_start_matches('/')
}

impl fmt::Debug for Router {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Router")
            .field("path", &self.path)
            .field("name", &self.name.as_ref().map_or_else(String::new, |v| v.to_owned()))
            .field("inherit", &self.inherit)
            .field("middle", &self.middleware.as_ref().map_or_else(|| 0, |v| v.len()))
            .field("routes", &self.routes.as_ref().map_or_else(|| &[][..], |v| v))
            .field("children", &self.children.as_ref().map_or_else(|| &[][..], |v| v))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use futures_executor::block_on;
    use hyper::body::to_bytes;

    use viz_core::{http, Context, Error, Response, Result};

    use crate::*;

    #[test]
    fn routing() {
        let routes = router()
            .with(m_0)
            // `/`
            .route(route("/").get(hello_world))
            .at("/*any", all(any))
            // `/users`
            .scope(
                "/users",
                router()
                    .with(m_1)
                    // `/users`
                    .route(get(index_users).post(create_user))
                    // `/users/new`
                    .at("/new", get(new_user))
                    // `/users/:id`
                    .scope(
                        "/:id",
                        router()
                            // `/users/:id`
                            .route(
                                    get(show_user)
                                    .patch(update_user)
                                    .put(update_user)
                                    .delete(delete_user),
                            )
                            // `/users/:id/edit`
                            .at("/edit", get(edit_user))
                            // `/users/*``
                            .at("*any", all(any)),
                    )
                    .scope(
                        "/:user_id",
                        router()
                            // .path("/posts")
                            // `/users/:user_id/posts`
                            .scope(
                                "/posts",
                                router()
                                    .inherit(false)
                                    .with(m_2)
                                    .at("/*any", all(any))
                                    // `/users/:user_id/posts`
                                    .route(get(index_posts).post(create_post))
                                    // `/users/:user_id/posts/new`
                                    .at("/new", get(new_post))
                                    // `/users/:user_id/posts/:id`
                                    .scope(
                                        "/:id",
                                        router()
                                            // `/users/:user_id/posts/:id`
                                            .route(
                                                    get(show_post),
                                            )
                                            .route(
                                                    // get(show_post)
                                                    patch(update_post)
                                                    .put(update_post)
                                                    .delete(delete_post),
                                            )
                                            // `/users/:user_id/posts/:id/edit`
                                            .at("/edit", get(edit_post)),
                                    ),
                            ),
                    ),
            )
            // `/posts`
            .scope(
                "/posts",
                router()
                    // `/posts`
                    .route(get(index_posts))
                    // `/posts/:id`
                    .at("/:id", get(show_post)),
            )
            // `/comments`
            .scope(
                "/comments",
                router()
                    // `/comments`
                    .route(get(index_comments).post(create_comment))
                    // `/comments/new`
                    .at("/new", get(new_comment))
                    // `/comments/:id`
                    .scope(
                        "/:id",
                        router()
                            // `/comments/:id`
                            .route(
                                get(show_comment).patch(update_comment).put(update_comment),
                            )
                            // `/comments/:id/edit`
                            .at("/edit", get(edit_comment)),
                    ),
            );

        let mut tree = HashMap::new();

        routes.finish(&mut tree);

        for i in 0..3 {
            println!();
            println!("request {}", i);

            let tr = tree.get(&Method::Verb(http::Method::GET));

            if let Some(t) = tr {
                // Open Edit User Page
                let mut req = http::Request::default();
                *req.uri_mut() = "/users/fundon/edit".parse().unwrap();
                *req.body_mut() = "Open Edit User Page".into();

                println!();
                println!("request {} {}", i, req.uri());

                if let Some(r) = t.find(&req.uri().to_string()) {
                    let mut cx = Context::from(req);

                    *cx.middleware_mut() = r.0.to_vec();

                    assert_eq!(r.1, vec![("id", "fundon")]);

                    assert!(
                        block_on(async move {
                            let res: http::Response = cx.next().await?.into();

                            assert_eq!(res.status(), 200);
                            assert_eq!(to_bytes(res.into_body()).await.unwrap(), "Edit user");

                            Ok::<_, Error>(())
                        })
                        .is_ok()
                    );
                }

                // Open Edit Post Page
                let mut req = http::Request::default();
                *req.uri_mut() = "/users/fundon/posts/233/edit".parse().unwrap();
                *req.body_mut() = "Open Edit Post Page".into();

                println!();
                println!("request {} {}", i, req.uri());

                if let Some(r) = t.find(&req.uri().to_string()) {
                    let mut cx = Context::from(req);

                    *cx.middleware_mut() = r.0.to_vec();

                    assert_eq!(r.1, vec![("user_id", "fundon"), ("id", "233")]);

                    assert!(
                        block_on(async move {
                            let res: http::Response = cx.next().await?.into();

                            assert_eq!(res.status(), 200);
                            assert_eq!(to_bytes(res.into_body()).await.unwrap(), "Edit post");

                            Ok::<_, Error>(())
                        })
                        .is_ok()
                    );
                }

                // Open Get Post Page
                let mut req = http::Request::default();
                *req.uri_mut() = "/users/fundon/posts/233".parse().unwrap();
                *req.body_mut() = "Open Get Post Page".into();

                println!();
                println!("request {} {} {}", i, req.uri(), req.method());

                if let Some(r) = t.find(&req.uri().to_string()) {
                    let mut cx = Context::from(req);

                    *cx.middleware_mut() = r.0.to_vec();

                    assert_eq!(r.1, vec![("user_id", "fundon"), ("id", "233")]);

                    assert!(
                        block_on(async move {
                            let res: http::Response = cx.next().await?.into();

                            assert_eq!(res.status(), 200);
                            assert_eq!(to_bytes(res.into_body()).await.unwrap(), "Show post");

                            Ok::<_, Error>(())
                        })
                        .is_ok()
                    );
                }
            }

            // let tr = tree.get(&Method::All);

            // if let Some(t) = tr {
            //     if let Some(r) = t.find("/user") {
            //         dbg!(r.1);
            //     }
            //     if let Some(r) = t.find("/users/fundon/edit") {
            //         dbg!(r.1);
            //     }
            //     if let Some(r) = t.find("/users/fundon/posts/233/edit") {
            //         dbg!(r.1);
            //     }
            // }
        }

        async fn m_0(cx: &mut Context) -> Result<Response> {
            println!("middleware 0");
            cx.next().await
        }

        async fn m_1(cx: &mut Context) -> Result<Response> {
            println!("middleware 1");
            cx.next().await
        }

        async fn m_2(cx: &mut Context) -> Result<Response> {
            println!("middleware 2");
            cx.next().await
        }

        // `/`
        async fn hello_world() -> &'static str {
            "Hello, world!"
        }

        // `/*`
        async fn any() -> &'static str {
            "* any!"
        }

        // `/users`
        // -----------------
        async fn new_user() -> &'static str {
            "New user"
        }

        async fn edit_user() -> &'static str {
            "Edit user"
        }

        async fn index_users() -> &'static str {
            "Index users"
        }

        async fn create_user() -> &'static str {
            "Create user"
        }

        async fn show_user() -> &'static str {
            "Show user"
        }

        async fn update_user() -> &'static str {
            "Update user"
        }

        async fn delete_user() -> &'static str {
            "Delete user"
        }
        // -----------------

        // `/posts`
        // -----------------
        async fn new_post() -> &'static str {
            "New post"
        }

        async fn edit_post() -> &'static str {
            "Edit post"
        }

        async fn index_posts() -> &'static str {
            "Index posts"
        }

        async fn create_post() -> &'static str {
            "Create post"
        }

        async fn show_post() -> &'static str {
            "Show post"
        }

        async fn update_post() -> &'static str {
            "Update post"
        }

        async fn delete_post() -> &'static str {
            "Delete post"
        }
        // -----------------

        // `/comments`
        // -----------------
        async fn new_comment() -> &'static str {
            "New comment"
        }

        async fn edit_comment() -> &'static str {
            "Edit comment"
        }

        async fn index_comments() -> &'static str {
            "Index comments"
        }

        async fn create_comment() -> &'static str {
            "Create comment"
        }

        async fn show_comment() -> &'static str {
            "Show comment"
        }

        async fn update_comment() -> &'static str {
            "Update comment"
        }
        // -----------------
    }
}
