use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub sui_json_rpc: SuiJsonRpcConfig,
    pub sui_contract: SuiContractConfig,
    pub backend: BackendConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SuiJsonRpcConfig {
    pub http_url: String,
    pub ws_url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SuiContractConfig {
    pub address: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BackendConfig {
    pub host: String,
    pub port: u16,
}

impl BackendConfig {
    pub fn graphql_url(&self) -> String {
        format!("http://{}:{}/graphql", self.host, self.port)
    }
}

pub fn load_config() -> Result<Config> {
    let config_path = std::env::current_dir()
        .context("Failed to determine the current directory")?
        .join("config");

    let env: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .parse()
        .context("Failed to parse APP_ENV")?;

    let env_filename = format!("{}.toml", env.as_str());
    let config = config::Config::builder()
        .add_source(config::File::from(config_path.join("base.toml")))
        .add_source(config::File::from(config_path.join(env_filename)))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
        .context("Failed to build config")?;

    config
        .try_deserialize()
        .context("Failed to deserialize config files into `Config`")
}

#[derive(Debug, Clone)]
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl FromStr for Environment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(anyhow!(
                "{} is not a supported environment. Use either `local` or `production`",
                other
            )),
        }
    }
}
