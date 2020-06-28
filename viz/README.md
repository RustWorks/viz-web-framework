<h1 align="center">Víz</h1>

<div align="center">
  <p><strong>Fast, flexible, minimalist web framework for Rust</strong></p>
</div>

<div align="center">
  <!-- Safety -->
  <a href="/">
    <img src="https://img.shields.io/badge/-safety!-success?style=flat-square" alt="Safety!" /></a>
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
  <!-- Twitter -->
  <a href="https://twitter.com/_fundon">
    <img src="https://img.shields.io/badge/twitter-@__fundon-blue.svg?style=flat-square" alt="Twitter: @_fundon" /></a>
</div>

## 🦀 Features

* **Safety** `#![deny(unsafe_code)]`

* Robust `routing`

* `Middleware` supports

* Based on fast [hyper][]

* Powerful extractors

## Todos

* [ ] Configuration
* [ ] Data State
* [ ] Sessions
* [ ] Basic middlewares: `logger`, `cors`, `serve static files`
* [ ] Websocket
* [ ] Template engines
* [ ] TLS

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

[hyper]: https://hyper.rs/
