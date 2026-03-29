#![allow(non_snake_case, dead_code)]

use crate::paths;
use serde::Serialize;
use serde_json::Value;
use std::{
    fs,
    sync::{Mutex, OnceLock},
    time::UNIX_EPOCH,
};

#[derive(Debug, Clone, Serialize)]
pub struct ReportSummaryEntry {
    pub id: String,
    pub fileName: String,
    pub action: String,
    pub traceId: String,
    pub mint: String,
    pub writtenAtMs: u128,
    pub displayTime: String,
    pub provider: String,
    pub transportType: String,
    pub signatureCount: usize,
    pub followEnabled: bool,
    pub followState: String,
    pub followActionCount: usize,
    pub followConfirmedCount: usize,
    pub followRunningCount: usize,
    pub followProblemCount: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReportCacheFileMeta {
    file_name: String,
    modified_ms: u128,
    len: u64,
}

#[derive(Debug, Clone)]
struct ReportSummaryCache {
    files: Vec<ReportCacheFileMeta>,
    newest: Vec<ReportSummaryEntry>,
    oldest: Vec<ReportSummaryEntry>,
}

fn format_report_time(written_at_ms: u128) -> String {
    if written_at_ms == 0 {
        return "Unknown time".to_string();
    }
    written_at_ms.to_string()
}

fn safe_json_parse(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| Value::Object(Default::default()))
}

