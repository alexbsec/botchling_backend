use std::collections::BTreeMap;
use std::fmt;
use std::sync::{OnceLock};
use std::sync::atomic::{AtomicU8, Ordering};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::{format::Writer, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn init(log_level: LogLevel) {
    let _ = LOGGER.set(Logger::new(log_level));
}

pub(crate) fn get() -> &'static Logger {
    LOGGER.get_or_init(|| Logger::new(LogLevel::Info))
}

pub fn should_log(level: LogLevel) -> bool {
    (level as u8) >= get().log_level_weight.load(Ordering::Relaxed)
}


pub fn set_log_level(level_str: &str) {
    let level = match string_to_log_level(level_str) {
        Some(lvl) => lvl,
        _ => {
            LogLevel::Info
        }
    };
    get().log_level_weight.store(level as u8, Ordering::Relaxed);
}

fn string_to_log_level(level: &str) -> Option<LogLevel> {
    match level.to_lowercase().as_str() {
        "trace" => Some(LogLevel::Trace),
        "debug" => Some(LogLevel::Debug),
        "info" => Some(LogLevel::Info),
        "warning" | "warn" => Some(LogLevel::Warning),
        "error" => Some(LogLevel::Error),
        "fatal" => Some(LogLevel::Fatal),
        _ => None,
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace = 1,
    Debug = 5,
    Info = 15,
    Warning = 50,
    Error = 100,
    Fatal = 255,
}

struct FieldCollector {
    fields: BTreeMap<String, serde_json::Value>,
    is_fatal: bool,
}

impl FieldCollector {
    fn new() -> Self {
        Self { fields: BTreeMap::new(), is_fatal: false }
    }
}

impl tracing::field::Visit for FieldCollector {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if let Some(n) = serde_json::Number::from_f64(value) {
            self.fields.insert(field.name().to_string(), serde_json::Value::Number(n));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if field.name() == "__fatal__" {
            self.is_fatal = value;
        } else {
            self.fields.insert(field.name().to_string(), serde_json::Value::Bool(value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields.insert(field.name().to_string(), serde_json::Value::String(value.to_string()));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() != "__fatal__" {
            self.fields.insert(field.name().to_string(), serde_json::Value::String(format!("{:?}", value)));
        }
    }
}

struct FatalFormatter;

impl<S, N> FormatEvent<S, N> for FatalFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        let mut collector = FieldCollector::new();
        event.record(&mut collector);

        let level = if collector.is_fatal { "FATAL" } else { metadata.level().as_str() };
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let entry = serde_json::json!({
            "timestamp": timestamp,
            "level": level,
            "target": metadata.target(),
            "fields": collector.fields,
        });

        writeln!(writer, "{}", entry)
    }
}

pub struct Logger {
    log_level_weight: AtomicU8,
}

impl Logger {
    fn new(log_level: LogLevel) -> Self {
        let _ = tracing_subscriber::fmt()
            .event_format(FatalFormatter)
            .try_init();
        Logger { log_level_weight: AtomicU8::new(log_level as u8) }
    }

    pub fn fatal(&self, message: &str) {
        if (LogLevel::Fatal as u8) >= self.log_level_weight.load(Ordering::Relaxed) {
            tracing::error!(__fatal__ = true, "{}", message);
        }
        std::process::exit(1);
    }

    pub fn log_error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    pub fn log_warn(&self, message: &str) {
        self.log(LogLevel::Warning, message);
    }

    pub fn log_info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    pub fn log_debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    pub fn log_trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }

    fn log(&self, level: LogLevel, message: &str) {
        if (level as u8) >= self.log_level_weight.load(Ordering::Relaxed) {
            match level {
                LogLevel::Trace => tracing::trace!("{}", message),
                LogLevel::Debug => tracing::debug!("{}", message),
                LogLevel::Info => tracing::info!("{}", message),
                LogLevel::Warning => tracing::warn!("{}", message),
                LogLevel::Error => tracing::error!("{}", message),
                LogLevel::Fatal => unreachable!(),
            }
        }
    }
}

#[macro_export]
macro_rules! log_trace {
    ($($key:ident = $val:expr),+ ; $fmt:expr $(, $arg:expr)*) => {{
        if $crate::infrastructure::logger::should_log($crate::infrastructure::logger::LogLevel::Trace) {
            tracing::trace!($($key = $val,)+ "{}", format!($fmt $(, $arg)*));
        }
    }};
    ($fmt:expr $(, $arg:expr)*) => {
        $crate::infrastructure::logger::get().log_trace(&format!($fmt $(, $arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($key:ident = $val:expr),+ ; $fmt:expr $(, $arg:expr)*) => {{
        if $crate::infrastructure::logger::should_log($crate::infrastructure::logger::LogLevel::Debug) {
            tracing::debug!($($key = $val,)+ "{}", format!($fmt $(, $arg)*));
        }
    }};
    ($fmt:expr $(, $arg:expr)*) => {
        $crate::infrastructure::logger::get().log_debug(&format!($fmt $(, $arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($key:ident = $val:expr),+ ; $fmt:expr $(, $arg:expr)*) => {{
        if $crate::infrastructure::logger::should_log($crate::infrastructure::logger::LogLevel::Info) {
            tracing::info!($($key = $val,)+ "{}", format!($fmt $(, $arg)*));
        }
    }};
    ($fmt:expr $(, $arg:expr)*) => {
        $crate::infrastructure::logger::get().log_info(&format!($fmt $(, $arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($key:ident = $val:expr),+ ; $fmt:expr $(, $arg:expr)*) => {{
        if $crate::infrastructure::logger::should_log($crate::infrastructure::logger::LogLevel::Warning) {
            tracing::warn!($($key = $val,)+ "{}", format!($fmt $(, $arg)*));
        }
    }};
    ($fmt:expr $(, $arg:expr)*) => {
        $crate::infrastructure::logger::get().log_warn(&format!($fmt $(, $arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($key:ident = $val:expr),+ ; $fmt:expr $(, $arg:expr)*) => {{
        if $crate::infrastructure::logger::should_log($crate::infrastructure::logger::LogLevel::Error) {
            tracing::error!($($key = $val,)+ "{}", format!($fmt $(, $arg)*));
        }
    }};
    ($fmt:expr $(, $arg:expr)*) => {
        $crate::infrastructure::logger::get().log_error(&format!($fmt $(, $arg)*))
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($($key:ident = $val:expr),+ ; $fmt:expr $(, $arg:expr)*) => {{
        if $crate::infrastructure::logger::should_log($crate::infrastructure::logger::LogLevel::Fatal) {
            tracing::error!(__fatal__ = true, $($key = $val,)+ "{}", format!($fmt $(, $arg)*));
        }
        std::process::exit(1);
    }};
    ($fmt:expr $(, $arg:expr)*) => {
        $crate::infrastructure::logger::get().fatal(&format!($fmt $(, $arg)*))
    };
}
