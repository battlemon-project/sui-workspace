use anyhow::{Context, Result};
use battlemon_backend::startup::App;
use battlemon_backend::{config, telemetry};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber =
        telemetry::get_subscriber("battlemon_backend".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber).context("Failed to init tracing subscriber")?;
    info!("Loading application config");
    let config = config::load_config().context("Failed to load app config")?;
    let app = App::build(config).await?;
    app.run_until_stopped().await?;

    Ok(())
}
