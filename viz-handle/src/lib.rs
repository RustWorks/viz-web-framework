use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
/// A Handle Trait for asynchronous context pipeline.
///
/// Maintain context in multiple handles.
#[async_trait]
pub trait Handle<'a, Context, Output>: Send + Sync + 'static {
    async fn call(&'a self, cx: Pin<&'a mut Context>) -> Output;
}

#[async_trait]
impl<'a, Context, Output, F, Fut> Handle<'a, Context, Output> for F
where
    F: Send + Sync + 'static + Clone + Fn(Pin<&'a mut Context>) -> Fut,
    Fut: Future<Output = Output> + Send + 'a,
    Context: 'a + Send,
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
        cx.index += 1;
        let fut = cx.next().await;
        cx.index += 1;
        println!("exec Fn a --{}<< {:>2}", repeat, cx.index);
        fut
    }

    fn b<'a>(mut cx: Pin<&'a mut Context>) -> BoxFuture<'a, Result> {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);
        println!("exec Fn b --{}>> {:>2}", repeat, cx.index);
        cx.index += 1;
        Box::pin(async move {
            let fut = cx.next().await;
            cx.index += 1;
            println!("exec Fn b --{}<< {:>2}", repeat, cx.index);
            fut
        })
    }

    fn c(mut cx: Pin<&mut Context>) -> BoxFuture<'_, Result> {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);
        println!("exec Fn c --{}>> {:>2}", repeat, cx.index);
        cx.index += 1;
        Box::pin(async move {
            let fut = cx.next().await;
            cx.index += 1;
            println!("exec Fn c --{}<< {:>2}", repeat, cx.index);
            fut
        })
    }

    fn d<'a>(mut cx: Pin<&'a mut Context>) -> impl Future<Output = Result> + 'a {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);
        println!("exec Fn d --{}>> {:>2}", repeat, cx.index);
        cx.index += 1;
        async move {
            let fut = cx.next().await;
            cx.index += 1;
            println!("exec Fn d --{}<< {:>2}", repeat, cx.index);
            fut
        }
    }

    fn e(mut cx: Pin<&mut Context>) -> impl Future<Output = Result> + '_ {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);
        println!("exec Fn e --{}>> {:>2}", repeat, cx.index);
        cx.index += 1;
        async move {
            let fut = cx.next().await;
            cx.index += 1;
            println!("exec Fn e --{}<< {:>2}", repeat, cx.index);
            fut
        }
    }

    async fn f(mut cx: Pin<&mut Context>) -> Result {
        let size = cx.middleware.len();
        let repeat = "-".repeat(2 * size);
        println!("exec Fn f --{}>> {:>2}", repeat, cx.index);
        cx.index += 1;
        let fut = cx.next().await;
        cx.index += 1;
        println!("exec Fn f --{}<< {:>2}", repeat, cx.index);
        fut
    }

    // let g = Apply::new(|mut cx: Pin<&mut Context>| {
    //     let size = cx.middleware.len();
    //     let repeat = "-".repeat(2 * size);
    //     println!("exec Fn g --{}>> {:>2}", repeat, cx.index);
    //     cx.index += 1;
    //     // let mut a = cx.as_mut();
    //     let fut = cx.get_mut().next();
    //     // cx.index += 1;
    //     async move {
    //         // println!("exec Fn g --{}<< {:>2}", repeat, cx.index);
    //         // fut
    //         fut.await
    //         // Ok(())
    //     }
    // });

    struct A {
        index: usize,
    }

    #[async_trait]
    impl<'a> Handle<'a, Context, Result> for A {
        async fn call(&'a self, mut cx: Pin<&'a mut Context>) -> Result {
            let size = cx.middleware.len();
            let repeat = "-".repeat(2 * size);
            println!("exec St A --{}>> {:>2}", repeat, cx.index);
            cx.index += self.index;
            let fut = cx.next().await;
            cx.index -= self.index;
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
            cx.index += self.index;
            let fut = cx.next().await;
            cx.index -= self.index;
            println!("exec St B --{}<< {:>2}", repeat, cx.index);
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
            // // let _ = g(cx.as_mut()).await;
            // let _ = (B {}).call(cx.as_mut()).await;
            // let _ = (A {}).call(cx.as_mut()).await;

            let mut v: Vec<Arc<dyn for<'a> Handle<'a, Context, Result>>> = vec![];
            v.push(Arc::new(a));
            v.push(Arc::new(b));
            v.push(Arc::new(c));
            v.push(Arc::new(d));
            v.push(Arc::new(e));
            v.push(Arc::new(f));
            // v.puArcBox::new(g));
            v.push(Arc::new(B { index: 1 }));
            v.push(Arc::new(A { index: 2 }));

            cx.as_mut().middleware = v.clone();
            println!("mw 0: {}", v.len());

            let result = cx.next().await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), ());

            println!("mw 1: {}", v.len());

            cx.as_mut().middleware = v.clone();

            let result = cx.next().await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), ());

            println!("mw 2: {}", v.len());

            cx.as_mut().middleware = v.clone();

            let result = cx.next().await;
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
        v.push(Arc::new(a));
        v.push(Arc::new(b));
        v.push(Arc::new(c));
        v.push(Arc::new(d));
        v.push(Arc::new(e));
        v.push(Arc::new(f));
        v.push(Arc::new(B { index: 1 }));
        v.push(Arc::new(A { index: 2 }));

        cx.as_mut().middleware = v.clone();
        println!("mw 0: {}", v.len());

        let result = cx.next().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());

        println!("mw 1: {}", v.len());

        cx.as_mut().middleware = v.clone();

        let result = cx.next().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());

        println!("mw 2: {}", v.len());

        cx.as_mut().middleware = v.clone();

        let result = cx.next().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());

        Ok(())
    }
}
