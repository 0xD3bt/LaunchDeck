#![allow(non_snake_case, dead_code)]

use serde::Serialize;
use serde_json::{Value, json};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::{paths, transport::TransportPlan};

#[derive(Debug, Clone, Serialize)]
pub struct TraceContext {
    pub traceId: String,
    pub startedAtMs: u128,
}

pub fn new_trace_context() -> TraceContext {
    let started_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    TraceContext {
        traceId: Uuid::new_v4().to_string(),
        startedAtMs: started_at_ms,
    }
}

pub fn log_event(event: &str, trace_id: &str, payload: Value) {
    let line = json!({
        "event": event,
        "traceId": trace_id,
        "payload": payload,
    });
    println!("{line}");
}

fn launch_log_dir() -> std::path::PathBuf {
    paths::reports_dir()
}

pub fn persist_launch_report(
    trace_id: &str,
    action: &str,
    transport_plan: &TransportPlan,
    report: &Value,
) -> Result<String, String> {
    let dir = launch_log_dir();
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let file_name = format!(
        "{}-{}-{}.json",
        current_time_ms(),
        action,
        trace_id.replace('-', "")
    );
    let path = dir.join(file_name);
    write_launch_report_file(&path, trace_id, action, transport_plan, report)?;
    Ok(path.display().to_string())
}

pub fn update_persisted_launch_report(
    path: &str,
    trace_id: &str,
    action: &str,
    transport_plan: &TransportPlan,
    report: &Value,
) -> Result<(), String> {
    write_launch_report_file(
        std::path::Path::new(path),
        trace_id,
        action,
        transport_plan,
        report,
    )
}

