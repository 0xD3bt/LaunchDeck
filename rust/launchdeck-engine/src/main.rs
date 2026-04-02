mod bags_native;
mod bonk_native;
mod config;
mod follow;
mod fs_utils;
mod image_library;
mod launchpad_dispatch;
mod launchpads;
mod observability;
mod paths;
mod providers;
mod pump_native;
mod report;
mod reports_browser;
mod rpc;
mod runtime;
mod strategies;
mod transport;
mod ui_bridge;
mod ui_config;
mod vamp;
mod wallet;

use crate::{
    bags_native::{
        PreparedBagsSendArtifacts, compile_launch_transaction as compile_bags_launch_transaction,
        prepare_native_bags_send, summarize_transactions as summarize_bags_transactions,
    },
    config::{NormalizedConfig, NormalizedFollowLaunch, RawConfig, normalize_raw_config},
    follow::{
        FOLLOW_RESPONSE_SCHEMA_VERSION, FollowArmRequest, FollowCancelRequest, FollowDaemonClient,
        FollowJobResponse, FollowReserveRequest, FollowStopAllRequest, build_action_records,
        should_use_post_setup_creator_vault_for_sell,
    },
    fs_utils::atomic_write,
    image_library::{
        build_image_library_payload, create_category, delete_image, save_image_bytes, update_image,
    },
    launchpad_dispatch::{
        compile_atomic_follow_buy_for_launchpad, quote_launch_for_launchpad,
        try_compile_native_launchpad,
    },
    launchpads::launchpad_registry,
    observability::{
        log_event, new_trace_context, persist_launch_report, update_persisted_launch_report,
    },
    providers::{provider_availability_registry, provider_registry},
    bonk_native::{
        derive_canonical_pool_id as derive_bonk_canonical_pool_id,
        predict_dev_buy_token_amount as predict_bonk_dev_buy_token_amount,
        warm_bonk_state,
    },
    pump_native::{predict_dev_buy_token_amount, warm_default_lookup_tables, warm_pump_global_state},
    report::{
        BenchmarkMode, ExecutionTimings, FollowJobTimings, LaunchReport,
        build_benchmark_timing_groups, build_report, configured_benchmark_mode, render_report,
        sanitize_execution_timings_for_mode,
    },
    reports_browser::{list_persisted_reports, read_persisted_report_bundle},
    rpc::{
        CompiledTransaction, confirm_submitted_transactions_for_transport,
        confirm_transactions_with_websocket_fallback, configured_warm_rpc_url,
        fetch_current_block_height,
        send_transactions_bundle,
        simulate_transactions, spawn_blockhash_refresh_task, prewarm_rpc_endpoint,
        submit_independent_transactions_for_transport, submit_transactions_for_transport,
        submit_transactions_sequential,
    },
    runtime::{
        RuntimeRegistry, RuntimeRequest, RuntimeResponse, fail_worker, heartbeat_worker,
        list_workers, restore_workers, start_worker, stop_worker,
    },
    strategies::strategy_registry,
    transport::{
        build_transport_plan, configured_helius_sender_endpoint,
        configured_helius_sender_endpoints_for_profile, configured_jito_bundle_endpoints,
        configured_provider_region, configured_shared_region, default_endpoint_profile,
        default_endpoint_profile_for_provider, estimate_transaction_count,
        helius_sender_endpoint_override_active, jito_bundle_endpoint_override_active,
    },
    ui_bridge::{build_raw_config_from_form, quote_from_form, upload_metadata_from_form},
    ui_config::{
        create_default_persistent_config, read_persistent_config, write_persistent_config,
    },
    vamp::{fetch_imported_token_metadata, import_remote_image_to_library},
    wallet::{
        enrich_wallet_statuses, list_solana_env_wallets, load_solana_wallet_by_env_key,
        public_key_from_secret, selected_wallet_key_or_default,
        selected_wallet_key_or_default_from_wallets,
    },
};
use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path as AxumPath, Query, State},
    http::{HeaderMap, Response, StatusCode, header},
    routing::{get, post},
};
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
struct AppState {
    auth_token: Option<String>,
    runtime: Arc<RuntimeRegistry>,
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    service: &'static str,
    version: &'static str,
    mode: &'static str,
}

#[derive(Deserialize, Default)]
struct EngineRequest {
    action: Option<String>,
    form: Option<Value>,
    #[serde(rename = "rawConfig")]
    raw_config: Option<Value>,
    #[serde(default)]
    #[serde(rename = "prepareRequestPayloadMs")]
    prepare_request_payload_ms: Option<u64>,
}

#[derive(Deserialize, Default)]
struct StatusRequest {
    wallet: Option<String>,
}

#[derive(Deserialize, Default)]
struct StatusQuery {
    wallet: Option<String>,
}

#[derive(Deserialize, Default)]
struct RunRequest {
    action: Option<String>,
    form: Option<Value>,
    #[serde(default)]
    #[serde(rename = "clientPreRequestMs")]
    client_pre_request_ms: Option<u64>,
    #[serde(default)]
    #[serde(rename = "prepareRequestPayloadMs")]
    prepare_request_payload_ms: Option<u64>,
}

#[derive(Deserialize)]
struct FollowCancelApiRequest {
    #[serde(rename = "traceId")]
    trace_id: String,
    #[serde(rename = "actionId")]
    action_id: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize, Default)]
struct FollowStopAllApiRequest {
    note: Option<String>,
}

#[derive(Deserialize, Default)]
struct MetadataUploadRequest {
    form: Option<Value>,
}

#[derive(Deserialize, Default)]
struct SettingsSaveRequest {
    config: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BagsStoredCredentials {
    #[serde(default)]
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(default)]
    #[serde(rename = "authToken")]
    auth_token: String,
    #[serde(default)]
    #[serde(rename = "agentUsername")]
    agent_username: String,
    #[serde(default)]
    #[serde(rename = "verifiedWallet")]
    verified_wallet: String,
}

#[derive(Deserialize, Default)]
struct BagsIdentityInitRequest {
    #[serde(default)]
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(default)]
    #[serde(rename = "saveApiKey")]
    save_api_key: bool,
    #[serde(default)]
    #[serde(rename = "agentUsername")]
    agent_username: String,
}

#[derive(Deserialize, Default)]
struct BagsIdentityVerifyRequest {
    #[serde(default)]
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(default)]
    #[serde(rename = "saveApiKey")]
    save_api_key: bool,
    #[serde(default)]
    #[serde(rename = "agentUsername")]
    agent_username: String,
    #[serde(default)]
    #[serde(rename = "publicIdentifier")]
    public_identifier: String,
    #[serde(default)]
    secret: String,
    #[serde(default)]
    #[serde(rename = "postId")]
    post_id: String,
    #[serde(default)]
    #[serde(rename = "walletEnvKey")]
    wallet_env_key: String,
}

#[derive(Deserialize, Default)]
struct BagsIdentityStatusQuery {
    wallet: Option<String>,
}

#[derive(Deserialize, Default)]
struct ReportsQuery {
    sort: Option<String>,
}

#[derive(Deserialize, Default)]
struct ReportViewQuery {
    id: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Default)]
struct QuoteQuery {
    launchpad: Option<String>,
    quoteAsset: Option<String>,
    launchMode: Option<String>,
    mode: Option<String>,
    amount: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Default)]
struct VampRequest {
    contractAddress: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Default)]
struct ImagesQuery {
    search: Option<String>,
    category: Option<String>,
    favoritesOnly: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Default)]
struct ImageUpdateRequest {
    id: Option<String>,
    name: Option<String>,
    tags: Option<Value>,
    category: Option<String>,
    isFavorite: Option<bool>,
}

#[derive(Deserialize, Default)]
struct ImageCategoryRequest {
    name: Option<String>,
}

#[derive(Deserialize, Default)]
struct ImageDeleteRequest {
    id: Option<String>,
}

const USD1_MINT: &str = "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB";
const DEFAULT_LOCAL_AUTH_TOKEN: &str = "4815927603149027";

fn configured_engine_port() -> u16 {
    std::env::var("LAUNCHDECK_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8789)
}

fn configured_auth_token() -> Option<String> {
    let token = std::env::var("LAUNCHDECK_ENGINE_AUTH_TOKEN")
        .unwrap_or_else(|_| DEFAULT_LOCAL_AUTH_TOKEN.to_string());
    let trimmed = token.trim();
    if trimmed.is_empty() {
        Some(DEFAULT_LOCAL_AUTH_TOKEN.to_string())
    } else {
        Some(trimmed.to_string())
    }
}

fn configured_runtime_state_path() -> std::path::PathBuf {
    paths::runtime_state_path()
}

fn authorize(headers: &HeaderMap, state: &AppState) -> Result<(), (StatusCode, Json<Value>)> {
    let Some(expected) = &state.auth_token else {
        return Ok(());
    };
    let actual = headers
        .get("x-launchdeck-engine-auth")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if actual == expected {
        Ok(())
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": "Unauthorized engine request.",
            })),
        ))
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        service: "launchdeck-engine",
        version: std::env!("CARGO_PKG_VERSION"),
        mode: "rust-native-only",
    })
}

fn configured_rpc_url() -> String {
    if let Ok(explicit) = std::env::var("SOLANA_RPC_URL") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "http://127.0.0.1:8899".to_string()
}

fn configured_startup_warm_enabled() -> bool {
    !matches!(
        std::env::var("LAUNCHDECK_DISABLE_STARTUP_WARM")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn configured_base_url() -> String {
    format!("http://127.0.0.1:{}", configured_engine_port())
}

fn configured_follow_daemon_port() -> u16 {
    std::env::var("LAUNCHDECK_FOLLOW_DAEMON_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8790)
}

fn configured_follow_daemon_base_url() -> String {
    std::env::var("LAUNCHDECK_FOLLOW_DAEMON_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("http://127.0.0.1:{}", configured_follow_daemon_port()))
}

fn configured_follow_daemon_transport() -> Result<String, String> {
    let transport = std::env::var("LAUNCHDECK_FOLLOW_DAEMON_TRANSPORT")
        .unwrap_or_else(|_| "local-http".to_string())
        .trim()
        .to_lowercase();
    match transport.as_str() {
        "" | "local-http" => Ok("local-http".to_string()),
        other => Err(format!(
            "Unsupported follow daemon transport: {other}. Expected local-http."
        )),
    }
}

async fn follow_daemon_status_payload() -> Value {
    let base_url = configured_follow_daemon_base_url();
    match configured_follow_daemon_transport() {
        Ok(transport) => {
            let client = FollowDaemonClient::new(&base_url);
            match client.health().await {
                Ok(health) => json!({
                    "configured": true,
                    "reachable": true,
                    "transport": transport,
                    "url": base_url,
                    "health": health,
                }),
                Err(error) => json!({
                    "configured": true,
                    "reachable": false,
                    "transport": transport,
                    "url": base_url,
                    "error": error,
                }),
            }
        }
        Err(error) => json!({
            "configured": false,
            "reachable": false,
            "url": base_url,
            "error": error,
        }),
    }
}

fn resolve_signer_source(selected_wallet_key: &str) -> String {
    if !selected_wallet_key.trim().is_empty() {
        return format!("env:{}", selected_wallet_key.trim());
    }
    if std::env::var("SOLANA_PRIVATE_KEY")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return "env:SOLANA_PRIVATE_KEY".to_string();
    }
    if std::env::var("SOLANA_KEYPAIR_PATH")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return "env:SOLANA_KEYPAIR_PATH".to_string();
    }
    "unknown".to_string()
}

fn guess_content_type(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

fn cache_control_for_path(path: &std::path::Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension == "html" {
        return "no-store";
    }
    if path.to_string_lossy().contains("uploads") {
        return "public, max-age=86400";
    }
    if matches!(
        extension.as_str(),
        "js" | "css" | "svg" | "png" | "jpg" | "jpeg" | "webp" | "gif"
    ) {
        return "no-cache";
    }
    "no-store"
}

fn file_response(path: std::path::PathBuf) -> Result<Response<Body>, (StatusCode, Json<Value>)> {
    let body = std::fs::read(&path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": "Not found",
            })),
        )
    })?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, guess_content_type(&path))
        .header(header::CACHE_CONTROL, cache_control_for_path(&path))
        .body(Body::from(body))
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "error": error.to_string(),
                })),
            )
        })
}

fn now_timestamp_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    millis.to_string()
}

fn current_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn attach_timing(mut payload: Value, started_at_ms: u128) -> Value {
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "timingMs".to_string(),
            Value::from(current_time_ms().saturating_sub(started_at_ms) as u64),
        );
    }
    payload
}

fn synthetic_mint_address(trace_id: &str) -> String {
    let clean = trace_id.replace('-', "");
    let mut bytes = Vec::with_capacity(32);
    while bytes.len() < 32 {
        bytes.extend_from_slice(clean.as_bytes());
    }
    bytes.truncate(32);
    bs58::encode(bytes).into_string()
}

fn render_report_value(report: &Value) -> Value {
    serde_json::from_value::<LaunchReport>(report.clone())
        .map(|parsed| Value::String(render_report(&parsed)))
        .unwrap_or_else(|_| Value::String(String::new()))
}

