use anyhow::Result;
use serde::{Deserialize, Serialize};
use sui_sdk::rpc_types::SuiEvent;

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
