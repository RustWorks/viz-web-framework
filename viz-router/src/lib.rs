//! Router for Viz Web Framework

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(ext))]

mod resource;
mod route;
mod router;
mod tree;

pub use path_tree::PathTree;
pub use resource::Resource;
pub use route::*;
pub use router::Router;
pub use tree::Tree;
