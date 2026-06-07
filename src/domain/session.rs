#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub account_id: u32,
    pub char_id: u32,
    pub ip_address: String,
    pub login_time: i64,
    pub logout_time: i64,
    pub cv_inter_action: f64,
    pub max_idle_gaps_ms: u64,
    pub teleport_count: u32,
    pub avg_post_tp_action_delay_ms: f64,
    pub std_post_tp_action_delay_ms: f64,
    pub chat_count: u32,
    pub bot_score: f64,
}

pub struct WelfordState {
    count: u64,
    mean: f64,
    m2: f64,
}

pub struct SessionState {
    pub account_id: u32,
    pub char_id: u32,
    pub ip: u32,
    pub login_time: i64,
    pub map: String,
    pub welford: WelfordState,
    pub last_event_at: i64,
    pub max_idle_gaps_ms: u64,
    pub chat_count: u32,
    pub teleport_count: u32,
    pub last_tp_at: Option<i64>,
    pub post_tp_delays: Vec<f64>,
}

impl SessionState {
    pub fn new(account_id: u32, char_id: u32, ip: u32, login_time: i64, map: String) -> Self {
        SessionState {
            account_id,
            char_id,
            ip,
            login_time,
            map,
            welford: WelfordState::new(),
            last_event_at: login_time,
            max_idle_gaps_ms: 0,
            chat_count: 0,
            teleport_count: 0,
            last_tp_at: None,
            post_tp_delays: Vec::new(),
        }
    }

    pub fn update_timing(&mut self, timestamp: i64) {
        if let Some(tp_at) = self.last_tp_at.take() {
            self.post_tp_delays.push((timestamp - tp_at) as f64);
        }
        let gap = (timestamp - self.last_event_at).max(0) as u64;
        if gap > self.max_idle_gaps_ms {
            self.max_idle_gaps_ms = gap;
        }
        self.welford.update(gap as f64);
        self.last_event_at = timestamp;
    }

    pub fn finalize(self, logout_time: i64) -> Session {
        let n = self.post_tp_delays.len();
        let avg_post_tp = if n == 0 {
            0.0
        } else {
            self.post_tp_delays.iter().sum::<f64>() / n as f64
        };
        let std_post_tp = if n < 2 {
            0.0
        } else {
            let variance = self.post_tp_delays.iter()
                .map(|&d| (d - avg_post_tp).powi(2))
                .sum::<f64>() / (n as f64 - 1.0);
            variance.sqrt()
        };

        let cv = self.welford.cv();
        let session_ms = (logout_time - self.login_time).max(0) as u64;

        let mut bot_score: f64 = 0.0;
        if cv < 0.3 { bot_score += 30.0; }
        if self.max_idle_gaps_ms < 5_000 && session_ms > 60_000 { bot_score += 25.0; }
        if self.chat_count == 0 && session_ms > 1_800_000 { bot_score += 20.0; }
        if avg_post_tp > 0.0 && avg_post_tp < 500.0 { bot_score += 25.0; }

        Session {
            account_id: self.account_id,
            char_id: self.char_id,
            ip_address: format!(
                "{}.{}.{}.{}",
                (self.ip >> 24) & 0xFF,
                (self.ip >> 16) & 0xFF,
                (self.ip >> 8) & 0xFF,
                self.ip & 0xFF,
            ),
            login_time: self.login_time,
            logout_time,
            cv_inter_action: cv,
            max_idle_gaps_ms: self.max_idle_gaps_ms,
            chat_count: self.chat_count,
            teleport_count: self.teleport_count,
            avg_post_tp_action_delay_ms: avg_post_tp,
            std_post_tp_action_delay_ms: std_post_tp,
            bot_score,
        }
    }
}

impl WelfordState {
    pub fn new() -> Self {
        WelfordState {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    pub fn variance(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        self.m2 / (self.count as f64 - 1.0)
    }

    pub fn stddev(&self) -> f64 {
        self.variance().sqrt()
    }

    pub fn cv(&self) -> f64 {
        if self.mean == 0.0 {
            return 0.0;
        }
        self.stddev() / self.mean
    }
}
