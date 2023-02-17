use eyre::{Context, Result};
use futures::StreamExt;
use indexer::{config, handle_contract_event, telemetry};
use models::sui_sdk::{rpc_types::SuiEventFilter, types::base_types::ObjectID, SuiClientBuilder};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("indexer".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber).context("Failed to init tracing subscriber")?;
    info!("Loading application config");
    let config = config::load_config().context("Failed to load app config")?;
    info!("Setup Sui Rust SDK");
    let sui = SuiClientBuilder::default()
        .ws_url(&config.sui_json_rpc.ws_url)
        .build(&config.sui_json_rpc.http_url)
        .await
        .context("Failed to build SuiClient")?;

    let contract = config.sui_contract.address.as_str();
    let event_filter = SuiEventFilter::Package(ObjectID::from_hex_literal(contract)?);
    let mut contract_events = sui
        .event_api()
        .subscribe_event(event_filter)
        .await
        .context("Failed to subscribe to events")?;

    info!("Start to poll Sui Node for contract `{contract}`");
    while let Some(contract_event) = contract_events.next().await {
        if let Err(e) = handle_contract_event(contract_event, &config).await {
            error!("While handling contract events obtain error: {e:?}");
        }
    }

    panic!("Failed to get new events from Sui Node");
}
