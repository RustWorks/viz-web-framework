mod logger;
mod recover;

pub use logger::LoggerMiddleware;
pub use recover::RecoverMiddleware;

pub fn logger() -> LoggerMiddleware {
    LoggerMiddleware::default()
}

pub fn recover() -> RecoverMiddleware {
    RecoverMiddleware::default()
}
