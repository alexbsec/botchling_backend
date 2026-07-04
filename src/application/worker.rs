use crate::domain::event::{BotchlingEvent, ChatSentEvent, LoginEvent, LogoutEvent, MapChangeEvent, TeleportEvent};
use crate::domain::session::SessionState;
use crate::infrastructure::discord;
use crate::infrastructure::mongo::{Database as MongoDatabase, SessionRepository};
use crate::{log_error, log_info};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

// Login-count milestones that trigger a Discord notification. Beyond the
// last one, every additional 100 logins fires again (see is_milestone).
const LOGIN_MILESTONES: &[u64] = &[1, 10, 50, 100];

fn is_milestone(count: u64) -> bool {
    LOGIN_MILESTONES.contains(&count) || (count > 100 && count % 100 == 0)
}

pub struct Worker {
    session_repo: SessionRepository,
    rx: Receiver<BotchlingEvent>,
    sessions: HashMap<u32, SessionState>,
    login_count: u64,
    discord_webhook_url: String,
}

impl Worker {
    pub fn new(db: &MongoDatabase, rx: Receiver<BotchlingEvent>, discord_webhook_url: String) -> Self {
        let session_collection = db.collection("sessions");
        let session_repo = SessionRepository::new(session_collection);
        Self {
            rx,
            session_repo,
            sessions: HashMap::new(),
            login_count: 0,
            discord_webhook_url,
        }
    }

    pub async fn run(mut self) {
        log_info!("Worker is running...");
        while let Some(event) = self.rx.recv().await {
            match event {
                BotchlingEvent::Login(e)     => self.handle_login(e),
                BotchlingEvent::Logout(e)    => self.handle_logout(e).await,
                BotchlingEvent::Walk(e)      => self.handle_timing(e.char_id, e.timestamp),
                BotchlingEvent::MobKill(e)   => self.handle_timing(e.char_id, e.timestamp),
                BotchlingEvent::TakeItem(e)  => self.handle_timing(e.char_id, e.timestamp),
                BotchlingEvent::Teleport(e)  => self.handle_teleport(e),
                BotchlingEvent::MapChange(e) => self.handle_map_change(e),
                BotchlingEvent::ChatSent(e)  => self.handle_chat(e),
            }
        }
    }

    fn handle_login(&mut self, e: LoginEvent) {
        if self.sessions.contains_key(&e.char_id) {
            log_error!("Duplicate login for char_id: {}", e.char_id);
            return;
        }
        self.sessions.insert(
            e.char_id,
            SessionState::new(e.account_id, e.char_id, e.ip, e.timestamp, e.map),
        );

        self.login_count += 1;
        if is_milestone(self.login_count) && !self.discord_webhook_url.is_empty() {
            let webhook_url = self.discord_webhook_url.clone();
            let login_count = self.login_count;
            let online_now = self.sessions.len();
            let content = format!(
                "\u{1F3AE} O servidor tem cara nova! **{}** jogador(es) entraram no servidor, com **{}** online(s) agora.",
                login_count, online_now
            );
            tokio::spawn(async move {
                if let Err(e) = discord::notify(&webhook_url, &content).await {
                    log_error!("Failed to send Discord milestone notification: {}", e.message);
                }
            });
        }
    }

    async fn handle_logout(&mut self, e: LogoutEvent) {
        if let Some(state) = self.sessions.remove(&e.char_id) {
            let session = state.finalize(e.timestamp);
            if let Err(err) = self.session_repo.insert(session).await {
                log_error!("Failed to persist session for char_id {}: {}", e.char_id, err.message);
            }
        } else {
            log_error!("Logout for unknown char_id: {}", e.char_id);
        }
    }

    fn handle_timing(&mut self, char_id: u32, timestamp: i64) {
        if let Some(state) = self.sessions.get_mut(&char_id) {
            state.update_timing(timestamp);
        }
    }

    fn handle_teleport(&mut self, e: TeleportEvent) {
        if let Some(state) = self.sessions.get_mut(&e.char_id) {
            state.update_timing(e.timestamp);
            state.teleport_count += 1;
            state.last_tp_at = Some(e.timestamp);
            state.map = e.map;
        }
    }

    fn handle_map_change(&mut self, e: MapChangeEvent) {
        if let Some(state) = self.sessions.get_mut(&e.char_id) {
            state.update_timing(e.timestamp);
            state.map = e.to_map;
        }
    }

    fn handle_chat(&mut self, e: ChatSentEvent) {
        if let Some(state) = self.sessions.get_mut(&e.char_id) {
            state.update_timing(e.timestamp);
            state.chat_count += 1;
        }
    }
}
