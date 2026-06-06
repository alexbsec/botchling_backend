use super::{BotchlingEvent, MAP_NAME_LEN, read_map};
use crate::error::Error;
use std::ptr::read_unaligned;

pub struct MobKillEvent {
    pub account_id: u32,
    pub char_id:    u32,
    pub timestamp:  i64,
    pub mob_id:     u16,
    pub x:          i16,
    pub y:          i16,
    pub map:        String,
}

pub fn parse(
    buf: &[u8],
    timestamp: i64,
    account_id: u32,
    char_id: u32,
) -> Result<BotchlingEvent, Error> {
    if buf.len() < 23 + MAP_NAME_LEN {
        return Err(Error::new("Buffer too short for MobKillEvent"));
    }

    Ok(BotchlingEvent::MobKill(MobKillEvent {
        timestamp,
        account_id,
        char_id,
        mob_id: unsafe { read_unaligned(buf.as_ptr().add(17) as *const u16) },
        x:      unsafe { read_unaligned(buf.as_ptr().add(19) as *const i16) },
        y:      unsafe { read_unaligned(buf.as_ptr().add(21) as *const i16) },
        map:    read_map(buf, 23),
    }))
}
