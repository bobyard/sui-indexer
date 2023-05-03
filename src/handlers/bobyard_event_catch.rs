use crate::models::lists::ListType;
use crate::models::offers::OfferType;
use crate::models::orders::OrderType;
use crate::models::{lists, offers, orders};
use crate::schema::offers::offer_time;
use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use diesel::{PgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use sui_sdk::rpc_types::SuiEvent;
use tracing::info;

// struct ListEvent<phantom T> has copy, drop {
// list_id: ID,
// list_item_id: ID,
// expire_time: u64,
// ask: u64,
// owner: address,
// }
//
// struct DeListEvent<phantom T> has copy, drop {
// list_id: ID,
// list_item_id: ID,
// expire_time: u64,
// ask: u64,
// owner: address,
// }
//
// struct BuyEvent<phantom T> has copy, drop {
// list_id: ID,
// ask: u64,
// owner: address,
// buyer: address,
// }
//
// struct AcceptOfferEvent<phantom T> has copy, drop {
// offer_id: ID,
// list_id: ID,
// offer_amount: u64,
// owner: address,
// buyer: address,
// }
//
// struct OfferEvent<phantom T> has copy, drop {
// offer_id: ID,
// list_id: ID,
// offer_amount: u64,
// expire_time: u64,
// owner: address,
// }
//
// struct CancelOfferEvent<phantom T> has copy, drop {
// offer_id: ID,
// list_id: ID,
// owner: address,
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    list_id: String,
    list_item_id: String,
    expire_time: String,
    ask: String,
    owner: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeList {
    list_id: String,
    list_item_id: String,
    expire_time: String,
    ask: String,
    owner: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Buy {
    list_id: String,
    ask: String,
    owner: String,
    buyer: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceptOffer {
    offer_id: String,
    list_id: String,
    offer_amount: String,
    owner: String,
    buyer: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MakeOffer {
    offer_id: String,
    list_id: String,
    offer_amount: String,
    expire_time: String,
    owner: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CancelOffer {
    offer_id: String,
    list_id: String,
    owner: String,
}

#[derive(Debug)]
pub enum BobYardEvent {
    List(List),
    DeList(DeList),
    Buy(Buy),
    AcceptOffer(AcceptOffer),
    MakeOffer(MakeOffer),
    CancelOffer(CancelOffer),
}

pub fn parse_bob_yard_event(
    events: &Vec<SuiEvent>,
    event_address: &str,
) -> Result<Vec<BobYardEvent>> {
    let bob_yard_events = events
        .into_iter()
        .filter_map(|e| {
            if &e.package_id.to_string() == event_address {
                let event_data = e.parsed_json.clone();
                let event_name = e.type_.name.clone().to_string();
                match event_name.as_str() {
                    "ListEvent" => {
                        let list: List = serde_json::from_value(event_data).unwrap();
                        Some(BobYardEvent::List(list))
                    }
                    "DeListEvent" => {
                        let de_list: DeList = serde_json::from_value(event_data).unwrap();
                        Some(BobYardEvent::DeList(de_list))
                    }
                    "BuyEvent" => {
                        let buy: Buy = serde_json::from_value(event_data).unwrap();
                        Some(BobYardEvent::Buy(buy))
                    }
                    "AcceptOfferEvent" => {
                        let accept_offer: AcceptOffer = serde_json::from_value(event_data).unwrap();
                        Some(BobYardEvent::AcceptOffer(accept_offer))
                    }
                    "OfferEvent" => {
                        let make_offer: MakeOffer = serde_json::from_value(event_data).unwrap();
                        Some(BobYardEvent::MakeOffer(make_offer))
                    }
                    "CancelOfferEvent" => {
                        let cancel_offer: CancelOffer = serde_json::from_value(event_data).unwrap();
                        Some(BobYardEvent::CancelOffer(cancel_offer))
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<BobYardEvent>>();

    Ok(bob_yard_events)
}

impl From<&List> for lists::List {
    fn from(list: &List) -> Self {
        lists::List {
            chain_id: 1,
            coin_id: 1,
            list_id: list.list_id.clone(),
            list_time: Utc::now().naive_utc(),
            token_id: list.list_item_id.clone(),
            seller_address: list.owner.clone(),
            seller_value: list.ask.parse().unwrap(),
            list_type: ListType::Listed,
            expire_time: NaiveDateTime::from_timestamp_millis(list.expire_time.parse().unwrap())
                .unwrap(),
            created_at: Some(Utc::now().naive_utc()),
            updated_at: Some(Utc::now().naive_utc()),
        }
    }
}

impl From<&Buy> for orders::Order {
    fn from(buy: &Buy) -> Self {
        orders::Order {
            chain_id: 1,
            coin_id: 1,
            token_id: buy.list_id.clone(),
            buyer_address: buy.buyer.clone(),
            value: buy.ask.parse().unwrap(),
            seller_address: buy.owner.clone(),
            order_type: OrderType::Sold,
            created_at: Some(Utc::now().naive_utc()),
            updated_at: Some(Utc::now().naive_utc()),
            list_id: buy.list_id.clone(),
            offer_id: None,
            sell_time: Default::default(),
            expire_time: Default::default(),
        }
    }
}

impl From<&MakeOffer> for offers::Offer {
    fn from(make_offer: &MakeOffer) -> Self {
        offers::Offer {
            chain_id: 1,
            coin_id: 1,
            offer_id: make_offer.offer_id.clone(),
            list_id: make_offer.list_id.clone(),
            buyer_address: make_offer.owner.clone(),
            offer_type: OfferType::Listed,
            offer_value: make_offer.offer_amount.parse().unwrap(),
            expire_time: NaiveDateTime::from_timestamp_millis(
                make_offer.expire_time.parse().unwrap(),
            )
            .unwrap(),
            offer_time: Default::default(),
            created_at: Some(Utc::now().naive_utc()),
            updated_at: Some(Utc::now().naive_utc()),
        }
    }
}

impl From<&AcceptOffer> for orders::Order {
    fn from(accept_offer: &AcceptOffer) -> Self {
        orders::Order {
            chain_id: 1,
            coin_id: 1,
            token_id: accept_offer.list_id.clone(),
            buyer_address: accept_offer.buyer.clone(),
            value: accept_offer.offer_amount.parse().unwrap(),
            seller_address: accept_offer.owner.clone(),
            order_type: OrderType::Offer,
            created_at: Some(Utc::now().naive_utc()),
            updated_at: Some(Utc::now().naive_utc()),
            list_id: accept_offer.list_id.clone(),
            offer_id: Some(accept_offer.offer_id.clone()),
            sell_time: Default::default(),
            expire_time: Default::default(),
        }
    }
}

pub fn event_handle(
    event: &Vec<BobYardEvent>,
    event_time: i64,
    pg: &mut PgConnection,
) -> Result<()> {
    let _ = event.into_iter().for_each(|e| {
        match e {
            BobYardEvent::List(list) => {
                let mut list: lists::List = list.into();
                list.list_time = NaiveDateTime::from_timestamp_millis(event_time as i64).unwrap();
                info!("list {:?}", list);
                lists::batch_insert(pg, &vec![list]).expect("batch_insert error");
            }
            BobYardEvent::DeList(de_list) => {
                info!("de_list {:?}", de_list);
                lists::delete(pg, &de_list.list_id).expect("batch_insert error");
            }
            BobYardEvent::Buy(buy) => {
                // delete the list.
                lists::delete(pg, &buy.list_id).expect("batch_insert error");
                // insert the order.
                let mut order: orders::Order = buy.into();
                info!("buy {:?}", order);
                orders::batch_insert(pg, &vec![order]).expect("batch_insert error");
            }
            BobYardEvent::AcceptOffer(accept_offer) => {
                // delete the list.
                lists::delete(pg, &accept_offer.list_id).expect("batch_insert error");
                offers::delete(pg, &accept_offer.offer_id).expect("batch_insert error");
                let mut order: orders::Order = accept_offer.into();
                info!("accept_offer {:?}", order);
                orders::batch_insert(pg, &vec![order]).expect("batch_insert error");
            }
            BobYardEvent::MakeOffer(make_offer) => {
                let mut offer_to_db: offers::Offer = make_offer.into();
                offer_to_db.offer_time =
                    NaiveDateTime::from_timestamp_millis(event_time as i64).unwrap();
                info!("offer_to_db {:?}", offer_to_db);
                offers::batch_insert(pg, &vec![offer_to_db]).expect("batch_insert error");
            }
            BobYardEvent::CancelOffer(cancel_offer) => {
                info!("cancel_offer {:?}", cancel_offer);
                offers::delete(pg, &cancel_offer.offer_id).expect("batch_insert error");
            }
        }
    });

    Ok(())
}
