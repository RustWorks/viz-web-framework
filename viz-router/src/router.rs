use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use viz_core::{Context, Middleware, Middlewares, Response, Result};
use viz_utils::log;

use crate::{Method, PathTree, Route, RouteHandler};

pub struct Router {
    // inherit parrent's middleware
    carry: bool,
    path: String,
    name: Option<String>,
    routes: Option<Vec<Route>>,
    children: Option<Vec<Router>>,
    middleware: Option<Middlewares>,
}

impl Router {
    pub fn new(path: &str) -> Self {
        Self {
            name: None,
            carry: true,
            routes: None,
            children: None,
            middleware: None,
            path: path.to_owned(),
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
        M: for<'a> Middleware<'a, Context, Output = Result<Response>>,
    {
        self.middleware.get_or_insert_with(Vec::new).insert(0, Arc::new(m));
        self
    }

    pub fn scope(mut self, path: &str, router: Router) -> Self {
        self.children.get_or_insert_with(Vec::new).push(router.path(path));
        self
    }

    pub fn route(mut self, route: Route) -> Self {
        self.routes.get_or_insert_with(Vec::new).push(route);
        self
    }

    pub fn at(mut self, path: &str, route: Route) -> Self {
        self.routes.get_or_insert_with(Vec::new).push(route.path(path));
        self
    }

    pub fn finish(mut self, tree: &mut HashMap<Method, PathTree<Middlewares>>) {
        let m0 = self.middleware.take().unwrap_or_default();
        let h0 = m0.len() > 0;

        if let Some(routes) = self.routes.take() {
            for mut route in routes {
                let m1 = route.middleware.take().unwrap_or_default();
                let carry = route.carry;
                let guard = route.guard.take().map(|g| Arc::new(g));
                let path = join_paths(&self.path, &route.path);

                for (method, handler) in route.handlers {
                    log::info!("{:>6}:{}", method.as_str(), &path);

                    let mut m = vec![if let Some(guard) = guard.clone().take() {
                        Arc::new(RouteHandler::new(guard, handler))
                    } else {
                        handler
                    }];

                    if m1.len() > 0 {
                        m.extend_from_slice(&m1);
                    }

                    if h0 && carry {
                        m.extend_from_slice(&m0);
                    }

                    tree.entry(method).or_insert_with(PathTree::new).insert(&path, m);
                }
            }
        }

        if let Some(children) = self.children.take() {
            for mut child in children {
                let path = join_paths(&self.path, &child.path);
                // log::debug!("{}", &path);
                child.path = path;
                if h0 && child.carry {
                    child.middleware.get_or_insert_with(Vec::new).extend_from_slice(&m0);
                }
                child.finish(tree);
            }
        }

        //         if h0 && self.path.is_empty() {
        //             let method = Method::All;
        //             let path = "/*";

        //             log::info!("{:>6}:{}", method.as_str(), &path);

        //             tree.entry(method)
        //                 .or_insert_with(PathTree::new)
        //                 .insert(path, m0);
        //         }
    }
}

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
            .field("carry", &self.carry)
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

    use viz_core::{http, into_guard, Context, Error, Response, Result};

    use crate::*;

    #[test]
    fn routing() {
        viz_utils::pretty_env_logger::init();

        let routes = router()
            .mid(m_0)
            // `/`
            .route(route().path("/").get(hello_world))
            .at("/*any", route().all(any))
            // `/users`
            .scope(
                "/users",
                router()
                    .mid(m_1)
                    // `/users`
                    .route(route().get(index_users).post(create_user))
                    // `/users/new`
                    .at("/new", route().guard(edit_guard).get(new_user))
                    // `/users/:id`
                    .scope(
                        "/:id",
                        router()
                            // `/users/:id`
                            .route(
                                route()
                                    .get(show_user)
                                    .patch(update_user)
                                    .put(update_user)
                                    .delete(delete_user),
                            )
                            // `/users/:id/edit`
                            .at("/edit", route().guard(edit_guard).get(edit_user))
                            // `/users/*``
                            .at("*any", route().all(any)),
                    )
                    .scope(
                        "/:user_id",
                        router()
                            // .path("/posts")
                            // `/users/:user_id/posts`
                            .scope(
                                "/posts",
                                router()
                                    .carry(false)
                                    .mid(m_2)
                                    .at("/*any", route().all(any))
                                    // `/users/:user_id/posts`
                                    .route(route().get(index_posts).post(create_post))
                                    // `/users/:user_id/posts/new`
                                    .at("/new", route().guard(edit_guard).get(new_post))
                                    // `/users/:user_id/posts/:id`
                                    .scope(
                                        "/:id",
                                        router()
                                            // `/users/:user_id/posts/:id`
                                            .route(
                                                route()
                                                    .guard(
                                                        into_guard(edit_guard)
                                                            & into_guard(get_guard),
                                                    )
                                                    .get(show_post),
                                            )
                                            .route(
                                                route()
                                                    // .get(show_post)
                                                    .patch(update_post)
                                                    .put(update_post)
                                                    .delete(delete_post),
                                            )
                                            // `/users/:user_id/posts/:id/edit`
                                            .at("/edit", route().guard(edit_guard).get(edit_post)),
                                    ),
                            ),
                    ),
            )
            // `/posts`
            .scope(
                "/posts",
                router()
                    // `/posts`
                    .route(route().get(index_posts))
                    // `/posts/:id`
                    .at("/:id", route().get(show_post)),
            )
            // `/comments`
            .scope(
                "/comments",
                router()
                    // `/comments`
                    .route(route().get(index_comments).post(create_comment))
                    // `/comments/new`
                    .at("/new", route().guard(edit_guard).get(new_comment))
                    // `/comments/:id`
                    .scope(
                        "/:id",
                        router()
                            // `/comments/:id`
                            .route(
                                route().get(show_comment).patch(update_comment).put(update_comment),
                            )
                            // `/comments/:id/edit`
                            .at("/edit", route().get(edit_comment)),
                    ),
            );

        let mut tree = HashMap::new();

        routes.finish(&mut tree);

        for i in 0..3 {
            println!("");
            println!("request {}", i);

            let tr = tree.get(&Method::Verb(http::Method::GET));

            if let Some(t) = tr {
                // Open Edit User Page
                let mut req = http::Request::default();
                *req.uri_mut() = "/users/fundon/edit".parse().unwrap();
                *req.body_mut() = "Open Edit User Page".into();

                println!("");
                println!("request {} {}", i, req.uri());

                if let Some(r) = t.find(&req.uri().to_string()) {
                    let mut cx = Context::from(req);

                    *cx.middleware_mut() = r.0.to_vec();

                    assert_eq!(r.1, vec![("id", "fundon")]);

                    assert!(block_on(async move {
                        let res: http::Response = cx.next().await?.into();

                        assert_eq!(res.status(), 200);
                        assert_eq!(to_bytes(res.into_body()).await.unwrap(), "Edit user");

                        Ok::<_, Error>(())
                    })
                    .is_ok());
                }

                // Open Edit Post Page
                let mut req = http::Request::default();
                *req.uri_mut() = "/users/fundon/posts/233/edit".parse().unwrap();
                *req.body_mut() = "Open Edit Post Page".into();

                println!("");
                println!("request {} {}", i, req.uri());

                if let Some(r) = t.find(&req.uri().to_string()) {
                    let mut cx = Context::from(req);

                    *cx.middleware_mut() = r.0.to_vec();

                    assert_eq!(r.1, vec![("user_id", "fundon"), ("id", "233")]);

                    assert!(block_on(async move {
                        let res: http::Response = cx.next().await?.into();

                        assert_eq!(res.status(), 200);
                        assert_eq!(to_bytes(res.into_body()).await.unwrap(), "Edit post");

                        Ok::<_, Error>(())
                    })
                    .is_ok());
                }

                // Open Get Post Page
                let mut req = http::Request::default();
                *req.uri_mut() = "/users/fundon/posts/233".parse().unwrap();
                *req.body_mut() = "Open Get Post Page".into();

                println!("");
                println!("request {} {}", i, req.uri());

                if let Some(r) = t.find(&req.uri().to_string()) {
                    let mut cx = Context::from(req);

                    *cx.middleware_mut() = r.0.to_vec();

                    assert_eq!(r.1, vec![("user_id", "fundon"), ("id", "233")]);

                    assert!(block_on(async move {
                        let res: http::Response = cx.next().await?.into();

                        assert_eq!(res.status(), 404);
                        assert_eq!(to_bytes(res.into_body()).await.unwrap(), "");

                        Ok::<_, Error>(())
                    })
                    .is_ok());
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

        fn edit_guard(_cx: &Context) -> bool {
            true
        }

        fn get_guard(_cx: &Context) -> bool {
            false
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
