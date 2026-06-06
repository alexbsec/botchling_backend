use super::{BotchlingEvent, MAP_NAME_LEN, read_map};
use crate::error::Error;
use std::ptr::read_unaligned;

pub struct LoginEvent {
    pub account_id: u32,
    pub char_id: u32,
    pub timestamp: i64,
    pub ip: u32,
    pub map: String,
}

pub fn parse(
    buf: &[u8],
    timestamp: i64,
    account_id: u32,
    char_id: u32,
) -> Result<BotchlingEvent, Error> {
    if buf.len() < 21 + MAP_NAME_LEN {
        return Err(Error::new("Buffer too short for login event"));
    }

    Ok(BotchlingEvent::Login(LoginEvent {
        timestamp,
        account_id,
        char_id,
        ip: unsafe { read_unaligned(buf.as_ptr().add(17) as *const u32) },
        map: read_map(buf, 21),
    }))
}
