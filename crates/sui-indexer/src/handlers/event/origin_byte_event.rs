use anyhow::Result;
use diesel::PgConnection;
use sui_sdk::rpc_types::SuiEvent;

#[derive(Debug)]
pub enum OriginByteEvent {}

pub fn event_parse(e: &SuiEvent) -> Option<super::EventIndex> {
    let event_data = e.parsed_json.clone();
    let event_name = e.type_.name.clone().to_string();
    let event_module = e.type_.module.to_string();

    match event_name.as_str() {
        _ => None,
    }
}

pub fn event_handle(
    e: &OriginByteEvent,
    event_time: i64,
    pg: &mut PgConnection,
) -> Result<()> {
    Ok(())
}
