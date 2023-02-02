use crate::config::Config;
use anyhow::{Context, Result};
use chrono::{DateTime as ChronoDateTime, Utc};
use futures::StreamExt;
use graphql_client::GraphQLQuery;
use models::sui_sdk::error::SuiRpcResult;
use models::sui_sdk::rpc_types::SuiEventEnvelope;
use models::sui_sdk::{rpc_types::SuiEventFilter, types::base_types::ObjectID, SuiClient};
use models::{events::Event, Item, Nft, Trait};
use reqwest::header;
use tracing::{error, info};

mod config;
mod startup;
mod telemetry;

type DateTime = ChronoDateTime<Utc>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/queries.graphql",
    response_derives = "Debug"
)]
struct InsertNft;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/queries.graphql",
    response_derives = "Debug"
)]
struct AddItem;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("indexer".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber).context("Failed to init tracing subscriber")?;
    info!("Loading application config");
    let config = config::load_config().context("Failed to load app config")?;
    info!("Parsing cli arguments");
    info!("Setup Sui Rust SDK");
    let sui = SuiClient::new(
        config.sui_json_rpc.http_url.as_str(),
        Some(config.sui_json_rpc.ws_url.as_str()),
        None,
    )
    .await
    .context("Failed to create SuiClient")?;

    let contract = config.sui_contract.address.as_str();
    let event_filter = SuiEventFilter::Package(ObjectID::from_hex_literal(contract)?);
    let mut contract_events = sui
        .event_api()
        .subscribe_event(event_filter)
        .await
        .context("Failed to subscribe to events")?;

    info!("Start to poll Sui Node");
    loop {
        let contract_event = contract_events.next().await;
        if let Err(e) = handle_contract_event(contract_event, &config).await {
            error!("While handling contract events obtain error: {e}");
        }
    }
}

async fn handle_contract_event(
    contract_event: Option<SuiRpcResult<SuiEventEnvelope>>,
    config: &Config,
) -> Result<()> {
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
) -> Result<reqwest::Response, reqwest::Error> {
    reqwest::Client::new()
        .post(config.backend.graphql_url())
        .header(header::CONTENT_TYPE, "application/json")
        .json(query)
        .send()
        .await
}

fn build_query(event: Event) -> Result<serde_json::Value> {
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
