pub mod login;
pub mod logout;
pub mod walk;
pub mod teleport;
pub mod map_change;
pub mod mob_kill;
pub mod take_item;
pub mod chat_sent;

pub use login::LoginEvent;
pub use logout::LogoutEvent;
pub use walk::WalkEvent;
pub use teleport::TeleportEvent;
pub use map_change::MapChangeEvent;
pub use mob_kill::MobKillEvent;
pub use take_item::TakeItemEvent;
pub use chat_sent::ChatSentEvent;

pub const MAP_NAME_LEN: usize = 16;
pub const CHAT_SIZE_MAX: usize = 256;
pub const HEADER_SIZE: usize = 17;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum BotchlingEvent {
    Login(LoginEvent),
    Logout(LogoutEvent),
    Walk(WalkEvent),
    Teleport(TeleportEvent),
    MapChange(MapChangeEvent),
    MobKill(MobKillEvent),
    TakeItem(TakeItemEvent),
    ChatSent(ChatSentEvent),
}

pub fn read_map(buf: &[u8], offset: usize) -> String {
    let slice = &buf[offset..offset + MAP_NAME_LEN];
    let end = slice.iter().position(|&b| b == 0).unwrap_or(MAP_NAME_LEN);
    String::from_utf8_lossy(&slice[..end]).into_owned()
}

unsafe fn read_at<T>(buf: &[u8], offset: usize) -> T {
    unsafe { std::ptr::read_unaligned(buf.as_ptr().add(offset) as *const T) }
}

impl TryFrom<&[u8]> for BotchlingEvent {
    type Error = crate::error::Error;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.len() < HEADER_SIZE {
            return Err(crate::error::Error::new("Buffer too short for header"));
        }

        let event_type = unsafe { read_at::<u8>(buf, 0) };
        let timestamp = unsafe { read_at::<i64>(buf, 1) };
        let account_id = unsafe { read_at::<u32>(buf, 9) };
        let char_id = unsafe { read_at::<u32>(buf, 13) };

        match event_type {
            1 => login::parse(buf, timestamp, account_id, char_id),
            2 => logout::parse(buf, timestamp, account_id, char_id),
            3 => walk::parse(buf, timestamp, account_id, char_id),
            4 => teleport::parse(buf, timestamp, account_id, char_id),
            5 => map_change::parse(buf, timestamp, account_id, char_id),
            6 => mob_kill::parse(buf, timestamp, account_id, char_id),
            7 => take_item::parse(buf, timestamp, account_id, char_id),
            8 => chat_sent::parse(buf, timestamp, account_id, char_id),
            _ => Err(crate::error::Error::new("Unknown event type")),
        }
    }
}
