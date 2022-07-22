<h1 align="center">Víz</h1>

<div align="center">
  <p><strong>Fast, robust, flexible, lightweight web framework for Rust</strong></p>
</div>

<div align="center">
  <!-- Safety -->
  <a href="/">
    <img src="https://img.shields.io/badge/-safety!-success?style=flat-square"
      alt="Safety!" /></a>
  <!-- Docs.rs docs -->
  <a href="https://docs.rs/viz">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="Docs.rs docs" /></a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/viz">
    <img src="https://img.shields.io/crates/v/viz.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/viz">
    <img src="https://img.shields.io/crates/d/viz.svg?style=flat-square"
      alt="Download" /></a>
  <!-- Discord -->
  <a href="https://discord.gg/cjX2KX">
     <img src="https://img.shields.io/discord/699908392105541722?logo=discord&style=flat-square"
     alt="Discord"></a>
</div>

## Features

* **Safety** `#![forbid(unsafe_code)]`

* Lightweight

* Robust [`Routing`](#routing)

* Handy [`Extractors`](#extractors)

* Simple + Flexible [`Handler`](#handler) & [`Middleware`](#middleware)

## Quick start

```rust
use std::net::SocketAddr;
use viz::{get, Request, Result, Router, Server, ServiceMaker};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new().route("/", get(index));

    if let Err(err) = Server::bind(&addr)
        .tcp_nodelay(true)
        .serve(ServiceMaker::from(app))
        .await
    {
        println!("{}", err);
    }

    Ok(())
}
```

More examples can be found [here](https://github.com/viz-rs/viz/tree/main/examples).

## Routing

The Viz router recognizes URLs and dispatches them to a handler.

### Simple routes 

```rust
let root = Router.new()
  .route("/", get(home))
  .route("/about", get(about));

let search = Router.new()
  .route("/", get(show_search));
```

### CRUD, Verbs

```rust
let todos = Router::new()
  .route("/", get(index).post(create))
  .route("/new", post(new))
  .route("/:id", get(show).patch(update).delete(destroy))
  .route("/:id/edit", get(edit));
```

### Resources

```rust
let users = Resource::default()
  .named("users")
  .route("/search", get(search_users))
  .index(index_users)
  .new(new_user)
  .create(create_user)
  .show(show_user)
  .edit(edit_user)
  .update(update_user)
  .destroy(delete_user);
```

### Nested

```rust
let app = Router::new()
  .nest("/", root) 
  .nest("/search", search) 
  .nest("/todos", todos.clone())  
  .nest("/users", users.nest("todos", todos))
  .route("/*", any(not_found));
```

## Handler

```rust
```

## Middleware

```rust
```
## Extractors

```rust
```
## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in Viz by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
