#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

//! Viz Router

/// Tree
pub type Tree = std::collections::HashMap<Method, PathTree<viz_core::VecMiddleware>>;

mod method;
mod route;
mod router;

pub use method::Method;
pub use path_tree::PathTree;
pub use route::*;
pub use router::{router, Router};
