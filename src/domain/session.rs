#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub account_id: u32,
    pub char_id: u32,
    pub ip_address: String,
    pub login_time: i64,
    pub logout_time: i64,
    pub cv_inter_action: f64,
    pub max_idle_gaps_ms: u32,
    pub teleport_count: u32,
    pub avg_teleport_per_map: f64,
    pub std_teleport_per_map: f64,
    pub avg_teleport_per_minute: f64,
    pub std_teleport_per_minute: f64,
    pub avg_post_tp_action_delay_ms: f64,
    pub std_post_tp_action_delay_ms: f64,
    pub bot_score: f64,
}
