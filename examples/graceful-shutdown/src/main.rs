#![deny(warnings)]
#![allow(clippy::unused_async)]

//! Graceful shutdown server.
//!
//! See <https://github.com/hyperium/hyper/blob/master/examples/graceful_shutdown.rs>

use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, pin};
use viz::{server::conn::http1, Io, Request, Responder, Result, Router, Tree};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let app = Router::new().get("/", index);
    let tree = Arc::new(Tree::from(app));

    // Use a 5 second timeout for incoming connections to the server.
    // If a request is in progress when the 5 second timeout elapses,
    // use a 2 second timeout for processing the final request and graceful shutdown.
    let connection_timeouts = vec![Duration::from_secs(5), Duration::from_secs(2)];

    loop {
        // Clone the connection_timeouts so they can be passed to the new task.
        let connection_timeouts_clone = connection_timeouts.clone();

        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();

        tokio::task::spawn(async move {
            // Pin the connection object so we can use tokio::select! below.
            let conn = http1::Builder::new()
                .serve_connection(Io::new(stream), Responder::new(tree, Some(addr)));
            pin!(conn);

            // Iterate the timeouts.  Use tokio::select! to wait on the
            // result of polling the connection itself,
            // and also on tokio::time::sleep for the current timeout duration.
            for (iter, sleep_duration) in connection_timeouts_clone.iter().enumerate() {
                println!("iter = {iter} sleep_duration = {sleep_duration:?}");
                tokio::select! {
                    res = conn.as_mut() => {
                        // Polling the connection returned a result.
                        // In this case print either the successful or error result for the connection
                        // and break out of the loop.
                        match res {
                            Ok(()) => println!("after polling conn, no error"),
                            Err(e) =>  println!("error serving connection: {e:?}"),
                        };
                        break;
                    }
                    () = tokio::time::sleep(*sleep_duration) => {
                        // tokio::time::sleep returned a result.
                        // Call graceful_shutdown on the connection and continue the loop.
                        println!("iter = {iter} got timeout_interval, calling conn.graceful_shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }
        });
    }
}
