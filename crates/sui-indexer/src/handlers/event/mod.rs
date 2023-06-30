use anyhow::Result;
use diesel::PgConnection;
use sui_sdk::rpc_types::SuiEvent;

pub mod bobyard_event;
pub mod kiosk_event;
pub mod origin_byte_event;

const SYSTEM_MODULE: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000002";

#[derive(Debug)]
pub enum EventIndex {
    BobYard(bobyard_event::BobYardEvent),
    OriginByte(origin_byte_event::OriginByteEvent),
    KioskEvent(kiosk_event::KioskEvent),
}

pub struct EventAccount {
    bob_yard: String,
    origin_byte: String,
}

impl EventAccount {
    pub fn new(bob_yard: String, origin_byte: String) -> Self {
        Self {
            bob_yard,
            origin_byte,
        }
    }
}

pub fn parse_event(
    events: &Vec<SuiEvent>,
    event_account: &EventAccount,
) -> Result<Vec<EventIndex>> {
    let events = events
        .into_iter()
        .filter_map(|e| {
            dbg!(&e.package_id.to_string());
            dbg!(&e.parsed_json);
            if e.package_id.to_string() == event_account.bob_yard {
                bobyard_event::event_parse(e)
            } else if e.package_id.to_string() == event_account.origin_byte {
                origin_byte_event::event_parse(e)
            } else if &e.package_id.to_string() == SYSTEM_MODULE {
                kiosk_event::event_parse(e)
            } else {
                None
            }
        })
        .collect::<Vec<EventIndex>>();

    Ok(events)
}

pub fn event_handle(
    event: &Vec<EventIndex>,
    event_time: i64,
    pg: &mut PgConnection,
) -> Result<()> {
    for e in event {
        match e {
            EventIndex::BobYard(e) => {
                bobyard_event::event_handle(e, event_time, pg)?;
            }
            EventIndex::OriginByte(e) => {
                origin_byte_event::event_handle(e, event_time, pg)?;
            }
            EventIndex::KioskEvent(e) => {
                dbg!("kiosk event: {:?}", e);
                kiosk_event::event_handle(e, event_time, pg)?;
            }
        }
    }

    Ok(())
}
