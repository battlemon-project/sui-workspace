use anyhow::{Context, Result};
use chrono::{DateTime as ChronoDateTime, Utc};
use futures::StreamExt;
use graphql_client::{GraphQLQuery, Response};
use reqwest::header;
use tracing::info;

use models::{NftToken, Trait};
use models::sui_sdk::{
    rpc_types::SuiEventFilter,
    SuiClient,
    types::base_types::ObjectID,
};

mod config;
mod startup;
mod telemetry;

type DateTime = ChronoDateTime<Utc>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries.graphql",
    response_derives = "Debug"
)]
struct InsertNftToken;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("indexer".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber).context("Failed to init tracing subscriber")?;
    info!("Loading application config");
    let config = config::load_config().context("Failed to load app config")?;
    info!("Setup Sui Rust SDK");
    let sui = SuiClient::new(
        config.sui_json_rpc.http_url.as_str(),
        Some(config.sui_json_rpc.ws_url.as_str()),
        None,
    )
    .await
    .context("Failed to create SuiClient")?;

    let contract = config.sui_contract.address.as_str();
    info!("Setup `SuiEventFilter`");
    let filter = vec![
        SuiEventFilter::Package(ObjectID::from_hex_literal(contract)?),
        SuiEventFilter::Module("lemon".to_string()),
        SuiEventFilter::MoveEventType(format!("{contract}::lemon::LemonCreated")),
    ];
    let mut lemon_events = sui
        .event_api()
        .subscribe_event(SuiEventFilter::All(filter))
        .await
        .context("Failed to subscribe to events")?;

    info!("Start to poll Sui Node");
    loop {
        match lemon_events.next().await {
            Some(event) => {
                info!("Getting Sui's event");
                let sui_event = event.context("Failed to get next `SuiEvent`")?.event;

                let NftToken {
                    id,
                    r#type,
                    owner,
                    url,
                    traits,
                    created_at,
                } = sui_event
                    .try_into()
                    .context("Failed to convert `SuiEvent` into `NftToken`")?;

                let traits = traits
                    .into_iter()
                    .map(|Trait { name, flavour }| insert_nft_token::TraitInput { name, flavour })
                    .collect();

                let query = InsertNftToken::build_query(insert_nft_token::Variables {
                    id,
                    type_: r#type,
                    owner,
                    url,
                    traits,
                    created_at,
                });

                info!("Send query to backend api");
                let resp = reqwest::Client::new()
                    .post(&config.backend.graphql_url())
                    .header(header::CONTENT_TYPE, "application/json")
                    .json(&query)
                    .send()
                    .await
                    .context("Failed to send request to GraphQL backend service")?;

                match resp.error_for_status() {
                    Ok(resp) => {
                        let resp: Response<insert_nft_token::ResponseData> =
                            resp.json().await.context("Failed to parse response body")?;
                        info!("Response: {:?}", resp);
                    }
                    Err(err) => {
                        info!("Error: {:?}", err);
                    }
                }
            }
            _ => {
                info!("No more events");
            }
        }
    }
}
