use crate::config::Config;
use anyhow::Context;
use graphql::{add_item, insert_nft, AddItem, InsertNft};
use graphql_client::GraphQLQuery;
use models::events::Event;
use models::sui_sdk::error::SuiRpcResult;
use models::sui_sdk::rpc_types::SuiEventEnvelope;
use models::{Item, Nft, Trait};
use reqwest::header;
use tracing::info;

pub mod config;
mod graphql;
pub mod telemetry;

pub async fn handle_contract_event(
    contract_event: Option<SuiRpcResult<SuiEventEnvelope>>,
    config: &Config,
) -> anyhow::Result<()> {
    let sui_event = contract_event
        .context("No more events")?
        .context("Sui Rpc error")?;

    info!("Getting Sui's event");
    let event = sui_event
        .event
        .try_into()
        .context("Failed to convert `SuiEvent` into `Event`")?;

    let query = build_query(event).context("Failed to build query for event")?;
    info!("Send query to backend api");
    send_graphql_query(config, &query)
        .await
        .context("Failed to send request to GraphQL backend service")?
        .error_for_status()
        .context("Response from server contains error")?;

    Ok(())
}

async fn send_graphql_query(
    config: &Config,
    query: &serde_json::Value,
) -> anyhow::Result<reqwest::Response, reqwest::Error> {
    reqwest::Client::new()
        .post(config.backend.graphql_url())
        .header(header::CONTENT_TYPE, "application/json")
        .json(query)
        .send()
        .await
}

fn build_query(event: Event) -> anyhow::Result<serde_json::Value> {
    let ret = match event {
        Event::Nft(Nft {
            id,
            r#type,
            owner,
            url,
            traits,
            items,
            created_at,
        }) => {
            let traits = traits
                .into_iter()
                .map(|Trait { name, flavour }| insert_nft::TraitInput { name, flavour })
                .collect();

            let items = items
                .into_iter()
                .map(|Trait { name, flavour }| insert_nft::TraitInput { name, flavour })
                .collect();

            let query = InsertNft::build_query(insert_nft::Variables {
                id,
                type_: r#type,
                owner,
                url,
                traits,
                items,
                created_at,
            });
            serde_json::to_value(query)?
        }
        Event::ItemAdded(Item { lemon_id, item_id }) => {
            let query = AddItem::build_query(add_item::Variables { lemon_id, item_id });
            serde_json::to_value(query)?
        }
        Event::ItemRemoved(Item { lemon_id, item_id }) => unimplemented!(),
    };

    Ok(ret)
}
