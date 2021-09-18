//! Authentication
//! Thanks: <https://ec.haxx.se/http/http-auth>

mod basic;
mod bearer;

pub use basic::Basic;
pub use bearer::Bearer;
