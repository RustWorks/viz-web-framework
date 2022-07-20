//! Unix Domain Socket
//!
//! ```sh
//! curl --unix-socket /tmp/viz.sock http://localhost/
//! ```
#![deny(warnings)]

#[cfg(unix)]
#[tokio::main]
async fn main() -> viz::Result<()> {
    use tokio::net::UnixListener;
    use tokio_stream::wrappers::UnixListenerStream;
    use viz::{get, FnExt, Result, Router, Server, ServiceMaker};

    async fn index() -> Result<&'static str> {
        Ok("Hello world!")
    }

    let path = "/tmp/viz.sock";
    println!("listening on {}", &path);

    let listener = UnixListener::bind(path).unwrap();
    let incoming = UnixListenerStream::new(listener);

    let app = Router::new().route("/", get(index.to_handler()));

    if let Err(err) = Server::builder(viz::accept_from_stream(incoming))
        .serve(ServiceMaker::from(app))
        .await
    {
        println!("{}", err);
    }

    Ok(())
}

#[cfg(not(unix))]
#[tokio::main]
async fn main() {
    panic!("Must run under Unix-like platform!");
}
