use std::sync::Arc;

use crate::{Context, Response, Result};

pub use handle::Handle as Middleware;

/// Dyn Middleware
pub type DynMiddleware = dyn for<'a> Middleware<'a, Context, Output = Result<Response>>;

/// Middleware List
pub type Middlewares = Vec<Arc<DynMiddleware>>;
