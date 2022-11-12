/// Unified TLS listener type.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Listener<T, A> {
    pub(crate) inner: T,
    pub(crate) acceptor: A,
}

impl<T, A> Listener<T, A> {
    /// Creates a new TLS listener.
    pub fn new(t: T, a: A) -> Self {
        Self {
            inner: t,
            acceptor: a,
        }
    }
}
