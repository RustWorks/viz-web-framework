<h1 align="center">Víz</h1>

<div align="center">
  <p><strong>Fast, flexible, minimalist web framework for Rust</strong></p>
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
  <!-- Twitter -->
  <a href="https://twitter.com/_fundon">
    <img src="https://img.shields.io/badge/twitter-@__fundon-blue.svg?style=flat-square"
      alt="Twitter: @_fundon" /></a>
</div>

## 🦀 Features

* **Safety** `#![forbid(unsafe_code)]`

* Robust `routing`

* `Middleware` supports

* Based on [hyper](https://hyper.rs/)

* Powerful extractors

## Middleware

* logger
* recover
* request_id
* timeout
* cookies
* sessions
* cors
* auth
* compression
* jwt

## Todos

* [ ] More friendly Routing
* [ ] Template engines
* [ ] TLS
* [ ] GraphQL?
* [ ] RPC?
* [x] Configuration
* [x] Data State
* [x] Error handling
* [x] Websocket
* [x] Server-Sent Events
* [x] Sessions
* [x] Middlewares
    * [x] `auth`
    * [x] `logger`
    * [x] `recover`
    * [x] `request_id`
    * [x] `timeout`
    * [x] `cookies`
    * [x] `session`
    * [x] `cors`
    * [x] `compression`
    * [x] `jwt`
    * [x] `serve static files`
* [x] Unix Domain Socket


## Thanks

Some ideas from them:

* [Actix Web](https://docs.rs/actix-web/)
* [Axum](https://docs.rs/axum/)
* [Ntex](https://docs.rs/ntex/)
* [Rocket](https://docs.rs/rocket/)
* [Tide](https://docs.rs/tide/)
* [Tower Web](https://docs.rs/tower-web/)
* [Warp](https://docs.rs/warp/)

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
