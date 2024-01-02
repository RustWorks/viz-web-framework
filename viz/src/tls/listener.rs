/// Unified TLS listener type.
#[derive(Debug)]
pub struct TlsListener<T, A> {
    pub(crate) inner: T,
    pub(crate) acceptor: A,
}

impl<T, A> TlsListener<T, A> {
    /// Creates a new TLS listener.
    pub fn new(t: T, a: A) -> Self {
        Self {
            inner: t,
            acceptor: a,
        }
    }
}
