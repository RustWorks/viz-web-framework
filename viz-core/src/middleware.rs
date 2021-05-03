use std::sync::Arc;

use crate::{Context, Middleware, Result};

/// Dyn Middleware
pub type DynMiddleware = dyn for<'a> Middleware<'a, Context, Output = Result>;

/// Middleware List
pub type Middlewares = Vec<Arc<DynMiddleware>>;
