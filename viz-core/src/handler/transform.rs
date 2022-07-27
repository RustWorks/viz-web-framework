/// Then `Transform` trait defines the interface of a handler factory that wraps inner handler to
/// a Handler during construction.
pub trait Transform<H> {
    type Output;

    fn transform(&self, h: H) -> Self::Output;
}