fn append_execution_warning(report: &mut Value, warning: &str) {
    let Some(execution) = report.get_mut("execution") else {
        return;
    };
    let mut existing_warnings = execution
        .get("warnings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    existing_warnings.push(Value::String(warning.to_string()));
    execution["warnings"] = Value::Array(existing_warnings);
}

fn append_execution_note(report: &mut Value, note: &str) {
    let Some(execution) = report.get_mut("execution") else {
        return;
    };
    let mut existing_notes = execution
        .get("notes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    existing_notes.push(Value::String(note.to_string()));
    execution["notes"] = Value::Array(existing_notes);
}

fn set_report_timing(report: &mut Value, key: &str, value_ms: u128) {
    if let Some(execution) = report.get_mut("execution") {
        if execution.get("timings").is_none()
            || execution.get("timings").is_some_and(Value::is_null)
        {
            execution["timings"] = json!({});
        }
        execution["timings"][key] = Value::from(value_ms as u64);
    }
}

fn set_optional_report_timing(report: &mut Value, key: &str, value_ms: Option<u128>) {
    if let Some(value_ms) = value_ms {
        set_report_timing(report, key, value_ms);
    }
}

fn set_report_benchmark_mode(report: &mut Value, mode: BenchmarkMode) {
    if let Some(execution) = report.get_mut("execution") {
        if execution.get("timings").is_none()
            || execution.get("timings").is_some_and(Value::is_null)
        {
            execution["timings"] = json!({});
        }
        execution["timings"]["benchmarkMode"] = Value::String(mode.as_str().to_string());
    }
}

fn report_benchmark_mode(report: &Value) -> BenchmarkMode {
    report
        .get("execution")
        .and_then(|value| value.get("timings"))
        .and_then(|timings| timings.get("benchmarkMode"))
        .and_then(Value::as_str)
        .map(BenchmarkMode::from_value)
        .unwrap_or_else(configured_benchmark_mode)
}

fn parse_report_execution_timings(report: &Value) -> ExecutionTimings {
    report
        .get("execution")
        .and_then(|value| value.get("timings"))
        .cloned()
        .and_then(|value| serde_json::from_value::<ExecutionTimings>(value).ok())
        .unwrap_or_default()
}

fn parse_follow_job_timings(report: &Value) -> Option<FollowJobTimings> {
    report
        .get("followDaemon")
        .and_then(|value| value.get("job"))
        .and_then(|job| job.get("timings"))
        .cloned()
        .and_then(|value| serde_json::from_value::<FollowJobTimings>(value).ok())
}

fn refresh_report_benchmark(report: &mut Value) {
    let mode = report_benchmark_mode(report);
    let timings = sanitize_execution_timings_for_mode(&parse_report_execution_timings(report), mode);
    if let Some(execution) = report.get_mut("execution") {
        execution["timings"] = serde_json::to_value(&timings).unwrap_or_else(|_| json!({}));
    }
    let sent_items = report
        .get("execution")
        .and_then(|value| value.get("sent"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let sent = sent_items
        .into_iter()
        .map(|item| {
            let send_height = item.get("sendObservedBlockHeight").and_then(Value::as_u64);
            let confirmed_height = item
                .get("confirmedObservedBlockHeight")
                .and_then(Value::as_u64);
            json!({
                "label": item.get("label").cloned().unwrap_or_else(|| Value::String("(unknown)".to_string())),
                "signature": item.get("signature").cloned().unwrap_or(Value::Null),
                "confirmationStatus": item.get("confirmationStatus").cloned().unwrap_or(Value::Null),
                "sendBlockHeight": send_height,
                "confirmedBlockHeight": confirmed_height,
                "blocksToConfirm": match (send_height, confirmed_height) {
                    (Some(send_height), Some(confirmed_height)) => Some(confirmed_height.saturating_sub(send_height)),
                    _ => None,
                },
                "confirmedSlot": item.get("confirmedSlot").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();
    let timing_groups = if mode == BenchmarkMode::Off {
        Vec::new()
    } else {
        build_benchmark_timing_groups(&timings, parse_follow_job_timings(report).as_ref())
    };
    report["benchmark"] = json!({
        "mode": mode.as_str(),
        "timings": timings,
        "timingGroups": timing_groups,
        "sent": sent,
    });
}

fn attach_follow_daemon_report(
    report: &mut Value,
    transport: Option<&str>,
    reserved: Option<&FollowJobResponse>,
    armed: Option<&FollowJobResponse>,
    latest: Option<&FollowJobResponse>,
    original_follow_launch: Option<&NormalizedFollowLaunch>,
) {
    let latest_response = latest.or(armed).or(reserved);
    let mut job = latest_response.and_then(|response| response.job.clone());
    if let Some(job_record) = job.as_mut()
        && let Some(original_follow_launch) = original_follow_launch
    {
        job_record.followLaunch = original_follow_launch.clone();
    }
    let health = latest_response.map(|response| response.health.clone());
    report["followDaemon"] = json!({
        "schemaVersion": FOLLOW_RESPONSE_SCHEMA_VERSION,
        "enabled": reserved.is_some() || armed.is_some(),
        "transport": transport,
        "reserved": reserved,
        "armed": armed,
        "job": job,
        "health": health,
    });
}

fn split_same_time_snipes(
    follow_launch: &NormalizedFollowLaunch,
) -> (
    Vec<crate::config::NormalizedFollowLaunchSnipe>,
    NormalizedFollowLaunch,
) {
    let mut same_time = Vec::new();
    let mut deferred = follow_launch.clone();
    deferred.snipes = follow_launch
        .snipes
        .iter()
        .filter_map(|snipe| {
            if !snipe.enabled {
                return None;
            }
            if snipe.submitWithLaunch {
                same_time.push(snipe.clone());
                if snipe.retryOnFailure {
                    let mut retry_snipe = snipe.clone();
                    retry_snipe.submitWithLaunch = false;
                    retry_snipe.submitDelayMs = 450;
                    retry_snipe.targetBlockOffset = None;
                    retry_snipe.skipIfTokenBalancePositive = true;
                    Some(retry_snipe)
                } else {
                    None
                }
            } else {
                Some(snipe.clone())
            }
        })
        .collect::<Vec<_>>();
    deferred.enabled = !deferred.snipes.is_empty()
        || deferred.devAutoSell.as_ref().is_some_and(|sell| sell.enabled);
    (same_time, deferred)
}

fn has_same_time_snipes(follow_launch: &NormalizedFollowLaunch) -> bool {
    follow_launch
        .snipes
        .iter()
        .any(|snipe| snipe.enabled && snipe.submitWithLaunch)
}

fn same_time_wallet_label(wallet_env_key: &str) -> String {
    let trimmed = wallet_env_key.trim();
    let suffix = trimmed.trim_start_matches("SOLANA_PRIVATE_KEY").trim();
    if suffix.is_empty() {
        "primary".to_string()
    } else {
        suffix.to_string()
    }
}

fn parse_sol_decimal_to_lamports(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Some(0);
    }
    let normalized = trimmed.replace(',', ".");
    let mut parts = normalized.split('.');
    let whole = parts.next()?;
    let fractional = parts.next().unwrap_or("");
    if parts.next().is_some()
        || !whole.chars().all(|char| char.is_ascii_digit())
        || !fractional.chars().all(|char| char.is_ascii_digit())
    {
        return None;
    }
    let whole_value = whole.parse::<u64>().ok()?;
    let mut fractional_text = fractional.to_string();
    if fractional_text.len() > 9 {
        fractional_text.truncate(9);
    }
    while fractional_text.len() < 9 {
        fractional_text.push('0');
    }
    let fractional_value = if fractional_text.is_empty() {
        0
    } else {
        fractional_text.parse::<u64>().ok()?
    };
    whole_value
        .checked_mul(1_000_000_000)
        .and_then(|base| base.checked_add(fractional_value))
}

fn format_lamports_to_sol_decimal(value: u64) -> String {
    let whole = value / 1_000_000_000;
    let fractional = value % 1_000_000_000;
    if fractional == 0 {
        return whole.to_string();
    }
    let mut fractional_text = format!("{fractional:09}");
    while fractional_text.ends_with('0') {
        fractional_text.pop();
    }
    format!("{whole}.{fractional_text}")
}

const HELIUS_SENDER_TIP_ACCOUNTS: [&str; 10] = [
    "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE",
    "D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ",
    "9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta",
    "5VY91ws6B2hMmBFRsXkoAAdsPHBJwRfBht4DXox3xkwn",
    "2nyhqdwKcJZR2vcqCyrYsaPVdAnFoJjiksCXJ7hfEYgD",
    "2q5pghRs6arqVjRvT5gfgWfWcHWmw1ZuCzphgd5KfWGJ",
    "wyvPkWjVZz1M8fHQnMMCDTQDbkManefNNhweYk5WkcF",
    "3KCKozbAaF75qEU33jtzozcJ29yJuaLJTy2jFdzUY8bT",
    "4vieeGHPYPG2MmyPRcYjdiDmmhN3ww7hsFNap8pVN3Ey",
    "4TQLFNWK8AovT1gFvda5jfw2oJeRMKEmw7aH6MGBJ3or",
];
const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];
const DEFAULT_AUTO_FEE_HELIUS_PRIORITY_LEVEL: &str = "veryhigh";
const DEFAULT_AUTO_FEE_JITO_TIP_PERCENTILE: &str = "p99";
const FEE_MARKET_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const FEE_MARKET_MAX_AGE: Duration = Duration::from_secs(15);

#[derive(Debug, Default, Clone)]
struct FeeMarketSnapshot {
    helius_priority_lamports: Option<u64>,
    jito_tip_p99_lamports: Option<u64>,
}

#[derive(Debug, Clone)]
struct CachedFeeMarketSnapshot {
    snapshot: FeeMarketSnapshot,
    fetched_at: Instant,
}

fn fee_market_cache() -> &'static Mutex<HashMap<String, CachedFeeMarketSnapshot>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedFeeMarketSnapshot>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn fee_market_cache_key(rpc_url: &str, helius_priority_level: &str, jito_tip_percentile: &str) -> String {
    format!("{rpc_url}|{helius_priority_level}|{jito_tip_percentile}")
}

fn get_cached_fee_market_snapshot(
    rpc_url: &str,
    helius_priority_level: &str,
    jito_tip_percentile: &str,
) -> Option<FeeMarketSnapshot> {
    let cache = fee_market_cache().lock().ok()?;
    let entry = cache.get(&fee_market_cache_key(
        rpc_url,
        helius_priority_level,
        jito_tip_percentile,
    ))?;
    if entry.fetched_at.elapsed() > FEE_MARKET_MAX_AGE {
        return None;
    }
    Some(entry.snapshot.clone())
}

fn cache_fee_market_snapshot(
    rpc_url: &str,
    helius_priority_level: &str,
    jito_tip_percentile: &str,
    snapshot: &FeeMarketSnapshot,
) {
    if let Ok(mut cache) = fee_market_cache().lock() {
        cache.insert(
            fee_market_cache_key(rpc_url, helius_priority_level, jito_tip_percentile),
            CachedFeeMarketSnapshot {
                snapshot: snapshot.clone(),
                fetched_at: Instant::now(),
            },
        );
    }
}

fn shared_fee_market_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("shared fee market client")
    })
}

fn auto_fee_helius_priority_level() -> String {
    let value = std::env::var("LAUNCHDECK_AUTO_FEE_HELIUS_PRIORITY_LEVEL")
        .unwrap_or_else(|_| DEFAULT_AUTO_FEE_HELIUS_PRIORITY_LEVEL.to_string());
    let trimmed = value.trim().to_lowercase();
    match trimmed.as_str() {
        "none" | "low" | "medium" | "high" => trimmed,
        "veryhigh" | "very_high" | "very-high" => "veryHigh".to_string(),
        "unsafemax" | "unsafe_max" | "unsafe-max" => "unsafeMax".to_string(),
        "recommended" => "recommended".to_string(),
        _ => "veryHigh".to_string(),
    }
}

fn auto_fee_jito_tip_percentile() -> String {
    let value = std::env::var("LAUNCHDECK_AUTO_FEE_JITO_TIP_PERCENTILE")
        .unwrap_or_else(|_| DEFAULT_AUTO_FEE_JITO_TIP_PERCENTILE.to_string());
    let trimmed = value.trim().to_lowercase();
    match trimmed.as_str() {
        "p25" | "25" | "25th" => "p25".to_string(),
        "p50" | "50" | "50th" | "median" => "p50".to_string(),
        "p75" | "75" | "75th" => "p75".to_string(),
        "p95" | "95" | "95th" => "p95".to_string(),
        "p99" | "99" | "99th" => "p99".to_string(),
        _ => DEFAULT_AUTO_FEE_JITO_TIP_PERCENTILE.to_string(),
    }
}

fn lamports_to_priority_fee_micro_lamports(priority_fee_lamports: u64) -> u64 {
    if priority_fee_lamports == 0 {
        0
    } else {
        priority_fee_lamports
    }
}

fn pick_tip_account_for_provider(provider: &str) -> String {
    match provider.trim() {
        "helius-sender" => HELIUS_SENDER_TIP_ACCOUNTS[0].to_string(),
        "jito-bundle" => JITO_TIP_ACCOUNTS[0].to_string(),
        _ => String::new(),
    }
}

fn normalize_estimate_to_lamports(value: Option<&Value>) -> Option<u64> {
    let numeric = match value {
        Some(Value::Number(raw)) => raw.as_f64()?,
        Some(Value::String(raw)) => raw.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    if !numeric.is_finite() || numeric <= 0.0 {
        return None;
    }
    let lamports = if numeric < 1.0 {
        (numeric * 1_000_000_000.0).round()
    } else {
        numeric.round()
    };
    if lamports <= 0.0 {
        None
    } else {
        Some(lamports as u64)
    }
}

fn parse_auto_fee_cap_lamports(value: &str) -> Option<u64> {
    parse_sol_decimal_to_lamports(value).filter(|lamports| *lamports > 0)
}

fn cap_auto_fee_lamports(estimate_lamports: u64, cap_lamports: Option<u64>) -> u64 {
    match cap_lamports {
        Some(cap) if cap > 0 => estimate_lamports.min(cap),
        _ => estimate_lamports,
    }
}

async fn fetch_fee_market_snapshot_live(rpc_url: &str) -> Result<FeeMarketSnapshot, String> {
    let helius_priority_level = auto_fee_helius_priority_level();
    let jito_tip_percentile = auto_fee_jito_tip_percentile();
    let client = shared_fee_market_http_client();
    let helius_options = if helius_priority_level == "recommended" {
        json!({
            "recommended": true
        })
    } else {
        json!({
            "includeAllPriorityFeeLevels": true
        })
    };
    let helius_request = client
        .post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "launchdeck-helius-priority-estimate",
            "method": "getPriorityFeeEstimate",
            "params": [{
                "options": helius_options
            }]
        }))
        .send();
    let jito_request = client
        .get("https://bundles.jito.wtf/api/v1/bundles/tip_floor")
        .send();
    let (helius_response, jito_response) = tokio::join!(helius_request, jito_request);

    let helius_priority_lamports = match helius_response {
        Ok(response) => {
            let payload = response
                .json::<Value>()
                .await
                .map_err(|error| format!("Failed to decode Helius fee estimate: {error}"))?;
            if let Some(error) = payload.get("error") {
                return Err(format!("Helius priority estimate failed: {error}"));
            }
            let result = payload.get("result").unwrap_or(&payload);
            normalize_estimate_to_lamports(
                match helius_priority_level.as_str() {
                    "recommended" => result
                        .get("priorityFeeEstimate")
                        .or_else(|| result.get("recommended")),
                    selected_level => result
                        .get("priorityFeeLevels")
                        .and_then(|levels| levels.get(selected_level)),
                }
                    .or_else(|| {
                        result
                            .get("priorityFeeLevels")
                            .and_then(|levels| levels.get("veryHigh"))
                    })
                    .or_else(|| {
                        result
                            .get("priorityFeeEstimate")
                            .or_else(|| result.get("recommended"))
                    })
                    .or_else(|| {
                        result
                            .get("priorityFeeLevels")
                            .and_then(|levels| levels.get("high"))
                    }),
            )
        }
        Err(error) => return Err(format!("Helius priority estimate request failed: {error}")),
    };

    let jito_tip_p99_lamports = match jito_response {
        Ok(response) => {
            let payload = response
                .json::<Value>()
                .await
                .map_err(|error| format!("Failed to decode Jito tip floor: {error}"))?;
            let sample = payload
                .as_array()
                .and_then(|entries| entries.first())
                .unwrap_or(&payload);
            normalize_estimate_to_lamports(
                match jito_tip_percentile.as_str() {
                    "p25" => sample
                        .get("p25")
                        .or_else(|| sample.get("percentile25"))
                        .or_else(|| sample.get("tipFloor25"))
                        .or_else(|| sample.get("landed_tips_25th_percentile")),
                    "p50" => sample
                        .get("p50")
                        .or_else(|| sample.get("percentile50"))
                        .or_else(|| sample.get("tipFloor50"))
                        .or_else(|| sample.get("landed_tips_50th_percentile")),
                    "p75" => sample
                        .get("p75")
                        .or_else(|| sample.get("percentile75"))
                        .or_else(|| sample.get("tipFloor75"))
                        .or_else(|| sample.get("landed_tips_75th_percentile")),
                    "p95" => sample
                        .get("p95")
                        .or_else(|| sample.get("percentile95"))
                        .or_else(|| sample.get("tipFloor95"))
                        .or_else(|| sample.get("landed_tips_95th_percentile")),
                    _ => sample
                        .get("p99")
                        .or_else(|| sample.get("percentile99"))
                        .or_else(|| sample.get("tipFloor99"))
                        .or_else(|| sample.get("landed_tips_99th_percentile")),
                },
            )
        }
        Err(error) => return Err(format!("Jito tip floor request failed: {error}")),
    };

    let snapshot = FeeMarketSnapshot {
        helius_priority_lamports,
        jito_tip_p99_lamports,
    };
    cache_fee_market_snapshot(
        rpc_url,
        &helius_priority_level,
        &jito_tip_percentile,
        &snapshot,
    );
    Ok(snapshot)
}

async fn fetch_fee_market_snapshot(rpc_url: &str) -> Result<FeeMarketSnapshot, String> {
    let helius_priority_level = auto_fee_helius_priority_level();
    let jito_tip_percentile = auto_fee_jito_tip_percentile();
    if let Some(snapshot) =
        get_cached_fee_market_snapshot(rpc_url, &helius_priority_level, &jito_tip_percentile)
    {
        return Ok(snapshot);
    }
    fetch_fee_market_snapshot_live(rpc_url).await
}

fn spawn_fee_market_snapshot_refresh_task(rpc_url: String) {
    tokio::spawn(async move {
        loop {
            let _ = fetch_fee_market_snapshot_live(&rpc_url).await;
            tokio::time::sleep(FEE_MARKET_REFRESH_INTERVAL).await;
        }
    });
}

