//! Authentication
//! Thanks: <https://ec.haxx.se/http/http-auth>

#[cfg(feature = "auth-basic")]
mod basic;
#[cfg(feature = "auth-basic")]
pub use basic::Basic;

#[cfg(feature = "auth-bearer")]
mod bearer;
#[cfg(feature = "auth-bearer")]
pub use bearer::Bearer;
