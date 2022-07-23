//! Fast, robust, flexible, lightweight web framework for Rust.
//!
//! # Features
//!
//! * **Safety** `#![forbid(unsafe_code)]`
//!
//! * Lightweight
//!
//! * Robust [`Routing`](#routing)
//!
//! * Handy `Extractors`
//!
//! * Simple + Flexible `Handler` & `Middleware`
//!
//! # Hello Viz
//!
//! ```rust,ignore
//! use std::net::SocketAddr;
//! use viz::{get, Request, Result, Router, Server, ServiceMaker};
//!
//! async fn index(_: Request) -> Result<&'static str> {
//!     Ok("Hello Viz")
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//!     println!("listening on {}", addr);
//!
//!     let app = Router::new().route("/", get(index));
//!
//!     if let Err(err) = Server::bind(&addr)
//!         .serve(ServiceMaker::from(app))
//!         .await
//!     {
//!         println!("{}", err);
//!     }
//!
//!     Ok(())
//! }
//! ```
//! More examples can be found [here](https://github.com/viz-rs/viz/tree/main/examples).
//!
//!
//! # Routing
//!
//! The Viz router recognizes URLs and dispatches them to a handler.
//!
//! ## Simple routes
//!
//! ```rust,ignore
//! let root = Router.new()
//!   .route("/", Route::new().get(index))
//!   .route("/about", Route::new().get(about));
//!
//! let search = Router.new()
//!   .route("/", Route::new().get(show_search));
//! ```
//!
//! ## CRUD, Verbs
//!
//! ```rust,ignore
//! let todos = Router::new()
//!   .route("/", get(index_todos).post(create_todo))
//!   .route("/new", post(new_todo))
//!   .route("/:id", get(show_todo).patch(update_todo).delete(destroy_todo))
//!   .route("/:id/edit", get(edit_todo));
//! ```
//!
//! ## Resources
//!
//! ```rust,ignore
//! let users = Resource::default()
//!   .named("users")
//!   .route("/search", get(search_users))
//!   .index(index_users)
//!   .new(new_user)
//!   .create(create_user)
//!   .show(show_user)
//!   .edit(edit_user)
//!   .update(update_user)
//!   .destroy(delete_user);
//! ```
//!
//! ## Nested
//!
//! ```rust,ignore
//! let app = Router::new()
//!   .nest("/", root)
//!   .nest("/search", search)
//!   .nest("/todos", todos.clone())
//!   .nest("/users", users.nest("todos", todos))
//!   .route("/*", any(not_found));
//! ```
//!
//! # Handler
//!
//! ### Simple handlers
//!
//! ```rust,ignore
//! async fn index(_: Request) -> Result<Response> {
//!   Ok(Response::text("Hello, World!"))
//! }
//!
//! async fn not_found(_, Request) -> Result<impl IntoResponse> {
//!   Ok("Not Found!")
//! }
//! ```
//!
//! ## Implemented `Handler` for handlers
//!
//! ```rust,ignore
//! #[derive(Clone, Serialize)]
//! struct About {
//!   title: &'static str,
//!   description: &'static str,
//! }
//!
//! #[async_trait]
//! impl Handler<Request> for About {
//!   type Output = Result<Response>;
//!
//!   async fn call(&self, req: Request) -> Self::Output {
//!     Ok(Response::json(self.clone()))
//!   }
//! }
//! ```
//!
//! ## Add `Extractors` on handlers
//!
//! If defined a `extract-handler`, must use `.to_handler()`, transform it to a basic handler.
//!
//! `eg: .route("/", get(show_todo_ext.to_handler()))`
//!
//! ```rust,ignore
//! async fn show_todo_ext(Params(id): Params<u64>) -> Result<impl IntoResponse> {
//!   Ok(format!("Hi, NO.{}", id))
//! }
//! ```
//!
//! ## Wrap handlers, add more operations
//!
//! ```rust,ignore
//! let posts = Router.new()
//!   .route("/:id", get(show_post.before(filter)));
//! ```
//!
//! ## Middleware
//!
//! ## Extractors
//!

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

mod server;

pub use viz_core::*;
pub use viz_router::*;

// https://github.com/hyperium/hyper/commit/ce72f73464d96fd67b59ceff08fd424733b43ffa#diff-1eaa7c1646ca4a8c2741ab2b4f80d22ab646d8ab031f99925a3adcc3ac242dcd
pub use hyper::server::accept::from_stream as accept_from_stream;
pub use hyper::Server;
pub use server::{ServiceMaker, Stream};

#[cfg(feature = "handlers")]
pub use viz_handlers as handlers;
#[cfg(feature = "macros")]
pub use viz_macros::handler;

#[cfg(any(feature = "rustls", feature = "native-tls"))]
pub mod tls {
    pub use crate::server::tls::*;
}
