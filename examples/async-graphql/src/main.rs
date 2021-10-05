use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_viz::{
    graphql_subscription, GraphQLRequest, GraphQLResponse, SecWebsocketProtocol,
};

use viz::prelude::{get, http, router, ws::Ws, Error, Header, Response, Result, Server, State};

mod starwars;

use starwars::{QueryRoot, StarWars, StarWarsSchema};

async fn graphql_handler(
    schema: State<StarWarsSchema>,
    GraphQLRequest(req): GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req).await.into()
}

async fn graphql_playground() -> Response {
    Response::html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

async fn graphql_subscription_handler(
    ws: Ws,
    State(schema): State<StarWarsSchema>,
    Header(protocol): Header<SecWebsocketProtocol>,
) -> Response {
    let mut res = ws.on_upgrade(move |websocket| graphql_subscription(websocket, schema, protocol));

    // must!
    res.headers_mut().append(
        "Sec-WebSocket-Protocol",
        http::HeaderValue::from_static(protocol.0.sec_websocket_protocol()),
    );

    res
}

#[tokio::main]
async fn main() -> Result<()> {
    let schema =
        Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data(StarWars::new()).finish();

    let mut app = viz::new();

    app.state(schema).routes(
        router()
            .at("/", get(graphql_playground).post(graphql_handler))
            .at("/ws", get(graphql_subscription_handler)),
    );

    Server::bind(&"127.0.0.1:3000".parse()?).serve(app.into_service()).await.map_err(Error::new)
}
