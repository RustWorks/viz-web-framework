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

* Robust `Routing`

* Handy `Extractors`

* Simple + Flexible `Handler` & `Middleware`

## Hello Viz

```rust
use std::net::SocketAddr;
use viz::{get, Request, Result, Router, Server, ServiceMaker};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello Viz")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new().route("/", get(index));

    if let Err(err) = Server::bind(&addr)
        .serve(ServiceMaker::from(app))
        .await
    {
        println!("{}", err);
    }

    Ok(())
}
```

More examples can be found [here](https://github.com/viz-rs/viz/tree/main/examples).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in Viz by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
