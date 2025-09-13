use std::time::{SystemTime, UNIX_EPOCH};

pub struct LogManager {}

impl LogManager {
    pub fn new_instance() -> Self {
        Self {}
    }

    pub fn send(&self, l: LogLevel, m: &str, t: &str) {
        let entry = LogEntry::new(l, m, t);
        todo!()
    }
}

pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub target: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: &str, target: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            level,
            timestamp,
            target: target.to_owned(),
            message: message.to_owned(),
        }
    }
}

pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}
