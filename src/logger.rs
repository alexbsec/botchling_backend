mod logger;

pub use self::logger::LogLevel;
pub use self::logger::init;
pub use self::logger::should_log;
pub(crate) use self::logger::get;
