use anyhow::anyhow;
use async_graphql::{InputObject, InputValueResult, ScalarType, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx_core::types::Json;
pub use sui_sdk;
use sui_sdk::rpc_types::{SuiEvent, SuiEventFilter, SuiMoveStruct, SuiMoveValue};
use sui_sdk::types::base_types::ObjectID;

#[derive(SimpleObject, InputObject, Serialize, Deserialize, Debug)]
#[graphql(input_name = "TraitInput")]
pub struct Trait {
    pub name: String,
    pub flavour: String,
}

#[derive(SimpleObject, InputObject, Serialize, Deserialize, Debug)]
#[graphql(input_name = "NftTokenInput")]
pub struct NftToken {
    pub id: String,
    pub r#type: String,
    pub owner: String,
    pub url: String,
    pub traits: Vec<Trait>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NftTokenSql {
    pub id: String,
    pub r#type: String,
    pub owner: String,
    pub url: String,
    pub traits: Json<Vec<Trait>>,
    pub created_at: DateTime<Utc>,
}

impl From<NftToken> for NftTokenSql {
    fn from(
        NftToken {
            id,
            r#type,
            owner,
            url,
            traits,
            created_at,
        }: NftToken,
    ) -> Self {
        Self {
            id,
            r#type,
            owner,
            url,
            traits: Json(traits),
            created_at,
        }
    }
}

impl From<NftTokenSql> for NftToken {
    fn from(
        NftTokenSql {
            id,
            r#type,
            owner,
            url,
            traits,
            created_at,
        }: NftTokenSql,
    ) -> Self {
        Self {
            id,
            r#type,
            owner,
            url,
            traits: traits.0,
            created_at,
        }
    }
}

impl TryFrom<SuiEvent> for NftToken {
    type Error = anyhow::Error;

    fn try_from(event: SuiEvent) -> Result<Self, Self::Error> {
        let SuiEvent::MoveEvent { sender,  fields, .. } = event else {
            return Err(anyhow!("Wrong `SuiEvent`"));
        };

        let Some(SuiMoveStruct::WithFields(fields)) = fields else {
            return Err(anyhow!("Wrong `SuiMoveStruct`"));
        };

        let Some(SuiMoveValue::Address(addr)) = fields.get("id") else {
            return Err(anyhow!("Wrong `SuiMoveValue`"));
        };

        let Some(SuiMoveValue::String(url)) = fields.get("url") else {
            return Err(anyhow!("Wrong `SuiMoveValue`"))
        };

        let Some(SuiMoveValue::Vector(traits)) = fields.get("traits") else {
            return Err(anyhow!("Wrong `SuiMoveValue`"));
        };

        let mut ret_traits = Vec::new();
        for item in traits {
            let SuiMoveValue::Struct(SuiMoveStruct::WithTypes { fields, .. }) = item else {
                continue
            };

            let Some(SuiMoveValue::String(name)) = fields.get("name").cloned() else {
                continue
            };

            let Some(SuiMoveValue::String(flavour)) = fields.get("flavour").cloned() else {
                continue
            };

            ret_traits.push(Trait { name, flavour })
        }

        Ok(NftToken {
            id: addr.to_string(),
            r#type: "lemon".to_string(),
            owner: sender.to_string(),
            url: url.to_owned(),
            traits: ret_traits,
            created_at: chrono::Utc::now(),
        })
    }
}
