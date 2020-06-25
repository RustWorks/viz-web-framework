use std::sync::Arc;

use crate::{Context, Response, Result};

pub use handle::Handle as Middleware;

pub type DynMiddleware = dyn for<'a> Middleware<'a, Context, Output = Result<Response>>;

pub type Middlewares = Vec<Arc<DynMiddleware>>;
