#![deny(warnings)]

use futures_util::StreamExt;
use std::{net::SocketAddr, sync::Arc};
use systemstat::{Platform, System};
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::IntervalStream;
use viz::{
    get,
    header::ACCEPT,
    types::{Data, Event, Sse},
    Error, HandlerExt, IntoResponse, Request, RequestExt, Response, ResponseExt, Result, Router,
    Server, ServiceMaker, StatusCode,
};

async fn index(_: Request) -> Result<Response> {
    Ok(Response::html::<&'static str>(include_str!(
        "../index.html"
    )))
}

async fn stats(mut req: Request) -> Result<impl IntoResponse> {
    // check request `Accept` header
    if !matches!(req.header::<_, String>(ACCEPT), Some(ts) if ts == mime::TEXT_EVENT_STREAM.as_ref())
    {
        Err(StatusCode::BAD_REQUEST.into_error())?
    }

    let Data(sys): Data<Arc<System>> = req.extract().await?;

    Ok(Sse::new(
        IntervalStream::new(interval(Duration::from_secs(10))).map(move |_| {
            match sys
                .load_average()
                .map_err(|e| Error::Normal(Box::new(e)))
                .and_then(|loadavg| {
                    serde_json::to_string(&loadavg).map_err(|e| Error::Normal(Box::new(e)))
                }) {
                Ok(loadavg) => Event::default().data(loadavg),
                Err(err) => {
                    println!("{}", err);
                    Event::default().retry(30)
                }
            }
        }),
    )
    .interval(Duration::from_secs(15)))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let sys = Arc::new(System::new());

    let app = Router::new()
        .route("/", get(index))
        .route("/stats", get(stats.with(Data::new(sys))));

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
