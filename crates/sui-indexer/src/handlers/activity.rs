use crate::handlers::bobyard_event_catch::BobYardEvent;
use crate::models::activities::{Activity, ActivityType};
use crate::models::tokens::Token;
use crate::ObjectStatus;

use super::bobyard_event_catch::EventIndex;

pub fn parse_tokens_activity(
    events: &Vec<EventIndex>,
    tokens: &Vec<(ObjectStatus, (Token, String))>,
) -> Vec<Activity> {
    let bob_yard_events = events
        .iter()
        .filter_map(|e| {
            if let EventIndex::BobYard(bob_yard_event) = e {
                Some(bob_yard_event.clone())
            } else {
                None
            }
        })
        .collect::<Vec<&BobYardEvent>>();

    let token_activities = Vec::new();
    let changed_tokens = tokens
        .iter()
        .filter_map(|(objects, token)| {
            if *objects == ObjectStatus::Wrapped
                || *objects == ObjectStatus::Mutated
            {
                return Some(token.clone());
            }
            None
        })
        .collect::<Vec<(Token, String)>>();

    if changed_tokens.len() <= 0 {
        //list to other market or stake to contract. no need to parse
        return token_activities;
    }

    let mut activity = vec![];
    for event in bob_yard_events {
        let _token_activity = match event {
            BobYardEvent::List(list) => {
                let _ = changed_tokens.iter().for_each(|token| {
                    if token.0.token_id == list.list_item_id {
                        let mut list_act = Activity::new_from_token_with_type(
                            ActivityType::Listed,
                            &token,
                        );
                        list_act.from_address = Some(token.1.clone());
                        list_act.to_address = Some(token.1.clone());
                        list_act.token_amount = list.ask.parse().unwrap();

                        activity.push(list_act);
                    }
                });
            }
            BobYardEvent::DeList(delist) => {
                let _ = changed_tokens.iter().for_each(|token| {
                    if token.0.token_id == delist.list_item_id {
                        let mut list_act = Activity::new_from_token_with_type(
                            ActivityType::Canceled,
                            &token,
                        );
                        list_act.from_address = Some(token.1.clone());
                        list_act.to_address = Some(token.1.clone());
                        list_act.token_amount = delist.ask.parse().unwrap();

                        activity.push(list_act);
                    }
                });
            }

            BobYardEvent::Buy(buy) => {
                let _ = changed_tokens.iter().for_each(|token| {
                    if token.0.token_id == buy.item_id {
                        let mut list_act = Activity::new_from_token_with_type(
                            ActivityType::Sold,
                            &token,
                        );
                        list_act.from_address = Some(buy.owner.clone());
                        list_act.to_address = Some(buy.buyer.clone());
                        list_act.token_amount = buy.ask.parse().unwrap();
                        activity.push(list_act);
                    }
                });
            }

            BobYardEvent::AcceptOffer(buy) => {
                let _ = changed_tokens.iter().for_each(|token| {
                    if token.0.token_id == buy.item_id {
                        let mut list_act = Activity::new_from_token_with_type(
                            ActivityType::Sold,
                            &token,
                        );
                        list_act.from_address = Some(buy.owner.clone());
                        list_act.to_address = Some(buy.buyer.clone());
                        list_act.token_amount =
                            buy.offer_amount.parse().unwrap();
                        activity.push(list_act);
                    }
                });
            }

            BobYardEvent::MakeOffer(_) => {}
            BobYardEvent::CancelOffer(_) => {}
        };
    }

    changed_tokens.iter().for_each(|token| {
        let mk_event = activity.clone();
        let mut have = false;
        mk_event.iter().for_each(|e| {
            if e.token_data_id_hash == token.0.token_id {
                have = true;
            }
        });
        if !have {
            let list_act = Activity::new_from_token_with_type(
                ActivityType::Transferred,
                &token,
            );
            activity.push(list_act);
        }
    });

    activity
}
