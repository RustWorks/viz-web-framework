//! Router for Viz Web Framework

#![forbid(unsafe_code)]

pub use path_tree::PathTree;

mod resource;
mod route;
mod router;
mod tree;

pub use resource::Resource;
pub use route::*;
pub use router::Router;
pub use tree::Tree;
