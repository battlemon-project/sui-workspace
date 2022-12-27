use crate::routes::BattlemonSchema;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Request, Response};
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::Json;

#[tracing::instrument(name = "Getting GraphQL playground")]
pub async fn graphql_playground() -> impl IntoResponse {
    let config = GraphQLPlaygroundConfig::new("/graphql/playground");
    Html(playground_source(config))
}

#[tracing::instrument(name = "Handling GraphQL request", skip_all)]
pub async fn graphql_handler(
    schema: State<BattlemonSchema>,
    Json(json): Json<Request>,
) -> Json<Response> {
    schema.execute(json).await.into()
}
