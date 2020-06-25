### Handler


- Extracts, like `Actix`, `Rocket`, `Warp` style

```rust
async fn login(user: User, query: Query) -> &'static str {
    "Hello Viz!"
}
```

- Only havs `Contex`


```rust
async fn hello(cx: &mut Context) -> &'static str {
    "Hello world!"
}
```

- `Contex` + Extracts, `Contex` must be first param

```rust
async fn hello_login(cx: &mut Context, user: User, query: Query) -> &'static str {
    // do something
    login.call(cx).await
}
```
