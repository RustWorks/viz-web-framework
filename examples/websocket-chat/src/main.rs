#![deny(warnings)]

use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::sync::broadcast::{channel, Sender};
use viz::{
    get, get_ext,
    types::{Data, Message, Params, WebSocket},
    HandlerExt, IntoResponse, Request, RequestExt, Response, ResponseExt, Result, Router, Server,
    ServiceMaker,
};

async fn index() -> Result<Response> {
    Ok(Response::html::<&'static str>(include_str!(
        "../index.html"
    )))
}

async fn ws(mut req: Request) -> Result<impl IntoResponse> {
    let (ws, Params(name), Data(sender)): (WebSocket, Params<String>, Data<Sender<String>>) =
        req.extract().await?;

    let tx = sender.clone();
    let mut rx = sender.subscribe();

    Ok(ws.on_upgrade(move |socket| async move {
        // Split the socket into a sender and receive of messages.
        let (mut ws_tx, mut ws_rx) = socket.split();

        tokio::task::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if ws_tx.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        while let Some(Ok(msg)) = ws_rx.next().await {
            if let Message::Text(text) = msg {
                // Maybe should check user name, dont send to current user.
                if tx.send(format!("{}: {}", name, text)).is_err() {
                    break;
                }
            }
        }

        println!("websocket was closed");
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let channel = channel::<String>(32);

    let app = Router::new()
        .route("/", get_ext(index))
        .route("/ws/:name", get(ws.with(Data::new(channel.0))));

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
