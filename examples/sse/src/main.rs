// #![deny(warnings)]

use futures_util::StreamExt;
use std::net::SocketAddr;
use tokio::time::{interval, Duration, Instant};
use tokio_stream::wrappers::IntervalStream;
use viz::{
    get,
    header::ACCEPT,
    types::{Event, Sse},
    IntoResponse, Request, RequestExt, Response, ResponseExt, Result, Router, Server, ServiceMaker,
    StatusCode,
};

async fn index(_: Request) -> Result<Response> {
    Ok(Response::html::<&'static str>(include_str!(
        "../index.html"
    )))
}

async fn stats(req: Request) -> Result<impl IntoResponse> {
    if !matches!(req.header::<_, String>(ACCEPT), Some(ts) if ts == mime::TEXT_EVENT_STREAM.as_ref())
    {
        Err(StatusCode::BAD_REQUEST.into_error())?
    }
    let now = Instant::now();
    Ok(Sse::new(
        IntervalStream::new(interval(Duration::from_secs(10)))
            .map(move |_| Event::default().data(now.elapsed().as_secs().to_string())),
    )
    .interval(Duration::from_secs(15)))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new()
        .route("/", get(index))
        .route("/stats", get(stats));

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
