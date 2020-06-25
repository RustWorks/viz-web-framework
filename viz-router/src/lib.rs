#![deny(unsafe_code)]

use std::collections::HashMap;

use viz_core::Middlewares;

pub type Tree = HashMap<Method, PathTree<Middlewares>>;

mod handler;
mod method;
mod route;
mod router;

pub use handler::RouteHandler;
pub use method::Method;
pub use path_tree::PathTree;
pub use route::{route, Route};
pub use router::{router, Router};