pub fn update_persisted_follow_daemon_snapshot(path: &str, snapshot: &Value) -> Result<(), String> {
    let existing = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut payload: Value = serde_json::from_str(&existing).map_err(|error| error.to_string())?;
    let report = payload
        .get_mut("report")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "Persisted launch report missing report payload.".to_string())?;
    report.insert("followDaemon".to_string(), snapshot.clone());
    fs::write(
        path,
        serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn write_launch_report_file(
    path: &std::path::Path,
    trace_id: &str,
    action: &str,
    transport_plan: &TransportPlan,
    report: &Value,
) -> Result<(), String> {
    let mint = report
        .get("mint")
        .and_then(Value::as_str)
        .unwrap_or("unknown-mint");
    let signatures = report
        .get("execution")
        .and_then(|execution| execution.get("sent"))
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get("signature").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let payload = json!({
        "traceId": trace_id,
        "action": action,
        "writtenAtMs": current_time_ms(),
        "mint": mint,
        "signatures": signatures,
        "transportPlan": transport_plan,
        "report": report,
    });
    fs::write(
        path,
        serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn current_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn persists_launch_report_with_trace_and_signatures() {
        let _guard = env_lock().lock().expect("lock env");
        let temp_dir = std::env::temp_dir().join(format!("launchdeck-send-log-{}", Uuid::new_v4()));
        unsafe {
            std::env::set_var("LAUNCHDECK_SEND_LOG_DIR", &temp_dir);
        }
        let plan = TransportPlan {
            requestedProvider: "helius-sender".to_string(),
            resolvedProvider: "helius-sender".to_string(),
            requestedEndpointProfile: "global".to_string(),
            resolvedEndpointProfile: "global".to_string(),
            executionClass: "single".to_string(),
            transportType: "helius-sender".to_string(),
            ordering: "single".to_string(),
            verified: true,
            supportsBundle: false,
            requiresInlineTip: true,
            requiresPriorityFee: true,
            separateTipTransaction: false,
            skipPreflight: true,
            maxRetries: 0,
            heliusSenderEndpoint: Some("https://sender.helius-rpc.com/fast".to_string()),
            heliusSenderEndpoints: vec!["https://sender.helius-rpc.com/fast".to_string()],
            watchEndpoint: Some("wss://mainnet.helius-rpc.com/?api-key=test".to_string()),
            watchEndpoints: vec!["wss://mainnet.helius-rpc.com/?api-key=test".to_string()],
            jitoBundleEndpoints: vec![],
            warnings: vec![],
        };
        let report = json!({
            "mint": "mint-test",
            "execution": {
                "sent": [
                    { "signature": "sig-1" },
                    { "signature": "sig-2" }
                ]
            }
        });
        let path =
            persist_launch_report("trace-123", "send", &plan, &report).expect("persist send log");
        let raw = fs::read_to_string(&path).expect("read persisted log");
        assert!(raw.contains("\"traceId\": \"trace-123\""));
        assert!(raw.contains("\"signature\""));
        unsafe {
            std::env::remove_var("LAUNCHDECK_SEND_LOG_DIR");
        }
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn updates_existing_launch_report_contents() {
        let _guard = env_lock().lock().expect("lock env");
        let temp_dir =
            std::env::temp_dir().join(format!("launchdeck-send-log-update-{}", Uuid::new_v4()));
        unsafe {
            std::env::set_var("LAUNCHDECK_SEND_LOG_DIR", &temp_dir);
        }
        let plan = TransportPlan {
            requestedProvider: "helius-sender".to_string(),
            resolvedProvider: "helius-sender".to_string(),
            requestedEndpointProfile: "global".to_string(),
            resolvedEndpointProfile: "global".to_string(),
            executionClass: "single".to_string(),
            transportType: "helius-sender".to_string(),
            ordering: "single".to_string(),
            verified: true,
            supportsBundle: false,
            requiresInlineTip: true,
            requiresPriorityFee: true,
            separateTipTransaction: false,
            skipPreflight: true,
            maxRetries: 0,
            heliusSenderEndpoint: Some("https://sender.helius-rpc.com/fast".to_string()),
            heliusSenderEndpoints: vec!["https://sender.helius-rpc.com/fast".to_string()],
            watchEndpoint: Some("wss://mainnet.helius-rpc.com/?api-key=test".to_string()),
            watchEndpoints: vec!["wss://mainnet.helius-rpc.com/?api-key=test".to_string()],
            jitoBundleEndpoints: vec![],
            warnings: vec![],
        };
        let initial_report = json!({
            "mint": "mint-test",
            "execution": {}
        });
        let path = persist_launch_report("trace-456", "simulate", &plan, &initial_report)
            .expect("persist initial log");
        let updated_report = json!({
            "mint": "mint-test",
            "benchmark": {
                "timings": {
                    "totalElapsedMs": 42
                }
            },
            "execution": {}
        });
        update_persisted_launch_report(&path, "trace-456", "simulate", &plan, &updated_report)
            .expect("update persisted log");
        let raw = fs::read_to_string(&path).expect("read updated log");
        assert!(raw.contains("\"benchmark\""));
        assert!(raw.contains("\"totalElapsedMs\": 42"));
        unsafe {
            std::env::remove_var("LAUNCHDECK_SEND_LOG_DIR");
        }
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn updates_follow_daemon_snapshot_in_persisted_report() {
        let _guard = env_lock().lock().expect("lock env");
        let temp_dir =
            std::env::temp_dir().join(format!("launchdeck-follow-log-update-{}", Uuid::new_v4()));
        unsafe {
            std::env::set_var("LAUNCHDECK_SEND_LOG_DIR", &temp_dir);
        }
        let plan = TransportPlan {
            requestedProvider: "helius-sender".to_string(),
            resolvedProvider: "helius-sender".to_string(),
            requestedEndpointProfile: "global".to_string(),
            resolvedEndpointProfile: "global".to_string(),
            executionClass: "single".to_string(),
            transportType: "helius-sender".to_string(),
            ordering: "single".to_string(),
            verified: true,
            supportsBundle: false,
            requiresInlineTip: true,
            requiresPriorityFee: true,
            separateTipTransaction: false,
            skipPreflight: true,
            maxRetries: 0,
            heliusSenderEndpoint: Some("https://sender.helius-rpc.com/fast".to_string()),
            heliusSenderEndpoints: vec!["https://sender.helius-rpc.com/fast".to_string()],
            watchEndpoint: Some("wss://mainnet.helius-rpc.com/?api-key=test".to_string()),
            watchEndpoints: vec!["wss://mainnet.helius-rpc.com/?api-key=test".to_string()],
            jitoBundleEndpoints: vec![],
            warnings: vec![],
        };
        let report = json!({
            "mint": "mint-test",
            "execution": {}
        });
        let path =
            persist_launch_report("trace-follow", "send", &plan, &report).expect("persist log");
        update_persisted_follow_daemon_snapshot(
            &path,
            &json!({
                "job": {
                    "traceId": "trace-follow",
                    "state": "running"
                }
            }),
        )
        .expect("update follow snapshot");
        let raw = fs::read_to_string(&path).expect("read updated log");
        assert!(raw.contains("\"followDaemon\""));
        assert!(raw.contains("\"traceId\": \"trace-follow\""));
        unsafe {
            std::env::remove_var("LAUNCHDECK_SEND_LOG_DIR");
        }
        let _ = fs::remove_dir_all(temp_dir);
    }
}
