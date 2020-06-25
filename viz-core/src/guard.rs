use std::ops::{BitAnd, BitOr, BitXor};

use crate::Context;

pub trait Guard: Send + Sync + 'static {
    fn check(&self, _: &Context) -> bool;
}

impl<F> Guard for F
where
    F: Send + Sync + 'static + Fn(&Context) -> bool,
{
    fn check(&self, cx: &Context) -> bool {
        (self)(cx)
    }
}

impl Guard for Box<dyn Guard> {
    fn check(&self, cx: &Context) -> bool {
        (**self).check(cx)
    }
}

impl<F> From<F> for Box<dyn Guard>
where
    F: Send + Sync + 'static + Fn(&Context) -> bool,
{
    fn from(f: F) -> Self {
        Box::new(f)
    }
}

pub fn into_guard<F>(f: F) -> Box<dyn Guard>
where
    F: Into<Box<dyn Guard>>,
{
    f.into()
}

impl BitAnd for Box<dyn Guard> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Box::new(move |cx: &Context| self.check(cx) & rhs.check(cx))
    }
}

impl BitOr for Box<dyn Guard> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Box::new(move |cx: &Context| self.check(cx) | (rhs.check(cx)))
    }
}

impl BitXor for Box<dyn Guard> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Box::new(move |cx: &Context| self.check(cx) ^ (rhs.check(cx)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{http, Context, Guard};

    #[test]
    fn guard() {
        let a: Box<dyn Guard> = Box::new(|_: &Context| true);
        let b: Box<dyn Guard> = Box::new(|_: &Context| false);
        let c: Box<dyn Guard> = Box::new(|_: &Context| true);
        let d: Box<dyn Guard> = Box::new(|_: &Context| false);
        let e: Box<dyn Guard> = (a & b) ^ (c | d);
        let cx: Context = http::Request::new("hello world".into()).into();
        let res = e.check(&cx);
        assert!(res);
    }
}
