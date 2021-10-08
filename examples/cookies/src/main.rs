//! Cookies
//! `curl 127.0.0.1:3000/ -H 'Cookie: count=0' -vvv`

use viz::middleware;
use viz::prelude::{get, router, Cookie, Cookies, Error, Response, Result, Server};

async fn index(cookies: Cookies) -> impl Into<Response> {
    let mut count: isize =
        cookies.get("count").and_then(|c| c.value().parse().ok()).unwrap_or_default();

    count += 1;

    let value = count.to_string();

    cookies.add(Cookie::new("count", value.clone()));

    value
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = viz::new();

    app.routes(router().with(middleware::Cookies::default()).at("/", get(index)));

    Server::bind(&"127.0.0.1:3000".parse()?).serve(app.into_service()).await.map_err(Error::new)
}
