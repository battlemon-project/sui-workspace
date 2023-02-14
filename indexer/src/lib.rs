use cynic::MutationBuilder;
use eyre::{ensure, Context};
use reqwest::header;
use serde::Serialize;
use serde_json::Value;
use tracing::info;

use graphql::insert_nft::{InsertNftMutation, InsertNftMutationArguments};
use models::events::Event;
use models::sui_sdk::error::SuiRpcResult;
use models::sui_sdk::rpc_types::SuiEventEnvelope;
use models::{Item, Nft};

use crate::config::Config;
use crate::graphql::add_item::{AddItemMutation, AddItemMutationArguments};
use crate::graphql::remove_item::{RemoveItemMutation, RemoveItemMutationArguments};

pub mod config;
mod graphql;
pub mod telemetry;

#[tracing::instrument(name = "Handling contract's event", err, skip_all)]
pub async fn handle_contract_event(
    contract_event: SuiRpcResult<SuiEventEnvelope>,
    config: &Config,
) -> eyre::Result<()> {
    let sui_event = contract_event.context("Sui Rpc error")?;
    info!("Getting new Sui's event");
    let event = sui_event
        .event
        .try_into()
        .context("Failed to convert `SuiEvent` into `Event`")?;
    let query = build_query(event);
    let resp = send_graphql_query(config, &query)
        .await
        .context("Failed to send request to GraphQL backend service")?;
    handle_errors(resp).await?;

    Ok(())
}

async fn handle_errors(resp: reqwest::Response) -> eyre::Result<()> {
    ensure!(
        resp.status().as_u16() == 200,
        "Response from backend contains error"
    );

    let graphql_resp: Value = resp.json().await?;
    if let Some(errors) = graphql_resp.get("errors") {
        ensure!(
            errors.as_array().unwrap().is_empty(),
            "GraphQL response contains errors: {errors:?}"
        );
    }

    Ok(())
}

#[tracing::instrument(name = "Sending GraphQL query to backend server", skip_all)]
async fn send_graphql_query(
    config: &Config,
    query: impl Serialize,
) -> eyre::Result<reqwest::Response, reqwest::Error> {
    reqwest::Client::new()
        .post(config.backend.graphql_url())
        .header(header::CONTENT_TYPE, "application/json")
        .json(&query)
        .send()
        .await
}

#[tracing::instrument(name = "Building query for graphql", skip_all)]
fn build_query(event: Event) -> Value {
    let ret = match event {
        Event::Nft(Nft {
            id,
            r#type,
            owner,
            url,
            traits,
            items,
            created_at,
            attached_to,
        }) => {
            let args = InsertNftMutationArguments {
                id,
                r#type,
                owner,
                url,
                traits: traits.into_iter().map(Into::into).collect(),
                items: items.into_iter().map(Into::into).collect(),
                created_at,
                attached_to,
            };
            let query = InsertNftMutation::build(args);
            serde_json::to_value(query).unwrap()
        }
        Event::ItemAdded(Item { lemon_id, item_id }) => {
            let args = AddItemMutationArguments { lemon_id, item_id };
            let query = AddItemMutation::build(args);
            serde_json::to_value(query).unwrap()
        }
        Event::ItemRemoved(Item { lemon_id, item_id }) => {
            let args = RemoveItemMutationArguments { lemon_id, item_id };
            let query = RemoveItemMutation::build(args);
            serde_json::to_value(query).unwrap()
        }
    };

    ret
}
