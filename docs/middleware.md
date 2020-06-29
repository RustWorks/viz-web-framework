## Middleware

Middleware is a trait, based on **[handle](https://github.com/viz-rs/handle)**.

### Async Functions

```rust
async fn middle(cx: &mut Context) -> Result<Response> {
    cx.next().await
}
```

### Implement `Middleware` for struct

```rust
struct Middle {}

impl<'a> Middleware<'a, Context> for Middle
{
    type Output = Result<Response>;

    #[inline]
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        // do something
        cx.next()
    }
}
```
