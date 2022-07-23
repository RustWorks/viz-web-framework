//! Router for Viz Web Framework

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

mod resources;
mod route;
mod router;
mod tree;

pub use path_tree::PathTree;
pub use resources::Resources;
pub use route::*;
pub use router::Router;
pub use tree::Tree;
