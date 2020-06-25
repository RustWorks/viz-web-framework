### Middleware

Middleware is a trait, based on **[handle]**.

- Async Functions

```rust
async fn mid(cx: &mut Context) -> Result<Response> {
    cx.next().await
}
```

- Implement `Middleware` for struct

```rust
struct Mid {}

impl<'a> Middleware<'a, Context> for Mid
{
    type Output = Result<Response>;

    #[inline]
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        // do something
        cx.next()
    }
}
```


[handle]: https://crates.io/crates/handle