async fn resolve_auto_execution_fees(
    rpc_url: &str,
    normalized: &mut NormalizedConfig,
    transport_plan: &crate::transport::TransportPlan,
) -> Result<Vec<String>, String> {
    let needs_auto = normalized.execution.autoGas
        || normalized.execution.buyAutoGas
        || normalized.execution.sellAutoGas;
    if !needs_auto {
        return Ok(vec![]);
    }
    let market = fetch_fee_market_snapshot(rpc_url).await?;
    let mut notes = Vec::new();

    if normalized.execution.autoGas {
        let cap_lamports = parse_auto_fee_cap_lamports(
            if !normalized.execution.maxPriorityFeeSol.trim().is_empty() {
                &normalized.execution.maxPriorityFeeSol
            } else {
                &normalized.execution.maxTipSol
            },
        );
        let provider = normalized.execution.provider.as_str();
        let uses_priority = match provider {
            "standard-rpc" => true,
            "helius-sender" => true,
            "jito-bundle" => transport_plan.executionClass != "bundle",
            _ => true,
        };
        let uses_tip = matches!(provider, "helius-sender")
            || (provider == "jito-bundle");

        if uses_priority {
            let estimated = market.helius_priority_lamports.ok_or_else(|| {
                "Creation auto fee is enabled but no Helius priority estimate was returned."
                    .to_string()
            })?;
            let resolved = cap_auto_fee_lamports(estimated.max(1), cap_lamports);
            normalized.execution.priorityFeeSol = format_lamports_to_sol_decimal(resolved);
            normalized.tx.computeUnitPriceMicroLamports =
                Some(lamports_to_priority_fee_micro_lamports(resolved) as i64);
        } else {
            normalized.execution.priorityFeeSol.clear();
            normalized.tx.computeUnitPriceMicroLamports = Some(0);
        }

        if uses_tip {
            let estimated = market.jito_tip_p99_lamports.ok_or_else(|| {
                "Creation auto fee is enabled but no Jito tip estimate was returned.".to_string()
            })?;
            let mut resolved = cap_auto_fee_lamports(estimated, cap_lamports);
            if provider == "helius-sender" && resolved > 0 && resolved < 200_000 {
                if cap_lamports.is_some() && cap_lamports.unwrap_or_default() < 200_000 {
                    return Err(
                        "Creation max auto fee is below the Helius Sender minimum tip of 0.0002 SOL."
                            .to_string(),
                    );
                }
                resolved = 200_000;
            }
            normalized.execution.tipSol = format_lamports_to_sol_decimal(resolved);
            normalized.tx.jitoTipLamports = resolved as i64;
            normalized.tx.jitoTipAccount = pick_tip_account_for_provider(provider);
        } else {
            normalized.execution.tipSol.clear();
            normalized.tx.jitoTipLamports = 0;
            normalized.tx.jitoTipAccount.clear();
        }

        notes.push(format!(
            "Creation auto fee resolved{}: priority={} SOL | tip={} SOL",
            cap_lamports
                .map(|cap| format!(" with cap {} SOL", format_lamports_to_sol_decimal(cap)))
                .unwrap_or_default(),
            if normalized.execution.priorityFeeSol.trim().is_empty() {
                "off".to_string()
            } else {
                normalized.execution.priorityFeeSol.clone()
            },
            if normalized.execution.tipSol.trim().is_empty() {
                "off".to_string()
            } else {
                normalized.execution.tipSol.clone()
            }
        ));
    }

    if normalized.execution.buyAutoGas {
        let cap_lamports = parse_auto_fee_cap_lamports(
            if !normalized.execution.buyMaxPriorityFeeSol.trim().is_empty() {
                &normalized.execution.buyMaxPriorityFeeSol
            } else {
                &normalized.execution.buyMaxTipSol
            },
        );
        let provider = normalized.execution.buyProvider.as_str();
        let uses_priority = provider != "jito-bundle" || true;
        let uses_tip = provider == "helius-sender" || provider == "jito-bundle";

        if uses_priority {
            let estimated = market.helius_priority_lamports.ok_or_else(|| {
                "Buy auto fee is enabled but no Helius priority estimate was returned.".to_string()
            })?;
            let resolved = cap_auto_fee_lamports(estimated.max(1), cap_lamports);
            normalized.execution.buyPriorityFeeSol = format_lamports_to_sol_decimal(resolved);
        } else {
            normalized.execution.buyPriorityFeeSol.clear();
        }

        if uses_tip {
            let estimated = market.jito_tip_p99_lamports.ok_or_else(|| {
                "Buy auto fee is enabled but no Jito tip estimate was returned.".to_string()
            })?;
            let mut resolved = cap_auto_fee_lamports(estimated, cap_lamports);
            if provider == "helius-sender" && resolved > 0 && resolved < 200_000 {
                if cap_lamports.is_some() && cap_lamports.unwrap_or_default() < 200_000 {
                    return Err(
                        "Buy max auto fee is below the Helius Sender minimum tip of 0.0002 SOL."
                            .to_string(),
                    );
                }
                resolved = 200_000;
            }
            normalized.execution.buyTipSol = format_lamports_to_sol_decimal(resolved);
        } else {
            normalized.execution.buyTipSol.clear();
        }

        notes.push(format!(
            "Buy auto fee resolved{}: priority={} SOL | tip={} SOL",
            cap_lamports
                .map(|cap| format!(" with cap {} SOL", format_lamports_to_sol_decimal(cap)))
                .unwrap_or_default(),
            if normalized.execution.buyPriorityFeeSol.trim().is_empty() {
                "off".to_string()
            } else {
                normalized.execution.buyPriorityFeeSol.clone()
            },
            if normalized.execution.buyTipSol.trim().is_empty() {
                "off".to_string()
            } else {
                normalized.execution.buyTipSol.clone()
            }
        ));
    }

    if normalized.execution.sellAutoGas {
        let cap_lamports = parse_auto_fee_cap_lamports(
            if !normalized.execution.sellMaxPriorityFeeSol.trim().is_empty() {
                &normalized.execution.sellMaxPriorityFeeSol
            } else {
                &normalized.execution.sellMaxTipSol
            },
        );
        let provider = normalized.execution.sellProvider.as_str();
        let uses_priority = provider != "jito-bundle" || true;
        let uses_tip = provider == "helius-sender" || provider == "jito-bundle";

        if uses_priority {
            let estimated = market.helius_priority_lamports.ok_or_else(|| {
                "Sell auto fee is enabled but no Helius priority estimate was returned."
                    .to_string()
            })?;
            let resolved = cap_auto_fee_lamports(estimated.max(1), cap_lamports);
            normalized.execution.sellPriorityFeeSol = format_lamports_to_sol_decimal(resolved);
        } else {
            normalized.execution.sellPriorityFeeSol.clear();
        }

        if uses_tip {
            let estimated = market.jito_tip_p99_lamports.ok_or_else(|| {
                "Sell auto fee is enabled but no Jito tip estimate was returned.".to_string()
            })?;
            let mut resolved = cap_auto_fee_lamports(estimated, cap_lamports);
            if provider == "helius-sender" && resolved > 0 && resolved < 200_000 {
                if cap_lamports.is_some() && cap_lamports.unwrap_or_default() < 200_000 {
                    return Err(
                        "Sell max auto fee is below the Helius Sender minimum tip of 0.0002 SOL."
                            .to_string(),
                    );
                }
                resolved = 200_000;
            }
            normalized.execution.sellTipSol = format_lamports_to_sol_decimal(resolved);
        } else {
            normalized.execution.sellTipSol.clear();
        }

        notes.push(format!(
            "Sell auto fee resolved{}: priority={} SOL | tip={} SOL",
            cap_lamports
                .map(|cap| format!(" with cap {} SOL", format_lamports_to_sol_decimal(cap)))
                .unwrap_or_default(),
            if normalized.execution.sellPriorityFeeSol.trim().is_empty() {
                "off".to_string()
            } else {
                normalized.execution.sellPriorityFeeSol.clone()
            },
            if normalized.execution.sellTipSol.trim().is_empty() {
                "off".to_string()
            } else {
                normalized.execution.sellTipSol.clone()
            }
        ));
    }

    Ok(notes)
}

fn apply_same_time_creation_fee_guard(
    normalized: &mut NormalizedConfig,
) -> Result<Option<String>, String> {
    if !has_same_time_snipes(&normalized.followLaunch) {
        return Ok(None);
    }
    let creation_priority = parse_sol_decimal_to_lamports(&normalized.execution.priorityFeeSol)
        .ok_or_else(|| "Invalid creation priority fee while applying same-time guard.".to_string())?;
    let buy_priority = parse_sol_decimal_to_lamports(&normalized.execution.buyPriorityFeeSol)
        .ok_or_else(|| "Invalid buy priority fee while applying same-time guard.".to_string())?;
    let creation_tip = parse_sol_decimal_to_lamports(&normalized.execution.tipSol)
        .ok_or_else(|| "Invalid creation tip while applying same-time guard.".to_string())?;
    let buy_tip = parse_sol_decimal_to_lamports(&normalized.execution.buyTipSol)
        .ok_or_else(|| "Invalid buy tip while applying same-time guard.".to_string())?;
    let mut adjusted_fields = Vec::new();
    if creation_priority <= buy_priority {
        let next_priority = buy_priority.saturating_add(1);
        if normalized.execution.autoGas {
            if let Some(cap_lamports) =
                parse_auto_fee_cap_lamports(&normalized.execution.maxPriorityFeeSol)
            {
                if next_priority > cap_lamports {
                    return Err(
                        "Same-time sniper safeguard needs a higher launch priority fee than your Creation max auto fee allows."
                            .to_string(),
                    );
                }
            }
        }
        let next_priority_text = format_lamports_to_sol_decimal(next_priority);
        normalized.execution.priorityFeeSol = next_priority_text.clone();
        normalized.execution.maxPriorityFeeSol = next_priority_text;
        adjusted_fields.push("priority fee");
    }
    if creation_tip <= buy_tip {
        let next_tip = buy_tip.saturating_add(1);
        if normalized.execution.autoGas {
            if let Some(cap_lamports) = parse_auto_fee_cap_lamports(&normalized.execution.maxTipSol)
            {
                if next_tip > cap_lamports {
                    return Err(
                        "Same-time sniper safeguard needs a higher launch tip than your Creation max auto fee allows."
                            .to_string(),
                    );
                }
            }
        }
        let next_tip_text = format_lamports_to_sol_decimal(next_tip);
        normalized.execution.tipSol = next_tip_text.clone();
        normalized.execution.maxTipSol = next_tip_text;
        normalized.tx.jitoTipLamports = next_tip as i64;
        adjusted_fields.push("tip");
    }
    if adjusted_fields.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!(
            "Same-time sniper safeguard raised launch {} above same-time buy fees.",
            adjusted_fields.join(" and ")
        )))
    }
}

async fn compile_same_time_snipes(
    rpc_url: &str,
    normalized: &crate::config::NormalizedConfig,
    mint: &str,
    launch_creator: &str,
    snipes: &[crate::config::NormalizedFollowLaunchSnipe],
    allow_ata_creation: bool,
) -> Result<Vec<CompiledTransaction>, String> {
    let tasks = snipes.iter().enumerate().map(|(index, snipe)| async move {
        let wallet_secret = load_solana_wallet_by_env_key(&snipe.walletEnvKey)?;
        let mut tx = compile_atomic_follow_buy_for_launchpad(
            &normalized.launchpad,
            &normalized.mode,
            &normalized.quoteAsset,
            rpc_url,
            &normalized.execution,
            normalized.token.mayhemMode,
            &normalized.tx.jitoTipAccount,
            &wallet_secret,
            mint,
            launch_creator,
            &snipe.buyAmountSol,
            allow_ata_creation,
        )
        .await?;
        tx.label = format!(
            "sniper-buy-{}-wallet-{}",
            index + 1,
            same_time_wallet_label(&snipe.walletEnvKey)
        );
        Ok::<Vec<CompiledTransaction>, String>(vec![tx])
    });
    join_all(tasks)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map(|groups| groups.into_iter().flatten().collect())
}

