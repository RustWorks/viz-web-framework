//! Authentication
//! https://ec.haxx.se/http/http-auth

mod basic;
mod bearer;

pub use basic::BasicMiddleware;
pub use bearer::BearerMiddleware;
