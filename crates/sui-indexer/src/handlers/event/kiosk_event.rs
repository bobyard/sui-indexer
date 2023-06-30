use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use diesel::PgConnection;

use crate::models::lists::{self, ListType, MarketType};
use serde::{Deserialize, Serialize};
use sui_sdk::rpc_types::SuiEvent;
use tracing::info;

use super::EventIndex;

#[derive(Debug)]
pub enum KioskEvent {
    ItemListed(ItemListedWithSender),
    ItemDelisted(ItemDelisted),
}

impl From<KioskEvent> for EventIndex {
    fn from(event: KioskEvent) -> Self { EventIndex::KioskEvent(event) }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemListed {
    id: String,
    kiosk: String,
    price: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemListedWithSender {
    id: String,
    kiosk: String,
    price: String,
    sender: String,
}

impl ItemListedWithSender {
    pub fn new(list: ItemListed, sender: String) -> Self {
        ItemListedWithSender {
            id: list.id,
            kiosk: list.kiosk,
            price: list.price,
            sender,
        }
    }
}

impl From<(&ItemListedWithSender)> for lists::List {
    fn from(list: &ItemListedWithSender) -> Self {
        lists::List {
            chain_id: 1,
            coin_id: 1,
            list_id: list.kiosk.clone(),
            list_time: Utc::now().naive_utc(),
            token_id: list.id.clone(),
            seller_address: list.sender.clone(),
            seller_value: list.price.parse().unwrap(),
            list_type: ListType::Listed,
            market_type: MarketType::Kiosk,
            expire_time: None,
            created_at: Some(Utc::now().naive_utc()),
            updated_at: Some(Utc::now().naive_utc()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemDelisted {
    id: String,
    kiosk: String,
}

pub fn event_parse(e: &SuiEvent) -> Option<super::EventIndex> {
    let event_data = e.parsed_json.clone();
    let event_module = e.type_.module.to_string();

    let event_name = e.type_.name.clone().to_string();

    match event_name.as_str() {
        "ItemListed" => {
            let list: ItemListed = serde_json::from_value(event_data).unwrap();
            let with_sender =
                ItemListedWithSender::new(list, e.sender.to_string());
            Some(KioskEvent::ItemListed(with_sender).into())
        }
        "ItemDelisted" => {
            let de_list: ItemDelisted =
                serde_json::from_value(event_data).unwrap();
            Some(KioskEvent::ItemDelisted(de_list).into())
        }
        _ => None,
    }
}

pub fn event_handle(
    e: &KioskEvent,
    event_time: i64,
    pg: &mut PgConnection,
) -> Result<()> {
    match e {
        KioskEvent::ItemListed(list) => {
            let mut list: lists::List = list.into();
            list.list_time =
                NaiveDateTime::from_timestamp_millis(event_time as i64)
                    .unwrap();

            info!("list {:?}", list);
            lists::batch_insert(pg, &vec![list]).expect("batch_insert error");
        }
        KioskEvent::ItemDelisted(de_list) => {
            info!("de_list {:?}", de_list);
            lists::delete(pg, &de_list.id).expect("batch_insert error");
        }
    }

    Ok(())
}