async fn compile_presigned_follow_actions(
    rpc_url: &str,
    normalized: &NormalizedConfig,
    mint: &str,
    launch_creator: &str,
    allow_ata_creation: bool,
) -> Result<Vec<crate::follow::FollowActionRecord>, String> {
    let mut actions = build_action_records(&normalized.followLaunch);
    let mut predicted_dev_buy_tokens: Option<Option<u64>> = None;
    let mut predicted_bonk_dev_buy_tokens: Option<Option<u64>> = None;
    let bonk_pool_id = if normalized.launchpad == "bonk" {
        Some(derive_bonk_canonical_pool_id(&normalized.quoteAsset, mint).await?)
    } else {
        None
    };
    for action in &mut actions {
        let wallet_key = selected_wallet_key_or_default(&action.walletEnvKey)
            .ok_or_else(|| format!("Wallet env key not found: {}", action.walletEnvKey))?;
        let wallet_secret = load_solana_wallet_by_env_key(&wallet_key)?;
        if let Some(pool_id) = bonk_pool_id.as_ref() {
            action.poolId = Some(pool_id.clone());
        }
        match action.kind {
            crate::follow::FollowActionKind::SniperBuy => {
                let Some(buy_amount_sol) = action.buyAmountSol.as_deref() else {
                    continue;
                };
                let mut tx = compile_atomic_follow_buy_for_launchpad(
                    &normalized.launchpad,
                    &normalized.mode,
                    &normalized.quoteAsset,
                    rpc_url,
                    &normalized.execution,
                    normalized.token.mayhemMode,
                    &normalized.tx.jitoTipAccount,
                    &wallet_secret,
                    mint,
                    launch_creator,
                    buy_amount_sol,
                    allow_ata_creation,
                )
                .await?;
                tx.label = action.actionId.clone();
                action.preSignedTransactions = vec![tx];
            }
            crate::follow::FollowActionKind::DevAutoSell
                if normalized.launchpad == "pump"
                    && action.walletEnvKey == normalized.selectedWalletKey =>
            {
                let predicted_tokens = match predicted_dev_buy_tokens {
                    Some(value) => value,
                    None => {
                        let value = predict_dev_buy_token_amount(rpc_url, normalized).await?;
                        predicted_dev_buy_tokens = Some(value);
                        value
                    }
                };
                let Some(predicted_tokens) = predicted_tokens else {
                    continue;
                };
                let Some(sell_percent) = action.sellPercent else {
                    continue;
                };
                let Some(mut tx) = crate::pump_native::compile_follow_sell_transaction_with_token_amount(
                    rpc_url,
                    &normalized.execution,
                    normalized.token.mayhemMode,
                    &normalized.tx.jitoTipAccount,
                    &wallet_secret,
                    mint,
                    launch_creator,
                    sell_percent,
                    should_use_post_setup_creator_vault_for_sell(
                        matches!(normalized.mode.as_str(), "agent-custom" | "agent-locked"),
                        action,
                    ),
                    Some(predicted_tokens),
                    Some(normalized.mode == "cashback"),
                )
                .await? else {
                    continue;
                };
                tx.label = action.actionId.clone();
                action.preSignedTransactions = vec![tx];
            }
            crate::follow::FollowActionKind::DevAutoSell
                if normalized.launchpad == "bonk"
                    && action.walletEnvKey == normalized.selectedWalletKey =>
            {
                let predicted_tokens = match predicted_bonk_dev_buy_tokens {
                    Some(value) => value,
                    None => {
                        let value = predict_bonk_dev_buy_token_amount(rpc_url, normalized).await?;
                        predicted_bonk_dev_buy_tokens = Some(value);
                        value
                    }
                };
                let Some(predicted_tokens) = predicted_tokens else {
                    continue;
                };
                let Some(sell_percent) = action.sellPercent else {
                    continue;
                };
                match crate::bonk_native::compile_follow_sell_transaction_with_token_amount(
                    rpc_url,
                    &normalized.quoteAsset,
                    &normalized.execution,
                    &normalized.tx.jitoTipAccount,
                    &wallet_secret,
                    mint,
                    sell_percent,
                    Some(predicted_tokens),
                    bonk_pool_id.as_deref(),
                    Some(&normalized.mode),
                    Some(launch_creator),
                )
                .await
                {
                    Ok(Some(mut tx)) => {
                        tx.label = action.actionId.clone();
                        action.preSignedTransactions = vec![tx];
                    }
                    Ok(None) => {}
                    Err(error) => {
                        action.lastError = Some(format!(
                            "Bonk dev-auto-sell pre-sign unavailable; daemon will compile live after launch: {error}"
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(actions)
}

async fn build_status_payload(
    state: &Arc<AppState>,
    requested_wallet: &str,
    strict_wallet_selection: bool,
) -> Result<Value, String> {
    let mut payload = build_bootstrap_fast_payload(requested_wallet, strict_wallet_selection)?;
    let wallet_status =
        build_wallet_status_payload(requested_wallet, strict_wallet_selection).await?;
    let runtime_status = build_runtime_status_payload(state).await;
    payload["rpcUrl"] = wallet_status
        .get("rpcUrl")
        .cloned()
        .unwrap_or(Value::String(configured_rpc_url()));
    payload["connected"] = wallet_status
        .get("connected")
        .cloned()
        .unwrap_or(Value::Bool(false));
    payload["wallets"] = wallet_status
        .get("wallets")
        .cloned()
        .unwrap_or(Value::Array(vec![]));
    payload["selectedWalletKey"] = wallet_status
        .get("selectedWalletKey")
        .cloned()
        .unwrap_or(Value::Null);
    payload["wallet"] = wallet_status.get("wallet").cloned().unwrap_or(Value::Null);
    payload["balanceLamports"] = wallet_status
        .get("balanceLamports")
        .cloned()
        .unwrap_or(Value::Null);
    payload["balanceSol"] = wallet_status
        .get("balanceSol")
        .cloned()
        .unwrap_or(Value::Null);
    payload["usd1Balance"] = wallet_status
        .get("usd1Balance")
        .cloned()
        .unwrap_or(Value::Null);
    payload["transport"] = runtime_status
        .get("transport")
        .cloned()
        .unwrap_or(Value::Null);
    payload["followDaemon"] = runtime_status
        .get("followDaemon")
        .cloned()
        .unwrap_or(Value::Null);
    payload["runtime"] = runtime_status
        .get("runtime")
        .cloned()
        .unwrap_or(Value::Null);
    payload["runtimeWorkers"] = runtime_status
        .get("runtimeWorkers")
        .cloned()
        .unwrap_or(Value::Array(vec![]));
    Ok(payload)
}

fn resolve_selected_wallet_key(
    requested_wallet: &str,
    strict_wallet_selection: bool,
    raw_wallets: &[crate::wallet::WalletSummary],
) -> Result<Option<String>, String> {
    let requested_wallet = requested_wallet.trim();
    let requested_wallet_is_known = requested_wallet.is_empty()
        || raw_wallets
            .iter()
            .any(|wallet| wallet.envKey == requested_wallet);
    if strict_wallet_selection && !requested_wallet_is_known {
        return Err(format!("Unknown wallet env key: {}", requested_wallet));
    }
    Ok(selected_wallet_key_or_default_from_wallets(
        if requested_wallet_is_known {
            requested_wallet
        } else {
            ""
        },
        raw_wallets,
    ))
}

fn build_bootstrap_fast_payload(
    requested_wallet: &str,
    strict_wallet_selection: bool,
) -> Result<Value, String> {
    let raw_wallets = list_solana_env_wallets();
    let selected_wallet_key =
        resolve_selected_wallet_key(requested_wallet, strict_wallet_selection, &raw_wallets)?;
    let selected_wallet = selected_wallet_key.as_ref().and_then(|env_key| {
        raw_wallets
            .iter()
            .find(|wallet| wallet.envKey == *env_key)
            .cloned()
    });
    Ok(json!({
        "ok": true,
        "service": "launchdeck-engine",
        "engineBackend": "rust",
        "implemented": true,
        "executionMode": "rust-native-only",
        "message": "Rust engine is online with native Pump execution, native RPC transport, and native runtime workers. Unsupported requests fail explicitly instead of falling back to JavaScript.",
        "connected": selected_wallet.is_some(),
        "wallets": raw_wallets,
        "selectedWalletKey": selected_wallet_key,
        "wallet": selected_wallet.as_ref().and_then(|wallet| wallet.publicKey.clone()),
        "providers": provider_availability_registry(),
        "providerRegistry": provider_registry(),
        "launchpads": launchpad_registry(),
        "startupWarm": {
            "enabled": configured_startup_warm_enabled(),
        },
        "regionRouting": build_region_routing_payload(),
        "config": read_persistent_config(),
    }))
}

async fn build_wallet_status_payload(
    requested_wallet: &str,
    strict_wallet_selection: bool,
) -> Result<Value, String> {
    let raw_wallets = list_solana_env_wallets();
    let selected_wallet_key =
        resolve_selected_wallet_key(requested_wallet, strict_wallet_selection, &raw_wallets)?;
    let rpc_url = configured_rpc_url();
    let wallets = enrich_wallet_statuses(&rpc_url, USD1_MINT, &raw_wallets).await;
    let selected_wallet = selected_wallet_key.as_ref().and_then(|env_key| {
        wallets
            .iter()
            .find(|wallet| wallet.envKey == *env_key)
            .cloned()
    });
    Ok(json!({
        "ok": true,
        "rpcUrl": rpc_url,
        "connected": selected_wallet.is_some(),
        "wallets": wallets,
        "selectedWalletKey": selected_wallet_key,
        "wallet": selected_wallet.as_ref().and_then(|wallet| wallet.publicKey.clone()),
        "balanceLamports": selected_wallet.as_ref().and_then(|wallet| wallet.balanceLamports),
        "balanceSol": selected_wallet.as_ref().and_then(|wallet| wallet.balanceSol),
        "usd1Balance": selected_wallet.as_ref().and_then(|wallet| wallet.usd1Balance),
    }))
}

async fn build_runtime_status_payload(state: &Arc<AppState>) -> Value {
    let (runtime_workers, follow_daemon) =
        tokio::join!(list_workers(&state.runtime), follow_daemon_status_payload(),);
    json!({
        "ok": true,
        "service": "launchdeck-engine",
        "transport": {
            "heliusSenderEndpoint": configured_helius_sender_endpoint(),
            "jitoBundleEndpoints": configured_jito_bundle_endpoints(),
        },
        "followDaemon": follow_daemon,
        "runtime": {
            "statePath": state.runtime.storage_path,
            "workerCount": runtime_workers.len(),
        },
        "runtimeWorkers": runtime_workers,
    })
}

fn build_region_routing_payload() -> Value {
    json!({
        "shared": {
            "configured": configured_shared_region(),
            "effective": default_endpoint_profile(),
        },
        "providers": {
            "helius-sender": {
                "configured": configured_provider_region("helius-sender"),
                "effective": default_endpoint_profile_for_provider("helius-sender"),
                "endpointOverrideActive": helius_sender_endpoint_override_active(),
            },
            "jito-bundle": {
                "configured": configured_provider_region("jito-bundle"),
                "effective": default_endpoint_profile_for_provider("jito-bundle"),
                "endpointOverrideActive": jito_bundle_endpoint_override_active(),
            }
        },
        "restartRequired": true,
    })
}

async fn engine_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<StatusRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    build_status_payload(&state, &payload.wallet.unwrap_or_default(), true)
        .await
        .map(Json)
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })
}

async fn execute_engine_action_payload(
    _state: &Arc<AppState>,
    payload: EngineRequest,
) -> Result<Value, (StatusCode, Json<Value>)> {
    let trace = new_trace_context();
    let action_started = Instant::now();
    let action = payload.action.unwrap_or_else(|| "unknown".to_string());
    let benchmark_mode = configured_benchmark_mode();
    let prepare_request_payload_ms = payload.prepare_request_payload_ms.map(u128::from);
    if action == "quote" {
        let form_value = payload.form.ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": "form is required for quote requests.",
                    "traceId": trace.traceId,
                })),
            )
        })?;
        let quote = quote_from_form(&configured_rpc_url(), form_value)
            .await
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": error,
                        "traceId": trace.traceId,
                    })),
                )
            })?;
        return Ok(json!({
            "ok": true,
            "service": "launchdeck-engine",
            "action": action,
            "implemented": true,
            "quote": quote,
            "traceId": trace.traceId,
            "elapsedMs": current_time_ms().saturating_sub(trace.startedAtMs),
        }));
    }
    let form_prepare_started = Instant::now();
    let (raw_config_value, prepared_metadata_uri, form_to_raw_config_ms) =
        if let Some(raw_config_value) = payload.raw_config {
            (raw_config_value, None, None)
        } else if let Some(form_value) = payload.form.clone() {
            let (raw_config, metadata_uri) = build_raw_config_from_form(&action, form_value)
                .await
                .map_err(|error| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "ok": false,
                            "error": error,
                            "traceId": trace.traceId,
                        })),
                    )
                })?;
            let raw_config_value = serde_json::to_value(raw_config).map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": error.to_string(),
                        "traceId": trace.traceId,
                    })),
                )
            })?;
            (
                raw_config_value,
                metadata_uri,
                Some(form_prepare_started.elapsed().as_millis()),
            )
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": "rawConfig or form is required.",
                    "traceId": trace.traceId,
                })),
            ));
        };
    let parsed: RawConfig = serde_json::from_value(raw_config_value.clone()).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": format!("Invalid rawConfig payload: {error}"),
                "traceId": trace.traceId,
            })),
        )
    })?;
    let normalize_started = Instant::now();
    let mut normalized = normalize_raw_config(parsed).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error.to_string(),
                "traceId": trace.traceId,
            })),
        )
    })?;
    let normalize_config_ms = normalize_started.elapsed().as_millis();
    log_event(
        "engine_action_received",
        &trace.traceId,
        json!({
            "action": action,
            "mode": normalized.mode,
            "launchpad": normalized.launchpad,
            "provider": normalized.execution.provider,
            "walletKey": raw_config_value.get("selectedWalletKey").cloned().unwrap_or(Value::Null),
        }),
    );
    let selected_wallet_key = raw_config_value
        .get("selectedWalletKey")
        .and_then(Value::as_str)
        .and_then(|value| selected_wallet_key_or_default(value))
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": "Creator keypair is required via selected wallet key or SOLANA_PRIVATE_KEY.",
                    "traceId": trace.traceId,
                })),
            )
        })?;
    let wallet_load_started = Instant::now();
    let wallet_secret = load_solana_wallet_by_env_key(&selected_wallet_key).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
                "traceId": trace.traceId,
            })),
        )
    })?;
    let creator_public_key = public_key_from_secret(&wallet_secret).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
                "traceId": trace.traceId,
            })),
        )
    })?;
    let wallet_load_ms = wallet_load_started.elapsed().as_millis();
    let preview_agent_authority = if normalized.mode == "regular" || normalized.mode == "cashback" {
        None
    } else if !normalized.agent.authority.trim().is_empty() {
        Some(normalized.agent.authority.clone())
    } else {
        Some(creator_public_key.clone())
    };
    let transport_plan_started = Instant::now();
    let transport_plan = build_transport_plan(
        &normalized.execution,
        estimate_transaction_count(&normalized),
    );
    let transport_plan_build_ms = transport_plan_started.elapsed().as_millis();
    let rpc_url = configured_rpc_url();
    let auto_fee_started = Instant::now();
    let auto_fee_notes = resolve_auto_execution_fees(
        &rpc_url,
        &mut normalized,
        &transport_plan,
    )
    .await
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
                "traceId": trace.traceId,
            })),
        )
    })?;
    let auto_fee_resolve_ms = auto_fee_started.elapsed().as_millis();
    let same_time_guard_started = Instant::now();
    let same_time_fee_guard_warning = apply_same_time_creation_fee_guard(&mut normalized).map_err(
        |error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                    "traceId": trace.traceId,
                })),
            )
        },
    )?;
    let same_time_fee_guard_ms = same_time_guard_started.elapsed().as_millis();
    let should_persist_report = normalized.tx.writeReport || action == "send";
    if action == "build" {
        let report_build_started = Instant::now();
        let report = build_report(
            &normalized,
            &transport_plan,
            now_timestamp_string(),
            configured_rpc_url(),
            creator_public_key.clone(),
            synthetic_mint_address(&trace.traceId),
            preview_agent_authority,
            Some("Rust native build".to_string()),
            vec![],
        );
        let report_build_ms = report_build_started.elapsed().as_millis();
        let mut report_value = serde_json::to_value(&report).map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error.to_string(),
                    "traceId": trace.traceId,
                })),
            )
        })?;
        if let Some(warning) = same_time_fee_guard_warning.as_deref() {
            append_execution_warning(&mut report_value, warning);
        }
        set_report_benchmark_mode(&mut report_value, benchmark_mode);
        set_optional_report_timing(
            &mut report_value,
            "prepareRequestPayloadMs",
            prepare_request_payload_ms,
        );
        for note in &auto_fee_notes {
            append_execution_note(&mut report_value, note);
        }
        if let Some(value) = form_to_raw_config_ms {
            set_report_timing(&mut report_value, "formToRawConfigMs", value);
        }
        set_report_timing(&mut report_value, "normalizeConfigMs", normalize_config_ms);
        set_report_timing(&mut report_value, "walletLoadMs", wallet_load_ms);
        set_report_timing(
            &mut report_value,
            "transportPlanBuildMs",
            transport_plan_build_ms,
        );
        set_report_timing(&mut report_value, "autoFeeResolveMs", auto_fee_resolve_ms);
        set_report_timing(
            &mut report_value,
            "sameTimeFeeGuardMs",
            same_time_fee_guard_ms,
        );
        set_report_timing(&mut report_value, "reportBuildMs", report_build_ms);
        let send_log_path = if should_persist_report {
            let persist_started = Instant::now();
            let path =
                persist_launch_report(&trace.traceId, &action, &transport_plan, &report_value)
                    .map_err(|error| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "ok": false,
                                "error": error,
                                "traceId": trace.traceId,
                            })),
                        )
                    })?;
            set_report_timing(
                &mut report_value,
                "persistReportMs",
                persist_started.elapsed().as_millis(),
            );
            set_report_timing(
                &mut report_value,
                "persistInitialSnapshotMs",
                persist_started.elapsed().as_millis(),
            );
            report_value["outPath"] = Value::String(path.clone());
            Some(path)
        } else {
            None
        };
        let backend_elapsed_ms = action_started.elapsed().as_millis();
        set_report_timing(&mut report_value, "executionTotalMs", backend_elapsed_ms);
        set_report_timing(&mut report_value, "backendTotalElapsedMs", backend_elapsed_ms);
        set_report_timing(&mut report_value, "totalElapsedMs", backend_elapsed_ms);
        refresh_report_benchmark(&mut report_value);
        if let Some(path) = send_log_path.as_ref() {
            let finalize_started = Instant::now();
            update_persisted_launch_report(
                path,
                &trace.traceId,
                &action,
                &transport_plan,
                &report_value,
            )
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": format!("Action completed but failed to finalize persisted report: {error}"),
                        "traceId": trace.traceId,
                    })),
                )
            })?;
            set_report_timing(
                &mut report_value,
                "persistFinalReportUpdateMs",
                finalize_started.elapsed().as_millis(),
            );
        }
        log_event(
            "engine_action_completed",
            &trace.traceId,
            json!({
                "action": action,
                "executor": "rust-native",
            }),
        );
        return Ok(json!({
            "ok": true,
            "service": "launchdeck-engine",
            "action": action,
            "implemented": true,
            "executionImplemented": true,
            "executor": "rust-native",
            "message": "Rust engine validated the request and built a native planning report.",
            "traceId": trace.traceId,
            "elapsedMs": current_time_ms().saturating_sub(trace.startedAtMs),
            "receivedForm": payload.form.is_some(),
            "receivedRawConfig": true,
            "normalizedConfig": redacted_normalized_config(&normalized),
            "transportPlan": transport_plan,
            "report": report_value,
            "sendLogPath": send_log_path,
            "text": render_report_value(&report_value),
            "metadataUri": prepared_metadata_uri,
        }));
    }
    let compile_started_ms = current_time_ms();
    let mut prepared_bags_send: Option<PreparedBagsSendArtifacts> = None;
    let native_artifacts = if action == "send" && normalized.launchpad == "bagsapp" {
        let prepared = prepare_native_bags_send(
            &configured_rpc_url(),
            &normalized,
            &transport_plan,
            &wallet_secret,
            now_timestamp_string(),
            creator_public_key.clone(),
            Some("Rust native compile".to_string()),
        )
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                    "traceId": trace.traceId,
                })),
            )
        })?;
        let native = prepared.native_artifacts.clone();
        prepared_bags_send = Some(prepared);
        Some(native.into())
    } else {
        try_compile_native_launchpad(
            &configured_rpc_url(),
            &normalized,
            &transport_plan,
            &wallet_secret,
            now_timestamp_string(),
            creator_public_key.clone(),
            Some("Rust native compile".to_string()),
            action == "send",
        )
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                    "traceId": trace.traceId,
                })),
            )
        })?
    };
    let compile_transactions_ms = current_time_ms().saturating_sub(compile_started_ms);
    let native = native_artifacts.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": format!(
                    "Native Rust engine does not support launchpad={} mode={} yet. JavaScript compile fallback has been removed.",
                    normalized.launchpad,
                    normalized.mode
                ),
                "traceId": trace.traceId,
            })),
        )
    })?;
    let (
        mut compiled_transactions,
        creation_transactions,
        deferred_setup_transactions,
        mut report_value,
        text_value,
        assembly_executor,
        compile_breakdown,
        compiled_mint,
        compiled_launch_creator,
    ) = (
        native.compiled_transactions,
        native.creation_transactions,
        native.deferred_setup_transactions,
        native.report,
        Value::String(native.text),
        "rust-native".to_string(),
        native.compile_timings,
        native.mint,
        native.launch_creator,
    );
    if let Some(warning) = same_time_fee_guard_warning.as_deref() {
        append_execution_warning(&mut report_value, warning);
    }
    for note in &auto_fee_notes {
        append_execution_note(&mut report_value, note);
    }
    set_report_timing(
        &mut report_value,
        "compileTransactionsMs",
        compile_transactions_ms,
    );
    set_report_timing(
        &mut report_value,
        "compileLaunchCreatorPrepMs",
        compile_breakdown.launch_creator_prep_ms,
    );
    set_report_timing(
        &mut report_value,
        "compileAltLoadMs",
        compile_breakdown.alt_load_ms,
    );
    set_report_timing(
        &mut report_value,
        "compileBlockhashFetchMs",
        compile_breakdown.blockhash_fetch_ms,
    );
    set_optional_report_timing(
        &mut report_value,
        "compileGlobalFetchMs",
        compile_breakdown.global_fetch_ms,
    );
    set_optional_report_timing(
        &mut report_value,
        "compileFollowUpPrepMs",
        compile_breakdown.follow_up_prep_ms,
    );
    set_report_timing(
        &mut report_value,
        "compileTxSerializeMs",
        compile_breakdown.tx_serialize_ms,
    );
    set_optional_report_timing(
        &mut report_value,
        "compileLaunchSerializeMs",
        compile_breakdown.launch_serialize_ms,
    );
    set_optional_report_timing(
        &mut report_value,
        "compileFollowUpSerializeMs",
        compile_breakdown.follow_up_serialize_ms,
    );
    set_optional_report_timing(
        &mut report_value,
        "compileTipSerializeMs",
        compile_breakdown.tip_serialize_ms,
    );

    if action == "simulate" {
        let simulate_started_ms = current_time_ms();
        let (simulation, warnings) = simulate_transactions(
            &configured_rpc_url(),
            &compiled_transactions,
            &normalized.execution.commitment,
        )
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                    "traceId": trace.traceId,
                })),
            )
        })?;
        let mut report = report_value;
        set_report_benchmark_mode(&mut report, benchmark_mode);
        set_optional_report_timing(&mut report, "prepareRequestPayloadMs", prepare_request_payload_ms);
        if let Some(value) = form_to_raw_config_ms {
            set_report_timing(&mut report, "formToRawConfigMs", value);
        }
        set_report_timing(&mut report, "normalizeConfigMs", normalize_config_ms);
        set_report_timing(&mut report, "walletLoadMs", wallet_load_ms);
        set_report_timing(&mut report, "transportPlanBuildMs", transport_plan_build_ms);
        set_report_timing(&mut report, "autoFeeResolveMs", auto_fee_resolve_ms);
        set_report_timing(&mut report, "sameTimeFeeGuardMs", same_time_fee_guard_ms);
        set_report_timing(
            &mut report,
            "compileTransactionsMs",
            compile_transactions_ms,
        );
        set_report_timing(
            &mut report,
            "compileLaunchCreatorPrepMs",
            compile_breakdown.launch_creator_prep_ms,
        );
        set_report_timing(
            &mut report,
            "compileAltLoadMs",
            compile_breakdown.alt_load_ms,
        );
        set_report_timing(
            &mut report,
            "compileBlockhashFetchMs",
            compile_breakdown.blockhash_fetch_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileGlobalFetchMs",
            compile_breakdown.global_fetch_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileFollowUpPrepMs",
            compile_breakdown.follow_up_prep_ms,
        );
        set_report_timing(
            &mut report,
            "compileTxSerializeMs",
            compile_breakdown.tx_serialize_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileLaunchSerializeMs",
            compile_breakdown.launch_serialize_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileFollowUpSerializeMs",
            compile_breakdown.follow_up_serialize_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileTipSerializeMs",
            compile_breakdown.tip_serialize_ms,
        );
        set_report_timing(
            &mut report,
            "simulateMs",
            current_time_ms().saturating_sub(simulate_started_ms),
        );
        if let Some(execution) = report.get_mut("execution") {
            execution["simulation"] =
                serde_json::to_value(simulation).unwrap_or(Value::Array(vec![]));
            let mut existing_warnings = execution
                .get("warnings")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            existing_warnings.extend(warnings.into_iter().map(Value::String));
            execution["warnings"] = Value::Array(existing_warnings);
        }
        let send_log_path = if should_persist_report {
            let persist_started = Instant::now();
            let path = persist_launch_report(&trace.traceId, &action, &transport_plan, &report)
                .map_err(|error| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "ok": false,
                            "error": error,
                            "traceId": trace.traceId,
                        })),
                    )
                })?;
            set_report_timing(
                &mut report,
                "persistReportMs",
                persist_started.elapsed().as_millis(),
            );
            set_report_timing(
                &mut report,
                "persistInitialSnapshotMs",
                persist_started.elapsed().as_millis(),
            );
            report["outPath"] = Value::String(path.clone());
            Some(path)
        } else {
            None
        };
        let backend_elapsed_ms = action_started.elapsed().as_millis();
        set_report_timing(&mut report, "executionTotalMs", backend_elapsed_ms);
        set_report_timing(&mut report, "backendTotalElapsedMs", backend_elapsed_ms);
        set_report_timing(&mut report, "totalElapsedMs", backend_elapsed_ms);
        refresh_report_benchmark(&mut report);
        if let Some(path) = send_log_path.as_ref() {
            let finalize_started = Instant::now();
            update_persisted_launch_report(
                path,
                &trace.traceId,
                &action,
                &transport_plan,
                &report,
            )
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": format!("Action completed but failed to finalize persisted report: {error}"),
                        "traceId": trace.traceId,
                    })),
                )
            })?;
            set_report_timing(
                &mut report,
                "persistFinalReportUpdateMs",
                finalize_started.elapsed().as_millis(),
            );
        }
        log_event(
            "engine_action_completed",
            &trace.traceId,
            json!({
                "action": action,
                "executor": "rust-rpc",
                "assemblyExecutor": assembly_executor,
            }),
        );
        return Ok(json!({
            "ok": true,
            "service": "launchdeck-engine",
            "action": action,
            "implemented": true,
            "executionImplemented": true,
            "executor": "rust-rpc",
            "message": "Rust engine validated the request, compiled transactions natively, and simulated them through native Rust RPC.",
            "traceId": trace.traceId,
            "elapsedMs": current_time_ms().saturating_sub(trace.startedAtMs),
            "receivedForm": payload.form.is_some(),
            "receivedRawConfig": true,
            "normalizedConfig": redacted_normalized_config(&normalized),
            "transportPlan": transport_plan,
            "assemblyExecutor": assembly_executor,
            "report": report,
            "sendLogPath": send_log_path,
            "text": render_report_value(&report),
            "metadataUri": prepared_metadata_uri,
        }));
    }

    if action == "send" {
        let execution_class = transport_plan.executionClass.clone();
        let use_phased_follow_pipeline = matches!(normalized.launchpad.as_str(), "pump" | "bonk");
        let (same_time_snipes, deferred_follow_launch) = if use_phased_follow_pipeline {
            (vec![], normalized.followLaunch.clone())
        } else {
            split_same_time_snipes(&normalized.followLaunch)
        };
        let same_time_retry_enabled = same_time_snipes.iter().any(|snipe| snipe.retryOnFailure);
        let follow_daemon_transport = configured_follow_daemon_transport().map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                    "traceId": trace.traceId,
                })),
            )
        })?;
        let follow_daemon_client = if deferred_follow_launch.enabled {
            Some(FollowDaemonClient::new(&configured_follow_daemon_base_url()))
        } else {
            None
        };
        let mut reserved_follow_job: Option<FollowJobResponse> = None;
        let mut armed_follow_job: Option<FollowJobResponse> = None;
        let mut same_time_sniper_compile_ms = 0u128;
        let mut same_time_independent_compiled: Vec<CompiledTransaction> = Vec::new();
        let mut same_time_sent: Vec<crate::rpc::SentResult> = Vec::new();
        let mut post_send_warnings: Vec<String> = Vec::new();
        let mut send_phase_errors: Vec<String> = Vec::new();
        let mut launch_confirmed = false;
        let mut same_time_failure_count = 0usize;
        let mut report_persisted = !should_persist_report;
        let mut report_finalized = !should_persist_report;
        let bags_same_time_compile_after_launch =
            normalized.launchpad == "bagsapp" && !same_time_snipes.is_empty();
        let rpc_url = configured_rpc_url();
        let mut bags_setup_sent: Vec<crate::rpc::SentResult> = Vec::new();
        let mut bags_setup_warnings: Vec<String> = Vec::new();
        let mut bags_setup_submit_ms = 0u128;
        let mut bags_setup_confirm_ms = 0u128;
        let mut follow_reserve_ms = 0u128;
        let mut follow_arm_ms = 0u128;
        let prebuilt_follow_actions = if use_phased_follow_pipeline && deferred_follow_launch.enabled
        {
            Some(
                compile_presigned_follow_actions(
                    &rpc_url,
                    &normalized,
                    &compiled_mint,
                    &compiled_launch_creator,
                    true,
                )
                .await
                .map_err(|error| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "ok": false,
                            "error": format!("Pre-signing follow actions failed: {error}"),
                            "traceId": trace.traceId,
                        })),
                    )
                })?,
            )
        } else {
            None
        };
        let mut reserve_follow_job_task = if let Some(client) = follow_daemon_client.as_ref() {
            let client = client.clone();
            let trace_id = trace.traceId.clone();
            let launchpad = normalized.launchpad.clone();
            let quote_asset = normalized.quoteAsset.clone();
            let launch_mode = normalized.mode.clone();
            let selected_wallet_key = normalized.selectedWalletKey.clone();
            let follow_launch = deferred_follow_launch.clone();
            let execution = normalized.execution.clone();
            let token_mayhem_mode = normalized.token.mayhemMode;
            let jito_tip_account = normalized.tx.jitoTipAccount.clone();
            let prefer_post_setup_creator_vault_for_sell =
                matches!(normalized.mode.as_str(), "agent-custom" | "agent-locked");
            let prebuilt_actions = prebuilt_follow_actions.clone().unwrap_or_default();
            let deferred_setup_transactions = deferred_setup_transactions.clone();
            Some(tokio::spawn(async move {
                let follow_reserve_started = Instant::now();
                let reserved = client
                    .reserve(&FollowReserveRequest {
                        traceId: trace_id,
                        launchpad,
                        quoteAsset: quote_asset,
                        launchMode: launch_mode,
                        selectedWalletKey: selected_wallet_key,
                        followLaunch: follow_launch,
                        execution,
                        tokenMayhemMode: token_mayhem_mode,
                        jitoTipAccount: jito_tip_account,
                        preferPostSetupCreatorVaultForSell: prefer_post_setup_creator_vault_for_sell,
                        prebuiltActions: prebuilt_actions,
                        deferredSetupTransactions: deferred_setup_transactions,
                    })
                    .await?;
                Ok::<_, String>((reserved, follow_reserve_started.elapsed().as_millis()))
            }))
        } else {
            None
        };
        if let Some(prepared) = prepared_bags_send.as_ref() {
            for bundle in &prepared.setup_bundles {
                let (mut bundle_sent, bundle_warnings, bundle_timing) = send_transactions_bundle(
                    &rpc_url,
                    &transport_plan.jitoBundleEndpoints,
                    bundle,
                    &normalized.execution.commitment,
                    normalized.execution.trackSendBlockHeight,
                )
                .await
                .map_err(|error| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "ok": false,
                            "error": format!("Bags setup bundle send failed: {error}"),
                            "traceId": trace.traceId,
                        })),
                    )
                })?;
                bags_setup_submit_ms += bundle_timing.submit_ms;
                bags_setup_confirm_ms += bundle_timing.confirm_ms;
                bags_setup_warnings.extend(bundle_warnings);
                bags_setup_sent.append(&mut bundle_sent);
            }
            if !prepared.setup_transactions.is_empty() {
                for setup_transaction in &prepared.setup_transactions {
                    let setup_batch = vec![setup_transaction.clone()];
                    let (mut setup_sent, setup_warnings, setup_submit_ms) =
                        submit_transactions_sequential(
                            &rpc_url,
                            &setup_batch,
                            &normalized.execution.commitment,
                            normalized.execution.skipPreflight,
                            normalized.execution.trackSendBlockHeight,
                        )
                        .await
                        .map_err(|error| {
                            (
                                StatusCode::BAD_REQUEST,
                                Json(json!({
                                    "ok": false,
                                    "error": format!("Bags setup transaction send failed for {}: {error}", setup_transaction.label),
                                    "traceId": trace.traceId,
                                })),
                            )
                        })?;
                    let (setup_confirm_warnings, setup_confirm_ms) =
                        confirm_transactions_with_websocket_fallback(
                            &rpc_url,
                            transport_plan
                                .watchEndpoint
                                .as_deref()
                                .or_else(|| transport_plan.watchEndpoints.first().map(String::as_str)),
                            &mut setup_sent,
                            &normalized.execution.commitment,
                            normalized.execution.trackSendBlockHeight,
                            225,
                            400,
                        )
                        .await
                        .map_err(|error| {
                            (
                                StatusCode::BAD_REQUEST,
                                Json(json!({
                                    "ok": false,
                                    "error": format!(
                                        "Bags setup transaction confirmation failed for {}: {error}",
                                        setup_transaction.label
                                    ),
                                    "traceId": trace.traceId,
                                })),
                            )
                        })?;
                    bags_setup_submit_ms += setup_submit_ms;
                    bags_setup_confirm_ms += setup_confirm_ms;
                    bags_setup_warnings.extend(setup_warnings);
                    bags_setup_warnings.extend(setup_confirm_warnings);
                    bags_setup_sent.append(&mut setup_sent);
                }
            }
            let launch_compiled = compile_bags_launch_transaction(
                &rpc_url,
                &normalized,
                &wallet_secret,
                &compiled_mint,
                &prepared.config_key,
                &prepared.metadata_uri,
            )
            .await
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": format!("Bags launch transaction build failed after setup send: {error}"),
                        "traceId": trace.traceId,
                    })),
                )
            })?;
            compiled_transactions = vec![launch_compiled];
            if let Some(transactions) = report_value
                .get_mut("transactions")
                .and_then(Value::as_array_mut)
            {
                let mut launch_summaries = serde_json::to_value(summarize_bags_transactions(
                    &compiled_transactions,
                    normalized.tx.dumpBase64,
                ))
                .unwrap_or_else(|_| Value::Array(vec![]));
                if let Some(items) = launch_summaries.as_array_mut() {
                    transactions.append(items);
                }
            }
        }
        if !same_time_snipes.is_empty() && !bags_same_time_compile_after_launch {
            let same_time_compile_started_ms = current_time_ms();
            let same_time_compiled = compile_same_time_snipes(
                &rpc_url,
                &normalized,
                &compiled_mint,
                &compiled_launch_creator,
                &same_time_snipes,
                action == "send",
            )
            .await
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": format!("Same-time sniper compile failed: {error}"),
                        "traceId": trace.traceId,
                    })),
                )
            })?;
            same_time_sniper_compile_ms =
                current_time_ms().saturating_sub(same_time_compile_started_ms);
            if transport_plan.transportType == "jito-bundle" {
                compiled_transactions.extend(same_time_compiled);
            } else {
                same_time_independent_compiled = same_time_compiled;
            }
        }
        if use_phased_follow_pipeline {
            compiled_transactions = creation_transactions.clone();
        }
        let submit_started_ms = current_time_ms();
        let (mut launch_sent, mut warnings, submit_ms) = if same_time_independent_compiled
            .is_empty()
        {
            submit_transactions_for_transport(
                &rpc_url,
                &transport_plan,
                &compiled_transactions,
                &normalized.execution.commitment,
                normalized.execution.skipPreflight,
                normalized.execution.trackSendBlockHeight,
            )
            .await
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": error,
                        "traceId": trace.traceId,
                    })),
                )
            })?
        } else if normalized.launchpad == "bonk" {
            let (launch_sent, mut launch_warnings, _launch_submit_ms) =
                submit_transactions_for_transport(
                    &rpc_url,
                    &transport_plan,
                    &compiled_transactions,
                    &normalized.execution.commitment,
                    normalized.execution.skipPreflight,
                    normalized.execution.trackSendBlockHeight,
                )
                .await
                .map_err(|error| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "ok": false,
                            "error": error,
                            "traceId": trace.traceId,
                        })),
                    )
                })?;
            launch_warnings.push(
                "Bonk same-time sniper buys are submitted immediately after the launch transaction on non-bundle transports so the mint exists before buy execution."
                    .to_string(),
            );
            match submit_independent_transactions_for_transport(
                &rpc_url,
                &transport_plan,
                &same_time_independent_compiled,
                &normalized.execution.commitment,
                normalized.execution.skipPreflight,
                normalized.execution.trackSendBlockHeight,
            )
            .await
            {
                Ok((sent, same_time_warnings, _same_time_submit_ms)) => {
                    same_time_sent = sent;
                    launch_warnings.extend(same_time_warnings);
                }
                Err(error) if same_time_retry_enabled => {
                    launch_warnings.push(format!(
                        "Same-time sniper submit failed after Bonk launch submit; daemon retry is armed for one retry attempt: {error}"
                    ));
                }
                Err(error) => {
                    same_time_failure_count = same_time_failure_count.saturating_add(1);
                    launch_warnings.push(format!(
                        "Same-time sniper submit failed after Bonk launch submit: {error}"
                    ));
                }
            }
            (
                launch_sent,
                launch_warnings,
                current_time_ms().saturating_sub(submit_started_ms),
            )
        } else {
            let launch_submit = submit_transactions_for_transport(
                &rpc_url,
                &transport_plan,
                &compiled_transactions,
                &normalized.execution.commitment,
                normalized.execution.skipPreflight,
                normalized.execution.trackSendBlockHeight,
            );
            let same_time_submit = submit_independent_transactions_for_transport(
                &rpc_url,
                &transport_plan,
                &same_time_independent_compiled,
                &normalized.execution.commitment,
                normalized.execution.skipPreflight,
                normalized.execution.trackSendBlockHeight,
            );
            let (launch_result, same_time_result) = tokio::join!(launch_submit, same_time_submit);
            let (launch_sent, mut launch_warnings, _launch_submit_ms) =
                launch_result.map_err(|error| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "ok": false,
                            "error": error,
                            "traceId": trace.traceId,
                        })),
                    )
                })?;
            match same_time_result {
                Ok((sent, same_time_warnings, _same_time_submit_ms)) => {
                    same_time_sent = sent;
                    launch_warnings.extend(same_time_warnings);
                }
                Err(error) if same_time_retry_enabled => {
                    launch_warnings.push(format!(
                        "Same-time sniper submit failed; daemon retry is armed for one retry attempt: {error}"
                    ));
                }
                Err(error) => {
                    same_time_failure_count = same_time_failure_count.saturating_add(1);
                    launch_warnings.push(format!(
                        "Same-time sniper submit failed after launch submit: {error}"
                    ));
                }
            }
            (
                launch_sent,
                launch_warnings,
                current_time_ms().saturating_sub(submit_started_ms),
            )
        };
        if !bags_setup_warnings.is_empty() {
            let launch_warnings = std::mem::take(&mut warnings);
            warnings = bags_setup_warnings;
            warnings.extend(launch_warnings);
        }
        let launch_signature = launch_sent
            .first()
            .and_then(|result| result.signature.clone())
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": "Launch submit completed without a signature, so follow actions cannot be armed safely.",
                        "traceId": trace.traceId,
                    })),
                )
            })?;
        if bags_same_time_compile_after_launch {
            let same_time_compile_started_ms = current_time_ms();
            match compile_same_time_snipes(
                &rpc_url,
                &normalized,
                &compiled_mint,
                &compiled_launch_creator,
                &same_time_snipes,
                true,
            )
            .await
            {
                Ok(same_time_compiled) => {
                    same_time_sniper_compile_ms = same_time_sniper_compile_ms.saturating_add(
                        current_time_ms().saturating_sub(same_time_compile_started_ms),
                    );
                    match submit_independent_transactions_for_transport(
                        &rpc_url,
                        &transport_plan,
                        &same_time_compiled,
                        &normalized.execution.commitment,
                        normalized.execution.skipPreflight,
                        normalized.execution.trackSendBlockHeight,
                    )
                    .await
                    {
                        Ok((sent, same_time_warnings, _same_time_submit_ms)) => {
                            same_time_sent = sent;
                            if let Some(execution) = report_value.get_mut("execution") {
                                let mut existing_warnings = execution
                                    .get("warnings")
                                    .and_then(Value::as_array)
                                    .cloned()
                                    .unwrap_or_default();
                                existing_warnings.push(Value::String(
                                    "Bags same-time snipes are compiled immediately after the launch transaction is submitted so the trade route can resolve against the live mint."
                                        .to_string(),
                                ));
                                existing_warnings
                                    .extend(same_time_warnings.into_iter().map(Value::String));
                                execution["warnings"] = Value::Array(existing_warnings);
                            }
                        }
                        Err(error) if same_time_retry_enabled => {
                            append_execution_warning(
                                &mut report_value,
                                &format!(
                                    "Same-time Bags sniper submit failed after launch submit; daemon retry is armed for one retry attempt: {error}"
                                ),
                            );
                        }
                        Err(error) => {
                            same_time_failure_count = same_time_failure_count.saturating_add(1);
                            append_execution_warning(
                                &mut report_value,
                                &format!(
                                    "Same-time Bags sniper submit failed after launch submit: {error}"
                                ),
                            );
                        }
                    }
                }
                Err(error) => {
                    same_time_failure_count = same_time_failure_count.saturating_add(1);
                    append_execution_warning(
                        &mut report_value,
                        &format!(
                            "Same-time Bags sniper compile failed after launch submit: {error}"
                        ),
                    );
                }
            }
        }
        let mut report = report_value;
        set_report_benchmark_mode(&mut report, benchmark_mode);
        set_optional_report_timing(&mut report, "prepareRequestPayloadMs", prepare_request_payload_ms);
        if let Some(value) = form_to_raw_config_ms {
            set_report_timing(&mut report, "formToRawConfigMs", value);
        }
        set_report_timing(&mut report, "normalizeConfigMs", normalize_config_ms);
        set_report_timing(&mut report, "walletLoadMs", wallet_load_ms);
        set_report_timing(&mut report, "transportPlanBuildMs", transport_plan_build_ms);
        set_report_timing(&mut report, "autoFeeResolveMs", auto_fee_resolve_ms);
        set_report_timing(&mut report, "sameTimeFeeGuardMs", same_time_fee_guard_ms);
        set_report_timing(
            &mut report,
            "compileTransactionsMs",
            compile_transactions_ms.saturating_add(same_time_sniper_compile_ms),
        );
        set_report_timing(
            &mut report,
            "compileLaunchCreatorPrepMs",
            compile_breakdown.launch_creator_prep_ms,
        );
        set_report_timing(
            &mut report,
            "compileAltLoadMs",
            compile_breakdown.alt_load_ms,
        );
        set_report_timing(
            &mut report,
            "compileBlockhashFetchMs",
            compile_breakdown.blockhash_fetch_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileGlobalFetchMs",
            compile_breakdown.global_fetch_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileFollowUpPrepMs",
            compile_breakdown.follow_up_prep_ms,
        );
        set_report_timing(
            &mut report,
            "compileTxSerializeMs",
            compile_breakdown.tx_serialize_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileLaunchSerializeMs",
            compile_breakdown.launch_serialize_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileFollowUpSerializeMs",
            compile_breakdown.follow_up_serialize_ms,
        );
        set_optional_report_timing(
            &mut report,
            "compileTipSerializeMs",
            compile_breakdown.tip_serialize_ms,
        );
        set_report_timing(
            &mut report,
            "sendMs",
            bags_setup_submit_ms
                .saturating_add(bags_setup_confirm_ms)
                .saturating_add(submit_ms),
        );
        set_report_timing(
            &mut report,
            "sendSubmitMs",
            bags_setup_submit_ms.saturating_add(submit_ms),
        );
        set_report_timing(&mut report, "sendTransportSubmitMs", submit_ms);
        set_report_timing(&mut report, "sendConfirmMs", bags_setup_confirm_ms);
        set_report_timing(&mut report, "sendTransportConfirmMs", 0);
        if let Some(task) = reserve_follow_job_task.take() {
            let (reserved, elapsed_ms) = task.await.map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": format!("Follow daemon reservation task failed: {error}"),
                        "traceId": trace.traceId,
                    })),
                )
            })?.map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": format!("Follow daemon reservation failed: {error}"),
                        "traceId": trace.traceId,
                    })),
                )
            })?;
            follow_reserve_ms = elapsed_ms;
            reserved_follow_job = Some(reserved);
        }
        set_optional_report_timing(&mut report, "followDaemonReserveMs", Some(follow_reserve_ms));
        if bags_setup_submit_ms > 0 || bags_setup_confirm_ms > 0 {
            set_report_timing(&mut report, "bagsSetupSubmitMs", bags_setup_submit_ms);
            set_report_timing(&mut report, "bagsSetupConfirmMs", bags_setup_confirm_ms);
        }
        attach_follow_daemon_report(
            &mut report,
            if deferred_follow_launch.enabled {
                Some(follow_daemon_transport.as_str())
            } else {
                None
            },
            reserved_follow_job.as_ref(),
            None,
            None,
            Some(&normalized.followLaunch),
        );
        let send_log_path = if should_persist_report {
            let persist_started = Instant::now();
            match persist_launch_report(&trace.traceId, &action, &transport_plan, &report) {
                Ok(path) => {
                    report_persisted = true;
                    set_report_timing(
                        &mut report,
                        "persistReportMs",
                        persist_started.elapsed().as_millis(),
                    );
                    set_report_timing(
                        &mut report,
                        "persistInitialSnapshotMs",
                        persist_started.elapsed().as_millis(),
                    );
                    report["outPath"] = Value::String(path.clone());
                    Some(path)
                }
                Err(error) => {
                    let warning = format!(
                        "Launch was submitted, but the initial report could not be persisted: {error}"
                    );
                    post_send_warnings.push(warning.clone());
                    send_phase_errors.push(warning);
                    None
                }
            }
        } else {
            None
        };
        let (mut confirm_warnings, mut confirm_ms) =
            match confirm_submitted_transactions_for_transport(
                &rpc_url,
                &transport_plan,
                &mut launch_sent,
                &normalized.execution.commitment,
                normalized.execution.trackSendBlockHeight,
            )
            .await
            {
                Ok(value) => {
                    launch_confirmed = true;
                    value
                }
                Err(error) => {
                    if let Some(client) = follow_daemon_client.as_ref() {
                        let _ = client
                            .cancel(&FollowCancelRequest {
                                traceId: trace.traceId.clone(),
                                actionId: None,
                                note: Some(format!("Launch confirmation failed: {error}")),
                            })
                            .await;
                    }
                    let warning = format!(
                        "Launch was submitted, but confirmation failed or remained incomplete: {error}"
                    );
                    post_send_warnings.push(warning.clone());
                    send_phase_errors.push(warning.clone());
                    (vec![warning], 0)
                }
            };
        if launch_confirmed {
            if let Some(client) = follow_daemon_client.as_ref() {
                let mut confirmed_follow_block_height = launch_sent
                    .first()
                    .and_then(|result| result.confirmedObservedBlockHeight);
                if confirmed_follow_block_height.is_none() {
                    confirmed_follow_block_height =
                        fetch_current_block_height(&rpc_url, "confirmed").await.ok();
                }
                let follow_arm_started = Instant::now();
                match client
                    .arm(&FollowArmRequest {
                        traceId: trace.traceId.clone(),
                        mint: compiled_mint.clone(),
                        launchCreator: compiled_launch_creator.clone(),
                        launchSignature: launch_signature.clone(),
                        submitAtMs: current_time_ms(),
                        sendObservedBlockHeight: launch_sent
                            .first()
                            .and_then(|result| result.sendObservedBlockHeight),
                        confirmedObservedBlockHeight: confirmed_follow_block_height,
                        reportPath: send_log_path.clone(),
                        transportPlan: transport_plan.clone(),
                    })
                    .await
                {
                    Ok(response) => {
                        follow_arm_ms = follow_arm_started.elapsed().as_millis();
                        armed_follow_job = Some(response);
                    }
                    Err(error) => {
                        follow_arm_ms = follow_arm_started.elapsed().as_millis();
                        let warning =
                            format!("Launch confirmed, but follow daemon arm failed: {error}");
                        post_send_warnings.push(warning.clone());
                        send_phase_errors.push(warning);
                    }
                }
            }
        }
        if let Some(_client) = follow_daemon_client.as_ref() {
            attach_follow_daemon_report(
                &mut report,
                Some(follow_daemon_transport.as_str()),
                reserved_follow_job.as_ref(),
                armed_follow_job.as_ref(),
                None,
                Some(&normalized.followLaunch),
            );
        }
        if !same_time_sent.is_empty() {
            match confirm_submitted_transactions_for_transport(
                &rpc_url,
                &transport_plan,
                &mut same_time_sent,
                &normalized.execution.commitment,
                normalized.execution.trackSendBlockHeight,
            )
            .await
            {
                Ok((same_time_confirm_warnings, same_time_confirm_ms)) => {
                    confirm_warnings.extend(same_time_confirm_warnings);
                    confirm_ms = confirm_ms.saturating_add(same_time_confirm_ms);
                }
                Err(error) if same_time_retry_enabled => {
                    confirm_warnings.push(format!(
                        "Same-time sniper confirmation failed; daemon retry will attempt one fallback buy: {error}"
                    ));
                }
                Err(error) => {
                    if let Some(client) = follow_daemon_client.as_ref() {
                        let _ = client
                            .cancel(&FollowCancelRequest {
                                traceId: trace.traceId.clone(),
                                actionId: None,
                                note: Some(format!(
                                    "Same-time sniper confirmation failed: {error}"
                                )),
                            })
                            .await;
                    }
                    same_time_failure_count = same_time_failure_count.saturating_add(1);
                    confirm_warnings.push(format!(
                        "Same-time sniper confirmation failed or remained incomplete: {error}"
                    ));
                }
            }
        }
        let mut sent = bags_setup_sent;
        sent.append(&mut launch_sent);
        sent.append(&mut same_time_sent);
        warnings.extend(confirm_warnings);
        let response_warning_count = warnings.len().saturating_add(post_send_warnings.len());
        set_report_timing(
            &mut report,
            "sendMs",
            bags_setup_submit_ms
                .saturating_add(bags_setup_confirm_ms)
                .saturating_add(submit_ms)
                .saturating_add(confirm_ms),
        );
        set_report_timing(
            &mut report,
            "sendSubmitMs",
            bags_setup_submit_ms.saturating_add(submit_ms),
        );
        set_report_timing(&mut report, "sendTransportSubmitMs", submit_ms);
        set_report_timing(
            &mut report,
            "sendConfirmMs",
            bags_setup_confirm_ms.saturating_add(confirm_ms),
        );
        set_report_timing(&mut report, "sendTransportConfirmMs", confirm_ms);
        set_report_timing(&mut report, "sendCreationSubmitMs", submit_ms);
        set_report_timing(&mut report, "sendCreationConfirmMs", confirm_ms);
        if use_phased_follow_pipeline {
            append_execution_note(
                &mut report,
                "Pump/Bonk phased send path reserved pre-signed follow actions in the daemon and moved setup into deferred post-confirm execution.",
            );
            if !deferred_setup_transactions.is_empty() {
                append_execution_note(
                    &mut report,
                    &format!(
                        "Deferred post-confirm setup prepared {} transaction(s) for daemon-managed submission.",
                        deferred_setup_transactions.len()
                    ),
                );
            }
        }
        set_optional_report_timing(&mut report, "followDaemonReserveMs", Some(follow_reserve_ms));
        set_optional_report_timing(&mut report, "followDaemonArmMs", Some(follow_arm_ms));
        if let Some(execution) = report.get_mut("execution") {
            execution["sent"] = serde_json::to_value(&sent).unwrap_or(Value::Array(vec![]));
            if let Some(actual_responder) = sent
                .iter()
                .find_map(|entry| entry.endpoint.as_ref().filter(|value| !value.is_empty()).cloned())
            {
                execution["heliusSenderEndpoint"] = Value::String(actual_responder);
            }
            let mut existing_warnings = execution
                .get("warnings")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            existing_warnings.extend(warnings.into_iter().map(Value::String));
            existing_warnings.extend(post_send_warnings.iter().cloned().map(Value::String));
            execution["warnings"] = Value::Array(existing_warnings);
        }
        let backend_elapsed_ms = action_started.elapsed().as_millis();
        set_report_timing(&mut report, "executionTotalMs", backend_elapsed_ms);
        set_report_timing(&mut report, "backendTotalElapsedMs", backend_elapsed_ms);
        set_report_timing(&mut report, "totalElapsedMs", backend_elapsed_ms);
        attach_follow_daemon_report(
            &mut report,
            if deferred_follow_launch.enabled {
                Some(follow_daemon_transport.as_str())
            } else {
                None
            },
            reserved_follow_job.as_ref(),
            armed_follow_job.as_ref(),
            None,
            Some(&normalized.followLaunch),
        );
        refresh_report_benchmark(&mut report);
        if let Some(path) = send_log_path.as_ref() {
            let finalize_started = Instant::now();
            match update_persisted_launch_report(
                path,
                &trace.traceId,
                &action,
                &transport_plan,
                &report,
            ) {
                Ok(()) => {
                    report_finalized = true;
                    set_report_timing(
                        &mut report,
                        "persistFinalReportUpdateMs",
                        finalize_started.elapsed().as_millis(),
                    );
                }
                Err(error) => {
                    let warning = format!(
                        "Launch completed, but the persisted report could not be finalized: {error}"
                    );
                    post_send_warnings.push(warning.clone());
                    send_phase_errors.push(warning);
                }
            }
        }
        if let Some(execution) = report.get_mut("execution") {
            let mut existing_warnings = execution
                .get("warnings")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for warning in &post_send_warnings {
                let value = Value::String(warning.clone());
                if !existing_warnings.contains(&value) {
                    existing_warnings.push(value);
                }
            }
            execution["warnings"] = Value::Array(existing_warnings);
        }
        log_event(
            "engine_action_completed",
            &trace.traceId,
            json!({
                "action": action,
                "executor": "rust-rpc",
                "assemblyExecutor": assembly_executor,
                "executionClass": execution_class,
            }),
        );
        return Ok(json!({
            "ok": true,
            "service": "launchdeck-engine",
            "action": action,
            "implemented": true,
            "executionImplemented": true,
            "executor": "rust-rpc",
            "message": "Rust engine validated the request, compiled transactions natively, and sent them through native Rust RPC/Jito transport.",
            "traceId": trace.traceId,
            "elapsedMs": current_time_ms().saturating_sub(trace.startedAtMs),
            "receivedForm": payload.form.is_some(),
            "receivedRawConfig": true,
            "normalizedConfig": redacted_normalized_config(&normalized),
            "transportPlan": transport_plan,
            "assemblyExecutor": assembly_executor,
            "report": report,
            "sendLogPath": send_log_path,
            "plainSummary": build_send_plain_summary(
                launch_confirmed,
                same_time_failure_count,
                report_persisted,
                report_finalized,
                reserved_follow_job.is_some(),
                armed_follow_job.is_some(),
                response_warning_count,
            ),
            "sendOutcome": {
                "launchSubmitted": true,
                "launchSignature": launch_signature,
                "launchConfirmed": launch_confirmed,
                "sameTimeFailureCount": same_time_failure_count,
                "reportPersisted": report_persisted,
                "reportFinalized": report_finalized,
                "followDaemonReserved": reserved_follow_job.is_some(),
                "followDaemonArmed": armed_follow_job.is_some(),
                "warnings": post_send_warnings,
                "errors": send_phase_errors,
            },
            "followDaemonTransport": if deferred_follow_launch.enabled {
                Some(follow_daemon_transport)
            } else {
                None::<String>
            },
            "followDaemonReserved": reserved_follow_job,
            "followDaemonArmed": armed_follow_job,
            "text": render_report_value(&report),
            "metadataUri": prepared_metadata_uri,
        }));
    }

    log_event(
        "engine_action_completed",
        &trace.traceId,
        json!({
            "action": action,
            "executor": assembly_executor,
        }),
    );
    Ok(json!({
        "ok": true,
        "service": "launchdeck-engine",
        "action": action,
        "implemented": true,
        "executionImplemented": true,
        "executor": assembly_executor,
        "message": "Rust engine validated the request and compiled the supported execution path natively.",
        "traceId": trace.traceId,
        "elapsedMs": current_time_ms().saturating_sub(trace.startedAtMs),
        "receivedForm": payload.form.is_some(),
        "receivedRawConfig": true,
        "normalizedConfig": redacted_normalized_config(&normalized),
        "transportPlan": transport_plan,
        "assemblyExecutor": assembly_executor,
        "report": report_value,
        "text": text_value,
        "metadataUri": prepared_metadata_uri,
    }))
}

