//! An HTTP server based on `hyper`.
//!
//! Run with:
//!
//! ```
//! cargo run --example hello-smol --release
//! ```
//!
//! Open in the browser any of these addresses:
//!
//! - http://localhost:8000/

use std::{
    collections::HashMap,
    future::Future,
    io,
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    // Multi-Threaded
    // thread,
};

use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use ramhorns::{Content, Template};
use serde::{Deserialize, Serialize};

use viz_core::{
    http, into_guard, Context as VizContext, Error, Extract, Params, Response as VizResponse,
    Result,
};

use viz_router::{route, router, Method, Tree};

use viz_utils::{
    anyhow::anyhow,
    futures::{
        // Multi-Threaded
        // future::{self, BoxFuture},
        future::BoxFuture,
        io::{AsyncRead, AsyncWrite},
        stream::Stream,
    },
    log, pretty_env_logger,
    smol::{self, Async, Task},
};

// Standard Mustache action here
const SOURCE: &str = "<h1>{{method}}</h1><h2>{{path}}</h2><p>{{params}}</p>";

#[derive(Serialize, Deserialize, Debug, Content)]
struct Info {
    method: String,
    path: String,
}

impl Info {
    fn render(&self) -> String {
        Template::new(SOURCE).unwrap().render(self)
    }
}

impl Extract for Info {
    type Error = Error;

    fn extract<'a>(cx: &'a mut VizContext) -> BoxFuture<'a, Result<Self, Self::Error>> {
        Box::pin(async move {
            let method = cx.method().to_string();
            if method == "PUT" {
                return Err(anyhow!("Wrong HTTP Method!"));
            }
            Ok(Info {
                method,
                path: cx.path().to_string(),
            })
        })
    }
}

/// Listens for incoming connections and serves them.
async fn listen(listener: Async<TcpListener>) -> Result<()> {
    let users = router()
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
        );

    let user_posts = router()
        .mid(m_1)
        // .carry(false)
        .mid(m_2)
        // .at("/*any", route().all(any))
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
                        .guard(into_guard(edit_guard) & into_guard(get_guard))
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
        );

    let posts = router()
        // `/posts`
        .route(route().get(index_posts))
        // `/posts/:id`
        .at("/:id", route().get(show_post));

    let comments = router()
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
                    route()
                        .get(show_comment)
                        .patch(update_comment)
                        .put(update_comment),
                )
                // `/comments/:id/edit`
                .at("/edit", route().get(edit_comment)),
        );

    let routes = router()
        .mid(m_0)
        // `/`
        .at("/", route().get(hello_world))
        .at("/*any", route().all(any))
        // `/users'
        .scope("/users", users)
        // `/users/:user_id/posts'
        .scope("/users/:user_id/posts", user_posts)
        // `/posts`
        .scope("/posts", posts)
        // `/comments`
        .scope("/comments", comments);

    let mut tree = HashMap::new();

    routes.finish(&mut tree);

    let tree = Arc::new(tree);

    /// Serves a request and returns a response.
    async fn serve(req: http::Request, tree: Arc<Tree>) -> Result<http::Response> {
        let mut cx: VizContext = req.into();
        let method = cx.method().to_owned();
        let path = cx.path();

        log::info!("{}:{}", &method, &path);
        let route = tree
            .get(&Method::Verb(method.to_owned()))
            .and_then(|t| t.find(path))
            .or_else(|| {
                if method == http::Method::HEAD {
                    tree.get(&Method::Verb(http::Method::GET))
                        .and_then(|t| t.find(path))
                } else {
                    None
                }
            })
            .or_else(|| tree.get(&Method::All).and_then(|t| t.find(path)));

        if let Some((handler, params)) = route {
            let params: Params = params.into();
            *cx.middleware_mut() = handler.to_owned();
            cx.extensions_mut().insert(params);
        }

        Ok(cx.next().await?.into())
    }

    // Start a hyper server.
    Server::builder(SmolListener::new(listener))
        .executor(SmolExecutor)
        .serve(make_service_fn(move |_| {
            // let middleware = middleware.clone();
            // async { Ok::<_, Error>(service_fn(move |req| serve(req, middleware.clone()))) }
            let tree = tree.clone();
            async move { Ok::<_, Error>(service_fn(move |req| serve(req, tree.clone()))) }
        }))
        .await?;

    Ok(())
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let addr: SocketAddr = ([127, 0, 0, 1], 8000).into();

    println!("Listening on http://{}", addr);

    // Start HTTP server.
    // Multi-Threaded
    // for _ in 0..num_cpus::get().max(1) {
    //     thread::spawn(|| smol::run(future::pending::<()>()));
    // }
    // smol::block_on(listen(Async::<TcpListener>::bind(&addr)?))

    // Single-Threaded
    smol::run(listen(Async::<TcpListener>::bind(&addr)?))
}

/// Spawns futures.
#[derive(Clone)]
struct SmolExecutor;

impl<F: Future + Send + 'static> hyper::rt::Executor<F> for SmolExecutor {
    fn execute(&self, fut: F) {
        Task::spawn(async { drop(fut.await) }).detach();
    }
}

/// Listens for incoming connections.
struct SmolListener {
    listener: Async<TcpListener>,
}

impl SmolListener {
    fn new(listener: Async<TcpListener>) -> Self {
        Self { listener }
    }
}

impl hyper::server::accept::Accept for SmolListener {
    type Conn = SmolStream;
    type Error = Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let poll = Pin::new(&mut self.listener.incoming()).poll_next(cx);
        let stream = futures::ready!(poll).unwrap()?;
        Poll::Ready(Some(Ok(SmolStream(stream))))
    }
}

/// A TCP connection.
struct SmolStream(Async<TcpStream>);

impl hyper::client::connect::Connection for SmolStream {
    fn connected(&self) -> hyper::client::connect::Connected {
        hyper::client::connect::Connected::new()
    }
}

impl tokio::io::AsyncRead for SmolStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for SmolStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.0.get_ref().shutdown(Shutdown::Write)?;
        Poll::Ready(Ok(()))
    }
}

fn edit_guard(_cx: &VizContext) -> bool {
    true
}

fn get_guard(_cx: &VizContext) -> bool {
    false
}

async fn m_0(cx: &mut VizContext) -> Result<VizResponse> {
    // println!("middleware 0");
    cx.next().await
}

async fn m_1(cx: &mut VizContext) -> Result<VizResponse> {
    // println!("middleware 1");
    cx.next().await
}

async fn m_2(cx: &mut VizContext) -> Result<VizResponse> {
    // println!("middleware 2");
    cx.next().await
}

// `/`
async fn hello_world() -> &'static str {
    "Hello, world!"
}

// `/*`
async fn any(info: Info) -> String {
    // format!("{:#?}", info)
    // "* any!"
    info.render()
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

// async fn edit_post() -> &'static str {
async fn edit_post(params: Params<(String, u32)>) -> String {
    // "Edit post"
    format!("{:#?}", params)
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
