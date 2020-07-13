use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use serde::{Deserialize, Serialize};

use viz::prelude::*;
use viz_utils::{log, pretty_env_logger, serde::json, thiserror::Error as ThisError};

const NOT_FOUND: &str = "404 - This is not the web page you are looking for.";

async fn my_mid(cx: &mut Context) -> Result<Response> {
    let num = cx.extract::<State<Arc<AtomicUsize>>>().await?;

    num.as_ref().fetch_add(1, Ordering::SeqCst);

    log::info!("IN  Mid: {} {} - {:?}", cx.method(), cx.path(), &num);

    let num = cx.state::<Arc<AtomicUsize>>()?;

    num.as_ref().fetch_add(1, Ordering::SeqCst);

    // log::info!("IN  Mid: {} {} - {:?}", cx.method(), cx.path(), num);

    let fut = cx.next().await;

    log::info!("OUT Mid: {} {}", cx.method(), cx.path());

    Ok(match fut {
        Ok(mut res) => {
            if res.status() == http::StatusCode::NOT_FOUND {
                *res.body_mut() = NOT_FOUND.into();
            }

            res
        }
        Err(e) => e.into(),
    })
}

#[derive(ThisError, Debug)]
enum UserError {
    #[error("User Not Found")]
    NotFound,
}

impl Into<Response> for UserError {
    fn into(self) -> Response {
        (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into()
    }
}

async fn not_found() -> http::StatusCode {
    log::info!("{:8}Exec: Not Found!", "");
    http::StatusCode::NOT_FOUND
}

async fn hello_world(num: State<Arc<AtomicUsize>>) -> String {
    num.as_ref().fetch_sub(1, Ordering::SeqCst);

    log::info!("{:8}Exec: Hello World! - {:?}", "", num);

    "Hello, World!".to_string()
}

async fn server_error() -> Result<Response> {
    // async fn server_error() -> Result<Response, UserError> {
    // Err(UserError::NotFound))
    // Err(how!(UserError::NotFound))
    reject!(UserError::NotFound)
}

fn allow_get(cx: &Context) -> bool {
    log::info!("{:>8} Get: {}", "", cx.method() == http::Method::GET);
    cx.method() == http::Method::GET
}

fn allow_head(cx: &Context) -> bool {
    log::info!("{:>8}Head: {}", "", cx.method() == http::Method::HEAD);
    cx.method() == http::Method::HEAD
}

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: usize,
    name: String,
}

async fn create_user(user: Json<User>) -> Result<String> {
    json::to_string_pretty(&*user).map_err(|e| anyhow!(e))
}

#[tokio::main]
async fn main() -> Result {
    pretty_env_logger::init();

    let app = viz::new().state(Arc::new(AtomicUsize::new(0))).routes(
        router()
            .mid(middleware::timeout())
            .mid(middleware::request_id())
            .mid(middleware::recover())
            .mid(middleware::logger())
            .mid(my_mid)
            .at(
                "/",
                route()
                    // .guard(allow_get)
                    .guard(into_guard(allow_get) | into_guard(allow_head))
                    .all(hello_world),
            )
            .at("/users", route().post(create_user))
            .at("/500", route().all(server_error))
            .at("/*", route().all(not_found)),
    );

    app.listen("127.0.0.1:8080").await
}