async fn engine_action(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<EngineRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    execute_engine_action_payload(&state, payload)
        .await
        .map(Json)
}

fn build_settings_payload() -> Value {
    json!({
        "ok": true,
        "config": read_persistent_config(),
        "defaults": create_default_persistent_config(),
        "regionRouting": build_region_routing_payload(),
        "strategies": strategy_registry(),
        "engine": {
            "backend": "rust",
            "url": configured_base_url(),
        },
    })
}

fn bags_api_base_url() -> String {
    std::env::var("BAGS_API_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://public-api-v2.bags.fm/api/v1".to_string())
}

fn read_bags_credentials_file(path: std::path::PathBuf) -> BagsStoredCredentials {
    let raw = fs::read_to_string(path).unwrap_or_default();
    if raw.trim().is_empty() {
        BagsStoredCredentials::default()
    } else {
        serde_json::from_str::<BagsStoredCredentials>(&raw).unwrap_or_default()
    }
}

fn write_bags_credentials_file(
    path: std::path::PathBuf,
    credentials: &BagsStoredCredentials,
) -> Result<(), String> {
    atomic_write(
        &path,
        &serde_json::to_vec_pretty(credentials).map_err(|error| error.to_string())?,
    )
}

fn read_persisted_bags_credentials() -> BagsStoredCredentials {
    read_bags_credentials_file(paths::bags_credentials_path())
}

