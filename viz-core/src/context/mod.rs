mod from_context;

pub use from_context::FromContext;

#[derive(Clone)]
pub struct Context {}

impl Context {
    pub fn new() -> Self {
        Self {}
    }
}
