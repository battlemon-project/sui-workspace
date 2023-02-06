use crate::errors::Error;
use crate::{Item, Nft, Trait};
use chrono::Utc;
use std::collections::BTreeMap;
use sui_sdk::rpc_types::{SuiEvent, SuiMoveStruct, SuiMoveValue};
use sui_sdk::types::base_types::SuiAddress;

const LEMON_ID: &str = "lemon_id";
const ITEM_ID: &str = "item_id";
const ID: &str = "id";
const URL: &str = "url";
const TRAITS: &str = "traits";
const NAME: &str = "name";
const FLAVOUR: &str = "flavour";

#[derive(Debug, Clone)]
pub enum Event {
    Nft(Nft),
    ItemAdded(Item),
    ItemRemoved(Item),
}

impl TryFrom<SuiEvent> for Event {
    type Error = Error;

    fn try_from(event: SuiEvent) -> Result<Self, Self::Error> {
        let SuiEvent::MoveEvent { sender, fields, type_: event_type, .. } = event else {
            return Err(Error::UnsupportedSuiEvent(event.get_event_type()));
        };

        let Some(SuiMoveStruct::WithFields(fields)) = fields else {
            return Err(Error::EventWithoutFields);
        };

        match event_type.rsplit("::").next() {
            Some("LemonCreated") => parse_event_nft_created(fields, sender, "lemon"),
            Some("ItemCreated") => parse_event_nft_created(fields, sender, "item"),
            Some("ItemAdded") => parse_event_item_added(fields),
            Some("ItemRemoved") => parse_event_item_removed(fields),
            None => Err(Error::EventTypeSplit),
            Some(rest) => Err(Error::UnsupportedEventType(rest.to_string())),
        }
    }
}

fn parse_event_item_added(mut fields: BTreeMap<String, SuiMoveValue>) -> Result<Event, Error> {
    let Some(SuiMoveValue::String(lemon_id)) = fields.remove(LEMON_ID) else {
        return Err(Error::WrongEventFieldName(LEMON_ID.to_string()))
    };

    let Some(SuiMoveValue::String(item_id)) = fields.remove(ITEM_ID) else {
        return Err(Error::WrongEventFieldName(ITEM_ID.to_string()))
    };

    Ok(Event::ItemAdded(Item { lemon_id, item_id }))
}

fn parse_event_item_removed(mut fields: BTreeMap<String, SuiMoveValue>) -> Result<Event, Error> {
    let Some(SuiMoveValue::String(lemon_id)) = fields.remove(LEMON_ID) else {
        return Err(Error::WrongEventFieldName(LEMON_ID.to_string()))
    };

    let Some(SuiMoveValue::String(item_id)) = fields.remove(ITEM_ID) else {
        return Err(Error::WrongEventFieldName(ITEM_ID.to_string()))
    };

    Ok(Event::ItemRemoved(Item { lemon_id, item_id }))
}

fn parse_event_nft_created(
    fields: BTreeMap<String, SuiMoveValue>,
    sender: SuiAddress,
    nft_type: &str,
) -> Result<Event, Error> {
    let Some(SuiMoveValue::Address(id)) = fields.get(ID) else {
        return Err(Error::WrongEventFieldName(ID.to_string()))
    };
    let Some(SuiMoveValue::String(url)) = fields.get(URL) else {
        return Err(Error::WrongEventFieldName(URL.to_string()))
    };
    let Some(SuiMoveValue::Vector(traits)) = fields.get(TRAITS) else {
        return Err(Error::WrongEventFieldName(TRAITS.to_string()))
    };

    let mut ret_traits = Vec::new();
    for item in traits {
        let SuiMoveValue::Struct(SuiMoveStruct::WithTypes { fields, .. }) = item else {
            continue
        };

        let Some(SuiMoveValue::String(name)) = fields.get(NAME).cloned() else {
            continue
        };

        let Some(SuiMoveValue::String(flavour)) = fields.get(FLAVOUR).cloned() else {
            continue
        };

        ret_traits.push(Trait { name, flavour })
    }

    Ok(Event::Nft(Nft {
        id: id.to_string(),
        r#type: nft_type.to_string(),
        owner: sender.to_string(),
        url: url.to_owned(),
        traits: ret_traits,
        items: Vec::new(),
        created_at: Utc::now(),
    }))
}
