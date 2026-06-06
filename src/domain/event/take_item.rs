use super::{BotchlingEvent, MAP_NAME_LEN, read_map};
use crate::error::Error;
use std::ptr::read_unaligned;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TakeItemEvent {
    pub account_id: u32,
    pub char_id:    u32,
    pub timestamp:  i64,
    pub item_id:    u32,
    pub amount:     u16,
    pub map:        String,
}

pub fn parse(
    buf: &[u8],
    timestamp: i64,
    account_id: u32,
    char_id: u32,
) -> Result<BotchlingEvent, Error> {
    if buf.len() < 23 + MAP_NAME_LEN {
        return Err(Error::new("Buffer too short for TakeItemEvent"));
    }

    Ok(BotchlingEvent::TakeItem(TakeItemEvent {
        timestamp,
        account_id,
        char_id,
        item_id: unsafe { read_unaligned(buf.as_ptr().add(17) as *const u32) },
        amount:  unsafe { read_unaligned(buf.as_ptr().add(21) as *const u16) },
        map:     read_map(buf, 23),
    }))
}
