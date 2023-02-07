use crate::config::Config;
use crate::routes::BattlemonSchema;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::response::{Html, IntoResponse};

#[tracing::instrument(name = "Getting GraphQL playground")]
pub async fn graphql_playground(config: State<Config>) -> impl IntoResponse {
    let graphql_config = GraphQLPlaygroundConfig::new(&config.graphql.playground_route);
    Html(playground_source(graphql_config))
}

#[tracing::instrument(name = "Handling GraphQL request", skip_all)]
pub async fn graphql_handler(
    schema: State<BattlemonSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
