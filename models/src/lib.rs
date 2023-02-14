pub mod errors;
pub mod events;

use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx_core::types::Json;
pub use sui_sdk;

#[derive(SimpleObject, InputObject, Serialize, Deserialize, Debug, Clone)]
#[graphql(input_name = "TraitInput")]
pub struct Trait {
    pub name: String,
    pub flavour: String,
}

#[derive(SimpleObject, InputObject, Serialize, Deserialize, Debug, Clone)]
#[graphql(input_name = "NftInput")]
pub struct Nft {
    pub id: String,
    pub r#type: String,
    pub owner: String,
    pub url: String,
    pub traits: Vec<Trait>,
    pub items: Vec<Nft>,
    pub created_at: DateTime<Utc>,
    pub attached_to: Option<String>,
}

#[derive(SimpleObject, Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub lemon_id: String,
    pub item_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NftSql {
    pub id: String,
    pub r#type: String,
    pub owner: String,
    pub url: String,
    pub traits: Json<Vec<Trait>>,
    pub items: Json<Vec<NftSql>>,
    pub created_at: DateTime<Utc>,
    pub attached_to: Option<String>,
}

impl From<Nft> for NftSql {
    fn from(
        Nft {
            id,
            r#type,
            owner,
            url,
            traits,
            items,
            created_at,
            attached_to,
        }: Nft,
    ) -> Self {
        let items = items.into_iter().map(Into::into).collect();
        Self {
            id,
            r#type,
            owner,
            url,
            traits: Json(traits),
            items: Json(items),
            created_at,
            attached_to,
        }
    }
}

impl From<NftSql> for Nft {
    fn from(
        NftSql {
            id,
            r#type,
            owner,
            url,
            traits,
            items,
            created_at,
            attached_to,
        }: NftSql,
    ) -> Self {
        let items = items.0.into_iter().map(Into::into).collect();
        Self {
            id,
            r#type,
            owner,
            url,
            traits: traits.0,
            items,
            created_at,
            attached_to,
        }
    }
}
