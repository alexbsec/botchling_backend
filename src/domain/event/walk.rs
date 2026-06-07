use super::{BotchlingEvent, MAP_NAME_LEN, read_map};
use crate::error::Error;
use std::ptr::read_unaligned;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct WalkEvent {
    pub account_id: u32,
    pub char_id: u32,
    pub timestamp: i64,
    pub from_x: i16,
    pub from_y: i16,
    pub to_x: i16,
    pub to_y: i16,
    pub map: String,
}

pub fn parse(
    buf: &[u8],
    timestamp: i64,
    account_id: u32,
    char_id: u32,
) -> Result<BotchlingEvent, Error> {
    if buf.len() < 25 + MAP_NAME_LEN {
        return Err(Error::new("Buffer too short for WalkEvent"));
    }

    Ok(BotchlingEvent::Walk(WalkEvent {
        timestamp,
        account_id,
        char_id,
        from_x: unsafe { read_unaligned(buf.as_ptr().add(17) as *const i16) },
        from_y: unsafe { read_unaligned(buf.as_ptr().add(19) as *const i16) },
        to_x: unsafe { read_unaligned(buf.as_ptr().add(21) as *const i16) },
        to_y: unsafe { read_unaligned(buf.as_ptr().add(23) as *const i16) },
        map: read_map(buf, 25),
    }))
}
