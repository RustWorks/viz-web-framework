mod logger;
mod recover;
mod request_id;
mod timeout;
mod cookies;

pub mod auth;
pub mod cors;
pub mod session;
pub mod compression;

pub use logger::LoggerMiddleware;
pub use recover::RecoverMiddleware;
pub use request_id::RequestIDMiddleware;
pub use timeout::TimeoutMiddleware;
pub use cookies::CookiesMiddleware;

pub fn logger() -> LoggerMiddleware {
    LoggerMiddleware::default()
}

pub fn recover() -> RecoverMiddleware {
    RecoverMiddleware::default()
}

pub fn request_id() -> RequestIDMiddleware {
    RequestIDMiddleware::default()
}

pub fn timeout() -> TimeoutMiddleware {
    TimeoutMiddleware::default()
}

pub fn cookies() -> CookiesMiddleware {
    CookiesMiddleware::default()
}
