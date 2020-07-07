mod logger;
mod recover;
mod request_id;

pub use logger::LoggerMiddleware;
pub use recover::RecoverMiddleware;
pub use request_id::RequestIDMiddleware;

pub fn logger() -> LoggerMiddleware {
    LoggerMiddleware::default()
}

pub fn recover() -> RecoverMiddleware {
    RecoverMiddleware::default()
}

pub fn request_id() -> RequestIDMiddleware {
    RequestIDMiddleware::default()
}