fn read_session_bags_credentials() -> BagsStoredCredentials {
    read_bags_credentials_file(paths::bags_session_path())
}

fn read_active_bags_credentials() -> BagsStoredCredentials {
    let persisted = read_persisted_bags_credentials();
    let session = read_session_bags_credentials();
    BagsStoredCredentials {
        api_key: if session.api_key.trim().is_empty() {
            persisted.api_key
        } else {
            session.api_key
        },
        auth_token: if session.auth_token.trim().is_empty() {
            persisted.auth_token
        } else {
            session.auth_token
        },
        agent_username: if session.agent_username.trim().is_empty() {
            persisted.agent_username
        } else {
            session.agent_username
        },
        verified_wallet: if session.verified_wallet.trim().is_empty() {
            persisted.verified_wallet
        } else {
            session.verified_wallet
        },
    }
}

fn persist_bags_credentials(
    credentials: BagsStoredCredentials,
    save_api_key: bool,
) -> Result<(), String> {
    let session_path = paths::bags_session_path();
    let persisted_path = paths::bags_credentials_path();
    if save_api_key {
        write_bags_credentials_file(persisted_path, &credentials)?;
        if session_path.exists() {
            let _ = fs::remove_file(session_path);
        }
        return Ok(());
    }
    write_bags_credentials_file(session_path, &credentials)
}

