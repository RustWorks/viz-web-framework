pub trait Transform<H> {
    type Output;

    fn transform(&self, h: H) -> Self::Output;
}
