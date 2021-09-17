use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_viz::{GraphQLRequest, GraphQLResponse};

use viz::prelude::{route, router, Error, Response, Result, Server, State};

mod starwars;

use starwars::{QueryRoot, StarWars, StarWarsSchema};

async fn graphql_handler(schema: State<StarWarsSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> Response {
    Response::html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[tokio::main]
async fn main() -> Result<()> {
    let schema =
        Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data(StarWars::new()).finish();

    let mut app = viz::new();

    app.state(schema)
        .routes(router().at("/", route().get(graphql_playground).post(graphql_handler)));

    Server::bind(&"127.0.0.1:3000".parse()?)
        .serve(app.into_make_service())
        .await
        .map_err(Error::new)
}
