use std::marker::PhantomData;

pub struct Listener<T, A, IO> {
    pub(crate) inner: T,
    pub(crate) acceptor: A,
    _marker: PhantomData<IO>,
}

impl<T, A, IO> Listener<T, A, IO> {
    pub fn new(t: T, a: A) -> Self {
        Self {
            inner: t,
            acceptor: a,
            _marker: PhantomData,
        }
    }
}