fn follow_actions(report: &Value) -> Vec<Value> {
    report
        .get("followDaemon")
        .and_then(|follow| follow.get("job"))
        .and_then(|job| job.get("actions"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn build_report_summary_entry_from_payload(
    file_name: &str,
    payload: &Value,
    written_at_ms: u128,
) -> ReportSummaryEntry {
    let report = payload.get("report").cloned().unwrap_or(Value::Null);
    let execution = report.get("execution").cloned().unwrap_or(Value::Null);
    let follow_actions = follow_actions(&report);
    let follow_state = report
        .get("followDaemon")
        .and_then(|follow| follow.get("job"))
        .and_then(|job| job.get("state"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    ReportSummaryEntry {
        id: file_name.to_string(),
        fileName: file_name.to_string(),
        action: payload
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .trim()
            .to_string(),
        traceId: payload
            .get("traceId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string(),
        mint: payload
            .get("mint")
            .and_then(Value::as_str)
            .or_else(|| report.get("mint").and_then(Value::as_str))
            .unwrap_or_default()
            .trim()
            .to_string(),
        writtenAtMs: written_at_ms,
        displayTime: format_report_time(written_at_ms),
        provider: execution
            .get("resolvedProvider")
            .and_then(Value::as_str)
            .or_else(|| execution.get("provider").and_then(Value::as_str))
            .unwrap_or_default()
            .trim()
            .to_string(),
        transportType: execution
            .get("transportType")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string(),
        signatureCount: payload
            .get("signatures")
            .and_then(Value::as_array)
            .map(|entries| entries.len())
            .unwrap_or(0),
        followEnabled: report
            .get("followDaemon")
            .and_then(|follow| follow.get("enabled"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        followState: follow_state,
        followActionCount: follow_actions.len(),
        followConfirmedCount: follow_actions
            .iter()
            .filter(|action| {
                action
                    .get("state")
                    .and_then(Value::as_str)
                    .is_some_and(|state| state == "confirmed")
            })
            .count(),
        followRunningCount: follow_actions
            .iter()
            .filter(|action| {
                action
                    .get("state")
                    .and_then(Value::as_str)
                    .is_some_and(|state| matches!(state, "running" | "eligible" | "armed" | "sent"))
            })
            .count(),
        followProblemCount: follow_actions
            .iter()
            .filter(|action| {
                action
                    .get("state")
                    .and_then(Value::as_str)
                    .is_some_and(|state| matches!(state, "failed" | "cancelled" | "expired"))
            })
            .count(),
    }
}

pub fn build_report_summary_entry(file_name: &str) -> Result<ReportSummaryEntry, String> {
    let file_path = paths::reports_dir().join(file_name);
    let stat = fs::metadata(&file_path).map_err(|error| error.to_string())?;
    let raw = fs::read_to_string(&file_path).map_err(|error| error.to_string())?;
    let payload = safe_json_parse(&raw);
    let written_at_ms = payload
        .get("writtenAtMs")
        .and_then(Value::as_u64)
        .map(|value| value as u128)
        .unwrap_or_else(|| {
            stat.modified()
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_millis())
                .unwrap_or(0)
        });
    Ok(build_report_summary_entry_from_payload(
        file_name,
        &payload,
        written_at_ms,
    ))
}

pub fn list_persisted_reports(sort: &str) -> Vec<ReportSummaryEntry> {
    let dir = paths::reports_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return vec![];
    };
    let mut files = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.to_ascii_lowercase().ends_with(".json") {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            let modified_ms = metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_millis())
                .unwrap_or(0);
            Some(ReportCacheFileMeta {
                file_name,
                modified_ms,
                len: metadata.len(),
            })
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    let cache = report_summary_cache();
    if let Ok(guard) = cache.lock()
        && let Some(cached) = guard.as_ref()
        && cached.files == files
    {
        return if sort == "oldest" {
            cached.oldest.clone()
        } else {
            cached.newest.clone()
        };
    }
    let mut newest = files
        .iter()
        .filter_map(|entry| build_report_summary_entry(&entry.file_name).ok())
        .collect::<Vec<_>>();
    newest.sort_by(|left, right| right.writtenAtMs.cmp(&left.writtenAtMs));
    let mut oldest = newest.clone();
    oldest.reverse();
    if let Ok(mut guard) = cache.lock() {
        *guard = Some(ReportSummaryCache {
            files,
            newest: newest.clone(),
            oldest: oldest.clone(),
        });
    }
    if sort == "oldest" { oldest } else { newest }
}

fn report_summary_cache() -> &'static Mutex<Option<ReportSummaryCache>> {
    static CACHE: OnceLock<Mutex<Option<ReportSummaryCache>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

fn build_report_text(file_name: &str, payload: &Value, fallback_raw: &str) -> String {
    let report = payload.get("report").cloned().unwrap_or(Value::Null);
    let execution = report.get("execution").cloned().unwrap_or(Value::Null);
    let mut lines = vec![
        format!(
            "[{}] {}",
            payload
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_uppercase(),
            format_report_time(
                payload
                    .get("writtenAtMs")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u128
            )
        ),
        format!("File: {file_name}"),
        format!(
            "Trace: {}",
            payload
                .get("traceId")
                .and_then(Value::as_str)
                .unwrap_or("(missing)")
        ),
        format!(
            "Mint: {}",
            payload
                .get("mint")
                .and_then(Value::as_str)
                .or_else(|| report.get("mint").and_then(Value::as_str))
                .unwrap_or("(missing)")
        ),
    ];
    if let Some(provider) = execution
        .get("resolvedProvider")
        .and_then(Value::as_str)
        .or_else(|| execution.get("provider").and_then(Value::as_str))
    {
        if !provider.is_empty() {
            lines.push(format!("Provider: {provider}"));
        }
    }
    if let Some(transport) = execution.get("transportType").and_then(Value::as_str) {
        if !transport.is_empty() {
            lines.push(format!("Transport: {transport}"));
        }
    }
    if let Some(profile) = execution
        .get("resolvedEndpointProfile")
        .and_then(Value::as_str)
    {
        if !profile.is_empty() {
            lines.push(format!("Endpoint profile: {profile}"));
        }
    }
    if let Some(signatures) = payload.get("signatures").and_then(Value::as_array) {
        if !signatures.is_empty() {
            lines.push(format!(
                "Signatures: {}",
                signatures
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    if let Some(sent_items) = execution.get("sent").and_then(Value::as_array) {
        if !sent_items.is_empty() {
            lines.push(String::new());
            lines.push("Sent:".to_string());
            for sent in sent_items {
                let mut summary = format!(
                    "- {}: signature={} | status={}",
                    sent.get("label")
                        .and_then(Value::as_str)
                        .unwrap_or("(unknown)"),
                    sent.get("signature")
                        .and_then(Value::as_str)
                        .unwrap_or("(missing)"),
                    sent.get("confirmationStatus")
                        .and_then(Value::as_str)
                        .unwrap_or("(pending)")
                );
                if let Some(block_height) =
                    sent.get("sendObservedBlockHeight").and_then(Value::as_u64)
                {
                    summary.push_str(&format!(" | send block height={block_height}"));
                }
                if let Some(block_height) = sent
                    .get("confirmedObservedBlockHeight")
                    .and_then(Value::as_u64)
                {
                    summary.push_str(&format!(" | confirmed block height={block_height}"));
                }
                if let (Some(send_height), Some(confirmed_height)) = (
                    sent.get("sendObservedBlockHeight").and_then(Value::as_u64),
                    sent.get("confirmedObservedBlockHeight")
                        .and_then(Value::as_u64),
                ) {
                    summary.push_str(&format!(
                        " | blocks to confirm={}",
                        confirmed_height.saturating_sub(send_height)
                    ));
                }
                if let Some(slot) = sent.get("confirmedSlot").and_then(Value::as_u64) {
                    summary.push_str(&format!(" | confirmed slot={slot}"));
                }
                lines.push(summary);
            }
        }
    }
    if let Some(follow) = report.get("followDaemon").and_then(Value::as_object) {
        let enabled = follow
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if enabled {
            lines.push(String::new());
            lines.push("Follow daemon:".to_string());
            if let Some(transport) = follow.get("transport").and_then(Value::as_str)
                && !transport.is_empty()
            {
                lines.push(format!("  Transport: {transport}"));
            }
            if let Some(job) = follow.get("job").and_then(Value::as_object) {
                if let Some(state) = job.get("state").and_then(Value::as_str) {
                    lines.push(format!("  Job state: {state}"));
                }
                if let Some(last_error) = job.get("lastError").and_then(Value::as_str)
                    && !last_error.is_empty()
                {
                    lines.push(format!("  Last error: {last_error}"));
                }
                if let Some(actions) = job.get("actions").and_then(Value::as_array) {
                    let confirmed = actions
                        .iter()
                        .filter(|action| {
                            action
                                .get("state")
                                .and_then(Value::as_str)
                                .is_some_and(|state| state == "confirmed")
                        })
                        .count();
                    let problems = actions
                        .iter()
                        .filter(|action| {
                            action
                                .get("state")
                                .and_then(Value::as_str)
                                .is_some_and(|state| {
                                    matches!(state, "failed" | "cancelled" | "expired")
                                })
                        })
                        .count();
                    lines.push(format!(
                        "  Actions: {} total | {} confirmed | {} problem",
                        actions.len(),
                        confirmed,
                        problems
                    ));
                    for action in actions {
                        let mut summary = format!(
                            "  - {} [{}]",
                            action
                                .get("actionId")
                                .and_then(Value::as_str)
                                .unwrap_or("(unknown)"),
                            action
                                .get("state")
                                .and_then(Value::as_str)
                                .unwrap_or("unknown")
                        );
                        if let Some(kind) = action.get("kind").and_then(Value::as_str) {
                            summary.push_str(&format!(" | kind={kind}"));
                        }
                        if let Some(signature) = action.get("signature").and_then(Value::as_str)
                            && !signature.is_empty()
                        {
                            summary.push_str(&format!(" | sig={signature}"));
                        }
                        if let Some(attempt_count) =
                            action.get("attemptCount").and_then(Value::as_u64)
                        {
                            summary.push_str(&format!(" | attempts={attempt_count}"));
                        }
                        if let Some(last_error) = action.get("lastError").and_then(Value::as_str)
                            && !last_error.is_empty()
                        {
                            summary.push_str(&format!(" | error={last_error}"));
                        }
                        lines.push(summary);
                    }
                }
            }
            if let Some(profiles) = follow.get("timingProfiles").and_then(Value::as_array)
                && !profiles.is_empty()
            {
                lines.push(
                    "  Advisory only: suggestions do not auto-change your configured follow timings."
                        .to_string(),
                );
                lines.push("  Timing profiles:".to_string());
                for profile in profiles {
                    let action_type = profile
                        .get("actionType")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let sample_count = profile
                        .get("sampleCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    let recommendation = profile
                        .get("recommendation")
                        .cloned()
                        .unwrap_or(Value::Null);
                    let suggested_delay = recommendation
                        .get("suggestedSubmitDelayMs")
                        .and_then(Value::as_u64);
                    let suggested_jitter = recommendation
                        .get("suggestedJitterMs")
                        .and_then(Value::as_u64);
                    let confidence = recommendation
                        .get("confidence")
                        .and_then(Value::as_str)
                        .unwrap_or("low");
                    let success_rate = recommendation
                        .get("successRate")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0);
                    let weighted_quality_score = recommendation
                        .get("weightedQualityScore")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0);
                    let mut summary = format!(
                        "  - {action_type}: samples={sample_count} | confidence={confidence} | success={:.0}% | quality={:.1}",
                        success_rate * 100.0,
                        weighted_quality_score
                    );
                    if let Some(delay) = suggested_delay {
                        summary.push_str(&format!(" | suggest delay={}ms", delay));
                    }
                    if let Some(jitter) = suggested_jitter {
                        summary.push_str(&format!(" | suggest jitter={}ms", jitter));
                    }
                    lines.push(summary);
                }
            }
        }
    }
    if let Some(benchmark) = report.get("benchmark").and_then(Value::as_object) {
        lines.push("Benchmark:".to_string());
        if let Some(timings) = benchmark.get("timings").and_then(Value::as_object) {
            let mut timing_parts = Vec::new();
            let push_timing = |parts: &mut Vec<String>, key: &str, label: &str| {
                if let Some(value) = timings.get(key).and_then(Value::as_u64) {
                    parts.push(format!("{label}={value}ms"));
                }
            };
            push_timing(&mut timing_parts, "totalElapsedMs", "total");
            push_timing(&mut timing_parts, "backendTotalElapsedMs", "backendTotal");
            push_timing(&mut timing_parts, "clientPreRequestMs", "preRequest");
            push_timing(&mut timing_parts, "formToRawConfigMs", "form");
            push_timing(&mut timing_parts, "normalizeConfigMs", "normalize");
            push_timing(&mut timing_parts, "walletLoadMs", "wallet");
            push_timing(&mut timing_parts, "reportBuildMs", "report");
            push_timing(&mut timing_parts, "compileTransactionsMs", "compile");
            push_timing(&mut timing_parts, "compileAltLoadMs", "altLoad");
            push_timing(&mut timing_parts, "compileBlockhashFetchMs", "blockhash");
            push_timing(&mut timing_parts, "compileGlobalFetchMs", "global");
            push_timing(&mut timing_parts, "compileFollowUpPrepMs", "followUpPrep");
            push_timing(&mut timing_parts, "compileTxSerializeMs", "serialize");
            push_timing(&mut timing_parts, "simulateMs", "simulate");
            push_timing(&mut timing_parts, "sendMs", "send");
            push_timing(&mut timing_parts, "sendSubmitMs", "submit");
            push_timing(&mut timing_parts, "sendConfirmMs", "confirm");
            push_timing(&mut timing_parts, "persistReportMs", "persist");
            if !timing_parts.is_empty() {
                lines.push(format!("  Timings: {}", timing_parts.join(" | ")));
            }
        }
        if let Some(sent_items) = benchmark.get("sent").and_then(Value::as_array) {
            for sent in sent_items {
                let label = sent
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or("(unknown)");
                let mut sent_parts = Vec::new();
                if let Some(value) = sent.get("sendBlockHeight").and_then(Value::as_u64) {
                    sent_parts.push(format!("send block height={value}"));
                }
                if let Some(value) = sent.get("confirmedBlockHeight").and_then(Value::as_u64) {
                    sent_parts.push(format!("confirmed block height={value}"));
                }
                if let Some(value) = sent.get("blocksToConfirm").and_then(Value::as_u64) {
                    sent_parts.push(format!("blocks to confirm={value}"));
                }
                if let Some(value) = sent.get("confirmedSlot").and_then(Value::as_u64) {
                    sent_parts.push(format!("confirmed slot={value}"));
                }
                if !sent_parts.is_empty() {
                    lines.push(format!("  {}: {}", label, sent_parts.join(" | ")));
                }
            }
        }
    }
    lines.push(String::new());
    lines.push("--- Report JSON ---".to_string());
    lines.push(
        serde_json::to_string_pretty(if report.is_null() { payload } else { &report })
            .unwrap_or_default(),
    );
    if report.is_null() {
        lines.push(String::new());
        lines.push("--- Raw File ---".to_string());
        lines.push(fallback_raw.to_string());
    }
    lines.join("\n")
}

pub fn read_persisted_report(file_name: &str) -> Result<(ReportSummaryEntry, String), String> {
    let (entry, text, _payload) = read_persisted_report_bundle(file_name)?;
    Ok((entry, text))
}

pub fn read_persisted_report_bundle(
    file_name: &str,
) -> Result<(ReportSummaryEntry, String, Value), String> {
    let safe_file_name = std::path::Path::new(file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    if safe_file_name.is_empty() {
        return Err("Report id is required.".to_string());
    }
    let file_path = paths::reports_dir().join(&safe_file_name);
    if !file_path.exists() {
        return Err("Report not found.".to_string());
    }
    let raw = fs::read_to_string(&file_path).map_err(|error| error.to_string())?;
    let payload = safe_json_parse(&raw);
    Ok((
        build_report_summary_entry(&safe_file_name)?,
        build_report_text(&safe_file_name, &payload, &raw),
        payload,
    ))
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
    fn summary_entry_includes_follow_job_counts() {
        let _guard = env_lock().lock().expect("lock env");
        let file_name = "follow-summary.json";
        let payload = serde_json::json!({
            "writtenAtMs": 123,
            "action": "send",
            "traceId": "trace-1",
            "mint": "mint-1",
            "report": {
                "execution": {
                    "provider": "helius-sender",
                    "transportType": "helius-sender"
                },
                "followDaemon": {
                    "enabled": true,
                    "job": {
                        "state": "running",
                        "actions": [
                            { "actionId": "a", "state": "confirmed" },
                            { "actionId": "b", "state": "failed" },
                            { "actionId": "c", "state": "running" }
                        ]
                    }
                }
            },
            "signatures": ["sig-1"]
        });
        let summary = build_report_summary_entry_from_payload(file_name, &payload, 123);
        assert!(summary.followEnabled);
        assert_eq!(summary.followState, "running");
        assert_eq!(summary.followActionCount, 3);
        assert_eq!(summary.followConfirmedCount, 1);
        assert_eq!(summary.followRunningCount, 1);
        assert_eq!(summary.followProblemCount, 1);
    }
}
