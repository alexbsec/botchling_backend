use super::BotchlingEvent;
use crate::error::Error;

pub struct LogoutEvent {
    pub account_id: u32,
    pub char_id: u32,
    pub timestamp: i64,
}

pub fn parse(
    _buf: &[u8],
    timestamp: i64,
    account_id: u32,
    char_id: u32,
) -> Result<BotchlingEvent, Error> {
    Ok(BotchlingEvent::Logout(LogoutEvent {
        timestamp,
        account_id,
        char_id,
    }))
}
