use crate::schema::{MutationRoot, QueryRoot};
use async_graphql::{EmptySubscription, Schema};
use std::{fs::File, io::Write};

#[path = "src/graphql.rs"]
mod schema;

fn main() {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();
    let path = std::env::current_dir()
        .expect("Failed to determine the current directory")
        .join("schema.graphql");

    let mut file = File::create(path).expect("Failed to create file.");
    file.write_all(schema.sdl().as_bytes())
        .expect("Failed to write data with graphql schema in file.");
}
