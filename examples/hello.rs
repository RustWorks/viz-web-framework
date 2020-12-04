use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};

use viz::prelude::*;
use viz_utils::{
    futures::{FutureExt, StreamExt},
    log, pretty_env_logger,
    serde::json,
    thiserror::Error as ThisError,
};

const NOT_FOUND: &str = "404 - This is not the web page you are looking for.";

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, Error>>>>>;

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

async fn echo(cx: &mut Context) -> Response {
    match cx.ws() {
        Ok(ws) => {
            ws.on_upgrade(|websocket| {
                // Just echo all messages back...
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        }
        Err(rs) => rs,
    }
}

async fn chat(cx: &mut Context) -> Result<Response> {
    let users = cx.state::<Users>()?;
    Ok(match cx.ws() {
        Ok(ws) => ws.on_upgrade(move |socket| user_connected(socket, users)),
        Err(rs) => rs,
    })
}

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Make an extra clone to give to our disconnection handler...
    let users2 = users.clone();

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users2).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Ok(Message::text(new_msg.clone()))) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>Warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/chat/';
        const ws = new WebSocket(uri);
        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }
        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };
        ws.onmessage = function(msg) {
            message(msg.data);
        };
        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };
        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;

#[tokio::main]
async fn main() -> Result {
    pretty_env_logger::init();

    let mut app = viz::new();

    let config = app.config().await;

    dbg!(config);

    let users = Users::default();

    app.state(Arc::new(AtomicUsize::new(0)))
        .state(users)
        .routes(
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
                .at("/echo", route().get2(echo))
                .at(
                    "/chat",
                    route().get(|| async { Response::html(INDEX_HTML) }),
                )
                .at("/chat/", route().get2(chat))
                .at("/*", route().all(not_found)),
        );

    app.listen("127.0.0.1:8080").await
}
