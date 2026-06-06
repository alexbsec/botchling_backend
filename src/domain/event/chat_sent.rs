use super::{BotchlingEvent, CHAT_SIZE_MAX, MAP_NAME_LEN, read_map};
use crate::error::Error;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ChatSentEvent {
    pub account_id: u32,
    pub char_id:    u32,
    pub timestamp:  i64,
    pub message:    String,
    pub map:        String,
}

pub fn parse(
    buf: &[u8],
    timestamp: i64,
    account_id: u32,
    char_id: u32,
) -> Result<BotchlingEvent, Error> {
    if buf.len() < 273 + MAP_NAME_LEN {
        return Err(Error::new("Buffer too short for ChatSentEvent"));
    }

    let msg_slice = &buf[17..17 + CHAT_SIZE_MAX];
    let msg_end = msg_slice.iter().position(|&b| b == 0).unwrap_or(CHAT_SIZE_MAX);
    let message = String::from_utf8_lossy(&msg_slice[..msg_end]).into_owned();

    Ok(BotchlingEvent::ChatSent(ChatSentEvent {
        timestamp,
        account_id,
        char_id,
        message,
        map: read_map(buf, 273),
    }))
}
