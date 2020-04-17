//! A handle trait for asynchronous context pipeline.
//!
//! Maintain context in multiple handlers.
//!
//! `Pin<&mut 🦀>` Safety!
//! Don't let him/her get away. Stay at home on 2020.
//!
//! Examples
//!
//! ```
//! use handle::Handle;
//! use async_trait::async_trait;
//! use futures::executor::block_on;
//! use std::{future::Future, pin::Pin, sync::Arc};
//!
//! type Result = anyhow::Result<()>;
//! type BoxFuture<'a, T = Result> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
//!
//! struct Context {
//!     index: usize,
//!     middleware: Vec<Box<dyn for<'a> Handle<'a, Context, Result>>>,
//! }
//!
//! impl Context {
//!     async fn next(mut self: Pin<&mut Context>) -> Result {
//!         if let Some(m) = self.middleware.pop() {
//!             m.call(self).await
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! async fn a(mut cx: Pin<&mut Context>) -> Result {
//!     let size = cx.middleware.len();
//!     let repeat = "-".repeat(2 * size);
//!     println!("exec Fn a --{}>> {:>2}", repeat, cx.index);
//!     cx.index += 1;
//!     let fut = cx.as_mut().next().await;
//!     cx.index += 1;
//!     println!("exec Fn a --{}<< {:>2}", repeat, cx.index);
//!     fut
//! }
//!
//! #[derive(Clone)]
//! struct A {
//!     index: usize,
//! }
//!
//! #[async_trait]
//! impl<'a> Handle<'a, Context, Result> for A {
//!     async fn call(&'a self, mut cx: Pin<&'a mut Context>) -> Result {
//!         let size = cx.middleware.len();
//!         let repeat = "-".repeat(2 * size);
//!         println!("exec St A --{}>> {:>2}", repeat, cx.index);
//!         cx.index += self.index;
//!         let fut = cx.as_mut().next().await;
//!         cx.index -= self.index;
//!         println!("exec St A --{}<< {:>2}", repeat, cx.index);
//!         fut
//!     }
//! }
//!
//! #[async_std::main]
//! async fn main() -> Result {
//!     let mut cx = Context {
//!         index: 0,
//!         middleware: vec![Box::new(a), Box::new(A { index: 2 })],
//!     };
//!
//!     let mut cx: Pin<&mut Context> = Pin::new(&mut cx);
//!
//!     let result = cx.as_mut().next().await;
//!     assert!(result.is_ok());
//!     assert_eq!(result.unwrap(), ());
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// A handle trait for asynchronous context pipeline.
#[async_trait]
pub trait Handle<'a, Context, Output>
where
    Self: Send + Sync,
{
    /// Invokes the handler within the given `Context` and then returns `Output`
    async fn call(&'a self, cx: Pin<&'a mut Context>) -> Output;
}

#[async_trait]
impl<'a, Context, Output, F, Fut> Handle<'a, Context, Output> for F
where
    F: Send + Sync + Fn(Pin<&'a mut Context>) -> Fut,
    Fut: Future<Output = Output> + Send + 'a,
    Context: Send + 'a,
    Output: 'a,
{
    #[inline]
    async fn call(&'a self, cx: Pin<&'a mut Context>) -> Output {
        (*self)(cx).await
    }
}

#[cfg(test)]
mod tests {
    use crate::Handle;
    use async_trait::async_trait;
    use futures::executor::block_on;
    use std::{future::Future, pin::Pin, sync::Arc};

    type Result = anyhow::Result<()>;
    type BoxFuture<'a, T = Result> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    struct Context {
        index: usize,
        middleware: Vec<Arc<dyn for<'a> Handle<'a, Context, Result>>>,
    }

    impl Context {
        async fn next(&mut self) -> Result {
            if let Some(m) = self.middleware.pop() {
                m.call(Pin::new(self)).await
            } else {
                Ok(())
            }
        }
    }

    async fn a(mut cx: Pin<&mut Context>) -> Result {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);

        println!("exec Fn a --{}>> {:>2}", repeat, cx.index);

        assert_eq!(cx.index, 0);
        cx.index += 1;
        assert_eq!(cx.index, 1);

        let fut = cx.next().await;

        assert_eq!(cx.index, 1);
        cx.index -= 1;
        assert_eq!(cx.index, 0);

        println!("exec Fn a --{}<< {:>2}", repeat, cx.index);

        fut
    }

    fn b<'a>(mut cx: Pin<&'a mut Context>) -> BoxFuture<'a, Result> {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);

        println!("exec Fn b --{}>> {:>2}", repeat, cx.index);

        assert_eq!(cx.index, 1);
        cx.index += 1;
        assert_eq!(cx.index, 2);

        Box::pin(async move {
            let fut = cx.next().await;

            assert_eq!(cx.index, 2);
            cx.index -= 1;
            assert_eq!(cx.index, 1);

            println!("exec Fn b --{}<< {:>2}", repeat, cx.index);

            fut
        })
    }

    fn c(mut cx: Pin<&mut Context>) -> BoxFuture<'_, Result> {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);

        println!("exec Fn c --{}>> {:>2}", repeat, cx.index);

        assert_eq!(cx.index, 2);
        cx.index += 1;
        assert_eq!(cx.index, 3);

        Box::pin(async move {
            let fut = cx.next().await;

            assert_eq!(cx.index, 3);
            cx.index -= 1;
            assert_eq!(cx.index, 2);

            println!("exec Fn c --{}<< {:>2}", repeat, cx.index);

            fut
        })
    }

    fn d<'a>(mut cx: Pin<&'a mut Context>) -> impl Future<Output = Result> + 'a {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);

        println!("exec Fn d --{}>> {:>2}", repeat, cx.index);

        assert_eq!(cx.index, 3);
        cx.index += 1;
        assert_eq!(cx.index, 4);

        async move {
            let fut = cx.next().await;

            assert_eq!(cx.index, 4);
            cx.index -= 1;
            assert_eq!(cx.index, 3);

            println!("exec Fn d --{}<< {:>2}", repeat, cx.index);

            fut
        }
    }

    fn e(mut cx: Pin<&mut Context>) -> impl Future<Output = Result> + '_ {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);

        println!("exec Fn e --{}>> {:>2}", repeat, cx.index);

        assert_eq!(cx.index, 4);
        cx.index += 1;
        assert_eq!(cx.index, 5);

        async move {
            let fut = cx.next().await;

            assert_eq!(cx.index, 5);
            cx.index -= 1;
            assert_eq!(cx.index, 4);

            println!("exec Fn e --{}<< {:>2}", repeat, cx.index);

            fut
        }
    }

    async fn f(mut cx: Pin<&mut Context>) -> Result {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);

        println!("exec Fn f --{}>> {:>2}", repeat, cx.index);

        assert_eq!(cx.index, 5);
        cx.index += 1;
        assert_eq!(cx.index, 6);

        let fut = cx.next().await;

        assert_eq!(cx.index, 6);
        cx.index -= 1;
        assert_eq!(cx.index, 5);

        println!("exec Fn f --{}<< {:>2}", repeat, cx.index);

        fut
    }

    #[derive(Clone)]
    struct A {
        index: usize,
    }

    #[async_trait]
    impl<'a> Handle<'a, Context, Result> for A {
        async fn call(&'a self, mut cx: Pin<&'a mut Context>) -> Result {
            let size = cx.middleware.len();
            let repeat = "-".repeat(2 * size);

            println!("exec St A --{}>> {:>2}", repeat, cx.index);

            assert_eq!(cx.index, 6);
            cx.index += self.index; // + 1
            assert_eq!(cx.index, 7);

            let fut = cx.next().await;

            assert_eq!(cx.index, 7);
            cx.index -= self.index; // - 1
            assert_eq!(cx.index, 6);

            println!("exec St A --{}<< {:>2}", repeat, cx.index);

            fut
        }
    }

    struct B {
        index: usize,
    }

    #[async_trait]
    impl<'a> Handle<'a, Context, Result> for B {
        async fn call(&'a self, mut cx: Pin<&'a mut Context>) -> Result {
            let size = cx.middleware.len();
            let repeat = "-".repeat(2 * size);

            println!("exec St B --{}>> {:>2}", repeat, cx.index);

            assert_eq!(cx.index, 7);
            cx.index += self.index; // + 2
            assert_eq!(cx.index, 9);

            let fut = cx.next().await;

            assert_eq!(cx.index, 9);
            cx.index -= self.index; // - 2
            assert_eq!(cx.index, 7);

            println!("exec St B --{}<< {:>2}", repeat, cx.index);

            fut
        }
    }

    struct C {
        index: usize,
    }

    #[async_trait]
    impl<'a> Handle<'a, Context, Result> for C {
        async fn call(&'a self, mut cx: Pin<&'a mut Context>) -> Result {
            let size = cx.middleware.len();
            let repeat = "-".repeat(2 * size);

            println!("exec St C --{}>> {:>2}", repeat, cx.index);

            assert_eq!(cx.index, 9);
            cx.index += self.index; // + 3
            assert_eq!(cx.index, 12);

            let fut = cx.next().await;

            assert_eq!(cx.index, 12);
            cx.index -= self.index; // - 3
            assert_eq!(cx.index, 9);

            println!("exec St C --{}<< {:>2}", repeat, cx.index);

            fut
        }
    }

    #[test]
    fn futures_rt() {
        block_on(async move {
            let mut cx = Context {
                index: 0,
                middleware: Vec::new(),
            };

            let mut cx: Pin<&mut Context> = Pin::new(&mut cx);

            // let _ = a(cx.as_mut()).await;
            // let _ = b(cx.as_mut()).await;
            // let _ = c(cx.as_mut()).await;
            // let _ = d(cx.as_mut()).await;
            // let _ = e(cx.as_mut()).await;
            // let _ = f(cx.as_mut()).await;
            // let _ = (B {}).call(cx.as_mut()).await;
            // let _ = (A {}).call(cx.as_mut()).await;

            let mut v: Vec<Box<dyn for<'a> Handle<'a, Context, Result>>> = vec![];
            v.push(Box::new(f));
            v.push(Box::new(e));
            v.push(Box::new(d));
            v.push(Box::new(c));
            v.push(Box::new(b));
            v.push(Box::new(a));
            v.push(Box::new(A { index: 1 }));
            v.push(Box::new(B { index: 2 }));
            v.push(Box::new(C { index: 3 }));
            v.reverse();
            assert_eq!(v.len(), 9);

            let mut v: Vec<Arc<dyn for<'a> Handle<'a, Context, Result>>> = vec![];

            // Handled it!
            // A Closure cant use `cx.next()`.
            v.push(Arc::new(|cx: Pin<&mut Context>| {
                assert_eq!(cx.index, 12);

                println!("We handled it!");

                async move {
                    // assert_eq!(cx.index, 12); // Error
                    Ok(())
                }
            }));
            v.push(Arc::new(C { index: 3 }));
            v.push(Arc::new(B { index: 2 }));
            v.push(Arc::new(A { index: 1 }));
            v.push(Arc::new(f));
            v.push(Arc::new(e));
            v.push(Arc::new(d));
            v.push(Arc::new(c));
            v.push(Arc::new(b));
            v.push(Arc::new(a));

            cx.as_mut().middleware = v.clone();
            println!("mw 0: {}", v.len());

            let result = cx.as_mut().next().await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), ());

            println!("mw 1: {}", v.len());

            cx.as_mut().middleware = v.clone();

            let result = cx.as_mut().next().await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), ());

            println!("mw 2: {}", v.len());

            cx.as_mut().middleware = v.clone();

            let result = cx.as_mut().next().await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), ());
        });
    }

    #[async_std::test]
    async fn async_std_rt() -> Result {
        let mut cx = Context {
            index: 0,
            middleware: Vec::new(),
        };

        let mut cx: Pin<&mut Context> = Pin::new(&mut cx);

        let mut v: Vec<Arc<dyn for<'a> Handle<'a, Context, Result>>> = vec![];
        v.insert(0, Arc::new(a));
        v.insert(0, Arc::new(b));
        v.insert(0, Arc::new(c));
        v.insert(0, Arc::new(d));
        v.insert(0, Arc::new(e));
        v.insert(0, Arc::new(f));
        v.insert(0, Arc::new(A { index: 1 }));
        v.insert(0, Arc::new(B { index: 2 }));
        v.insert(0, Arc::new(C { index: 3 }));
        // Handled it!
        async fn handler(cx: Pin<&mut Context>) -> Result {
            assert_eq!(cx.index, 12);

            println!("We handled it!");

            Ok(())
        }
        v.insert(0, Arc::new(handler));

        cx.as_mut().middleware = v.clone();
        println!("mw 0: {}", v.len());

        let result = cx.as_mut().next().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());

        println!("mw 1: {}", v.len());

        cx.as_mut().middleware = v.clone();

        let result = cx.as_mut().next().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());

        println!("mw 2: {}", v.len());

        cx.as_mut().middleware = v.clone();

        let result = cx.as_mut().next().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());

        Ok(())
    }
}
