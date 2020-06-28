### Handler

#### Generics Async Functions

```rust
async fn hey() -> &'static str {
    "Hey Viz!"
}
```

#### Extracts

Follows `Actix`, `Rocket`, `Warp` style

```rust
async fn login(user: User, query: Query) -> &'static str {
    "Hello Viz!"
}
```

#### `Context`

Has only `Context` parameter

```rust
async fn hello(cx: &mut Context) -> &'static str {
    "Hello world!"
}
```

#### `Context` + Extracts

`Context` must be first parameter

```rust
async fn hello_login(cx: &mut Context, user: User, query: Query) -> &'static str {
    // do something
    login(user, query).await
}
```