fn clear_bags_session_credentials() {
    let path = paths::bags_session_path();
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn redacted_normalized_config(config: &NormalizedConfig) -> Value {
    let mut value = serde_json::to_value(config).unwrap_or_else(|_| Value::Null);
    if let Some(object) = value.as_object_mut() {
        if let Some(signer) = object.get_mut("signer").and_then(Value::as_object_mut) {
            signer.insert(
                "secretKey".to_string(),
                Value::String("[redacted]".to_string()),
            );
        }
        if let Some(bags) = object.get_mut("bags").and_then(Value::as_object_mut) {
            if bags.contains_key("authToken") {
                bags.insert(
                    "authToken".to_string(),
                    Value::String("[redacted]".to_string()),
                );
            }
        }
        if object.contains_key("vanityPrivateKey") {
            object.insert(
                "vanityPrivateKey".to_string(),
                Value::String("[redacted]".to_string()),
            );
        }
    }
    value
}

fn build_send_plain_summary(
    launch_confirmed: bool,
    same_time_failures: usize,
    report_persisted: bool,
    report_finalized: bool,
    follow_reserved: bool,
    follow_armed: bool,
    warning_count: usize,
) -> String {
    let mut parts = vec![if launch_confirmed {
        "Launch submitted and confirmed.".to_string()
    } else {
        "Launch submitted, but confirmation is incomplete or failed.".to_string()
    }];
    if same_time_failures > 0 {
        parts.push(format!(
            "{same_time_failures} same-time sniper stage(s) failed or remain unconfirmed."
        ));
    }
    if !report_persisted {
        parts.push("Initial report persistence failed.".to_string());
    } else if !report_finalized {
        parts.push("Report persistence completed only partially.".to_string());
    }
    if follow_reserved && !follow_armed {
        parts.push("Follow automation was reserved but not armed.".to_string());
    }
    if warning_count > 0 {
        parts.push(format!("{warning_count} warning(s) were recorded."));
    }
    parts.join(" ")
}

fn resolve_wallet_public_key_for_bags(wallet_env_key: &str) -> Result<String, String> {
    let key = wallet_env_key.trim();
    if key.is_empty() {
        return Ok(String::new());
    }
    let secret = load_solana_wallet_by_env_key(key)?;
    public_key_from_secret(&secret)
}

fn extract_string_field(value: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .unwrap_or_default()
        .trim()
        .to_string()
}

async fn bags_api_request(
    method: reqwest::Method,
    route: &str,
    api_key: &str,
    auth_token: Option<&str>,
    body: Option<Value>,
) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|error| format!("Failed to build Bags HTTP client: {error}"))?;
    let url = format!("{}{}", bags_api_base_url(), route);
    let mut request = client
        .request(method, &url)
        .header("x-api-key", api_key)
        .header("content-type", "application/json");
    if let Some(token) = auth_token.filter(|value| !value.trim().is_empty()) {
        request = request.bearer_auth(token.trim());
    }
    if let Some(payload) = body {
        request = request.json(&payload);
    }
    let response = request
        .send()
        .await
        .map_err(|error| format!("Bags API request failed: {error}"))?;
    let status = response.status();
    let payload: Value = response
        .json()
        .await
        .map_err(|error| format!("Failed to decode Bags API response: {error}"))?;
    if payload
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(status.is_success())
    {
        Ok(payload.get("response").cloned().unwrap_or(payload))
    } else {
        Err(payload
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("Bags API request failed.")
            .to_string())
    }
}

async fn bags_identity_wallet_matches(
    api_key: &str,
    auth_token: &str,
    expected_wallet: &str,
) -> Result<(bool, String), String> {
    if expected_wallet.trim().is_empty() {
        return Ok((false, String::new()));
    }
    let payload = bags_api_request(
        reqwest::Method::GET,
        "/agent/wallet/list",
        api_key,
        Some(auth_token),
        None,
    )
    .await?;
    let wallets = if let Some(array) = payload.as_array() {
        array.clone()
    } else {
        payload
            .get("wallets")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let mut resolved_username = String::new();
    for wallet in wallets {
        if !resolved_username.trim().is_empty() {
            // Keep the first useful username while still checking all wallets.
        } else {
            resolved_username =
                extract_string_field(&wallet, &["providerUsername", "agentUsername", "username"]);
        }
        let wallet_address = extract_string_field(&wallet, &["wallet", "address", "publicKey"]);
        if !wallet_address.is_empty() && wallet_address == expected_wallet.trim() {
            let username =
                extract_string_field(&wallet, &["providerUsername", "agentUsername", "username"]);
            return Ok((
                true,
                if username.is_empty() {
                    resolved_username
                } else {
                    username
                },
            ));
        }
    }
    Ok((false, resolved_username))
}

async fn api_bags_identity_status(
    Query(query): Query<BagsIdentityStatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let active = read_active_bags_credentials();
    let wallet_env_key = query.wallet.unwrap_or_default();
    let wallet = resolve_wallet_public_key_for_bags(&wallet_env_key).unwrap_or_default();
    let mut verified = false;
    let mut resolved_username = active.agent_username.clone();
    if !wallet.is_empty()
        && !active.api_key.trim().is_empty()
        && !active.auth_token.trim().is_empty()
    {
        if let Ok((matched, username)) =
            bags_identity_wallet_matches(&active.api_key, &active.auth_token, &wallet).await
        {
            verified = matched;
            if !username.trim().is_empty() {
                resolved_username = username;
            }
        }
    }
    Ok(Json(attach_timing(
        json!({
            "ok": true,
            "configuredApiKey": !active.api_key.trim().is_empty(),
            "verified": verified,
            "mode": if verified { "linked" } else { "wallet-only" },
            "agentUsername": resolved_username,
            "verifiedWallet": if verified { wallet } else { String::new() },
            "authToken": if verified { active.auth_token } else { String::new() },
        }),
        started_at_ms,
    )))
}

async fn api_bags_identity_init(
    Json(payload): Json<BagsIdentityInitRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let current = read_active_bags_credentials();
    let api_key = if payload.api_key.trim().is_empty() {
        current.api_key
    } else {
        payload.api_key.trim().to_string()
    };
    if api_key.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "Bags API key is required." })),
        ));
    }
    let response = bags_api_request(
        reqwest::Method::POST,
        "/agent/auth/init",
        &api_key,
        None,
        Some(json!({
            "agentUsername": payload.agent_username.trim(),
        })),
    )
    .await
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": error })),
        )
    })?;
    let init_agent_username = {
        let resolved = extract_string_field(&response, &["agentUsername", "providerUsername"]);
        if resolved.trim().is_empty() {
            payload.agent_username.trim().to_string()
        } else {
            resolved
        }
    };
    persist_bags_credentials(
        BagsStoredCredentials {
            api_key: api_key.clone(),
            auth_token: current.auth_token,
            agent_username: init_agent_username.clone(),
            verified_wallet: current.verified_wallet,
        },
        payload.save_api_key,
    )
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": error })),
        )
    })?;
    Ok(Json(attach_timing(
        json!({
            "ok": true,
            "configuredApiKey": true,
            "agentUsername": init_agent_username,
            "publicIdentifier": extract_string_field(&response, &["publicIdentifier"]),
            "secret": extract_string_field(&response, &["secret"]),
            "verificationPostContent": extract_string_field(&response, &["verificationPostContent", "content", "message"]),
        }),
        started_at_ms,
    )))
}

async fn api_bags_identity_verify(
    Json(payload): Json<BagsIdentityVerifyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let current = read_active_bags_credentials();
    let api_key = if payload.api_key.trim().is_empty() {
        current.api_key
    } else {
        payload.api_key.trim().to_string()
    };
    if api_key.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "Bags API key is required." })),
        ));
    }
    let wallet = resolve_wallet_public_key_for_bags(&payload.wallet_env_key).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": error })),
        )
    })?;
    let login_response = bags_api_request(
        reqwest::Method::POST,
        "/agent/auth/login",
        &api_key,
        None,
        Some(json!({
            "agentUsername": payload.agent_username.trim(),
            "publicIdentifier": payload.public_identifier.trim(),
            "secret": payload.secret.trim(),
            "postId": payload.post_id.trim(),
        })),
    )
    .await
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": error })),
        )
    })?;
    let auth_token = extract_string_field(&login_response, &["token", "authToken", "jwt"]);
    if auth_token.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "Bags login did not return an auth token." })),
        ));
    }
    let (verified, resolved_username) =
        bags_identity_wallet_matches(&api_key, &auth_token, &wallet)
            .await
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "ok": false, "error": error })),
                )
            })?;
    if !verified {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "Linked Bags identity requires the selected LaunchDeck wallet to belong to the authenticated Bags account.",
            })),
        ));
    }
    let agent_username = if resolved_username.trim().is_empty() {
        payload.agent_username.trim().to_string()
    } else {
        resolved_username
    };
    persist_bags_credentials(
        BagsStoredCredentials {
            api_key: api_key,
            auth_token: auth_token.clone(),
            agent_username: agent_username.clone(),
            verified_wallet: wallet.clone(),
        },
        payload.save_api_key,
    )
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": error })),
        )
    })?;
    Ok(Json(attach_timing(
        json!({
            "ok": true,
            "configuredApiKey": true,
            "verified": true,
            "agentUsername": agent_username,
            "authToken": auth_token,
            "verifiedWallet": wallet,
        }),
        started_at_ms,
    )))
}

async fn api_bags_identity_clear() -> Json<Value> {
    clear_bags_session_credentials();
    Json(json!({ "ok": true }))
}

async fn api_bootstrap_fast(
    Query(query): Query<StatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let requested_wallet = query.wallet.unwrap_or_default();
    let mut payload = build_bootstrap_fast_payload(&requested_wallet, false).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    payload["backend"] = Value::String("rust".to_string());
    payload["signerSource"] = Value::String(resolve_signer_source(
        payload
            .get("selectedWalletKey")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    ));
    Ok(Json(attach_timing(payload, started_at_ms)))
}

async fn api_bootstrap(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let requested_wallet = query.wallet.unwrap_or_default();
    let mut payload = build_status_payload(&state, &requested_wallet, false)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })?;
    payload["backend"] = Value::String("rust".to_string());
    payload["signerSource"] = Value::String(resolve_signer_source(
        payload
            .get("selectedWalletKey")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    ));
    payload["config"] = read_persistent_config();
    Ok(Json(attach_timing(payload, started_at_ms)))
}

async fn api_lookup_tables_warm() -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rpc_url = configured_rpc_url();
    let loaded = warm_default_lookup_tables(&rpc_url)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })?;
    Ok(Json(json!({
        "ok": true,
        "loaded": loaded,
    })))
}

async fn api_pump_global_warm() -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rpc_url = configured_rpc_url();
    warm_pump_global_state(&rpc_url).await.map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    Ok(Json(json!({
        "ok": true,
    })))
}

async fn api_startup_warm() -> Json<Value> {
    let started_at_ms = current_time_ms();
    if !configured_startup_warm_enabled() {
        return Json(attach_timing(
            json!({
                "ok": true,
                "skipped": true,
                "startupWarm": {
                    "enabled": false,
                    "reason": "disabled-by-env",
                },
            }),
            started_at_ms,
        ));
    }
    let main_rpc_url = configured_rpc_url();
    let rpc_url = configured_warm_rpc_url(&main_rpc_url);
    let helius_sender_endpoints =
        configured_helius_sender_endpoints_for_profile(&default_endpoint_profile_for_provider(
            "helius-sender",
        ));
    let (lookup_tables, pump_global, bonk_state, fee_market, sender_prewarm) = tokio::join!(
        warm_default_lookup_tables(&rpc_url),
        warm_pump_global_state(&rpc_url),
        warm_bonk_state(&rpc_url),
        fetch_fee_market_snapshot(&main_rpc_url),
        async {
            join_all(
                helius_sender_endpoints
                    .iter()
                    .map(|endpoint| async move { (endpoint.clone(), prewarm_rpc_endpoint(endpoint).await) }),
            )
            .await
        },
    );
    Json(attach_timing(
        json!({
            "ok": true,
            "lookupTables": match lookup_tables {
                Ok(loaded) => json!({
                    "ok": true,
                    "loaded": loaded,
                }),
                Err(error) => json!({
                    "ok": false,
                    "error": error,
                }),
            },
            "pumpGlobal": match pump_global {
                Ok(()) => json!({
                    "ok": true,
                }),
                Err(error) => json!({
                    "ok": false,
                    "error": error,
                }),
            },
            "bonkState": match bonk_state {
                Ok(payload) => payload,
                Err(error) => json!({
                    "ok": false,
                    "error": error,
                }),
            },
            "feeMarket": match fee_market {
                Ok(snapshot) => json!({
                    "ok": true,
                    "heliusPriorityLamports": snapshot.helius_priority_lamports,
                    "jitoTipP99Lamports": snapshot.jito_tip_p99_lamports,
                }),
                Err(error) => json!({
                    "ok": false,
                    "error": error,
                }),
            },
            "heliusSender": sender_prewarm
                .into_iter()
                .map(|(endpoint, result)| {
                    json!({
                        "endpoint": endpoint,
                        "ok": result.is_ok(),
                        "error": result.err(),
                    })
                })
                .collect::<Vec<_>>(),
        }),
        started_at_ms,
    ))
}

async fn api_wallet_status(
    Query(query): Query<StatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let wallet = query.wallet.unwrap_or_default();
    let mut payload = build_wallet_status_payload(&wallet, true)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })?;
    payload["backend"] = Value::String("rust".to_string());
    payload["signerSource"] = Value::String(resolve_signer_source(
        payload
            .get("selectedWalletKey")
            .and_then(Value::as_str)
            .unwrap_or(wallet.as_str()),
    ));
    Ok(Json(attach_timing(payload, started_at_ms)))
}

async fn api_runtime_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let started_at_ms = current_time_ms();
    Json(attach_timing(
        build_runtime_status_payload(&state).await,
        started_at_ms,
    ))
}

fn follow_daemon_browser_client() -> Result<FollowDaemonClient, String> {
    configured_follow_daemon_transport().map(|_| ())?;
    Ok(FollowDaemonClient::new(&configured_follow_daemon_base_url()))
}

fn follow_jobs_payload(response: FollowJobResponse, started_at_ms: u128) -> Json<Value> {
    let FollowJobResponse {
        schemaVersion,
        ok,
        job,
        jobs,
        health,
    } = response;
    Json(attach_timing(
        json!({
            "ok": ok,
            "schemaVersion": schemaVersion,
            "job": job,
            "jobs": jobs,
            "health": health,
        }),
        started_at_ms,
    ))
}

async fn api_follow_jobs() -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let client = follow_daemon_browser_client().map_err(|error| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    client
        .list()
        .await
        .map(|response| follow_jobs_payload(response, started_at_ms))
        .map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })
}

async fn api_follow_cancel(
    Json(payload): Json<FollowCancelApiRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let client = follow_daemon_browser_client().map_err(|error| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    client
        .cancel(&FollowCancelRequest {
            traceId: payload.trace_id,
            actionId: payload.action_id,
            note: payload.note,
        })
        .await
        .map(|response| follow_jobs_payload(response, started_at_ms))
        .map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })
}

async fn api_follow_stop_all(
    Json(payload): Json<FollowStopAllApiRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let client = follow_daemon_browser_client().map_err(|error| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    client
        .stop_all(&FollowStopAllRequest { note: payload.note })
        .await
        .map(|response| follow_jobs_payload(response, started_at_ms))
        .map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })
}

async fn api_status(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let wallet = query.wallet.unwrap_or_default();
    let mut payload = build_status_payload(&state, &wallet, true)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })?;
    payload["backend"] = Value::String("rust".to_string());
    payload["signerSource"] = Value::String(resolve_signer_source(
        payload
            .get("selectedWalletKey")
            .and_then(Value::as_str)
            .unwrap_or(wallet.as_str()),
    ));
    payload["config"] = read_persistent_config();
    Ok(Json(attach_timing(payload, started_at_ms)))
}

