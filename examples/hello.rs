use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use viz::prelude::*;
use viz_utils::{log, pretty_env_logger};

const NOT_FOUND: &str = "404 - This is not the web page you are looking for.";

async fn logger(cx: &mut Context) -> Result<Response> {
    let num = cx.extract::<State<Arc<AtomicUsize>>>().await?;

    num.as_ref().fetch_add(1, Ordering::SeqCst);

    log::info!("IN  Mid: {} {} - {:?}", cx.method(), cx.path(), num);

    let num = cx.state::<Arc<AtomicUsize>>()?;

    num.as_ref().fetch_add(1, Ordering::SeqCst);

    log::info!("IN  Mid: {} {} - {:?}", cx.method(), cx.path(), num);

    let fut = cx.next().await;

    log::info!("OUT Mid: {} {}", cx.method(), cx.path());

    fut.map(|mut res| {
        if res.status() == http::StatusCode::NOT_FOUND {
            *res.body_mut() = NOT_FOUND.into();
        }

        res
    })
}

async fn not_found() -> http::StatusCode {
    log::info!("{:>8}Exec: Not Found!", "");
    http::StatusCode::NOT_FOUND
}

async fn hello_world(num: State<Arc<AtomicUsize>>) -> &'static str {
    num.as_ref().fetch_sub(1, Ordering::SeqCst);

    log::info!("{:>8}Exec: Hello World! - {:?}", "", num);

    "Hello, World!"
}

fn allow_get(cx: &Context) -> bool {
    log::info!("{:>9}Get: {}", "", cx.method() == http::Method::GET);
    cx.method() == http::Method::GET
}

fn allow_head(cx: &Context) -> bool {
    log::info!("{:>8}Head: {}", "", cx.method() == http::Method::HEAD);
    cx.method() == http::Method::HEAD
}

#[tokio::main]
async fn main() -> Result {
    pretty_env_logger::init();

    let app = viz::new().state(Arc::new(AtomicUsize::new(0))).routes(
        router()
            .mid(logger)
            .at(
                "/",
                route()
                    // .guard(allow_get)
                    .guard(into_guard(allow_get) | into_guard(allow_head))
                    .all(hello_world),
            )
            .at("/*", route().all(not_found)),
    );

    app.listen("127.0.0.1:8000").await
}
