use crate::config::Config;
use crate::graphql::{MutationRoot, QueryRoot};
use async_graphql::{EmptySubscription, Schema};
use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use graphql::*;
use healthcheck::*;
use sqlx::PgPool;

mod graphql;
mod healthcheck;

pub type BattlemonSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: PgPool,
    pub graphql_schema: BattlemonSchema,
    pub config: Config,
}

#[rustfmt::skip]
#[tracing::instrument(name = "Setup routes", skip_all)]
pub fn setup_router(state: AppState) -> Router {
    Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/graphql", post(graphql_handler))
        .route("/graphql/playground",get(graphql_playground).post(graphql_handler))
        .with_state(state)
}