async fn api_run(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RunRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let client_pre_request_ms = payload.client_pre_request_ms.map(u128::from);
    let action = payload.action.unwrap_or_else(|| "build".to_string());
    if !["build", "simulate", "send"].contains(&action.trim().to_lowercase().as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": format!("Unsupported action: {}", action.trim()),
            })),
        ));
    }
    let selected_wallet_key = payload
        .form
        .as_ref()
        .and_then(|value| value.get("selectedWalletKey"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut response = execute_engine_action_payload(
        &state,
        EngineRequest {
            action: Some(action.clone()),
            form: payload.form,
            raw_config: None,
            prepare_request_payload_ms: payload.prepare_request_payload_ms,
        },
    )
    .await?;
    let persisted_path = response
        .get("sendLogPath")
        .and_then(Value::as_str)
        .map(str::to_string);
    let response_trace_id = response
        .get("traceId")
        .and_then(Value::as_str)
        .map(str::to_string);
    let response_transport_plan = response.get("transportPlan").cloned();
    let mut rendered_text: Option<Value> = None;
    let mut persisted_report_snapshot: Option<Value> = None;
    if let Some(report) = response.get_mut("report") {
        if let Some(pre_request_ms) = client_pre_request_ms {
            let backend_total_ms = report
                .get("execution")
                .and_then(|execution| execution.get("timings"))
                .and_then(|timings| timings.get("totalElapsedMs"))
                .and_then(Value::as_u64)
                .map(u128::from);
            set_optional_report_timing(report, "backendTotalElapsedMs", backend_total_ms);
            set_report_timing(
                report,
                "totalElapsedMs",
                backend_total_ms
                    .unwrap_or_default()
                    .saturating_add(pre_request_ms),
            );
            set_report_timing(report, "clientPreRequestMs", pre_request_ms);
        }
        refresh_report_benchmark(report);
        let render_started = Instant::now();
        rendered_text = Some(render_report_value(report));
        set_report_timing(report, "reportRenderMs", render_started.elapsed().as_millis());
        refresh_report_benchmark(report);
        persisted_report_snapshot = Some(report.clone());
    }
    if let Some(text) = rendered_text {
        response["text"] = text;
    }
    if let (Some(path), Some(trace_id), Some(transport_plan_value), Some(report_snapshot)) = (
        persisted_path.as_deref(),
        response_trace_id.as_deref(),
        response_transport_plan,
        persisted_report_snapshot,
    ) && let Ok(transport_plan) =
        serde_json::from_value::<crate::transport::TransportPlan>(transport_plan_value)
    {
        let _ = update_persisted_launch_report(
            path,
            trace_id,
            &action,
            &transport_plan,
            &report_snapshot,
        );
    }
    response["backend"] = Value::String("rust".to_string());
    response["metadataUri"] = response
        .get("metadataUri")
        .cloned()
        .unwrap_or(Value::String(String::new()));
    response["signerSource"] = Value::String(resolve_signer_source(&selected_wallet_key));
    Ok(Json(response))
}

async fn api_engine_health() -> Json<Value> {
    let engine = health().await;
    let follow_daemon = follow_daemon_status_payload().await;
    Json(json!({
        "ok": true,
        "backend": "rust",
        "url": configured_base_url(),
        "engine": engine.0,
        "followDaemon": follow_daemon,
    }))
}

async fn api_quote(
    Query(query): Query<QuoteQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let launchpad = query.launchpad.unwrap_or_else(|| "pump".to_string());
    let quote_asset = query.quoteAsset.unwrap_or_else(|| "sol".to_string());
    let launch_mode = query.launchMode.unwrap_or_default();
    let mode = query.mode.unwrap_or_default();
    let amount = query.amount.unwrap_or_default();
    if mode.trim().is_empty() || amount.trim().is_empty() {
        return Ok(Json(attach_timing(
            json!({
                "ok": true,
                "quote": Value::Null,
            }),
            started_at_ms,
        )));
    }
    let quote = quote_launch_for_launchpad(
        &configured_rpc_url(),
        &launchpad,
        &quote_asset,
        &launch_mode,
        &mode,
        &amount,
    )
    .await
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    Ok(Json(attach_timing(
        json!({
            "ok": true,
            "quote": quote,
        }),
        started_at_ms,
    )))
}

async fn api_settings_get() -> Json<Value> {
    let started_at_ms = current_time_ms();
    Json(attach_timing(build_settings_payload(), started_at_ms))
}

async fn api_settings_save(
    Json(payload): Json<SettingsSaveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let config = payload
        .config
        .unwrap_or_else(create_default_persistent_config);
    let saved_path = write_persistent_config(config).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    let mut response = build_settings_payload();
    response["savedPath"] = Value::String(saved_path);
    Ok(Json(attach_timing(response, started_at_ms)))
}

async fn api_reports_list(Query(query): Query<ReportsQuery>) -> Json<Value> {
    let started_at_ms = current_time_ms();
    let sort = if query
        .sort
        .unwrap_or_else(|| "newest".to_string())
        .trim()
        .eq_ignore_ascii_case("oldest")
    {
        "oldest"
    } else {
        "newest"
    };
    Json(attach_timing(
        json!({
            "ok": true,
            "sort": sort,
            "reports": list_persisted_reports(sort),
        }),
        started_at_ms,
    ))
}

async fn api_reports_view(
    Query(query): Query<ReportViewQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let id = query.id.unwrap_or_default();
    let (entry, text, payload) = read_persisted_report_bundle(&id).map_err(|error| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    Ok(Json(attach_timing(
        json!({
            "ok": true,
            "entry": entry,
            "text": text,
            "payload": payload,
        }),
        started_at_ms,
    )))
}

async fn api_upload_image(
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let mut bytes = Vec::new();
    let mut filename = "image".to_string();
    let mut content_type = String::new();
    while let Some(field) = multipart.next_field().await.map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error.to_string(),
            })),
        )
    })? {
        if field.name().unwrap_or_default() != "file" {
            continue;
        }
        filename = field.file_name().unwrap_or("image").to_string();
        content_type = field.content_type().unwrap_or_default().to_string();
        bytes = field
            .bytes()
            .await
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "ok": false,
                        "error": error.to_string(),
                    })),
                )
            })?
            .to_vec();
        break;
    }
    if bytes.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "Image file is required.",
            })),
        ));
    }
    let extension = match content_type.trim().to_ascii_lowercase().as_str() {
        "image/png" => ".png",
        "image/jpeg" => ".jpg",
        "image/webp" => ".webp",
        "image/gif" => ".gif",
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": "Only png, jpg, webp, and gif images are supported.",
                })),
            ));
        }
    };
    let record = save_image_bytes(&bytes, extension, &filename, None).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    Ok(Json(attach_timing(
        json!({
            "ok": true,
            "id": record.id,
            "fileName": record.fileName,
            "name": record.name,
            "tags": record.tags,
            "category": record.category,
            "isFavorite": record.isFavorite,
            "createdAt": record.createdAt,
            "updatedAt": record.updatedAt,
            "previewUrl": record.previewUrl,
        }),
        started_at_ms,
    )))
}

async fn api_metadata_upload(
    Json(payload): Json<MetadataUploadRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let form = payload.form.unwrap_or(Value::Null);
    let metadata_uri = upload_metadata_from_form(form).await.map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    Ok(Json(attach_timing(
        json!({
            "ok": true,
            "metadataUri": metadata_uri,
        }),
        started_at_ms,
    )))
}

async fn api_images_list(
    Query(query): Query<ImagesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let started_at_ms = current_time_ms();
    let payload = build_image_library_payload(
        query.search.as_deref().unwrap_or_default(),
        query.category.as_deref().unwrap_or_default(),
        query
            .favoritesOnly
            .as_deref()
            .unwrap_or_default()
            .eq_ignore_ascii_case("true"),
    )
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    Ok(Json(attach_timing(
        serde_json::to_value(payload).unwrap_or(json!({"ok": false})),
        started_at_ms,
    )))
}

async fn api_image_update(
    Json(payload): Json<ImageUpdateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tags = payload.tags.map(|value| match value {
        Value::Array(entries) => entries
            .into_iter()
            .filter_map(|entry| entry.as_str().map(|value| value.to_string()))
            .collect::<Vec<_>>(),
        Value::String(raw) => raw
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>(),
        _ => vec![],
    });
    let (library_payload, image) = update_image(
        payload.id.as_deref().unwrap_or_default(),
        payload.name.as_deref(),
        tags,
        payload.category.as_deref(),
        payload.isFavorite,
    )
    .map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": error,
            })),
        )
    })?;
    let mut response = serde_json::to_value(library_payload).unwrap_or(json!({"ok": true}));
    response["image"] = serde_json::to_value(image).unwrap_or(Value::Null);
    Ok(Json(response))
}

async fn api_image_category_create(
    Json(payload): Json<ImageCategoryRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (library_payload, category) = create_category(payload.name.as_deref().unwrap_or_default())
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })?;
    let mut response = serde_json::to_value(library_payload).unwrap_or(json!({"ok": true}));
    response["category"] = Value::String(category);
    Ok(Json(response))
}

async fn api_image_delete(
    Json(payload): Json<ImageDeleteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let library_payload =
        delete_image(payload.id.as_deref().unwrap_or_default()).map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })?;
    Ok(Json(
        serde_json::to_value(library_payload).unwrap_or(json!({"ok": true})),
    ))
}

async fn api_vamp_import(
    Json(payload): Json<VampRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let contract_address = payload.contractAddress.unwrap_or_default();
    if contract_address.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "Contract address is required.",
            })),
        ));
    }
    let mint = solana_sdk::pubkey::Pubkey::try_from(contract_address.trim()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": "Invalid Solana contract address.",
            })),
        )
    })?;
    let rpc_url = configured_rpc_url();
    let imported = fetch_imported_token_metadata(&mint.to_string(), &rpc_url)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": error,
                })),
            )
        })?;
    let mut image = Value::Null;
    let mut warning = String::new();
    if !imported.imageUrl.is_empty() {
        match import_remote_image_to_library(
            &imported.imageUrl,
            &format!(
                "{}-vamp",
                if !imported.symbol.is_empty() {
                    imported.symbol.clone()
                } else if !imported.name.is_empty() {
                    imported.name.clone()
                } else {
                    mint.to_string()
                }
            ),
            if !imported.name.is_empty() {
                &imported.name
            } else if !imported.symbol.is_empty() {
                &imported.symbol
            } else {
                "Imported token image"
            },
        )
        .await
        {
            Ok(Some(record)) => {
                image = serde_json::to_value(record).unwrap_or(Value::Null);
            }
            Ok(None) => {}
            Err(error) => warning = error,
        }
    }
    Ok(Json(json!({
        "ok": true,
        "source": if imported.source.is_empty() { "metadata" } else { &imported.source },
        "token": {
            "name": imported.name,
            "symbol": imported.symbol,
            "description": imported.description,
            "website": imported.website,
            "twitter": imported.twitter,
            "telegram": imported.telegram,
            "launchpad": imported.launchpad,
            "mode": imported.mode,
            "quoteAsset": imported.quoteAsset,
            "routes": imported.routes,
            "detection": imported.detection,
        },
        "image": image,
        "warning": warning,
    })))
}

async fn static_handler(
    AxumPath(requested): AxumPath<String>,
) -> Result<Response<Body>, (StatusCode, Json<Value>)> {
    if requested.is_empty() || requested == "index.html" {
        return file_response(paths::ui_dir().join("index.html"));
    }
    if requested.starts_with("uploads/") {
        let file_name = std::path::Path::new(requested.trim_start_matches("uploads/"))
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        return file_response(paths::uploads_dir().join(file_name));
    }
    let safe_name = std::path::Path::new(&requested)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    if safe_name.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": "Not found",
            })),
        ));
    }
    file_response(paths::ui_dir().join(safe_name))
}

async fn runtime_action(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<RuntimeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    let worker = payload.worker.unwrap_or_else(|| "default".to_string());
    let path = headers
        .get("x-launchdeck-runtime-action")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let response = match path.as_str() {
        "start" => start_worker(&state.runtime, &worker, payload.config).await,
        "stop" => stop_worker(&state.runtime, &worker, payload.note.clone()).await,
        "heartbeat" => heartbeat_worker(&state.runtime, &worker, payload.note.clone()).await,
        "fail" => fail_worker(&state.runtime, &worker, payload.note.clone()).await,
        _ => RuntimeResponse {
            ok: true,
            worker: Some(worker),
            active: None,
            state: None,
            workers: list_workers(&state.runtime).await,
        },
    };
    Ok(Json(json!({
        "ok": response.ok,
        "service": "launchdeck-engine",
        "implemented": true,
        "runtime": response,
    })))
}

async fn runtime_start(
    state: State<Arc<AppState>>,
    mut headers: HeaderMap,
    payload: Json<RuntimeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    headers.insert(
        "x-launchdeck-runtime-action",
        "start".parse().expect("valid header"),
    );
    runtime_action(state, headers, payload).await
}

async fn runtime_stop(
    state: State<Arc<AppState>>,
    mut headers: HeaderMap,
    payload: Json<RuntimeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    headers.insert(
        "x-launchdeck-runtime-action",
        "stop".parse().expect("valid header"),
    );
    runtime_action(state, headers, payload).await
}

async fn runtime_status(
    state: State<Arc<AppState>>,
    mut headers: HeaderMap,
    payload: Json<RuntimeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    headers.insert(
        "x-launchdeck-runtime-action",
        "status".parse().expect("valid header"),
    );
    runtime_action(state, headers, payload).await
}

async fn runtime_heartbeat(
    state: State<Arc<AppState>>,
    mut headers: HeaderMap,
    payload: Json<RuntimeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    headers.insert(
        "x-launchdeck-runtime-action",
        "heartbeat".parse().expect("valid header"),
    );
    runtime_action(state, headers, payload).await
}

async fn runtime_fail(
    state: State<Arc<AppState>>,
    mut headers: HeaderMap,
    payload: Json<RuntimeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    headers.insert(
        "x-launchdeck-runtime-action",
        "fail".parse().expect("valid header"),
    );
    runtime_action(state, headers, payload).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_reports_rust_native_only_mode() {
        let response = health().await;
        assert!(response.ok);
        assert_eq!(response.service, "launchdeck-engine");
        assert_eq!(response.mode, "rust-native-only");
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    clear_bags_session_credentials();
    let rpc_url = configured_rpc_url();
    let state = Arc::new(AppState {
        auth_token: configured_auth_token(),
        runtime: Arc::new(RuntimeRegistry::new(configured_runtime_state_path())),
    });
    spawn_fee_market_snapshot_refresh_task(rpc_url.clone());
    spawn_blockhash_refresh_task(rpc_url.clone(), "processed");
    spawn_blockhash_refresh_task(rpc_url.clone(), "confirmed");
    spawn_blockhash_refresh_task(rpc_url, "finalized");
    let restored_workers = restore_workers(&state.runtime).await;
    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/",
            get(|| async { file_response(paths::ui_dir().join("index.html")) }),
        )
        .route(
            "/index.html",
            get(|| async { file_response(paths::ui_dir().join("index.html")) }),
        )
        .route("/api/bootstrap-fast", get(api_bootstrap_fast))
        .route("/api/bootstrap", get(api_bootstrap))
        .route("/api/startup-warm", post(api_startup_warm))
        .route("/api/lookup-tables/warm", post(api_lookup_tables_warm))
        .route("/api/pump-global/warm", post(api_pump_global_warm))
        .route("/api/wallet-status", get(api_wallet_status))
        .route("/api/runtime-status", get(api_runtime_status))
        .route("/api/follow/jobs", get(api_follow_jobs))
        .route("/api/follow/cancel", post(api_follow_cancel))
        .route("/api/follow/stop-all", post(api_follow_stop_all))
        .route("/api/status", get(api_status))
        .route("/api/run", post(api_run))
        .route("/api/engine/health", get(api_engine_health))
        .route("/api/quote", get(api_quote))
        .route("/api/bags/identity/status", get(api_bags_identity_status))
        .route("/api/bags/identity/init", post(api_bags_identity_init))
        .route("/api/bags/identity/verify", post(api_bags_identity_verify))
        .route("/api/bags/identity/clear", post(api_bags_identity_clear))
        .route(
            "/api/settings",
            get(api_settings_get).post(api_settings_save),
        )
        .route("/api/settings/save", post(api_settings_save))
        .route("/api/reports", get(api_reports_list))
        .route("/api/reports/view", get(api_reports_view))
        .route("/api/upload-image", post(api_upload_image))
        .route("/api/metadata/upload", post(api_metadata_upload))
        .route("/api/images", get(api_images_list))
        .route("/api/images/update", post(api_image_update))
        .route("/api/images/categories", post(api_image_category_create))
        .route("/api/images/delete", post(api_image_delete))
        .route("/api/vamp", post(api_vamp_import))
        .route("/engine/status", post(engine_status))
        .route("/engine/quote", post(engine_action))
        .route("/engine/build", post(engine_action))
        .route("/engine/simulate", post(engine_action))
        .route("/engine/send", post(engine_action))
        .route("/engine/runtime/start", post(runtime_start))
        .route("/engine/runtime/stop", post(runtime_stop))
        .route("/engine/runtime/status", post(runtime_status))
        .route("/engine/runtime/heartbeat", post(runtime_heartbeat))
        .route("/engine/runtime/fail", post(runtime_fail))
        .route("/{*path}", get(static_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], configured_engine_port()));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind LaunchDeck engine listener");
    if !restored_workers.is_empty() {
        println!(
            "Restored {} runtime worker(s) from disk.",
            restored_workers.len()
        );
    }
    println!("LaunchDeck engine running at http://{}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("LaunchDeck engine server failed");
}
