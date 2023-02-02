use chrono::{DateTime as ChronoDateTime, Utc};
use graphql_client::GraphQLQuery;

type DateTime = ChronoDateTime<Utc>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/queries.graphql",
    response_derives = "Debug"
)]
pub struct InsertNft;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/queries.graphql",
    response_derives = "Debug"
)]
pub struct AddItem;
