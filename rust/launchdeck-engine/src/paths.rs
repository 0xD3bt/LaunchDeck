#![allow(dead_code)]

use std::{env, path::PathBuf};

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
        })
}

pub fn ui_dir() -> PathBuf {
    repo_root().join("ui")
}

pub fn local_root_dir() -> PathBuf {
    if let Ok(explicit) = env::var("LAUNCHDECK_LOCAL_DATA_DIR") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    repo_root().join(".local").join("launchdeck")
}

pub fn uploads_dir() -> PathBuf {
    local_root_dir().join("uploads")
}

pub fn reports_dir() -> PathBuf {
    if let Ok(explicit) = env::var("LAUNCHDECK_SEND_LOG_DIR") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    local_root_dir().join("send-reports")
}

pub fn image_library_path() -> PathBuf {
    local_root_dir().join("image-library.json")
}

pub fn app_config_path() -> PathBuf {
    local_root_dir().join("app-config.json")
}

pub fn bags_credentials_path() -> PathBuf {
    local_root_dir().join("bags-credentials.json")
}

pub fn bags_session_path() -> PathBuf {
    local_root_dir().join("bags-session.json")
}

pub fn lookup_table_cache_path() -> PathBuf {
    local_root_dir().join("lookup-tables.json")
}

pub fn bonk_lookup_table_cache_path() -> PathBuf {
    local_root_dir().join("bonk-lookup-tables.json")
}

pub fn runtime_state_path() -> PathBuf {
    if let Ok(explicit) = env::var("LAUNCHDECK_ENGINE_RUNTIME_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    repo_root().join(".local").join("engine-runtime.json")
}

pub fn follow_daemon_state_path() -> PathBuf {
    if let Ok(explicit) = env::var("LAUNCHDECK_FOLLOW_DAEMON_STATE_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    local_root_dir().join("follow-daemon-state.json")
}
