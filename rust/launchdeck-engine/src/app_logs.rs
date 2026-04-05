#![allow(non_snake_case, dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::paths;

const LIVE_LOG_LIMIT: usize = 100;
const ERROR_LOG_DEFAULT_LIMIT: usize = 250;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppLogEntry {
    pub id: String,
    pub timestampMs: u64,
    pub level: String,
    pub source: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    pub persisted: bool,
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn live_logs() -> &'static Mutex<VecDeque<AppLogEntry>> {
    static LOGS: OnceLock<Mutex<VecDeque<AppLogEntry>>> = OnceLock::new();
    LOGS.get_or_init(|| Mutex::new(VecDeque::with_capacity(LIVE_LOG_LIMIT)))
}

fn error_log_path() -> std::path::PathBuf {
    paths::local_root_dir().join("error-logs.jsonl")
}

fn push_live_entry(entry: AppLogEntry) {
    let mut logs = live_logs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    logs.push_back(entry);
    while logs.len() > LIVE_LOG_LIMIT {
        logs.pop_front();
    }
}

fn append_error_entry(entry: &AppLogEntry) {
    let path = error_log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let Ok(line) = serde_json::to_string(entry) else {
        return;
    };
    let _ = writeln!(file, "{line}");
}

fn record_entry(level: &str, source: &str, message: impl Into<String>, context: Option<Value>) {
    let entry = AppLogEntry {
        id: Uuid::new_v4().to_string(),
        timestampMs: current_time_ms(),
        level: level.to_string(),
        source: source.trim().to_string(),
        message: message.into(),
        context,
        persisted: level.eq_ignore_ascii_case("error"),
    };
    push_live_entry(entry.clone());
    if entry.persisted {
        append_error_entry(&entry);
    }
}

pub fn record_info(source: &str, message: impl Into<String>, context: Option<Value>) {
    record_entry("info", source, message, context);
}

pub fn record_warn(source: &str, message: impl Into<String>, context: Option<Value>) {
    record_entry("warn", source, message, context);
}

pub fn record_error(source: &str, message: impl Into<String>, context: Option<Value>) {
    record_entry("error", source, message, context);
}

pub fn list_live_logs(limit: usize) -> Vec<AppLogEntry> {
    let logs = live_logs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    logs.iter().rev().take(limit.max(1)).cloned().collect()
}

pub fn list_error_logs(limit: Option<usize>) -> Vec<AppLogEntry> {
    let path = error_log_path();
    let Ok(file) = OpenOptions::new().read(true).open(path) else {
        return Vec::new();
    };
    let reader = BufReader::new(file);
    let mut entries = reader
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str::<AppLogEntry>(&line).ok())
        .collect::<Vec<_>>();
    entries.reverse();
    entries.truncate(limit.unwrap_or(ERROR_LOG_DEFAULT_LIMIT).max(1));
    entries
}

#[cfg(test)]
pub fn clear_live_logs() {
    let mut logs = live_logs()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    logs.clear();
}
