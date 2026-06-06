use super::{BotchlingEvent, MAP_NAME_LEN, read_map};
use crate::error::Error;
use std::ptr::read_unaligned;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MapChangeEvent {
    pub account_id: u32,
    pub char_id:    u32,
    pub timestamp:  i64,
    pub x:          i16,
    pub y:          i16,
    pub from_map:   String,
    pub to_map:     String,
}

pub fn parse(
    buf: &[u8],
    timestamp: i64,
    account_id: u32,
    char_id: u32,
) -> Result<BotchlingEvent, Error> {
    if buf.len() < 37 + MAP_NAME_LEN {
        return Err(Error::new("Buffer too short for MapChangeEvent"));
    }

    Ok(BotchlingEvent::MapChange(MapChangeEvent {
        timestamp,
        account_id,
        char_id,
        x:        unsafe { read_unaligned(buf.as_ptr().add(17) as *const i16) },
        y:        unsafe { read_unaligned(buf.as_ptr().add(19) as *const i16) },
        from_map: read_map(buf, 21),
        to_map:   read_map(buf, 37),
    }))
}
