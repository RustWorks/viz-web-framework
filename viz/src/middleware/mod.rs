mod logger;
mod recover;
mod request_id;
mod session;
mod timeout;

pub use logger::LoggerMiddleware;
pub use recover::RecoverMiddleware;
pub use request_id::RequestIDMiddleware;
pub use session::SessionMiddleware;
pub use timeout::TimeoutMiddleware;

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

pub fn session<Store>() -> SessionMiddleware<Store> {
    SessionMiddleware::default()
}
