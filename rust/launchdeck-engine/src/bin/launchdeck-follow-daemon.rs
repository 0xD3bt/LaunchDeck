#![allow(non_snake_case)]

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt, future::join_all};
use launchdeck_engine::{
    bags_native::{
        compile_follow_buy_transaction as compile_bags_follow_buy_transaction,
        compile_follow_sell_transaction as compile_bags_follow_sell_transaction,
        fetch_bags_market_snapshot,
    },
    bonk_native::{
        compile_follow_buy_transaction as compile_bonk_follow_buy_transaction,
        compile_follow_sell_transaction_with_token_amount as compile_bonk_follow_sell_transaction_with_token_amount,
        fetch_bonk_market_snapshot,
    },
    launchpad_dispatch::compile_atomic_follow_buy_for_launchpad,
    follow::{
        DeferredSetupState, FOLLOW_RESPONSE_SCHEMA_VERSION, FollowActionKind, FollowActionRecord,
        FollowActionState, FollowArmRequest, FollowCancelRequest, FollowDaemonHealth,
        FollowDaemonStore,
        FollowJobRecord, FollowJobResponse, FollowJobState, FollowReadyRequest,
        FollowReadyResponse, FollowReserveRequest, FollowStopAllRequest, FollowWatcherHealth,
        follow_job_response, follow_ready_response, should_use_post_setup_creator_vault_for_buy,
        should_use_post_setup_creator_vault_for_sell,
    },
    observability::update_persisted_follow_daemon_snapshot,
    paths,
    pump_native::{
        PreparedFollowBuyRuntime, PreparedFollowBuyStatic, compile_follow_sell_transaction,
        fetch_pump_market_snapshot, finalize_follow_buy_transaction, prepare_follow_buy_runtime,
        prepare_follow_buy_static, pump_bonding_curve_address,
    },
    rpc::{
        confirm_submitted_transactions_for_transport, fetch_current_block_height,
        fetch_current_block_height_fresh,
        spawn_blockhash_refresh_task, submit_transactions_for_transport,
    },
    transport::{
        TransportPlan, build_transport_plan, configured_watch_endpoints_for_provider,
        supports_helius_transaction_subscribe,
    },
    report::{FollowActionTimings, FollowJobTimings, configured_benchmark_mode},
    wallet::{
        fetch_balance_lamports, fetch_token_balance, load_solana_wallet_by_env_key,
        public_key_from_secret, selected_wallet_key_or_default,
    },
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex, OwnedSemaphorePermit, Semaphore, watch},
    task::{JoinHandle, JoinSet},
    time::{Instant, sleep, timeout},
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const USD1_MINT: &str = "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB";

#[derive(Clone)]
struct AppState {
    auth_token: Option<String>,
    rpc_url: String,
    store: FollowDaemonStore,
    max_active_jobs: Option<usize>,
    max_concurrent_compiles: Option<usize>,
    max_concurrent_sends: Option<usize>,
    capacity_wait_ms: u64,
    active_jobs: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    wallet_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    report_write_lock: Arc<Mutex<()>>,
    compile_slots: Option<Arc<Semaphore>>,
    send_slots: Option<Arc<Semaphore>>,
    watch_hubs: Arc<Mutex<HashMap<String, Arc<JobWatchHub>>>>,
    prepared_follow_buys: Arc<Mutex<HashMap<String, PreparedFollowBuyStatic>>>,
    hot_follow_buy_runtime: Arc<Mutex<HashMap<String, CachedFollowBuyRuntime>>>,
    hot_follow_buy_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    follow_report_flushes: Arc<Mutex<HashMap<String, PendingFollowReportFlush>>>,
    follow_report_flush_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    watch_endpoint_health: Arc<Mutex<HashMap<String, CachedWatchEndpointHealth>>>,
    wallet_precheck_cache: Arc<Mutex<HashMap<String, CachedWalletPrecheck>>>,
    offset_worker: Arc<OffsetWorkerHub>,
}

#[derive(Clone)]
struct CachedFollowBuyRuntime {
    prepared: PreparedFollowBuyRuntime,
    refreshed_at_ms: u128,
}

#[derive(Clone)]
struct PendingFollowReportFlush {
    report_path: String,
    snapshot: Value,
    delay_ms: u64,
}

#[derive(Clone)]
struct CachedWatchEndpointHealth {
    endpoint: String,
    checked_at_ms: u128,
    healthy: bool,
    error: Option<String>,
    provider: Option<String>,
    endpoint_profile: Option<String>,
}

#[derive(Clone)]
struct CachedWalletPrecheck {
    quote_asset: String,
    action: FollowActionRecord,
    checked_at_ms: u128,
    result: Result<(), String>,
}

struct JobWatchHub {
    signature_tx: watch::Sender<Option<Result<u64, String>>>,
    market_tx: watch::Sender<Option<Result<u64, String>>>,
    started: Mutex<JobWatchStarted>,
}

struct OffsetWorkerHub {
    consumers: Mutex<HashMap<String, OffsetConsumer>>,
    running: Mutex<bool>,
    last_observed_block_height: Mutex<Option<u64>>,
    last_observed_at_ms: Mutex<Option<u128>>,
}

struct OffsetConsumer {
    trace_id: String,
    action_id: String,
    base_confirmed_block_height: u64,
    confirmation_detected_at_ms: u128,
    target_block_offset: u64,
    completion_tx: watch::Sender<Option<Result<(), String>>>,
}

#[derive(Default)]
struct JobWatchStarted {
    signature: bool,
    market: bool,
}

#[derive(Clone, Copy)]
enum WatcherKind {
    Slot,
    Signature,
    Market,
}

#[derive(Clone, Copy)]
enum FollowReportSyncMode {
    Debounced,
    Immediate,
    Final,
}

const WATCHER_MAX_RECONNECT_ATTEMPTS: u32 = 5;
const WATCHER_BACKOFF_BASE_MS: u64 = 200;
const FOLLOW_BUY_PRECHECK_BUFFER_LAMPORTS: u64 = 2_000_000;
const DEFAULT_LOCAL_AUTH_TOKEN: &str = "4815927603149027";
const HOT_FOLLOW_BUY_REFRESH_MS: u64 = 250;
const HOT_FOLLOW_BUY_MAX_AGE_MS: u128 = 900;
const FOLLOW_REPORT_SYNC_DEBOUNCE_MS: u64 = 150;
const FOLLOW_READY_WATCH_REFRESH_MS: u64 = 5_000;
const FOLLOW_READY_WATCH_TTL_MS: u128 = 10_000;
const FOLLOW_READY_PRECHECK_TTL_MS: u128 = 5_000;
const FOLLOW_READY_PRECHECK_REFRESH_MS: u64 = 3_000;
const DEFAULT_FOLLOW_OFFSET_POLL_INTERVAL_MS: u64 = 400;
const OFFSET_SAME_BLOCK_REPOLL_DELAY_MS: u64 = 25;
const OFFSET_SAME_BLOCK_REPOLL_LIMIT: usize = 3;

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or_default()
}

fn configured_follow_daemon_port() -> u16 {
    env::var("LAUNCHDECK_FOLLOW_DAEMON_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8790)
}

fn configured_follow_daemon_base_url() -> String {
    env::var("LAUNCHDECK_FOLLOW_DAEMON_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("http://127.0.0.1:{}", configured_follow_daemon_port()))
}

fn parse_optional_limit_value(value: &str) -> Option<usize> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return None;
    }
    trimmed.parse::<usize>().ok().filter(|value| *value > 0)
}

fn configured_limit(var_name: &str) -> Option<usize> {
    let Ok(value) = env::var(var_name) else {
        return None;
    };
    let trimmed = value.trim();
    let parsed = parse_optional_limit_value(trimmed);
    if parsed.is_none() && !trimmed.is_empty() && trimmed != "0" {
        eprintln!(
            "Warning: {var_name}={trimmed:?} is invalid. Use blank or 0 for uncapped, or a positive integer for a cap. Treating it as uncapped."
        );
    }
    parsed
}

fn configured_capacity_wait_ms() -> u64 {
    env::var("LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(5_000)
}

fn configured_follow_offset_poll_interval_ms() -> u64 {
    env::var("LAUNCHDECK_FOLLOW_OFFSET_POLL_INTERVAL_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_FOLLOW_OFFSET_POLL_INTERVAL_MS)
}

fn configured_enable_approximate_follow_offset_timer() -> bool {
    matches!(
        env::var("LAUNCHDECK_ENABLE_APPROXIMATE_FOLLOW_OFFSET_TIMER")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn configured_auth_token() -> Option<String> {
    let token = env::var("LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN")
        .unwrap_or_else(|_| DEFAULT_LOCAL_AUTH_TOKEN.to_string());
    let trimmed = token.trim();
    if trimmed.is_empty() {
        Some(DEFAULT_LOCAL_AUTH_TOKEN.to_string())
    } else {
        Some(trimmed.to_string())
    }
}

fn configured_startup_watch_endpoint() -> Option<String> {
    env::var("SOLANA_WS_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            configured_watch_endpoints_for_provider("standard-rpc", "")
                .into_iter()
                .next()
        })
}

async fn build_health(state: &Arc<AppState>) -> FollowDaemonHealth {
    let mut health = state.store.health().await;
    health.maxActiveJobs = state.max_active_jobs;
    health.maxConcurrentCompiles = state.max_concurrent_compiles;
    health.maxConcurrentSends = state.max_concurrent_sends;
    health.availableCompileSlots = state
        .compile_slots
        .as_ref()
        .map(|semaphore| semaphore.available_permits());
    health.availableSendSlots = state
        .send_slots
        .as_ref()
        .map(|semaphore| semaphore.available_permits());
    if health.activeJobs == 0 {
        health.slotWatcherMode = None;
        health.signatureWatcherMode = None;
        health.marketWatcherMode = None;
        health.watchEndpoint = None;
        health.lastError = None;
    }
    health
}

fn configured_rpc_url() -> String {
    if let Ok(explicit) = env::var("SOLANA_RPC_URL") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "http://127.0.0.1:8899".to_string()
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
                "error": "Unauthorized follow daemon request.",
            })),
        ))
    }
}

fn watch_endpoint_cache_key(endpoint: &str) -> String {
    endpoint.to_string()
}

fn wallet_precheck_cache_key(quote_asset: &str, action: &FollowActionRecord) -> String {
    format!(
        "{quote_asset}|{:?}|{}|{}",
        action.kind,
        action.walletEnvKey,
        action.buyAmountSol.as_deref().unwrap_or_default()
    )
}

fn parse_sol_to_lamports(value: &str, label: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} was empty."));
    }
    let negative = trimmed.starts_with('-');
    if negative {
        return Err(format!("{label} must be non-negative."));
    }
    let mut parts = trimmed.split('.');
    let whole = parts
        .next()
        .unwrap_or("0")
        .parse::<u64>()
        .map_err(|error| format!("{label} whole amount was invalid: {error}"))?;
    let fractional_raw = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return Err(format!("{label} must be a decimal amount."));
    }
    if fractional_raw.len() > 9 {
        return Err(format!("{label} supports at most 9 decimal places."));
    }
    let mut fractional = fractional_raw.to_string();
    while fractional.len() < 9 {
        fractional.push('0');
    }
    let fractional_lamports = if fractional.is_empty() {
        0
    } else {
        fractional
            .parse::<u64>()
            .map_err(|error| format!("{label} fractional amount was invalid: {error}"))?
    };
    whole
        .checked_mul(1_000_000_000)
        .and_then(|base| base.checked_add(fractional_lamports))
        .ok_or_else(|| format!("{label} overflowed lamport conversion."))
}

fn watcher_backoff_ms(attempt: u32) -> u64 {
    let factor = 1u64 << attempt.min(5);
    (WATCHER_BACKOFF_BASE_MS.saturating_mul(factor)).min(5_000)
}

async fn set_watcher_health(
    state: &Arc<AppState>,
    kind: WatcherKind,
    status: FollowWatcherHealth,
    mode: Option<String>,
    endpoint: Option<String>,
    last_error: Option<String>,
) {
    let current = state.store.health().await;
    let slot = match kind {
        WatcherKind::Slot => status.clone(),
        _ => current.slotWatcher.clone(),
    };
    let slot_mode = match kind {
        WatcherKind::Slot => mode.clone(),
        _ => current.slotWatcherMode.clone(),
    };
    let signature = match kind {
        WatcherKind::Signature => status.clone(),
        _ => current.signatureWatcher.clone(),
    };
    let signature_mode = match kind {
        WatcherKind::Signature => mode.clone(),
        _ => current.signatureWatcherMode.clone(),
    };
    let market = match kind {
        WatcherKind::Market => status,
        _ => current.marketWatcher.clone(),
    };
    let market_mode = match kind {
        WatcherKind::Market => mode,
        _ => current.marketWatcherMode.clone(),
    };
    let _ = state
        .store
        .update_health(
            endpoint.or(current.watchEndpoint),
            slot,
            slot_mode,
            signature,
            signature_mode,
            market,
            market_mode,
            last_error,
        )
        .await;
}

async fn wallet_public_key(wallet_key: &str) -> Result<String, String> {
    let wallet_secret = load_solana_wallet_by_env_key(wallet_key)?;
    public_key_from_secret(&wallet_secret)
}

fn follow_buy_cache_key(trace_id: &str, action_id: &str) -> String {
    format!("{trace_id}:{action_id}")
}

fn hot_follow_buy_runtime_cache_key(trace_id: &str, prefer_post_setup_creator_vault: bool) -> String {
    format!(
        "{trace_id}:{}",
        if prefer_post_setup_creator_vault {
            "post-setup"
        } else {
            "launch-creator"
        }
    )
}

async fn cache_prepared_follow_buy(
    state: &Arc<AppState>,
    trace_id: &str,
    action_id: &str,
    prepared: PreparedFollowBuyStatic,
) {
    let key = follow_buy_cache_key(trace_id, action_id);
    let mut cache = state.prepared_follow_buys.lock().await;
    cache.insert(key, prepared);
}

async fn get_prepared_follow_buy(
    state: &Arc<AppState>,
    trace_id: &str,
    action_id: &str,
) -> Option<PreparedFollowBuyStatic> {
    let key = follow_buy_cache_key(trace_id, action_id);
    let cache = state.prepared_follow_buys.lock().await;
    cache.get(&key).cloned()
}

async fn cache_hot_follow_buy_runtime(
    state: &Arc<AppState>,
    trace_id: &str,
    prefer_post_setup_creator_vault: bool,
    prepared: PreparedFollowBuyRuntime,
) {
    let mut cache = state.hot_follow_buy_runtime.lock().await;
    cache.insert(
        hot_follow_buy_runtime_cache_key(trace_id, prefer_post_setup_creator_vault),
        CachedFollowBuyRuntime {
            prepared,
            refreshed_at_ms: now_ms(),
        },
    );
}

async fn get_hot_follow_buy_runtime(
    state: &Arc<AppState>,
    trace_id: &str,
    prefer_post_setup_creator_vault: bool,
) -> Option<CachedFollowBuyRuntime> {
    let cache = state.hot_follow_buy_runtime.lock().await;
    cache.get(&hot_follow_buy_runtime_cache_key(
        trace_id,
        prefer_post_setup_creator_vault,
    ))
    .cloned()
}

async fn clear_follow_buy_caches(state: &Arc<AppState>, trace_id: &str) {
    {
        let mut prepared = state.prepared_follow_buys.lock().await;
        prepared.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut runtime = state.hot_follow_buy_runtime.lock().await;
        runtime.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    let handle = {
        let mut tasks = state.hot_follow_buy_tasks.lock().await;
        tasks.remove(trace_id)
    };
    if let Some(handle) = handle {
        handle.abort();
    }
}

async fn clear_follow_buy_cache_entries_only(state: &Arc<AppState>, trace_id: &str) {
    {
        let mut prepared = state.prepared_follow_buys.lock().await;
        prepared.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut runtime = state.hot_follow_buy_runtime.lock().await;
        runtime.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
}

async fn spawn_hot_follow_buy_refresh_if_needed(state: Arc<AppState>, trace_id: String) {
    let mut tasks = state.hot_follow_buy_tasks.lock().await;
    if tasks.contains_key(&trace_id) {
        return;
    }
    let task_state = state.clone();
    let task_trace_id = trace_id.clone();
    let handle = tokio::spawn(async move {
        loop {
            let Some(job) = get_job(&task_state, &task_trace_id).await else {
                break;
            };
            if job.cancelRequested
                || matches!(
                    job.state,
                    FollowJobState::Completed
                        | FollowJobState::CompletedWithFailures
                        | FollowJobState::Cancelled
                        | FollowJobState::Failed
                )
            {
                break;
            }
            let Some(mint) = job.mint.as_deref() else {
                sleep(Duration::from_millis(HOT_FOLLOW_BUY_REFRESH_MS)).await;
                continue;
            };
            let Some(launch_creator) = job.launchCreator.as_deref() else {
                sleep(Duration::from_millis(HOT_FOLLOW_BUY_REFRESH_MS)).await;
                continue;
            };
            for prefer_post_setup_creator_vault in [false, true] {
                if let Ok(prepared) = prepare_follow_buy_runtime(
                    &task_state.rpc_url,
                    mint,
                    launch_creator,
                    prefer_post_setup_creator_vault,
                )
                .await
                {
                    cache_hot_follow_buy_runtime(
                        &task_state,
                        &task_trace_id,
                        prefer_post_setup_creator_vault,
                        prepared,
                    )
                    .await;
                }
            }
            sleep(Duration::from_millis(HOT_FOLLOW_BUY_REFRESH_MS)).await;
        }
        clear_follow_buy_cache_entries_only(&task_state, &task_trace_id).await;
        let mut tasks = task_state.hot_follow_buy_tasks.lock().await;
        tasks.remove(&task_trace_id);
    });
    tasks.insert(trace_id, handle);
}

async fn prepare_follow_job_buy_caches(state: Arc<AppState>, job: &FollowJobRecord) {
    let Some(mint) = job.mint.as_deref() else {
        return;
    };
    let Some(launch_creator) = job.launchCreator.as_deref() else {
        return;
    };
    let buy_actions = job
        .actions
        .iter()
        .filter(|action| matches!(action.kind, FollowActionKind::SniperBuy))
        .cloned()
        .collect::<Vec<_>>();
    if buy_actions.is_empty() {
        return;
    }
    if job.launchpad != "pump" {
        return;
    }
    for prefer_post_setup_creator_vault in [false, true] {
        if let Ok(prepared_runtime) = prepare_follow_buy_runtime(
            &state.rpc_url,
            mint,
            launch_creator,
            prefer_post_setup_creator_vault,
        )
        .await
        {
            cache_hot_follow_buy_runtime(
                &state,
                &job.traceId,
                prefer_post_setup_creator_vault,
                prepared_runtime,
            )
            .await;
        }
    }
    let trace_id = job.traceId.clone();
    let rpc_url = state.rpc_url.clone();
    let execution = job.execution.clone();
    let buy_tip_account = job.buyTipAccount.clone();
    let tasks = buy_actions.into_iter().map(|action| {
        let trace_id = trace_id.clone();
        let rpc_url = rpc_url.clone();
        let execution = execution.clone();
        let buy_tip_account = buy_tip_account.clone();
        let mint = mint.to_string();
        let launch_creator = launch_creator.to_string();
        async move {
            let wallet_key = selected_wallet_key_or_default(&action.walletEnvKey)
                .ok_or_else(|| format!("Wallet env key not found: {}", action.walletEnvKey))?;
            let wallet_secret = load_solana_wallet_by_env_key(&wallet_key)?;
            let buy_amount = action
                .buyAmountSol
                .as_deref()
                .ok_or_else(|| "Follow buy missing amount.".to_string())?;
            let prepared = prepare_follow_buy_static(
                &rpc_url,
                &execution,
                &buy_tip_account,
                &wallet_secret,
                &mint,
                &launch_creator,
                buy_amount,
            )
            .await?;
            Ok::<_, String>((trace_id, action.actionId, prepared))
        }
    });
    for result in join_all(tasks).await {
        if let Ok((trace_id, action_id, prepared)) = result {
            cache_prepared_follow_buy(&state, &trace_id, &action_id, prepared).await;
        }
    }
    spawn_hot_follow_buy_refresh_if_needed(state, job.traceId.clone()).await;
}

async fn resolve_prepared_follow_buy(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    wallet_secret: &[u8],
    launch_creator: &str,
    buy_amount: &str,
) -> Result<PreparedFollowBuyStatic, String> {
    if let Some(prepared) = get_prepared_follow_buy(state, &job.traceId, &action.actionId).await {
        return Ok(prepared);
    }
    let mint = job
        .mint
        .as_deref()
        .ok_or_else(|| "Follow job missing mint.".to_string())?;
    let prepared = prepare_follow_buy_static(
        &state.rpc_url,
        &job.execution,
        &job.buyTipAccount,
        wallet_secret,
        mint,
        launch_creator,
        buy_amount,
    )
    .await?;
    cache_prepared_follow_buy(state, &job.traceId, &action.actionId, prepared.clone()).await;
    Ok(prepared)
}

async fn resolve_hot_follow_buy_runtime_for_job(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    launch_creator: &str,
    prefer_post_setup_creator_vault: bool,
) -> Result<PreparedFollowBuyRuntime, String> {
    if let Some(cached) =
        get_hot_follow_buy_runtime(state, &job.traceId, prefer_post_setup_creator_vault).await
        && now_ms().saturating_sub(cached.refreshed_at_ms) <= HOT_FOLLOW_BUY_MAX_AGE_MS
    {
        return Ok(cached.prepared);
    }
    let mint = job
        .mint
        .as_deref()
        .ok_or_else(|| "Follow job missing mint.".to_string())?;
    let prepared = prepare_follow_buy_runtime(
        &state.rpc_url,
        mint,
        launch_creator,
        prefer_post_setup_creator_vault,
    )
    .await?;
    cache_hot_follow_buy_runtime(
        state,
        &job.traceId,
        prefer_post_setup_creator_vault,
        prepared.clone(),
    )
    .await;
    Ok(prepared)
}

async fn required_action_precheck(
    rpc_url: &str,
    quote_asset: &str,
    action: &FollowActionRecord,
) -> Result<(), String> {
    let wallet_key = selected_wallet_key_or_default(&action.walletEnvKey)
        .ok_or_else(|| format!("Wallet env key not found: {}", action.walletEnvKey))?;
    let public_key = wallet_public_key(&wallet_key).await?;
    if let Some(amount_sol) = action.buyAmountSol.as_deref() {
        if quote_asset.eq_ignore_ascii_case("usd1") {
            let required_usd1 = amount_sol
                .trim()
                .parse::<f64>()
                .map_err(|error| format!("Invalid USD1 follow buy amount: {error}"))?;
            let usd1_balance =
                fetch_token_balance(rpc_url, &public_key, USD1_MINT, "processed").await?;
            let balance_lamports = fetch_balance_lamports(rpc_url, &public_key).await?;
            if balance_lamports < FOLLOW_BUY_PRECHECK_BUFFER_LAMPORTS {
                return Err(format!(
                    "Wallet {wallet_key} has insufficient SOL headroom for follow buy fees. Need at least {} lamports, found {balance_lamports}.",
                    FOLLOW_BUY_PRECHECK_BUFFER_LAMPORTS
                ));
            }
            if usd1_balance + 0.000_001 < required_usd1
                && balance_lamports < FOLLOW_BUY_PRECHECK_BUFFER_LAMPORTS.saturating_mul(2)
            {
                return Err(format!(
                    "Wallet {wallet_key} is short on USD1 and does not have enough SOL headroom to fund an automatic top-up."
                ));
            }
        } else {
            let required_lamports = parse_sol_to_lamports(amount_sol, "follow buy amount")?
                .saturating_add(FOLLOW_BUY_PRECHECK_BUFFER_LAMPORTS);
            let balance_lamports = fetch_balance_lamports(rpc_url, &public_key).await?;
            if balance_lamports < required_lamports {
                return Err(format!(
                    "Wallet {wallet_key} has insufficient funds for follow buy. Need at least {required_lamports} lamports, found {balance_lamports}."
                ));
            }
        }
    }
    Ok(())
}

async fn run_cached_action_precheck(
    state: &Arc<AppState>,
    quote_asset: &str,
    action: &FollowActionRecord,
) -> Result<(), String> {
    let key = wallet_precheck_cache_key(quote_asset, action);
    {
        let cache = state.wallet_precheck_cache.lock().await;
        if let Some(entry) = cache.get(&key)
            && now_ms().saturating_sub(entry.checked_at_ms) <= FOLLOW_READY_PRECHECK_TTL_MS
        {
            return entry.result.clone();
        }
    }
    let result = required_action_precheck(&state.rpc_url, quote_asset, action).await;
    let cached = CachedWalletPrecheck {
        quote_asset: quote_asset.to_string(),
        action: action.clone(),
        checked_at_ms: now_ms(),
        result: result.clone(),
    };
    let mut cache = state.wallet_precheck_cache.lock().await;
    cache.insert(key, cached);
    result
}

fn spawn_action_precheck_refresh(
    state: Arc<AppState>,
    quote_asset: String,
    action: FollowActionRecord,
) {
    tokio::spawn(async move {
        let _ = run_cached_action_precheck(&state, &quote_asset, &action).await;
    });
}

fn build_follow_buy_precheck_action(
    snipe: &launchdeck_engine::config::NormalizedFollowLaunchSnipe,
) -> FollowActionRecord {
    FollowActionRecord {
        actionId: snipe.actionId.clone(),
        kind: FollowActionKind::SniperBuy,
        walletEnvKey: snipe.walletEnvKey.clone(),
        state: FollowActionState::Queued,
        buyAmountSol: Some(snipe.buyAmountSol.clone()),
        sellPercent: None,
        submitDelayMs: Some(snipe.submitDelayMs),
        targetBlockOffset: snipe.targetBlockOffset,
        delayMs: None,
        marketCap: None,
        jitterMs: Some(snipe.jitterMs),
        feeJitterBps: Some(snipe.feeJitterBps),
        precheckRequired: true,
        requireConfirmation: false,
        skipIfTokenBalancePositive: false,
        attemptCount: 0,
        scheduledForMs: None,
        submitStartedAtMs: None,
        submittedAtMs: None,
        confirmedAtMs: None,
        provider: None,
        endpointProfile: None,
        transportType: None,
        watcherMode: None,
        watcherFallbackReason: None,
        sendObservedBlockHeight: None,
        confirmedObservedBlockHeight: None,
        blocksToConfirm: None,
        signature: None,
        explorerUrl: None,
        endpoint: None,
        bundleId: None,
        lastError: None,
        triggerKey: None,
        orderIndex: 0,
        preSignedTransactions: vec![],
        poolId: None,
        timings: FollowActionTimings::default(),
    }
}

async fn cached_action_precheck_result(
    state: &Arc<AppState>,
    quote_asset: &str,
    action: &FollowActionRecord,
) -> Option<CachedWalletPrecheck> {
    let key = wallet_precheck_cache_key(quote_asset, action);
    let cache = state.wallet_precheck_cache.lock().await;
    cache.get(&key).cloned()
}

async fn run_launch_gate_prechecks_advisory(
    state: &Arc<AppState>,
    payload: &FollowReadyRequest,
) -> Result<(), String> {
    if !payload.followLaunch.constraints.blockOnRequiredPrechecks {
        return Ok(());
    }
    for snipe in &payload.followLaunch.snipes {
        if !snipe.enabled {
            continue;
        }
        let action = build_follow_buy_precheck_action(snipe);
        let cached = cached_action_precheck_result(state, &payload.quoteAsset, &action).await;
        if let Some(entry) = cached {
            if now_ms().saturating_sub(entry.checked_at_ms) <= FOLLOW_READY_PRECHECK_TTL_MS {
                entry.result?;
                continue;
            }
        }
        spawn_action_precheck_refresh(state.clone(), payload.quoteAsset.clone(), action);
    }
    Ok(())
}

async fn skip_retry_if_wallet_already_holds_token(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    wallet_key: &str,
    mint: &str,
) -> Result<bool, String> {
    if !action.skipIfTokenBalancePositive || !matches!(action.kind, FollowActionKind::SniperBuy) {
        return Ok(false);
    }
    let public_key = wallet_public_key(wallet_key).await?;
    let token_balance = fetch_token_balance(&state.rpc_url, &public_key, mint, "processed").await?;
    if token_balance <= 0.0 {
        return Ok(false);
    }
    state
        .store
        .update_action(&job.traceId, &action.actionId, |record| {
            record.state = FollowActionState::Cancelled;
            record.lastError =
                Some("Skipped retry because wallet already holds the launch token.".to_string());
        })
        .await?;
    sync_follow_job_report(state, &job.traceId).await;
    Ok(true)
}

async fn run_launch_gate_prechecks(
    state: &Arc<AppState>,
    payload: &FollowReadyRequest,
) -> Result<(), String> {
    if !payload.followLaunch.constraints.blockOnRequiredPrechecks {
        return Ok(());
    }
    for snipe in &payload.followLaunch.snipes {
        if !snipe.enabled {
            continue;
        }
        let action = build_follow_buy_precheck_action(snipe);
        run_cached_action_precheck(state, &payload.quoteAsset, &action).await?;
    }
    Ok(())
}

fn has_capacity_for_new_job(health: &FollowDaemonHealth) -> bool {
    health
        .maxActiveJobs
        .is_none_or(|max_active_jobs| health.activeJobs < max_active_jobs)
}

async fn acquire_capacity_slot(
    semaphore: Option<Arc<Semaphore>>,
    wait_ms: u64,
    label: &str,
) -> Result<Option<OwnedSemaphorePermit>, String> {
    let Some(semaphore) = semaphore else {
        return Ok(None);
    };
    timeout(Duration::from_millis(wait_ms), semaphore.acquire_owned())
        .await
        .map_err(|_| format!("Timed out waiting for follow daemon {label} capacity."))?
        .map_err(|_| format!("Follow daemon {label} capacity is unavailable."))
        .map(Some)
}

async fn daemon_health(State(state): State<Arc<AppState>>) -> Json<FollowDaemonHealth> {
    Json(build_health(&state).await)
}

fn requires_realtime_watchers(request: &FollowReadyRequest) -> bool {
    if !request.followLaunch.enabled {
        return false;
    }
    request.followLaunch.snipes.iter().any(|snipe| {
        snipe.enabled
            && (snipe.targetBlockOffset.is_some()
                || snipe
                    .postBuySell
                    .as_ref()
                    .is_some_and(sell_requires_realtime_watchers))
    }) || request
        .followLaunch
        .devAutoSell
        .as_ref()
        .is_some_and(sell_requires_realtime_watchers)
}

fn sell_requires_realtime_watchers(
    sell: &launchdeck_engine::config::NormalizedFollowLaunchSell,
) -> bool {
    sell.requireConfirmation || sell.targetBlockOffset.is_some() || sell.marketCap.is_some()
}

fn phase_two_follow_actions_enabled(
    follow: &launchdeck_engine::config::NormalizedFollowLaunch,
) -> bool {
    follow
        .snipes
        .iter()
        .any(|snipe| snipe.postBuySell.is_some())
}

fn resolve_watch_endpoint(request: &FollowReadyRequest) -> Option<String> {
    request.watchEndpoint.clone().or_else(|| {
        configured_watch_endpoints_for_provider(
            &request.execution.provider,
            &request.execution.endpointProfile,
        )
        .into_iter()
        .next()
    })
}

fn selected_realtime_watcher_mode(
    provider: &str,
    endpoint_profile: &str,
    watch_endpoint: Option<&str>,
) -> String {
    if configured_enable_helius_transaction_subscribe()
        && supports_helius_transaction_subscribe(provider, endpoint_profile, watch_endpoint)
    {
        "helius-transaction-subscribe".to_string()
    } else {
        "standard-ws".to_string()
    }
}

fn selected_market_watcher_mode(
    provider: &str,
    endpoint_profile: &str,
    watch_endpoint: Option<&str>,
) -> String {
    selected_realtime_watcher_mode(provider, endpoint_profile, watch_endpoint)
}

async fn remember_watch_endpoint(
    state: &Arc<AppState>,
    endpoint: &str,
) {
    let key = watch_endpoint_cache_key(endpoint);
    let mut cache = state.watch_endpoint_health.lock().await;
    cache.entry(key).or_insert_with(|| CachedWatchEndpointHealth {
        endpoint: endpoint.to_string(),
        checked_at_ms: 0,
        healthy: false,
        error: Some("Watcher health has not been validated yet.".to_string()),
        provider: None,
        endpoint_profile: None,
    });
}

async fn cached_watch_endpoint_health(
    state: &Arc<AppState>,
    endpoint: &str,
) -> Option<CachedWatchEndpointHealth> {
    let key = watch_endpoint_cache_key(endpoint);
    let cache = state.watch_endpoint_health.lock().await;
    cache.get(&key).cloned()
}

async fn store_watch_endpoint_health(
    state: &Arc<AppState>,
    endpoint: &str,
    healthy: bool,
    error: Option<String>,
    provider: Option<&str>,
    endpoint_profile: Option<&str>,
) {
    let key = watch_endpoint_cache_key(endpoint);
    let previous = {
        let cache = state.watch_endpoint_health.lock().await;
        cache.get(&key).cloned()
    };
    let provider = provider
        .map(str::to_string)
        .or_else(|| previous.as_ref().and_then(|entry| entry.provider.clone()));
    let endpoint_profile = endpoint_profile
        .map(str::to_string)
        .or_else(|| previous.as_ref().and_then(|entry| entry.endpoint_profile.clone()));
    let entry = CachedWatchEndpointHealth {
        endpoint: endpoint.to_string(),
        checked_at_ms: now_ms(),
        healthy,
        error: error.clone(),
        provider: provider.clone(),
        endpoint_profile: endpoint_profile.clone(),
    };
    {
        let mut cache = state.watch_endpoint_health.lock().await;
        cache.insert(key, entry);
    }
    let current = state.store.health().await;
    if current.watchEndpoint.as_deref() == Some(endpoint) || current.watchEndpoint.is_none() {
        let watcher_status = if healthy {
            FollowWatcherHealth::Healthy
        } else {
            FollowWatcherHealth::Failed
        };
        let mode = provider.as_deref().map(|value| {
            selected_realtime_watcher_mode(
                value,
                endpoint_profile.as_deref().unwrap_or_default(),
                Some(endpoint),
            )
        });
        let market_mode = provider.as_deref().map(|value| {
            selected_market_watcher_mode(
                value,
                endpoint_profile.as_deref().unwrap_or_default(),
                Some(endpoint),
            )
        });
        let _ = state
            .store
            .update_health(
                Some(endpoint.to_string()),
                watcher_status.clone(),
                mode.clone(),
                watcher_status.clone(),
                mode,
                watcher_status,
                market_mode,
                error,
            )
            .await;
    }
}

async fn refresh_watch_endpoint_health(
    state: &Arc<AppState>,
    endpoint: &str,
    provider: Option<&str>,
    endpoint_profile: Option<&str>,
) -> Result<(), String> {
    let result = validate_watch_endpoint(endpoint).await;
    match &result {
        Ok(()) => {
            store_watch_endpoint_health(state, endpoint, true, None, provider, endpoint_profile).await;
        }
        Err(error) => {
            store_watch_endpoint_health(
                state,
                endpoint,
                false,
                Some(error.clone()),
                provider,
                endpoint_profile,
            )
            .await;
        }
    }
    result
}

async fn refresh_watch_endpoint_health_cache(state: &Arc<AppState>) {
    let endpoints = {
        let cache = state.watch_endpoint_health.lock().await;
        cache.values().cloned().collect::<Vec<_>>()
    };
    for entry in endpoints {
        if entry.checked_at_ms > 0
            && now_ms().saturating_sub(entry.checked_at_ms) < FOLLOW_READY_WATCH_REFRESH_MS as u128
        {
            continue;
        }
        let _ = refresh_watch_endpoint_health(
            state,
            &entry.endpoint,
            entry.provider.as_deref(),
            entry.endpoint_profile.as_deref(),
        )
        .await;
    }
}

fn spawn_watch_endpoint_health_monitor(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            refresh_watch_endpoint_health_cache(&state).await;
            sleep(Duration::from_millis(FOLLOW_READY_WATCH_REFRESH_MS)).await;
        }
    });
}

async fn refresh_wallet_precheck_cache(state: &Arc<AppState>) {
    let entries = {
        let cache = state.wallet_precheck_cache.lock().await;
        cache.values().cloned().collect::<Vec<_>>()
    };
    for entry in entries {
        if now_ms().saturating_sub(entry.checked_at_ms) < FOLLOW_READY_PRECHECK_REFRESH_MS as u128 {
            continue;
        }
        let result =
            required_action_precheck(&state.rpc_url, &entry.quote_asset, &entry.action).await;
        let refreshed = CachedWalletPrecheck {
            quote_asset: entry.quote_asset.clone(),
            action: entry.action.clone(),
            checked_at_ms: now_ms(),
            result,
        };
        let key = wallet_precheck_cache_key(&entry.quote_asset, &entry.action);
        let mut cache = state.wallet_precheck_cache.lock().await;
        cache.insert(key, refreshed);
    }
}

fn spawn_wallet_precheck_monitor(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            refresh_wallet_precheck_cache(&state).await;
            sleep(Duration::from_millis(FOLLOW_READY_PRECHECK_REFRESH_MS)).await;
        }
    });
}

async fn validate_watch_endpoint(endpoint: &str) -> Result<(), String> {
    timeout(Duration::from_secs(3), connect_async(endpoint))
        .await
        .map_err(|_| format!("Timed out connecting to websocket endpoint: {endpoint}"))?
        .map(|_| ())
        .map_err(|error| error.to_string())
}

async fn ensure_follow_request_prerequisites(
    state: &Arc<AppState>,
    payload: &FollowReadyRequest,
    requires_websocket: bool,
    watch_endpoint: Option<&String>,
    strict: bool,
) -> Result<(), String> {
    if payload.followLaunch.enabled {
        if strict {
            run_launch_gate_prechecks(state, payload).await?;
        } else {
            run_launch_gate_prechecks_advisory(state, payload).await?;
        }
    }
    if !requires_websocket {
        return Ok(());
    }
    let endpoint = watch_endpoint
        .map(|value| value.as_str())
        .ok_or_else(|| "No websocket watch endpoint configured. Set SOLANA_WS_URL.".to_string())?;
    remember_watch_endpoint(state, endpoint).await;
    let cached = cached_watch_endpoint_health(state, endpoint).await;
    if strict {
        if let Some(entry) = cached {
            if entry.checked_at_ms > 0
                && now_ms().saturating_sub(entry.checked_at_ms) <= FOLLOW_READY_WATCH_TTL_MS
            {
                return if entry.healthy {
                    Ok(())
                } else {
                    Err(entry
                        .error
                        .unwrap_or_else(|| "Watcher health check failed.".to_string()))
                };
            }
        }
        return refresh_watch_endpoint_health(
            state,
            endpoint,
            Some(&payload.execution.provider),
            Some(&payload.execution.endpointProfile),
        )
        .await;
    }
    if let Some(entry) = cached {
        if entry.checked_at_ms > 0
            && now_ms().saturating_sub(entry.checked_at_ms) <= FOLLOW_READY_WATCH_TTL_MS
        {
            return if entry.healthy {
                Ok(())
            } else {
                Err(entry
                    .error
                    .unwrap_or_else(|| "Watcher health check failed.".to_string()))
            };
        }
    }
    tokio::spawn({
        let state = state.clone();
        let endpoint = endpoint.to_string();
        let provider = payload.execution.provider.clone();
        let endpoint_profile = payload.execution.endpointProfile.clone();
        async move {
            let _ = refresh_watch_endpoint_health(
                &state,
                &endpoint,
                Some(&provider),
                Some(&endpoint_profile),
            )
            .await;
        }
    });
    Ok(())
}

async fn daemon_ready(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<FollowReadyRequest>,
) -> Result<Json<FollowReadyResponse>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    let requires_websocket = requires_realtime_watchers(&payload);
    let watch_endpoint = resolve_watch_endpoint(&payload);
    if payload.followLaunch.enabled && payload.followLaunch.schemaVersion == 0 {
        let health = build_health(&state).await;
        return Ok(Json(follow_ready_response(
            health,
            watch_endpoint,
            requires_websocket,
            false,
            Some("Unsupported follow launch schema version.".to_string()),
        )));
    }
    if payload.followLaunch.enabled && payload.execution.send == false {
        let health = build_health(&state).await;
        return Ok(Json(follow_ready_response(
            health,
            watch_endpoint,
            requires_websocket,
            false,
            Some("Follow launch requires execution.send=true.".to_string()),
        )));
    }
    if payload.followLaunch.enabled && phase_two_follow_actions_enabled(&payload.followLaunch) {
        let health = build_health(&state).await;
        return Ok(Json(follow_ready_response(
            health,
            watch_endpoint,
            requires_websocket,
            false,
            Some(
                "Per-sniper post-buy sells are not shipped yet. Phase 1 supports multi-sniper buys plus dev auto-sell only."
                    .to_string(),
            ),
        )));
    }
    let health = build_health(&state).await;
    if payload.followLaunch.enabled && !has_capacity_for_new_job(&health) {
        return Ok(Json(follow_ready_response(
            health,
            watch_endpoint,
            requires_websocket,
            false,
            Some("Follow daemon active job capacity is full.".to_string()),
        )));
    }
    if let Err(error) = ensure_follow_request_prerequisites(
        &state,
        &payload,
        requires_websocket,
        watch_endpoint.as_ref(),
        false,
    )
    .await
    {
        if requires_websocket && watch_endpoint.is_none() {
            let _ = state
                .store
                .update_health(
                    None,
                    FollowWatcherHealth::Failed,
                    None,
                    FollowWatcherHealth::Failed,
                    None,
                    FollowWatcherHealth::Failed,
                    None,
                    Some("No websocket watch endpoint configured for follow daemon. Set SOLANA_WS_URL.".to_string()),
                )
                .await;
        }
        let health = build_health(&state).await;
        return Ok(Json(follow_ready_response(
            health,
            watch_endpoint,
            requires_websocket,
            false,
            Some(error),
        )));
    }
    let health = build_health(&state).await;
    Ok(Json(follow_ready_response(
        health,
        watch_endpoint,
        requires_websocket,
        true,
        None,
    )))
}

async fn reserve_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<FollowReserveRequest>,
) -> Result<Json<FollowJobResponse>, (StatusCode, Json<Value>)> {
    let reserve_started = Instant::now();
    authorize(&headers, &state)?;
    if payload.followLaunch.enabled && phase_two_follow_actions_enabled(&payload.followLaunch) {
        return Err(internal_error(
            "Per-sniper post-buy sells are not shipped yet. Phase 1 supports multi-sniper buys plus dev auto-sell only."
                .to_string(),
        ));
    }
    let ready_request = FollowReadyRequest {
        followLaunch: payload.followLaunch.clone(),
        quoteAsset: payload.quoteAsset.clone(),
        execution: payload.execution.clone(),
        watchEndpoint: None,
    };
    let requires_websocket = requires_realtime_watchers(&ready_request);
    let watch_endpoint = resolve_watch_endpoint(&ready_request);
    ensure_follow_request_prerequisites(
        &state,
        &ready_request,
        requires_websocket,
        watch_endpoint.as_ref(),
        true,
    )
        .await
        .map_err(internal_error)?;
    let existing = get_job(&state, &payload.traceId).await;
    let health = build_health(&state).await;
    if existing.is_none() && !has_capacity_for_new_job(&health) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "ok": false,
                "error": "Follow daemon active job capacity is full.",
                "health": health,
            })),
        ));
    }
    let job = state
        .store
        .reserve_job(payload)
        .await
        .map_err(internal_error)?;
    update_job_timings(&state, &job.traceId, |timings| {
        timings.reserveMs = Some(reserve_started.elapsed().as_millis());
    })
    .await;
    let health = state.store.health().await;
    Ok(Json(follow_job_response(health, Some(job), vec![])))
}

async fn arm_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<FollowArmRequest>,
) -> Result<Json<FollowJobResponse>, (StatusCode, Json<Value>)> {
    let arm_started = Instant::now();
    authorize(&headers, &state)?;
    let trace_id = payload.traceId.clone();
    let job = state.store.arm_job(payload).await.map_err(internal_error)?;
    update_job_timings(&state, &trace_id, |timings| {
        timings.armMs = Some(arm_started.elapsed().as_millis());
    })
    .await;
    spawn_job_if_needed(state.clone(), trace_id.clone()).await;
    let arm_task_state = state.clone();
    let arm_task_trace_id = trace_id.clone();
    let arm_task_job = job.clone();
    tokio::spawn(async move {
        sync_follow_job_report_immediate(&arm_task_state, &arm_task_trace_id).await;
        let cache_started = Instant::now();
        prepare_follow_job_buy_caches(arm_task_state.clone(), &arm_task_job).await;
        update_job_timings(&arm_task_state, &arm_task_trace_id, |timings| {
            add_time(&mut timings.cachePrepMs, cache_started.elapsed().as_millis());
        })
        .await;
    });
    let health = state.store.health().await;
    Ok(Json(follow_job_response(health, Some(job), vec![])))
}

async fn cancel_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<FollowCancelRequest>,
) -> Result<Json<FollowJobResponse>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    let trace_id = payload.traceId.clone();
    let job = state
        .store
        .cancel_job(payload)
        .await
        .map_err(internal_error)?;
    sync_follow_job_report_final(&state, &trace_id).await;
    Ok(Json(job_response(&state, Some(job)).await))
}

async fn job_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(trace_id): Path<String>,
) -> Result<Json<FollowJobResponse>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    let job = get_job(&state, &trace_id).await;
    Ok(Json(job_response(&state, job).await))
}

async fn list_jobs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<FollowJobResponse>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    Ok(Json(job_response(&state, None).await))
}

async fn stop_all_jobs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<FollowStopAllRequest>,
) -> Result<Json<FollowJobResponse>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    state
        .store
        .cancel_all_jobs(payload.note)
        .await
        .map_err(internal_error)?;
    let trace_ids = state
        .store
        .list_jobs()
        .await
        .into_iter()
        .map(|job| job.traceId)
        .collect::<Vec<_>>();
    for trace_id in trace_ids {
        sync_follow_job_report_final(&state, &trace_id).await;
    }
    Ok(Json(job_response(&state, None).await))
}

fn internal_error(error: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "ok": false,
            "error": error,
        })),
    )
}

async fn get_job(state: &Arc<AppState>, trace_id: &str) -> Option<FollowJobRecord> {
    state
        .store
        .list_jobs()
        .await
        .into_iter()
        .find(|job| job.traceId == trace_id)
}

fn sum_follow_times(values: impl IntoIterator<Item = Option<u128>>) -> Option<u128> {
    let mut total = 0u128;
    let mut has_any = false;
    for value in values.into_iter().flatten() {
        total = total.saturating_add(value);
        has_any = true;
    }
    has_any.then_some(total)
}

fn add_time(slot: &mut Option<u128>, delta_ms: u128) {
    if delta_ms == 0 {
        return;
    }
    *slot = Some(slot.unwrap_or_default().saturating_add(delta_ms));
}

fn refresh_follow_job_timing_rollups(job: &mut FollowJobRecord) {
    let action_watcher_wait = sum_follow_times(
        job.actions
            .iter()
            .map(|action| action.timings.watcherWaitMs),
    );
    let action_eligibility = sum_follow_times(
        job.actions
            .iter()
            .map(|action| action.timings.eligibilityMs),
    );
    let action_compile = sum_follow_times(job.actions.iter().map(|action| action.timings.compileMs));
    let action_submit = sum_follow_times(job.actions.iter().map(|action| action.timings.submitMs));
    let action_confirm = sum_follow_times(job.actions.iter().map(|action| action.timings.confirmMs));
    let action_execution = sum_follow_times(
        job.actions
            .iter()
            .map(|action| action.timings.executionTotalMs),
    );
    job.timings.benchmarkMode = Some(configured_benchmark_mode().as_str().to_string());
    job.timings.watcherWaitMs = action_watcher_wait;
    job.timings.eligibilityMs = action_eligibility;
    job.timings.compileMs = action_compile;
    job.timings.submitMs = action_submit;
    job.timings.confirmMs = action_confirm;
    job.timings.executionTotalMs = sum_follow_times([
        job.timings.reserveMs,
        job.timings.armMs,
        job.timings.cachePrepMs,
        action_execution,
    ]);
    job.timings.reportingOverheadMs = sum_follow_times([
        job.timings.reportSyncMs,
        job.timings.followSnapshotFlushMs,
    ]);
}

async fn update_job_timings(
    state: &Arc<AppState>,
    trace_id: &str,
    mutator: impl FnOnce(&mut FollowJobTimings),
) {
    let _ = state
        .store
        .update_job(trace_id, |job| {
            mutator(&mut job.timings);
            refresh_follow_job_timing_rollups(job);
        })
        .await;
}

async fn update_action_timings(
    state: &Arc<AppState>,
    trace_id: &str,
    action_id: &str,
    mutator: impl FnOnce(&mut FollowActionTimings),
) {
    let _ = state
        .store
        .update_action(trace_id, action_id, |record| {
            mutator(&mut record.timings);
        })
        .await;
    let _ = state
        .store
        .update_job(trace_id, |job| refresh_follow_job_timing_rollups(job))
        .await;
}

async fn job_response(state: &Arc<AppState>, job: Option<FollowJobRecord>) -> FollowJobResponse {
    let health = build_health(state).await;
    let jobs = state.store.list_jobs().await;
    follow_job_response(health, job, jobs)
}

async fn spawn_follow_report_flush_task(state: Arc<AppState>, trace_id: String) {
    let mut tasks = state.follow_report_flush_tasks.lock().await;
    if tasks.contains_key(&trace_id) {
        return;
    }
    let task_state = state.clone();
    let task_trace_id = trace_id.clone();
    let handle = tokio::spawn(async move {
        loop {
            let delay_ms = {
                let pending = task_state.follow_report_flushes.lock().await;
                pending.get(&task_trace_id).map(|entry| entry.delay_ms)
            };
            let Some(delay_ms) = delay_ms else {
                break;
            };
            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }
            let pending = {
                let mut flushes = task_state.follow_report_flushes.lock().await;
                flushes.remove(&task_trace_id)
            };
            let Some(pending) = pending else {
                break;
            };
            let flush_started = Instant::now();
            let _report_guard = task_state.report_write_lock.lock().await;
            if update_persisted_follow_daemon_snapshot(&pending.report_path, &pending.snapshot)
                .is_ok()
            {
                let flush_ms = flush_started.elapsed().as_millis();
                update_job_timings(&task_state, &task_trace_id, |timings| {
                    add_time(&mut timings.followSnapshotFlushMs, flush_ms);
                })
                .await;
            }
            let has_more = {
                let pending = task_state.follow_report_flushes.lock().await;
                pending.contains_key(&task_trace_id)
            };
            if !has_more {
                break;
            }
        }
        let mut tasks = task_state.follow_report_flush_tasks.lock().await;
        tasks.remove(&task_trace_id);
    });
    tasks.insert(trace_id, handle);
}

async fn queue_follow_job_report_sync(
    state: &Arc<AppState>,
    trace_id: &str,
    mode: FollowReportSyncMode,
) {
    let sync_started = Instant::now();
    let Some(job) = get_job(state, trace_id).await else {
        return;
    };
    let Some(report_path) = job.reportPath.clone() else {
        return;
    };
    let health = build_health(state).await;
    let snapshot = json!({
        "schemaVersion": FOLLOW_RESPONSE_SCHEMA_VERSION,
        "transport": health.controlTransport,
        "job": job,
        "health": health,
    });
    update_job_timings(state, trace_id, |timings| {
        add_time(&mut timings.reportSyncMs, sync_started.elapsed().as_millis());
    })
    .await;
    let delay_ms = match mode {
        FollowReportSyncMode::Debounced => FOLLOW_REPORT_SYNC_DEBOUNCE_MS,
        FollowReportSyncMode::Immediate | FollowReportSyncMode::Final => 0,
    };
    {
        let mut flushes = state.follow_report_flushes.lock().await;
        flushes.insert(
            trace_id.to_string(),
            PendingFollowReportFlush {
                report_path,
                snapshot,
                delay_ms,
            },
        );
    }
    if !matches!(mode, FollowReportSyncMode::Debounced) {
        let existing = {
            let mut tasks = state.follow_report_flush_tasks.lock().await;
            tasks.remove(trace_id)
        };
        if let Some(handle) = existing {
            handle.abort();
        }
    }
    spawn_follow_report_flush_task(state.clone(), trace_id.to_string()).await;
}

async fn sync_follow_job_report(state: &Arc<AppState>, trace_id: &str) {
    queue_follow_job_report_sync(state, trace_id, FollowReportSyncMode::Debounced).await;
}

async fn sync_follow_job_report_immediate(state: &Arc<AppState>, trace_id: &str) {
    queue_follow_job_report_sync(state, trace_id, FollowReportSyncMode::Immediate).await;
}

async fn sync_follow_job_report_final(state: &Arc<AppState>, trace_id: &str) {
    queue_follow_job_report_sync(state, trace_id, FollowReportSyncMode::Final).await;
}

async fn get_job_watch_hub(state: &Arc<AppState>, trace_id: &str) -> Arc<JobWatchHub> {
    let mut hubs = state.watch_hubs.lock().await;
    if let Some(existing) = hubs.get(trace_id) {
        return existing.clone();
    }
    let (signature_tx, _) = watch::channel(None);
    let (market_tx, _) = watch::channel(None);
    let hub = Arc::new(JobWatchHub {
        signature_tx,
        market_tx,
        started: Mutex::new(JobWatchStarted::default()),
    });
    hubs.insert(trace_id.to_string(), hub.clone());
    hub
}

async fn remove_job_watch_hub(state: &Arc<AppState>, trace_id: &str) {
    let mut hubs = state.watch_hubs.lock().await;
    hubs.remove(trace_id);
}

async fn spawn_job_if_needed(state: Arc<AppState>, trace_id: String) {
    let mut active_jobs = state.active_jobs.lock().await;
    if active_jobs.contains_key(&trace_id) {
        return;
    }
    let runner_state = state.clone();
    let runner_trace = trace_id.clone();
    let handle = tokio::spawn(async move {
        run_job(runner_state.clone(), runner_trace.clone()).await;
        let mut active_jobs = runner_state.active_jobs.lock().await;
        active_jobs.remove(&runner_trace);
    });
    active_jobs.insert(trace_id, handle);
}

async fn restore_jobs(state: Arc<AppState>) {
    let _ = state.store.recover_jobs_for_restart().await;
    for job in state.store.list_jobs().await {
        if matches!(job.state, FollowJobState::Armed | FollowJobState::Running) {
            prepare_follow_job_buy_caches(state.clone(), &job).await;
            spawn_job_if_needed(state.clone(), job.traceId.clone()).await;
        }
    }
}

async fn run_job(state: Arc<AppState>, trace_id: String) {
    let Some(mut job) = get_job(&state, &trace_id).await else {
        return;
    };
    if !matches!(job.state, FollowJobState::Armed | FollowJobState::Running) {
        return;
    }
    let _ = state
        .store
        .finalize_job_state(&trace_id, FollowJobState::Running, None)
        .await;
    job = match get_job(&state, &trace_id).await {
        Some(job) => job,
        None => return,
    };
    let mut grouped_action_ids: Vec<Vec<String>> = Vec::new();
    let mut grouped_by_trigger: HashMap<String, Vec<FollowActionRecord>> = HashMap::new();
    for action in &job.actions {
        let key = action
            .triggerKey
            .clone()
            .unwrap_or_else(|| action.actionId.clone());
        grouped_by_trigger.entry(key).or_default().push(action.clone());
    }
    for (_, mut actions) in grouped_by_trigger {
        actions.sort_by_key(action_group_sort_key);
        grouped_action_ids.push(actions.into_iter().map(|action| action.actionId).collect());
    }
    let mut had_failure = false;
    let mut action_tasks = JoinSet::new();
    for action_ids in grouped_action_ids {
        let task_state = state.clone();
        let task_trace_id = trace_id.clone();
        if action_ids.len() == 1 {
            let action_id = action_ids.into_iter().next().unwrap_or_default();
            action_tasks
                .spawn(async move { run_action_task(task_state, task_trace_id, action_id).await });
        } else {
            action_tasks.spawn(async move {
                run_action_batch_task(task_state, task_trace_id, action_ids).await
            });
        }
    }
    if job.deferredSetup.is_some() {
        let setup_state = state.clone();
        let setup_trace_id = trace_id.clone();
        action_tasks.spawn(async move { run_deferred_setup_task(setup_state, setup_trace_id).await });
    }
    while let Some(result) = action_tasks.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(_error)) => {
                had_failure = true;
            }
            Err(_join_error) => {
                had_failure = true;
            }
        }
    }
    let Some(final_job) = get_job(&state, &trace_id).await else {
        return;
    };
    let has_failed_actions = final_job.actions.iter().any(|action| {
        matches!(
            action.state,
            FollowActionState::Failed | FollowActionState::Expired
        )
    });
    let has_confirmed_actions = final_job
        .actions
        .iter()
        .any(|action| matches!(action.state, FollowActionState::Confirmed));
    let setup_failed = final_job
        .deferredSetup
        .as_ref()
        .is_some_and(|setup| matches!(setup.state, DeferredSetupState::Failed));
    let setup_confirmed = final_job
        .deferredSetup
        .as_ref()
        .is_some_and(|setup| matches!(setup.state, DeferredSetupState::Confirmed));
    let final_state =
        if final_job.cancelRequested || matches!(final_job.state, FollowJobState::Cancelled) {
            FollowJobState::Cancelled
        } else if (has_failed_actions || setup_failed) && (has_confirmed_actions || setup_confirmed) {
            FollowJobState::CompletedWithFailures
        } else if had_failure || has_failed_actions || setup_failed {
            FollowJobState::Failed
        } else {
            FollowJobState::Completed
        };
    let _ = state
        .store
        .finalize_job_state(&trace_id, final_state, None)
        .await;
    sync_follow_job_report_final(&state, &trace_id).await;
    clear_follow_buy_caches(&state, &trace_id).await;
    remove_job_watch_hub(&state, &trace_id).await;
}

async fn run_action_task(
    state: Arc<AppState>,
    trace_id: String,
    action_id: String,
) -> Result<(), String> {
    execute_action_with_retry(state, trace_id, action_id).await
}

fn action_group_sort_key(action: &FollowActionRecord) -> (u8, u32, String) {
    let class_rank = match action.kind {
        FollowActionKind::DevAutoSell => 0,
        FollowActionKind::SniperBuy => 1,
        FollowActionKind::SniperSell => 2,
    };
    (class_rank, action.orderIndex, action.actionId.clone())
}

async fn execute_action_with_retry(
    state: Arc<AppState>,
    trace_id: String,
    action_id: String,
) -> Result<(), String> {
    loop {
        let Some(current_job) = get_job(&state, &trace_id).await else {
            return Ok(());
        };
        if current_job.cancelRequested || matches!(current_job.state, FollowJobState::Cancelled) {
            return Ok(());
        }
        let Some(action) = current_job
            .actions
            .iter()
            .find(|action| action.actionId == action_id)
            .cloned()
        else {
            return Ok(());
        };
        if matches!(
            action.state,
            FollowActionState::Confirmed
                | FollowActionState::Cancelled
                | FollowActionState::Failed
                | FollowActionState::Expired
                | FollowActionState::Sent
                | FollowActionState::Running
        ) {
            return Ok(());
        }
        if let Err(error) = execute_action(state.clone(), &current_job, &action).await {
            if should_rebuild_expired_presigned_action(&action, &error) {
                let retry_delay_ms = action_retry_backoff_ms(action.attemptCount);
                let _ = state
                    .store
                    .update_action(&trace_id, &action.actionId, |record| {
                        record.state = FollowActionState::Armed;
                        record.preSignedTransactions.clear();
                        record.lastError = Some(format!(
                            "Expired pre-signed dev-auto-sell payload; rebuilding with fresh blockhash in {}ms: {}",
                            retry_delay_ms, error
                        ));
                    })
                    .await;
                sync_follow_job_report(&state, &trace_id).await;
                sleep(Duration::from_millis(retry_delay_ms)).await;
                continue;
            }
            if is_expired_action_error(&error) {
                record_action_expired(&state, &current_job, &action, &error).await;
                return Ok(());
            }
            let Some(latest_job) = get_job(&state, &trace_id).await else {
                return Err(error);
            };
            let Some(latest_action) = latest_job
                .actions
                .iter()
                .find(|record| record.actionId == action.actionId)
                .cloned()
            else {
                return Err(error);
            };
            if should_rebuild_presigned_pump_buy_creator_vault_mismatch(
                &latest_job,
                &latest_action,
                &error,
            ) {
                let retry_delay_ms = action_retry_backoff_ms(latest_action.attemptCount);
                let _ = state
                    .store
                    .update_action(&trace_id, &action.actionId, |record| {
                        reset_action_for_presigned_rebuild(
                            record,
                            format!(
                                "Pump buy hit creator_vault mismatch; rebuilding with refreshed creator-vault state in {}ms: {}",
                                retry_delay_ms, error
                            ),
                        );
                    })
                    .await;
                sync_follow_job_report(&state, &trace_id).await;
                sleep(Duration::from_millis(retry_delay_ms)).await;
                continue;
            }
            if should_retry_pump_sell_creator_vault_mismatch(&latest_job, &latest_action, &error) {
                let retry_delay_ms = action_retry_backoff_ms(latest_action.attemptCount);
                let _ = state
                    .store
                    .update_action(&trace_id, &action.actionId, |record| {
                        reset_action_for_presigned_rebuild(
                            record,
                            format!(
                                "Pump sell hit creator_vault mismatch; rebuilding with refreshed creator-vault state in {}ms: {}",
                                retry_delay_ms, error
                            ),
                        );
                    })
                    .await;
                sync_follow_job_report(&state, &trace_id).await;
                sleep(Duration::from_millis(retry_delay_ms)).await;
                continue;
            }
            if should_rebuild_presigned_pump_sell_onchain_slippage(
                &latest_job,
                &latest_action,
                &error,
            ) {
                let retry_delay_ms = action_retry_backoff_ms(latest_action.attemptCount);
                let _ = state
                    .store
                    .update_action(&trace_id, &action.actionId, |record| {
                        reset_action_for_presigned_rebuild(
                            record,
                            format!(
                                "Pump pre-signed sell hit on-chain slippage; rebuilding with refreshed sell quote in {}ms: {}",
                                retry_delay_ms, error
                            ),
                        );
                    })
                    .await;
                sync_follow_job_report(&state, &trace_id).await;
                sleep(Duration::from_millis(retry_delay_ms)).await;
                continue;
            }
            if should_retry_action(&latest_job, &latest_action, &error) {
                let retry_delay_ms = action_retry_backoff_ms(latest_action.attemptCount);
                let _ = state
                    .store
                    .update_action(&trace_id, &action.actionId, |record| {
                        record.state = FollowActionState::Armed;
                        record.lastError = Some(format!(
                            "Retrying after attempt {} in {}ms: {}",
                            latest_action.attemptCount, retry_delay_ms, error
                        ));
                    })
                    .await;
                sync_follow_job_report(&state, &trace_id).await;
                sleep(Duration::from_millis(retry_delay_ms)).await;
                continue;
            }
            record_action_failure(&state, &current_job, &action, &error).await;
            return Err(error);
        }
        return Ok(());
    }
}

async fn run_action_batch_task(
    state: Arc<AppState>,
    trace_id: String,
    action_ids: Vec<String>,
) -> Result<(), String> {
    let Some(job) = get_job(&state, &trace_id).await else {
        return Ok(());
    };
    let mut actions = job
        .actions
        .iter()
        .filter(|action| action_ids.iter().any(|id| id == &action.actionId))
        .cloned()
        .collect::<Vec<_>>();
    if actions.is_empty() {
        return Ok(());
    }
    actions.sort_by_key(action_group_sort_key);
    let lead_action = actions[0].clone();
    let transport_plan = follow_action_transport_plan(&job, &lead_action);
    let eligibility_timing = wait_for_action_eligibility(state.clone(), &job, &lead_action).await?;
    for action in &actions {
        let _ = state
            .store
            .update_action(&trace_id, &action.actionId, |record| {
                if matches!(record.state, FollowActionState::Queued) {
                    record.state = FollowActionState::Armed;
                }
                record.state = FollowActionState::Eligible;
                record.provider = Some(transport_plan.resolvedProvider.clone());
                record.endpointProfile = Some(transport_plan.resolvedEndpointProfile.clone());
                record.transportType = Some(transport_plan.transportType.clone());
            })
            .await;
        update_action_timings(&state, &trace_id, &action.actionId, |timings| {
            timings.watcherWaitMs = Some(eligibility_timing.watcher_wait_ms);
            timings.eligibilityMs = Some(eligibility_timing.total_ms);
        })
        .await;
    }
    sync_follow_job_report(&state, &trace_id).await;
    let batch_tasks = actions.into_iter().map(|action| {
        let state = state.clone();
        let trace_id = trace_id.clone();
        async move { execute_action_with_retry(state, trace_id, action.actionId).await }
    });
    let _ = join_all(batch_tasks).await;
    Ok(())
}

async fn run_deferred_setup_task(state: Arc<AppState>, trace_id: String) -> Result<(), String> {
    loop {
        let Some(job) = get_job(&state, &trace_id).await else {
            return Ok(());
        };
        let Some(setup) = job.deferredSetup.clone() else {
            return Ok(());
        };
        if matches!(setup.state, DeferredSetupState::Confirmed | DeferredSetupState::Failed) {
            return Ok(());
        }
        wait_for_signature_confirmation(state.clone(), &job, "post-confirm-setup").await?;
        let blocking_follow = job
            .actions
            .iter()
            .any(|action| should_block_deferred_setup_for_action(action, now_ms()));
        if blocking_follow {
            sleep(Duration::from_millis(50)).await;
            continue;
        }
        return execute_deferred_setup(state.clone(), &job).await;
    }
}

async fn record_action_failure(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    error: &str,
) {
    let _ = state
        .store
        .update_action(&job.traceId, &action.actionId, |record| {
            record.state = FollowActionState::Failed;
            record.lastError = Some(error.to_string());
        })
        .await;
    sync_follow_job_report(state, &job.traceId).await;
}

async fn record_action_expired(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    reason: &str,
) {
    let _ = state
        .store
        .update_action(&job.traceId, &action.actionId, |record| {
            record.state = FollowActionState::Expired;
            record.lastError = Some(reason.to_string());
        })
        .await;
    sync_follow_job_report(state, &job.traceId).await;
}

fn is_pump_creator_vault_retry_error(error: &str) -> bool {
    is_creator_vault_seed_mismatch(error) || is_pump_custom_2006_seed_mismatch(error)
}

fn should_retry_action(job: &FollowJobRecord, action: &FollowActionRecord, error: &str) -> bool {
    if action.attemptCount > job.followLaunch.constraints.retryBudget {
        return false;
    }
    if is_expired_action_error(error) {
        return false;
    }
    if matches!(
        action.state,
        FollowActionState::Sent
            | FollowActionState::Confirmed
            | FollowActionState::Cancelled
            | FollowActionState::Expired
            | FollowActionState::Failed
    ) {
        return false;
    }
    is_retryable_action_error(error)
}

fn should_retry_pump_sell_creator_vault_mismatch(
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    error: &str,
) -> bool {
    if action.attemptCount > job.followLaunch.constraints.retryBudget {
        return false;
    }
    if job.launchpad != "pump" {
        return false;
    }
    if !matches!(
        action.kind,
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell
    ) {
        return false;
    }
    is_pump_creator_vault_retry_error(error)
}

fn should_rebuild_presigned_pump_buy_creator_vault_mismatch(
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    error: &str,
) -> bool {
    if action.attemptCount > job.followLaunch.constraints.retryBudget {
        return false;
    }
    if job.launchpad != "pump" {
        return false;
    }
    if !matches!(action.kind, FollowActionKind::SniperBuy) {
        return false;
    }
    if action.preSignedTransactions.is_empty() {
        return false;
    }
    is_pump_creator_vault_retry_error(error)
}

fn is_pump_custom_6003_slippage(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("instructionerror")
        && (normalized.contains("\"custom\":6003")
            || normalized.contains("custom:6003")
            || normalized.contains("custom: 6003"))
}

fn should_rebuild_presigned_pump_sell_onchain_slippage(
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    error: &str,
) -> bool {
    if action.attemptCount > job.followLaunch.constraints.retryBudget {
        return false;
    }
    if job.launchpad != "pump" {
        return false;
    }
    if !matches!(
        action.kind,
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell
    ) {
        return false;
    }
    if action.preSignedTransactions.is_empty() {
        return false;
    }
    is_pump_custom_6003_slippage(error)
}

fn is_retryable_action_error(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    if normalized.contains("cancel")
        || normalized.contains("insufficient funds")
        || normalized.contains("wallet env key not found")
        || normalized.contains("wallet env var is missing")
        || normalized.contains("missing mint")
        || normalized.contains("missing launch creator")
        || normalized.contains("missing transport plan")
        || normalized.contains("had nothing to send")
        || normalized.contains("invalid")
        || normalized.contains("precheck")
    {
        return false;
    }
    normalized.contains("timeout")
        || normalized.contains("timed out")
        || normalized.contains("websocket")
        || normalized.contains("connection")
        || normalized.contains("tempor")
        || normalized.contains("was not found")
        || normalized.contains("capacity")
        || normalized.contains("provider")
        || normalized.contains("rpc")
        || normalized.contains("429")
        || normalized.contains("too many requests")
        || normalized.contains("unavailable")
}

fn action_retry_backoff_ms(attempt_count: u32) -> u64 {
    let multiplier = 1u64 << attempt_count.min(4);
    (150u64.saturating_mul(multiplier)).min(5_000)
}

fn market_cap_scan_expired_error(action_id: &str, scan_timeout_seconds: u64) -> String {
    format!(
        "__expired__ market-cap scan window elapsed for {action_id} after {scan_timeout_seconds} second(s)."
    )
}

fn is_expired_action_error(error: &str) -> bool {
    error.trim().starts_with("__expired__")
}

fn should_rebuild_expired_presigned_action(action: &FollowActionRecord, error: &str) -> bool {
    matches!(action.kind, FollowActionKind::DevAutoSell)
        && !action.preSignedTransactions.is_empty()
        && is_expired_action_error(error)
}

fn action_waiting_for_presigned_rebuild(action: &FollowActionRecord) -> bool {
    if action.preSignedTransactions.is_empty()
        && matches!(
            action.state,
            FollowActionState::Queued
                | FollowActionState::Armed
                | FollowActionState::Eligible
                | FollowActionState::Running
        )
    {
        let normalized = action
            .lastError
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        return normalized.contains("rebuilding with refreshed creator-vault state")
            || normalized.contains("rebuilding with refreshed sell quote");
    }
    false
}

fn reset_action_for_presigned_rebuild(record: &mut FollowActionRecord, message: String) {
    record.state = FollowActionState::Armed;
    record.preSignedTransactions.clear();
    record.submitStartedAtMs = None;
    record.submittedAtMs = None;
    record.confirmedAtMs = None;
    record.sendObservedBlockHeight = None;
    record.confirmedObservedBlockHeight = None;
    record.blocksToConfirm = None;
    record.signature = None;
    record.explorerUrl = None;
    record.endpoint = None;
    record.bundleId = None;
    record.lastError = Some(message);
}

fn should_block_deferred_setup_for_action(action: &FollowActionRecord, now_ms: u128) -> bool {
    if action.marketCap.is_some() {
        return false;
    }
    if !matches!(
        action.state,
        FollowActionState::Queued
            | FollowActionState::Armed
            | FollowActionState::Eligible
            | FollowActionState::Running
    ) {
        return false;
    }
    if action_waiting_for_presigned_rebuild(action) {
        return false;
    }
    if action.requireConfirmation {
        return true;
    }
    if let Some(offset) = action.targetBlockOffset {
        return offset <= 2;
    }
    action
        .scheduledForMs
        .is_some_and(|scheduled_for_ms| scheduled_for_ms <= now_ms.saturating_add(1_500))
}

async fn execute_action(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Result<(), String> {
    let trace_id = job.traceId.clone();
    let transport_plan = follow_action_transport_plan(job, action);
    if !matches!(action.state, FollowActionState::Eligible) {
        ensure_action_not_cancelled(&state, &trace_id, &action.actionId).await?;
        state
            .store
            .update_action(&trace_id, &action.actionId, |record| {
                if matches!(record.state, FollowActionState::Queued) {
                    record.state = FollowActionState::Armed;
                }
            })
            .await?;
        sync_follow_job_report(&state, &trace_id).await;
        let eligibility_timing = wait_for_action_eligibility(state.clone(), job, action).await?;
        update_action_timings(&state, &trace_id, &action.actionId, |timings| {
            timings.watcherWaitMs = Some(eligibility_timing.watcher_wait_ms);
            timings.eligibilityMs = Some(eligibility_timing.total_ms);
        })
        .await;
        ensure_action_not_cancelled(&state, &trace_id, &action.actionId).await?;
        state
            .store
            .update_action(&trace_id, &action.actionId, |record| {
                record.state = FollowActionState::Eligible;
                record.provider = Some(transport_plan.resolvedProvider.clone());
                record.endpointProfile = Some(transport_plan.resolvedEndpointProfile.clone());
                record.transportType = Some(transport_plan.transportType.clone());
            })
            .await?;
        sync_follow_job_report(&state, &trace_id).await;
    }
    let wallet_key = selected_wallet_key_or_default(&action.walletEnvKey)
        .ok_or_else(|| format!("Wallet env key not found: {}", action.walletEnvKey))?;
    let wallet_secret = load_solana_wallet_by_env_key(&wallet_key)?;
    if action.precheckRequired {
        required_action_precheck(&state.rpc_url, &job.quoteAsset, action).await?;
    }
    ensure_action_not_cancelled(&state, &trace_id, &action.actionId).await?;
    let wallet_lock = wallet_lock(&state, &wallet_key).await;
    let _wallet_guard = wallet_lock.lock().await;
    state
        .store
        .update_action(&trace_id, &action.actionId, |record| {
            record.state = FollowActionState::Running;
            record.attemptCount = record.attemptCount.saturating_add(1);
            record.submitStartedAtMs = Some(now_ms());
        })
        .await?;
    sync_follow_job_report(&state, &trace_id).await;
    let mint = job
        .mint
        .as_deref()
        .ok_or_else(|| "Follow job missing mint.".to_string())?;
    if skip_retry_if_wallet_already_holds_token(&state, job, action, &wallet_key, mint).await? {
        return Ok(());
    }
    let launch_creator = job
        .launchCreator
        .as_deref()
        .ok_or_else(|| "Follow job missing launch creator.".to_string())?;
    let compile_started = Instant::now();
    let prefer_post_setup_creator_vault_for_buy = should_use_post_setup_creator_vault_for_buy(
        job.preferPostSetupCreatorVaultForSell,
        action,
    );
    let prefer_post_setup_creator_vault_for_sell = should_use_post_setup_creator_vault_for_sell(
        job.preferPostSetupCreatorVaultForSell,
        action,
    );
    let sell_percent = match action.kind {
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => Some(
            action
                .sellPercent
                .ok_or_else(|| "Follow sell missing percent.".to_string())?,
        ),
        FollowActionKind::SniperBuy => None,
    };
    let compiled_from_presign = !action.preSignedTransactions.is_empty();
    let compiled = if let Some(tx) = action.preSignedTransactions.first().cloned() {
        let current_block_height = current_shared_block_height(state.clone(), &trace_id).await?;
        if current_block_height > tx.lastValidBlockHeight {
            return Err(format!(
                "__expired__ pre-signed payload for {} expired at block height {} before send.",
                action.actionId, tx.lastValidBlockHeight
            ));
        }
        Some(tx)
    } else {
        let _compile_permit = acquire_capacity_slot(
            state.compile_slots.clone(),
            state.capacity_wait_ms,
            "compile",
        )
        .await?;
        match action.kind {
        FollowActionKind::SniperBuy => {
            let amount = action
                .buyAmountSol
                .as_deref()
                .ok_or_else(|| "Follow buy missing amount.".to_string())?;
            if job.launchpad == "bonk" {
                Some(
                    if job.quoteAsset == "usd1" {
                        compile_atomic_follow_buy_for_launchpad(
                            &job.launchpad,
                            &job.launchMode,
                            &job.quoteAsset,
                            &state.rpc_url,
                            &job.execution,
                            job.tokenMayhemMode,
                            &job.buyTipAccount,
                            &wallet_secret,
                            mint,
                            launch_creator,
                            amount,
                            true,
                        )
                        .await?
                    } else {
                        compile_bonk_follow_buy_transaction(
                            &state.rpc_url,
                            &job.quoteAsset,
                            &job.execution,
                            job.tokenMayhemMode,
                            &job.buyTipAccount,
                            &wallet_secret,
                            mint,
                            launch_creator,
                            amount,
                            true,
                        )
                        .await?
                    }
                )
            } else if job.launchpad == "bagsapp" {
                Some(
                    compile_bags_follow_buy_transaction(
                        &state.rpc_url,
                        &job.execution,
                        job.tokenMayhemMode,
                        &job.buyTipAccount,
                        &wallet_secret,
                        mint,
                        launch_creator,
                        amount,
                    )
                    .await?,
                )
            } else {
                let prepared = resolve_prepared_follow_buy(
                    &state,
                    job,
                    action,
                    &wallet_secret,
                    launch_creator,
                    amount,
                )
                .await?;
                let runtime =
                    resolve_hot_follow_buy_runtime_for_job(
                        &state,
                        job,
                        launch_creator,
                        prefer_post_setup_creator_vault_for_buy,
                    )
                    .await?;
                Some(
                    finalize_follow_buy_transaction(
                        &state.rpc_url,
                        &job.execution,
                        job.tokenMayhemMode,
                        &wallet_secret,
                        &prepared,
                        &runtime,
                    )
                    .await?,
                )
            }
        }
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => {
            if job.launchpad == "bonk" {
                compile_bonk_follow_sell_transaction_with_token_amount(
                    &state.rpc_url,
                    &job.quoteAsset,
                    &job.execution,
                    &job.sellTipAccount,
                    &wallet_secret,
                    mint,
                    sell_percent.unwrap_or_default(),
                    None,
                    action.poolId.as_deref(),
                    None,
                    None,
                )
                .await?
            } else if job.launchpad == "bagsapp" {
                compile_bags_follow_sell_transaction(
                    &state.rpc_url,
                    &job.execution,
                    job.tokenMayhemMode,
                    &job.sellTipAccount,
                    &wallet_secret,
                    mint,
                    launch_creator,
                    sell_percent.unwrap_or_default(),
                    prefer_post_setup_creator_vault_for_sell,
                )
                .await?
            } else {
                compile_follow_sell_transaction(
                    &state.rpc_url,
                    &job.execution,
                    job.tokenMayhemMode,
                    &job.sellTipAccount,
                    &wallet_secret,
                    mint,
                    launch_creator,
                    sell_percent.unwrap_or_default(),
                    prefer_post_setup_creator_vault_for_sell,
                )
                .await?
            }
        }
        }
    };
    let Some(mut compiled) = compiled else {
        return Err("Action had nothing to send for the current wallet state.".to_string());
    };
    let compile_ms = compile_started.elapsed().as_millis();
    update_action_timings(&state, &trace_id, &action.actionId, |timings| {
        timings.compileMs = Some(if compiled_from_presign { 0 } else { compile_ms });
    })
    .await;
    ensure_action_not_cancelled(&state, &trace_id, &action.actionId).await?;
    let _send_permit =
        acquire_capacity_slot(state.send_slots.clone(), state.capacity_wait_ms, "send").await?;
    let mut retried_creator_vault = false;
    let (mut submitted, mut warnings, submit_ms, submit_latency) = loop {
        let started = Instant::now();
        match submit_transactions_for_transport(
            &state.rpc_url,
            &transport_plan,
            &[compiled.clone()],
            &job.execution.commitment,
            job.execution.skipPreflight,
            job.execution.trackSendBlockHeight,
        )
        .await
        {
            Ok((submitted, warnings, submit_ms)) => {
                break (
                    submitted,
                    warnings,
                    submit_ms,
                    started.elapsed().as_millis(),
                );
            }
            Err(error)
                if !retried_creator_vault
                    && sell_percent.is_some()
                    && should_retry_pump_sell_creator_vault_mismatch(job, action, &error) =>
            {
                retried_creator_vault = true;
                sleep(Duration::from_millis(200)).await;
                compiled = compile_follow_sell_transaction(
                    &state.rpc_url,
                    &job.execution,
                    job.tokenMayhemMode,
                    &job.sellTipAccount,
                    &wallet_secret,
                    mint,
                    launch_creator,
                    sell_percent.unwrap_or_default(),
                    prefer_post_setup_creator_vault_for_sell,
                )
                .await?
                .ok_or_else(|| {
                    "Action had nothing to send after creator vault retry.".to_string()
                })?;
            }
            Err(error) => return Err(error),
        }
    };
    if retried_creator_vault {
        warnings.push(
            "Retried sell after creator_vault seed mismatch; refreshed fee-vault authority."
                .to_string(),
        );
    }
    let sent = submitted
        .pop()
        .ok_or_else(|| "Follow daemon submit returned no transactions.".to_string())?;
    update_action_timings(&state, &trace_id, &action.actionId, |timings| {
        timings.submitMs = Some(submit_latency.max(submit_ms));
    })
    .await;
    state
        .store
        .update_action(&trace_id, &action.actionId, |record| {
            record.state = FollowActionState::Sent;
            record.submittedAtMs = Some(now_ms());
            record.sendObservedBlockHeight = sent.sendObservedBlockHeight;
            record.signature = sent.signature.clone();
            record.explorerUrl = sent.explorerUrl.clone();
            record.endpoint = sent.endpoint.clone();
            record.bundleId = sent.bundleId.clone();
            record.lastError = if warnings.is_empty() {
                None
            } else {
                Some(warnings.join(" | "))
            };
        })
        .await?;
    sync_follow_job_report(&state, &trace_id).await;
    let mut confirm_input = vec![sent.clone()];
    let (_, confirm_ms) = confirm_submitted_transactions_for_transport(
        &state.rpc_url,
        &transport_plan,
        &mut confirm_input,
        &job.execution.commitment,
        job.execution.trackSendBlockHeight,
    )
    .await?;
    update_action_timings(&state, &trace_id, &action.actionId, |timings| {
        timings.confirmMs = Some(confirm_ms);
        timings.executionTotalMs = Some(
            timings
                .eligibilityMs
                .unwrap_or_default()
                .saturating_add(timings.compileMs.unwrap_or_default())
                .saturating_add(timings.submitMs.unwrap_or_default())
                .saturating_add(confirm_ms),
        );
    })
    .await;
    let confirmed = confirm_input.pop().unwrap_or(sent);
    state
        .store
        .update_action(&trace_id, &action.actionId, |record| {
            record.state = FollowActionState::Confirmed;
            record.confirmedAtMs = Some(now_ms());
            record.confirmedObservedBlockHeight = confirmed.confirmedObservedBlockHeight;
            record.blocksToConfirm = match (
                record.sendObservedBlockHeight,
                confirmed.confirmedObservedBlockHeight,
            ) {
                (Some(send_block), Some(confirm_block)) if confirm_block >= send_block => {
                    Some(confirm_block - send_block)
                }
                _ => None,
            };
            if record.signature.is_none() {
                record.signature = confirmed.signature.clone();
            }
            if record.explorerUrl.is_none() {
                record.explorerUrl = confirmed.explorerUrl.clone();
            }
            if record.endpoint.is_none() {
                record.endpoint = confirmed.endpoint.clone();
            }
            if record.bundleId.is_none() {
                record.bundleId = confirmed.bundleId.clone();
            }
        })
        .await?;
    sync_follow_job_report(&state, &trace_id).await;
    Ok(())
}

async fn execute_deferred_setup(state: Arc<AppState>, job: &FollowJobRecord) -> Result<(), String> {
    let Some(setup) = job.deferredSetup.clone() else {
        return Ok(());
    };
    let Some(transport_plan) = job.transportPlan.clone() else {
        return Err("Follow job missing transport plan for deferred setup.".to_string());
    };
    let trace_id = job.traceId.clone();
    let mut attempt = setup.attemptCount;
    loop {
        let _ = state.store.update_job(&trace_id, |record| {
            if let Some(setup) = record.deferredSetup.as_mut() {
                setup.state = DeferredSetupState::Running;
                setup.attemptCount = setup.attemptCount.saturating_add(1);
            }
        }).await;
        sync_follow_job_report(&state, &trace_id).await;
        let _send_permit =
            acquire_capacity_slot(state.send_slots.clone(), state.capacity_wait_ms, "send").await?;
        match submit_transactions_for_transport(
            &state.rpc_url,
            &transport_plan,
            &setup.transactions,
            &job.execution.commitment,
            job.execution.skipPreflight,
            job.execution.trackSendBlockHeight,
        )
        .await
        {
            Ok((mut submitted, warnings, _submit_ms)) => {
                let signatures = submitted
                    .iter()
                    .filter_map(|entry| entry.signature.clone())
                    .collect::<Vec<_>>();
                let _ = state.store.update_job(&trace_id, |record| {
                    if let Some(setup) = record.deferredSetup.as_mut() {
                        setup.state = DeferredSetupState::Sent;
                        setup.submittedAtMs = Some(now_ms());
                        setup.signatures = signatures.clone();
                        setup.lastError = if warnings.is_empty() {
                            None
                        } else {
                            Some(warnings.join(" | "))
                        };
                    }
                }).await;
                let _ = confirm_submitted_transactions_for_transport(
                    &state.rpc_url,
                    &transport_plan,
                    &mut submitted,
                    &job.execution.commitment,
                    job.execution.trackSendBlockHeight,
                )
                .await?;
                let _ = state.store.update_job(&trace_id, |record| {
                    if let Some(setup) = record.deferredSetup.as_mut() {
                        setup.state = DeferredSetupState::Confirmed;
                        setup.confirmedAtMs = Some(now_ms());
                    }
                }).await;
                sync_follow_job_report(&state, &trace_id).await;
                return Ok(());
            }
            Err(error) if attempt < 2 && is_retryable_action_error(&error) => {
                attempt = attempt.saturating_add(1);
                let _ = state.store.update_job(&trace_id, |record| {
                    if let Some(setup) = record.deferredSetup.as_mut() {
                        setup.state = DeferredSetupState::Queued;
                        setup.lastError = Some(error.clone());
                    }
                }).await;
                sleep(Duration::from_millis(action_retry_backoff_ms(attempt))).await;
            }
            Err(error) => {
                let _ = state.store.update_job(&trace_id, |record| {
                    if let Some(setup) = record.deferredSetup.as_mut() {
                        setup.state = DeferredSetupState::Failed;
                        setup.lastError = Some(error.clone());
                    }
                }).await;
                sync_follow_job_report(&state, &trace_id).await;
                return Err(error);
            }
        }
    }
}

fn follow_action_execution(job: &FollowJobRecord, action: &FollowActionRecord) -> launchdeck_engine::config::NormalizedExecution {
    let mut execution = job.execution.clone();
    match action.kind {
        FollowActionKind::SniperBuy => {
            execution.provider = job.execution.buyProvider.clone();
            execution.endpointProfile = job.execution.buyEndpointProfile.clone();
        }
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => {
            execution.provider = job.execution.sellProvider.clone();
            execution.endpointProfile = job.execution.sellEndpointProfile.clone();
        }
    }
    execution
}

fn follow_action_transport_plan(job: &FollowJobRecord, action: &FollowActionRecord) -> TransportPlan {
    build_transport_plan(&follow_action_execution(job, action), 1)
}

fn is_creator_vault_seed_mismatch(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("creator_vault")
        && (normalized.contains("constraintseeds")
            || normalized.contains("seeds constraint was violated"))
}

fn is_pump_custom_2006_seed_mismatch(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("instructionerror")
        && (normalized.contains("\"custom\":2006")
            || normalized.contains("custom:2006")
            || normalized.contains("custom: 2006"))
}

#[derive(Default)]
struct EligibilityTiming {
    watcher_wait_ms: u128,
    total_ms: u128,
}

async fn wallet_lock(state: &Arc<AppState>, wallet_key: &str) -> Arc<Mutex<()>> {
    let mut locks = state.wallet_locks.lock().await;
    locks
        .entry(wallet_key.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

async fn wait_for_action_eligibility(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Result<EligibilityTiming, String> {
    let eligibility_started = Instant::now();
    let mut watcher_wait_ms = 0u128;
    let confirmed_launch_block_height =
        if action.requireConfirmation || action.targetBlockOffset.is_some() {
            let wait_started = Instant::now();
            let confirmed =
                wait_for_signature_confirmation(state.clone(), job, &action.actionId).await?;
            watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            Some(confirmed)
        } else {
            None
        };
    match action.kind {
        FollowActionKind::SniperBuy => {
            if let Some(schedule_ms) = action.scheduledForMs {
                wait_until_ms(
                    state.clone(),
                    &job.traceId,
                    Some(&action.actionId),
                    schedule_ms,
                )
                .await?;
            }
            if action.targetBlockOffset.unwrap_or_default() > 0 {
                let wait_started = Instant::now();
                wait_for_slot_offset(
                    state.clone(),
                    job,
                    &action.actionId,
                    confirmed_launch_block_height,
                    u64::from(action.targetBlockOffset.unwrap()),
                )
                .await?;
                watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            }
        }
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => {
            let has_time = action.scheduledForMs.is_some();
            let has_market = action.marketCap.is_some();
            let has_slot = action.targetBlockOffset.unwrap_or_default() > 0;
            if has_slot && has_market {
                let wait_started = Instant::now();
                tokio::select! {
                    result = wait_for_slot_offset(
                        state.clone(),
                        job,
                        &action.actionId,
                        confirmed_launch_block_height,
                        u64::from(action.targetBlockOffset.unwrap()),
                    ) => result?,
                    result = wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId) => result?,
                }
                watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            } else if has_slot {
                let wait_started = Instant::now();
                wait_for_slot_offset(
                    state.clone(),
                    job,
                    &action.actionId,
                    confirmed_launch_block_height,
                    u64::from(action.targetBlockOffset.unwrap()),
                )
                .await?;
                watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            } else if has_time && has_market {
                let wait_started = Instant::now();
                tokio::select! {
                    result = wait_until_ms(state.clone(), &job.traceId, Some(&action.actionId), action.scheduledForMs.unwrap()) => result?,
                    result = wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId) => result?,
                }
                watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            } else if let Some(schedule_ms) = action.scheduledForMs {
                wait_until_ms(
                    state.clone(),
                    &job.traceId,
                    Some(&action.actionId),
                    schedule_ms,
                )
                .await?;
            } else if has_market {
                let wait_started = Instant::now();
                wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId).await?;
                watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            }
        }
    }
    Ok(EligibilityTiming {
        watcher_wait_ms,
        total_ms: eligibility_started.elapsed().as_millis(),
    })
}

async fn wait_until_ms(
    state: Arc<AppState>,
    trace_id: &str,
    action_id: Option<&str>,
    target_ms: u128,
) -> Result<(), String> {
    loop {
        if let Some(action_id) = action_id {
            ensure_action_not_cancelled(&state, trace_id, action_id).await?;
        } else {
            ensure_job_not_cancelled(&state, trace_id).await?;
        }
        let now = now_ms();
        if now >= target_ms {
            return Ok(());
        }
        let sleep_ms = (target_ms - now).min(50) as u64;
        sleep(Duration::from_millis(sleep_ms)).await;
    }
}

async fn ensure_job_not_cancelled(state: &Arc<AppState>, trace_id: &str) -> Result<(), String> {
    let Some(job) = get_job(state, trace_id).await else {
        return Err("Follow job disappeared while waiting.".to_string());
    };
    if job.cancelRequested || matches!(job.state, FollowJobState::Cancelled) {
        Err("Follow job was cancelled.".to_string())
    } else {
        Ok(())
    }
}

async fn ensure_action_not_cancelled(
    state: &Arc<AppState>,
    trace_id: &str,
    action_id: &str,
) -> Result<(), String> {
    let Some(job) = get_job(state, trace_id).await else {
        return Err("Follow job disappeared while waiting.".to_string());
    };
    if job.cancelRequested || matches!(job.state, FollowJobState::Cancelled) {
        return Err("Follow job was cancelled.".to_string());
    }
    let Some(action) = job
        .actions
        .iter()
        .find(|action| action.actionId == action_id)
    else {
        return Err(format!(
            "Follow action disappeared while waiting: {action_id}"
        ));
    };
    if matches!(action.state, FollowActionState::Cancelled) {
        Err(format!("Follow action was cancelled: {action_id}"))
    } else {
        Ok(())
    }
}

async fn handle_watcher_retry(
    state: &Arc<AppState>,
    trace_id: &str,
    kind: WatcherKind,
    endpoint: &str,
    attempt: u32,
    error: String,
) -> Result<(), String> {
    ensure_job_not_cancelled(state, trace_id).await?;
    let current = state.store.health().await;
    let current_mode = match kind {
        WatcherKind::Slot => current.slotWatcherMode.clone(),
        WatcherKind::Signature => current.signatureWatcherMode.clone(),
        WatcherKind::Market => current.marketWatcherMode.clone(),
    };
    if attempt >= WATCHER_MAX_RECONNECT_ATTEMPTS {
        set_watcher_health(
            state,
            kind,
            FollowWatcherHealth::Failed,
            current_mode.clone(),
            Some(endpoint.to_string()),
            Some(error.clone()),
        )
        .await;
        return Err(error);
    }
    set_watcher_health(
        state,
        kind,
        FollowWatcherHealth::Degraded,
        current_mode,
        Some(endpoint.to_string()),
        Some(error),
    )
    .await;
    sleep(Duration::from_millis(watcher_backoff_ms(attempt))).await;
    Ok(())
}

async fn ensure_signature_watcher(
    state: Arc<AppState>,
    job: &FollowJobRecord,
) -> watch::Receiver<Option<Result<u64, String>>> {
    let hub = get_job_watch_hub(&state, &job.traceId).await;
    let mut started = hub.started.lock().await;
    if !started.signature {
        started.signature = true;
        let task_state = state.clone();
        let task_job = job.clone();
        let task_tx = hub.signature_tx.clone();
        tokio::spawn(async move {
            run_signature_watcher(task_state, task_job, task_tx).await;
        });
    }
    drop(started);
    hub.signature_tx.subscribe()
}

fn offset_consumer_key(trace_id: &str, action_id: &str) -> String {
    format!("{trace_id}:{action_id}")
}

async fn current_shared_block_height(state: Arc<AppState>, _trace_id: &str) -> Result<u64, String> {
    fetch_current_block_height_fresh(&state.rpc_url, "confirmed").await
}

async fn register_offset_consumer(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action_id: &str,
    base_confirmed_block_height: u64,
    target_block_offset: u64,
    confirmation_detected_at_ms: u128,
) -> watch::Receiver<Option<Result<(), String>>> {
    let key = offset_consumer_key(&job.traceId, action_id);
    let (completion_tx, rx) = watch::channel(None);
    let consumer = OffsetConsumer {
        trace_id: job.traceId.clone(),
        action_id: action_id.to_string(),
        base_confirmed_block_height,
        confirmation_detected_at_ms,
        target_block_offset,
        completion_tx,
    };
    {
        let current_height = *state.offset_worker.last_observed_block_height.lock().await;
        if current_height.is_some_and(|height| {
            height >= base_confirmed_block_height.saturating_add(target_block_offset)
        }) {
            let _ = consumer.completion_tx.send(Some(Ok(())));
            return rx;
        }
    }
    let mut should_start = false;
    {
        let mut consumers = state.offset_worker.consumers.lock().await;
        consumers.insert(key, consumer);
    }
    {
        let mut running = state.offset_worker.running.lock().await;
        if !*running {
            *running = true;
            should_start = true;
        }
    }
    if should_start {
        let task_state = state.clone();
        tokio::spawn(async move {
            run_offset_worker(task_state).await;
        });
    }
    let approximate_mode = configured_enable_approximate_follow_offset_timer();
    set_watcher_health(
        &state,
        WatcherKind::Slot,
        FollowWatcherHealth::Healthy,
        Some(if approximate_mode {
            "approx-local-timer".to_string()
        } else {
            "real-blockheight".to_string()
        }),
        if approximate_mode {
            None
        } else {
            Some(state.rpc_url.clone())
        },
        None,
    )
    .await;
    rx
}

async fn unregister_offset_consumer(state: &Arc<AppState>, trace_id: &str, action_id: &str) {
    let key = offset_consumer_key(trace_id, action_id);
    let mut consumers = state.offset_worker.consumers.lock().await;
    consumers.remove(&key);
}

async fn current_offset_consumers(state: &Arc<AppState>) -> Vec<(String, u64)> {
    let consumers = state.offset_worker.consumers.lock().await;
    consumers
        .values()
        .map(|consumer| {
            (
                offset_consumer_key(&consumer.trace_id, &consumer.action_id),
                consumer
                    .base_confirmed_block_height
                    .saturating_add(consumer.target_block_offset),
            )
        })
        .collect()
}

async fn compute_offset_poll_delay_ms(state: &Arc<AppState>, interval_ms: u64) -> u64 {
    let now = now_ms();
    let last_observed_at_ms = *state.offset_worker.last_observed_at_ms.lock().await;
    if let Some(last_observed_at_ms) = last_observed_at_ms {
        let elapsed_ms = now.saturating_sub(last_observed_at_ms) as u64;
        let conservative_lost_ms = elapsed_ms.saturating_mul(3) / 4;
        return interval_ms
            .saturating_sub(conservative_lost_ms.min(interval_ms.saturating_sub(100)))
            .max(100);
    }
    let earliest_confirmation_detected_at_ms = {
        let consumers = state.offset_worker.consumers.lock().await;
        consumers
            .values()
            .map(|consumer| consumer.confirmation_detected_at_ms)
            .min()
    };
    let Some(earliest_confirmation_detected_at_ms) = earliest_confirmation_detected_at_ms else {
        return interval_ms;
    };
    let elapsed_ms = now.saturating_sub(earliest_confirmation_detected_at_ms) as u64;
    let conservative_lost_ms = (elapsed_ms / 2).min(interval_ms.saturating_sub(100));
    interval_ms.saturating_sub(conservative_lost_ms).max(100)
}

async fn complete_offset_consumers_for_approximate_tick(
    state: &Arc<AppState>,
    interval_ms: u64,
    correction_ms: u64,
) {
    let now = now_ms();
    let completed = {
        let consumers = state.offset_worker.consumers.lock().await;
        consumers
            .iter()
            .filter_map(|(key, consumer)| {
                let elapsed_ms = now.saturating_sub(consumer.confirmation_detected_at_ms) as u64;
                let adjusted_elapsed_ms = elapsed_ms.saturating_add(correction_ms);
                let estimated_blocks_elapsed = adjusted_elapsed_ms / interval_ms;
                (estimated_blocks_elapsed >= consumer.target_block_offset).then(|| {
                    (key.clone(), consumer.completion_tx.clone())
                })
            })
            .collect::<Vec<_>>()
    };
    if completed.is_empty() {
        return;
    }
    {
        let mut consumers = state.offset_worker.consumers.lock().await;
        for (key, _) in &completed {
            consumers.remove(key);
        }
    }
    for (_, tx) in completed {
        let _ = tx.send(Some(Ok(())));
    }
}

async fn complete_offset_consumers_for_height(state: &Arc<AppState>, block_height: u64) {
    let completed = {
        let consumers = state.offset_worker.consumers.lock().await;
        consumers
            .iter()
            .filter_map(|(key, consumer)| {
                let target_height = consumer
                    .base_confirmed_block_height
                    .saturating_add(consumer.target_block_offset);
                (block_height >= target_height).then(|| (key.clone(), consumer.completion_tx.clone()))
            })
            .collect::<Vec<_>>()
    };
    if completed.is_empty() {
        return;
    }
    {
        let mut consumers = state.offset_worker.consumers.lock().await;
        for (key, _) in &completed {
            consumers.remove(key);
        }
    }
    for (_, tx) in completed {
        let _ = tx.send(Some(Ok(())));
    }
}

async fn run_offset_worker(state: Arc<AppState>) {
    let interval_ms = configured_follow_offset_poll_interval_ms();
    let approximate_mode = configured_enable_approximate_follow_offset_timer();
    let mut delay_ms = compute_offset_poll_delay_ms(&state, interval_ms).await;
    let mut next_tick_correction_ms = interval_ms.saturating_sub(delay_ms);
    loop {
        if current_offset_consumers(&state).await.is_empty() {
            {
                let mut running = state.offset_worker.running.lock().await;
                *running = false;
            }
            if current_offset_consumers(&state).await.is_empty() {
                return;
            }
            let mut running = state.offset_worker.running.lock().await;
            if !*running {
                *running = true;
            }
        }
        sleep(Duration::from_millis(delay_ms)).await;
        if approximate_mode {
            set_watcher_health(
                &state,
                WatcherKind::Slot,
                FollowWatcherHealth::Healthy,
                Some("approx-local-timer".to_string()),
                None,
                None,
            )
            .await;
            complete_offset_consumers_for_approximate_tick(
                &state,
                interval_ms,
                next_tick_correction_ms,
            )
                .await;
            next_tick_correction_ms = 0;
            delay_ms = interval_ms;
            continue;
        }
        let mut repolls_remaining = OFFSET_SAME_BLOCK_REPOLL_LIMIT;
        loop {
            let result = fetch_current_block_height_fresh(&state.rpc_url, "confirmed").await;
            match result {
                Ok(block_height) => {
                    let previous_block_height =
                        { *state.offset_worker.last_observed_block_height.lock().await };
                    {
                        let mut last_height = state.offset_worker.last_observed_block_height.lock().await;
                        *last_height = Some(block_height);
                    }
                    {
                        let mut last_observed_at = state.offset_worker.last_observed_at_ms.lock().await;
                        *last_observed_at = Some(now_ms());
                    }
                    set_watcher_health(
                        &state,
                        WatcherKind::Slot,
                        FollowWatcherHealth::Healthy,
                        Some("real-blockheight".to_string()),
                        Some(state.rpc_url.clone()),
                        None,
                    )
                    .await;
                    complete_offset_consumers_for_height(&state, block_height).await;
                    if previous_block_height == Some(block_height) && repolls_remaining > 0 {
                        repolls_remaining = repolls_remaining.saturating_sub(1);
                        sleep(Duration::from_millis(OFFSET_SAME_BLOCK_REPOLL_DELAY_MS)).await;
                        continue;
                    }
                }
                Err(error) => {
                    set_watcher_health(
                        &state,
                        WatcherKind::Slot,
                        FollowWatcherHealth::Degraded,
                        Some("real-blockheight".to_string()),
                        Some(state.rpc_url.clone()),
                        Some(error),
                    )
                    .await;
                }
            }
            break;
        }
        delay_ms = interval_ms;
    }
}

async fn ensure_market_watcher(
    state: Arc<AppState>,
    job: &FollowJobRecord,
) -> watch::Receiver<Option<Result<u64, String>>> {
    let hub = get_job_watch_hub(&state, &job.traceId).await;
    let mut started = hub.started.lock().await;
    if !started.market {
        started.market = true;
        let task_state = state.clone();
        let task_job = job.clone();
        let task_tx = hub.market_tx.clone();
        tokio::spawn(async move {
            run_market_watcher(task_state, task_job, task_tx).await;
        });
    }
    drop(started);
    hub.market_tx.subscribe()
}

fn configured_enable_helius_transaction_subscribe() -> bool {
    matches!(
        env::var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

async fn capture_action_watcher_metadata(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action_id: &str,
    kind: WatcherKind,
) {
    let health = state.store.health().await;
    let inferred_endpoint = resolve_job_watch_endpoint(job).ok();
    let inferred_mode = match kind {
        WatcherKind::Slot | WatcherKind::Signature => Some(selected_realtime_watcher_mode(
            &job.execution.provider,
            &job.execution.endpointProfile,
            inferred_endpoint.as_deref(),
        )),
        WatcherKind::Market => Some(selected_market_watcher_mode(
            &job.execution.provider,
            &job.execution.endpointProfile,
            inferred_endpoint.as_deref(),
        )),
    };
    let fallback_reason = health.lastError.clone().filter(|value| !value.trim().is_empty());
    let mode = match kind {
        WatcherKind::Slot => health.slotWatcherMode.clone().or_else(|| inferred_mode.clone()),
        WatcherKind::Signature => {
            if fallback_reason.is_some() {
                health
                    .signatureWatcherMode
                    .clone()
                    .or_else(|| inferred_mode.clone())
            } else {
                inferred_mode
                    .clone()
                    .or_else(|| health.signatureWatcherMode.clone())
            }
        }
        WatcherKind::Market => {
            if fallback_reason.is_some() {
                health
                    .marketWatcherMode
                    .clone()
                    .or_else(|| inferred_mode.clone())
            } else {
                inferred_mode
                    .clone()
                    .or_else(|| health.marketWatcherMode.clone())
            }
        }
    };
    let _ = state
        .store
        .update_action(&job.traceId, action_id, |record| {
            record.watcherMode = mode;
            record.watcherFallbackReason = fallback_reason;
        })
        .await;
}

#[allow(dead_code)]
async fn run_slot_watcher_ws_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    endpoint: &str,
) -> Result<(), String> {
    let mut ws = open_subscription_socket(endpoint).await?;
    subscribe(&mut ws, "slotSubscribe", json!([])).await?;
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        let message = next_json_message(&mut ws).await?;
        if message.get("params").is_none() {
            continue;
        }
        let block_height = fetch_current_block_height(&state.rpc_url, "confirmed").await?;
        let _ = tx.send(Some(Ok(block_height)));
        set_watcher_health(
            state,
            WatcherKind::Slot,
            FollowWatcherHealth::Healthy,
            Some("standard-ws".to_string()),
            Some(endpoint.to_string()),
            None,
        )
        .await;
    }
}

#[allow(dead_code)]
async fn run_helius_transaction_slot_watcher_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    endpoint: &str,
) -> Result<(), String> {
    let mut ws = open_subscription_socket(endpoint).await?;
    subscribe(
        &mut ws,
        "transactionSubscribe",
        json!([
            {
                "failed": false,
                "vote": false
            },
            {
                "commitment": "processed",
                "encoding": "jsonParsed",
                "transactionDetails": "none",
                "showRewards": false,
                "maxSupportedTransactionVersion": 0
            }
        ]),
    )
    .await?;
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        let message = next_json_message(&mut ws).await?;
        if message.get("params").is_none() {
            continue;
        }
        let block_height = fetch_current_block_height(&state.rpc_url, "confirmed").await?;
        let _ = tx.send(Some(Ok(block_height)));
        set_watcher_health(
            state,
            WatcherKind::Slot,
            FollowWatcherHealth::Healthy,
            Some("helius-transaction-subscribe".to_string()),
            Some(endpoint.to_string()),
            None,
        )
        .await;
    }
}

#[allow(dead_code)]
async fn run_slot_watcher_polling_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    note: Option<String>,
) -> Result<(), String> {
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        let block_height = fetch_current_block_height(&state.rpc_url, "confirmed").await?;
        let _ = tx.send(Some(Ok(block_height)));
        set_watcher_health(
            state,
            WatcherKind::Slot,
            FollowWatcherHealth::Healthy,
            Some("rpc-polling".to_string()),
            Some(state.rpc_url.clone()),
            note.clone(),
        )
        .await;
        sleep(Duration::from_millis(200)).await;
    }
}

async fn recompute_market_cap_for_job(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    mint: &str,
) -> Result<u64, String> {
    if job.launchpad == "bonk" {
        let snapshot = fetch_bonk_market_snapshot(&state.rpc_url, mint, &job.quoteAsset).await?;
        return snapshot
            .marketCapLamports
            .parse::<u64>()
            .map_err(|error| format!("Invalid Bonk market cap payload: {error}"));
    }
    if job.launchpad == "bagsapp" {
        let snapshot = fetch_bags_market_snapshot(&state.rpc_url, mint).await?;
        return snapshot
            .marketCapLamports
            .parse::<u64>()
            .map_err(|error| format!("Invalid Bags market cap payload: {error}"));
    }
    let snapshot = fetch_pump_market_snapshot(&state.rpc_url, mint).await?;
    Ok(snapshot.marketCapLamports)
}

fn market_watch_account(job: &FollowJobRecord, mint: &str) -> Result<String, String> {
    if job.launchpad == "pump" {
        return pump_bonding_curve_address(mint);
    }
    Ok(mint.to_string())
}

async fn run_standard_market_watcher_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    endpoint: &str,
    mint: &str,
    note: Option<String>,
) -> Result<(), String> {
    if job.launchpad == "pump" {
        let bonding_curve = market_watch_account(job, mint)?;
        let mut ws = open_subscription_socket(endpoint).await?;
        subscribe(
            &mut ws,
            "accountSubscribe",
            json!([
                bonding_curve,
                {
                    "encoding": "base64",
                    "commitment": "processed"
                }
            ]),
        )
        .await?;
        loop {
            ensure_job_not_cancelled(state, &job.traceId).await?;
            let message = next_json_message(&mut ws).await?;
            if message.get("params").is_none() {
                continue;
            }
            let market_cap = recompute_market_cap_for_job(state, job, mint).await?;
            let _ = tx.send(Some(Ok(market_cap)));
            set_watcher_health(
                state,
                WatcherKind::Market,
                FollowWatcherHealth::Healthy,
                Some("standard-ws".to_string()),
                Some(endpoint.to_string()),
                note.clone(),
            )
            .await;
        }
    }
    let mut ws = open_subscription_socket(endpoint).await?;
    subscribe(
        &mut ws,
        "logsSubscribe",
        json!([
            {
                "mentions": [mint]
            },
            {
                "commitment": "processed"
            }
        ]),
    )
    .await?;
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        let message = next_json_message(&mut ws).await?;
        if message.get("params").is_none() {
            continue;
        }
        let market_cap = recompute_market_cap_for_job(state, job, mint).await?;
        let _ = tx.send(Some(Ok(market_cap)));
        set_watcher_health(
            state,
            WatcherKind::Market,
            FollowWatcherHealth::Healthy,
            Some("standard-ws".to_string()),
            Some(endpoint.to_string()),
            note.clone(),
        )
        .await;
    }
}

async fn run_helius_transaction_market_watcher_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    endpoint: &str,
    mint: &str,
) -> Result<(), String> {
    let watch_account = market_watch_account(job, mint)?;
    let mut ws = open_subscription_socket(endpoint).await?;
    subscribe(
        &mut ws,
        "transactionSubscribe",
        json!([
            {
                "accountInclude": [watch_account],
                "failed": false,
                "vote": false
            },
            {
                "commitment": "processed",
                "encoding": "jsonParsed",
                "transactionDetails": "none",
                "showRewards": false,
                "maxSupportedTransactionVersion": 0
            }
        ]),
    )
    .await?;
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        let message = next_json_message(&mut ws).await?;
        if message.get("params").is_none() {
            continue;
        }
        let market_cap = recompute_market_cap_for_job(state, job, mint).await?;
        let _ = tx.send(Some(Ok(market_cap)));
        set_watcher_health(
            state,
            WatcherKind::Market,
            FollowWatcherHealth::Healthy,
            Some("helius-transaction-subscribe".to_string()),
            Some(endpoint.to_string()),
            None,
        )
        .await;
    }
}

fn extract_transaction_notification_signature(message: &Value) -> Option<String> {
    [
        message.pointer("/params/result/signature"),
        message.pointer("/params/result/transaction/signature"),
        message.pointer("/params/result/transaction/signatures/0"),
        message.pointer("/params/result/transaction/transaction/signatures/0"),
    ]
    .into_iter()
    .flatten()
    .find_map(|value| value.as_str().map(str::to_string))
}

async fn run_helius_transaction_signature_watcher_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    endpoint: &str,
    signature: &str,
) -> Result<u64, String> {
    let filter = if let Some(creator) = job.launchCreator.as_ref() {
        json!({
            "accountInclude": [creator],
            "failed": false,
            "vote": false
        })
    } else {
        json!({
            "failed": false,
            "vote": false
        })
    };
    let mut ws = open_subscription_socket(endpoint).await?;
    subscribe(
        &mut ws,
        "transactionSubscribe",
        json!([
            filter,
            {
                "commitment": "confirmed",
                "encoding": "jsonParsed",
                "transactionDetails": "full",
                "showRewards": false,
                "maxSupportedTransactionVersion": 0
            }
        ]),
    )
    .await?;
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        let message = next_json_message(&mut ws).await?;
        if message.get("params").is_none() {
            continue;
        }
        let Some(observed_signature) = extract_transaction_notification_signature(&message) else {
            continue;
        };
        if observed_signature != signature {
            continue;
        }
        let confirmed_block_height =
            current_shared_block_height(state.clone(), &job.traceId).await?;
        let _ = tx.send(Some(Ok(confirmed_block_height)));
        set_watcher_health(
            state,
            WatcherKind::Signature,
            FollowWatcherHealth::Healthy,
            Some("helius-transaction-subscribe".to_string()),
            Some(endpoint.to_string()),
            None,
        )
        .await;
        return Ok(confirmed_block_height);
    }
}

async fn run_signature_watcher(
    state: Arc<AppState>,
    job: FollowJobRecord,
    tx: watch::Sender<Option<Result<u64, String>>>,
) {
    let Some(signature) = job.launchSignature.clone() else {
        let _ = tx.send(Some(
            Err("Follow job missing launch signature.".to_string()),
        ));
        return;
    };
    let Ok(endpoint) = resolve_job_watch_endpoint(&job) else {
        let _ = tx.send(Some(Err(
            "No websocket watch endpoint configured for follow job. Set SOLANA_WS_URL.".to_string(),
        )));
        return;
    };
    let mut attempt: u32 = 0;
    let prefers_helius_transaction_subscribe = configured_enable_helius_transaction_subscribe()
        && supports_helius_transaction_subscribe(
            &job.execution.provider,
            &job.execution.endpointProfile,
            Some(&endpoint),
        );
    loop {
        let session: Result<u64, String> = async {
            if prefers_helius_transaction_subscribe {
                match run_helius_transaction_signature_watcher_session(
                    &state,
                    &job,
                    &tx,
                    &endpoint,
                    &signature,
                )
                .await
                {
                    Ok(confirmed_block_height) => return Ok(confirmed_block_height),
                    Err(error) => {
                        let fallback_note = format!(
                            "Helius transactionSubscribe signature watcher failed: {error}. Falling back to standard websocket."
                        );
                        set_watcher_health(
                            &state,
                            WatcherKind::Signature,
                            FollowWatcherHealth::Healthy,
                            Some("standard-ws".to_string()),
                            Some(endpoint.clone()),
                            Some(fallback_note),
                        )
                        .await;
                    }
                }
            }
            let mut ws = open_subscription_socket(&endpoint).await?;
            subscribe(
                &mut ws,
                "signatureSubscribe",
                json!([
                    signature,
                    {
                        "commitment": "confirmed"
                    }
                ]),
            )
            .await?;
            loop {
                ensure_job_not_cancelled(&state, &job.traceId).await?;
                let message = next_json_message(&mut ws).await?;
                if let Some(params) = message.get("params") {
                    let value = params
                        .get("result")
                        .and_then(|result| result.get("value"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    let err = value.get("err").cloned().unwrap_or(Value::Null);
                    if !err.is_null() {
                        return Err(format!(
                            "Launch signature notification reported error: {err}"
                        ));
                    }
                    set_watcher_health(
                        &state,
                        WatcherKind::Signature,
                        FollowWatcherHealth::Healthy,
                        Some("standard-ws".to_string()),
                        Some(endpoint.clone()),
                        None,
                    )
                    .await;
                    let confirmed_block_height =
                        fetch_current_block_height(&state.rpc_url, "confirmed").await?;
                    return Ok(confirmed_block_height);
                }
            }
        }
        .await;
        match session {
            Ok(confirmed_block_height) => {
                let _ = tx.send(Some(Ok(confirmed_block_height)));
                return;
            }
            Err(error) => {
                attempt = attempt.saturating_add(1);
                if let Err(final_error) = handle_watcher_retry(
                    &state,
                    &job.traceId,
                    WatcherKind::Signature,
                    &endpoint,
                    attempt,
                    error,
                )
                .await
                {
                    let _ = tx.send(Some(Err(final_error)));
                    return;
                }
            }
        }
    }
}

#[allow(dead_code)]
async fn run_slot_watcher(
    state: Arc<AppState>,
    job: FollowJobRecord,
    tx: watch::Sender<Option<Result<u64, String>>>,
) {
    let mut attempt: u32 = 0;
    let watch_endpoint = resolve_job_watch_endpoint(&job).ok();
    let prefers_helius_transaction_subscribe = watch_endpoint.as_deref().is_some_and(|endpoint| {
        configured_enable_helius_transaction_subscribe()
            && supports_helius_transaction_subscribe(
                &job.execution.provider,
                &job.execution.endpointProfile,
                Some(endpoint),
            )
    });
    loop {
        let session = if let Some(endpoint) = watch_endpoint.as_deref() {
            if prefers_helius_transaction_subscribe {
                match run_helius_transaction_slot_watcher_session(&state, &job, &tx, endpoint).await
                {
                    Ok(()) => Ok(()),
                    Err(error) => {
                        let fallback_note = format!(
                            "Helius transactionSubscribe slot watcher failed: {error}. Falling back to standard websocket."
                        );
                        set_watcher_health(
                            &state,
                            WatcherKind::Slot,
                            FollowWatcherHealth::Healthy,
                            Some("standard-ws".to_string()),
                            Some(endpoint.to_string()),
                            Some(fallback_note.clone()),
                        )
                        .await;
                        match run_slot_watcher_ws_session(&state, &job, &tx, endpoint).await {
                            Ok(()) => Ok(()),
                            Err(error) => {
                                let fallback_note = format!(
                                    "{fallback_note} Standard websocket slot watcher failed: {error}. Falling back to RPC polling."
                                );
                                run_slot_watcher_polling_session(
                                    &state,
                                    &job,
                                    &tx,
                                    Some(fallback_note),
                                )
                                .await
                            }
                        }
                    }
                }
            } else {
                match run_slot_watcher_ws_session(&state, &job, &tx, endpoint).await {
                    Ok(()) => Ok(()),
                    Err(error) => {
                        let fallback_note = format!(
                            "Standard websocket slot watcher failed: {error}. Falling back to RPC polling."
                        );
                        run_slot_watcher_polling_session(&state, &job, &tx, Some(fallback_note)).await
                    }
                }
            }
        } else {
            run_slot_watcher_polling_session(
                &state,
                &job,
                &tx,
                Some("No websocket watch endpoint configured for slot watcher. Falling back to RPC polling.".to_string()),
            )
            .await
        };
        match session {
            Ok(()) => return,
            Err(error) => {
                attempt = attempt.saturating_add(1);
                let terminal_error = error.clone();
                if handle_watcher_retry(
                    &state,
                    &job.traceId,
                    WatcherKind::Slot,
                    watch_endpoint.as_deref().unwrap_or(&state.rpc_url),
                    attempt,
                    error,
                )
                .await
                .is_err()
                {
                    let _ = tx.send(Some(Err(terminal_error)));
                    return;
                }
            }
        }
    }
}

async fn run_market_watcher(
    state: Arc<AppState>,
    job: FollowJobRecord,
    tx: watch::Sender<Option<Result<u64, String>>>,
) {
    let Some(mint) = job.mint.clone() else {
        let _ = tx.send(Some(Err("Market watcher missing mint.".to_string())));
        return;
    };
    let Ok(endpoint) = resolve_job_watch_endpoint(&job) else {
        let _ = tx.send(Some(Err(
            "Missing watch endpoint for market watcher.".to_string()
        )));
        return;
    };
    let mut attempt: u32 = 0;
    let prefers_helius_transaction_subscribe = configured_enable_helius_transaction_subscribe()
        && supports_helius_transaction_subscribe(
            &job.execution.provider,
            &job.execution.endpointProfile,
            Some(&endpoint),
        );
    loop {
        let session = if prefers_helius_transaction_subscribe {
            set_watcher_health(
                &state,
                WatcherKind::Market,
                FollowWatcherHealth::Healthy,
                Some("helius-transaction-subscribe".to_string()),
                Some(endpoint.clone()),
                None,
            )
            .await;
            match run_helius_transaction_market_watcher_session(&state, &job, &tx, &endpoint, &mint)
                .await
            {
                Ok(()) => Ok(()),
                Err(error) => {
                    let fallback_note = format!(
                        "Helius transactionSubscribe market watcher failed: {error}. Falling back to standard websocket."
                    );
                    set_watcher_health(
                        &state,
                        WatcherKind::Market,
                        FollowWatcherHealth::Healthy,
                        Some("standard-ws".to_string()),
                        Some(endpoint.clone()),
                        Some(fallback_note.clone()),
                    )
                    .await;
                    run_standard_market_watcher_session(
                        &state,
                        &job,
                        &tx,
                        &endpoint,
                        &mint,
                        Some(fallback_note),
                    )
                    .await
                }
            }
        } else {
            set_watcher_health(
                &state,
                WatcherKind::Market,
                FollowWatcherHealth::Healthy,
                Some("standard-ws".to_string()),
                Some(endpoint.clone()),
                None,
            )
            .await;
            run_standard_market_watcher_session(&state, &job, &tx, &endpoint, &mint, None).await
        };
        match session {
            Ok(()) => return,
            Err(error) => {
                attempt = attempt.saturating_add(1);
                let terminal_error = error.clone();
                if handle_watcher_retry(
                    &state,
                    &job.traceId,
                    WatcherKind::Market,
                    &endpoint,
                    attempt,
                    error,
                )
                .await
                .is_err()
                {
                    let _ = tx.send(Some(Err(terminal_error)));
                    return;
                }
            }
        }
    }
}

async fn wait_for_signature_confirmation(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action_id: &str,
) -> Result<u64, String> {
    if let Some(confirmed_block_height) = job.confirmedObservedBlockHeight {
        capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Signature).await;
        return Ok(confirmed_block_height);
    }
    let mut rx = ensure_signature_watcher(state.clone(), job).await;
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, action_id).await?;
        let current = rx.borrow().clone();
        match current {
            Some(Ok(confirmed_block_height)) => {
                capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Signature)
                    .await;
                return Ok(confirmed_block_height);
            }
            Some(Err(error)) => return Err(error),
            None => {
                rx.changed()
                    .await
                    .map_err(|_| "Shared signature watcher stopped unexpectedly.".to_string())?;
            }
        }
    }
}

async fn wait_for_slot_offset(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action_id: &str,
    confirmed_launch_block_height: Option<u64>,
    target_offset: u64,
) -> Result<(), String> {
    if target_offset == 0 {
        capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Signature).await;
        return Ok(());
    }
    let base_block_height = confirmed_launch_block_height
        .or(job.confirmedObservedBlockHeight)
        .ok_or_else(|| {
            "Launch confirmation block height was unavailable for On Confirmed Block trigger."
                .to_string()
        })?;
    let confirmation_detected_at_ms = now_ms();
    let mut rx = register_offset_consumer(
        state.clone(),
        job,
        action_id,
        base_block_height,
        target_offset,
        confirmation_detected_at_ms,
    )
    .await;
    loop {
        if let Err(error) = ensure_action_not_cancelled(&state, &job.traceId, action_id).await {
            unregister_offset_consumer(&state, &job.traceId, action_id).await;
            return Err(error);
        }
        let current = rx.borrow().clone();
        if let Some(result) = current {
            unregister_offset_consumer(&state, &job.traceId, action_id).await;
            match result {
                Ok(()) => {
                    capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Slot)
                        .await;
                    return Ok(());
                }
                Err(error) => return Err(error),
            }
        }
        tokio::select! {
            result = rx.changed() => {
                if let Err(_) = result {
                    unregister_offset_consumer(&state, &job.traceId, action_id).await;
                    return Err("Shared offset worker stopped unexpectedly.".to_string());
                }
            }
            _ = sleep(Duration::from_millis(50)) => {}
        }
    }
}

async fn wait_for_market_cap_trigger(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    action_id: &str,
) -> Result<(), String> {
    let trigger = action
        .marketCap
        .as_ref()
        .ok_or_else(|| "Market-cap trigger missing.".to_string())?;
    let threshold = trigger.threshold.parse::<u64>().map_err(|error| {
        format!(
            "Invalid market-cap threshold '{}': {error}",
            trigger.threshold
        )
    })?;
    let timeout_deadline =
        tokio::time::Instant::now() + Duration::from_secs(trigger.scanTimeoutSeconds);
    let mut rx = ensure_market_watcher(state.clone(), job).await;
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, action_id).await?;
        if tokio::time::Instant::now() >= timeout_deadline {
            if trigger.timeoutAction == "sell" {
                capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Market).await;
                return Ok(());
            }
            capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Market).await;
            return Err(market_cap_scan_expired_error(
                action_id,
                trigger.scanTimeoutSeconds,
            ));
        }
        let current_market_cap = rx.borrow().clone();
        if let Some(result) = current_market_cap {
            match result {
                Ok(market_cap) => {
                    let matches = match trigger.direction.as_str() {
                        "lte" => market_cap <= threshold,
                        _ => market_cap >= threshold,
                    };
                    if matches {
                        let current_health = state.store.health().await;
                        set_watcher_health(
                            &state,
                            WatcherKind::Market,
                            FollowWatcherHealth::Healthy,
                            current_health.marketWatcherMode.clone(),
                            job.transportPlan
                                .as_ref()
                                .and_then(|plan| plan.watchEndpoint.clone()),
                            current_health.lastError.clone(),
                        )
                        .await;
                        capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Market)
                            .await;
                        return Ok(());
                    }
                }
                Err(error) => return Err(error),
            }
        }
        let wait_for_change = timeout_deadline
            .checked_duration_since(tokio::time::Instant::now())
            .unwrap_or_default();
        match timeout(wait_for_change, rx.changed()).await {
            Ok(result) => {
                result.map_err(|_| "Shared market watcher stopped unexpectedly.".to_string())?;
            }
            Err(_) => {
                if trigger.timeoutAction == "sell" {
                    capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Market)
                        .await;
                    return Ok(());
                }
                capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Market).await;
                return Err(market_cap_scan_expired_error(
                    action_id,
                    trigger.scanTimeoutSeconds,
                ));
            }
        }
    }
}

fn resolve_job_watch_endpoint(job: &FollowJobRecord) -> Result<String, String> {
    if let Some(plan) = &job.transportPlan {
        if let Some(endpoint) = &plan.watchEndpoint {
            if !endpoint.trim().is_empty() {
                return Ok(endpoint.clone());
            }
        }
        if let Some(endpoint) = plan.watchEndpoints.first() {
            if !endpoint.trim().is_empty() {
                return Ok(endpoint.clone());
            }
        }
    }
    configured_watch_endpoints_for_provider(&job.execution.provider, &job.execution.endpointProfile)
        .into_iter()
        .next()
        .ok_or_else(|| {
            "No websocket watch endpoint configured for follow job. Set SOLANA_WS_URL.".to_string()
        })
}

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn open_subscription_socket(endpoint: &str) -> Result<WsStream, String> {
    timeout(Duration::from_secs(5), connect_async(endpoint))
        .await
        .map_err(|_| format!("Timed out connecting to websocket endpoint: {endpoint}"))?
        .map(|(stream, _)| stream)
        .map_err(|error| error.to_string())
}

async fn subscribe(ws: &mut WsStream, method: &str, params: Value) -> Result<(), String> {
    ws.send(Message::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        })
        .to_string()
        .into(),
    ))
    .await
    .map_err(|error| error.to_string())?;
    loop {
        let payload = next_json_message(ws).await?;
        if payload.get("id").and_then(Value::as_i64) == Some(1) {
            if payload.get("error").is_some() {
                return Err(format!("Subscription failed: {payload}"));
            }
            return Ok(());
        }
    }
}

async fn next_json_message(ws: &mut WsStream) -> Result<Value, String> {
    loop {
        let Some(message) = ws.next().await else {
            return Err("Websocket stream closed.".to_string());
        };
        let message = message.map_err(|error| error.to_string())?;
        match message {
            Message::Text(text) => {
                let parsed =
                    serde_json::from_str::<Value>(&text).map_err(|error| error.to_string())?;
                return Ok(parsed);
            }
            Message::Binary(bytes) => {
                let parsed =
                    serde_json::from_slice::<Value>(&bytes).map_err(|error| error.to_string())?;
                return Ok(parsed);
            }
            Message::Ping(payload) => {
                ws.send(Message::Pong(payload))
                    .await
                    .map_err(|error| error.to_string())?;
            }
            Message::Pong(_) => {}
            Message::Frame(_) => {}
            Message::Close(_) => return Err("Websocket stream closed.".to_string()),
        }
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let base_url = configured_follow_daemon_base_url();
    let max_active_jobs = configured_limit("LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS");
    let max_concurrent_compiles = configured_limit("LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES");
    let max_concurrent_sends = configured_limit("LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS");
    let capacity_wait_ms = configured_capacity_wait_ms();
    let state = Arc::new(AppState {
        auth_token: configured_auth_token(),
        rpc_url: configured_rpc_url(),
        store: FollowDaemonStore::load_or_default(paths::follow_daemon_state_path()),
        max_active_jobs,
        max_concurrent_compiles,
        max_concurrent_sends,
        capacity_wait_ms,
        active_jobs: Arc::new(Mutex::new(HashMap::new())),
        wallet_locks: Arc::new(Mutex::new(HashMap::new())),
        report_write_lock: Arc::new(Mutex::new(())),
        compile_slots: max_concurrent_compiles.map(|limit| Arc::new(Semaphore::new(limit))),
        send_slots: max_concurrent_sends.map(|limit| Arc::new(Semaphore::new(limit))),
        watch_hubs: Arc::new(Mutex::new(HashMap::new())),
        prepared_follow_buys: Arc::new(Mutex::new(HashMap::new())),
        hot_follow_buy_runtime: Arc::new(Mutex::new(HashMap::new())),
        hot_follow_buy_tasks: Arc::new(Mutex::new(HashMap::new())),
        follow_report_flushes: Arc::new(Mutex::new(HashMap::new())),
        follow_report_flush_tasks: Arc::new(Mutex::new(HashMap::new())),
        watch_endpoint_health: Arc::new(Mutex::new(HashMap::new())),
        wallet_precheck_cache: Arc::new(Mutex::new(HashMap::new())),
        offset_worker: Arc::new(OffsetWorkerHub {
            consumers: Mutex::new(HashMap::new()),
            running: Mutex::new(false),
            last_observed_block_height: Mutex::new(None),
            last_observed_at_ms: Mutex::new(None),
        }),
    });
    if let Some(endpoint) = configured_startup_watch_endpoint() {
        remember_watch_endpoint(&state, &endpoint).await;
        let startup_state = state.clone();
        tokio::spawn(async move {
            let _ = refresh_watch_endpoint_health(&startup_state, &endpoint, None, None).await;
        });
    }
    spawn_watch_endpoint_health_monitor(state.clone());
    spawn_wallet_precheck_monitor(state.clone());
    spawn_blockhash_refresh_task(state.rpc_url.clone(), "processed");
    spawn_blockhash_refresh_task(state.rpc_url.clone(), "confirmed");
    spawn_blockhash_refresh_task(state.rpc_url.clone(), "finalized");
    let _ = state
        .store
        .update_supervision(
            true,
            Some(std::process::id()),
            Some("local-http".to_string()),
            Some(base_url.clone()),
            None,
        )
        .await;
    restore_jobs(state.clone()).await;
    let shutdown_state = state.clone();
    let shutdown_base_url = base_url.clone();
    let app = Router::new()
        .route("/health", get(daemon_health))
        .route("/ready", post(daemon_ready))
        .route("/jobs", get(list_jobs))
        .route("/jobs/reserve", post(reserve_job))
        .route("/jobs/arm", post(arm_job))
        .route("/jobs/cancel", post(cancel_job))
        .route("/jobs/stop-all", post(stop_all_jobs))
        .route("/jobs/{trace_id}", get(job_status))
        .with_state(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], configured_follow_daemon_port()));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind LaunchDeck follow daemon listener");
    println!("LaunchDeck follow daemon running at {}", base_url);
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = shutdown_state
                .store
                .update_supervision(
                    false,
                    Some(std::process::id()),
                    Some("local-http".to_string()),
                    Some(shutdown_base_url),
                    None,
                )
                .await;
        })
        .await
        .expect("LaunchDeck follow daemon server failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use launchdeck_engine::config::{
        NormalizedExecution, NormalizedFollowLaunch, NormalizedFollowLaunchConstraints,
        NormalizedFollowLaunchMarketCapTrigger, NormalizedFollowLaunchSell,
        NormalizedFollowLaunchSnipe,
    };
    use launchdeck_engine::rpc::CompiledTransaction;
    use serde_json::json;
    use std::path::PathBuf;

    fn sample_execution() -> NormalizedExecution {
        serde_json::from_value(json!({
            "simulate": false,
            "send": true,
            "txFormat": "base64",
            "commitment": "confirmed",
            "skipPreflight": false,
            "trackSendBlockHeight": true,
            "provider": "helius-sender",
            "endpointProfile": "us",
            "autoGas": false,
            "autoMode": "off",
            "priorityFeeSol": "0.001",
            "tipSol": "0.01",
            "maxPriorityFeeSol": "",
            "maxTipSol": "",
            "buyProvider": "standard-rpc",
            "buyEndpointProfile": "",
            "buyAutoGas": false,
            "buyAutoMode": "off",
            "buyPriorityFeeSol": "0.001",
            "buyTipSol": "",
            "buySlippagePercent": "90",
            "buyMaxPriorityFeeSol": "",
            "buyMaxTipSol": "",
            "sellAutoGas": false,
            "sellAutoMode": "off",
            "sellProvider": "jito-bundle",
            "sellEndpointProfile": "eu",
            "sellPriorityFeeSol": "0.001",
            "sellTipSol": "0.01",
            "sellSlippagePercent": "90",
            "sellMaxPriorityFeeSol": "",
            "sellMaxTipSol": ""
        }))
        .expect("sample execution should deserialize")
    }

    fn sample_job() -> FollowJobRecord {
        FollowJobRecord {
            schemaVersion: 1,
            traceId: "trace".to_string(),
            jobId: "job".to_string(),
            state: FollowJobState::Armed,
            createdAtMs: 0,
            updatedAtMs: 0,
            launchpad: "pump".to_string(),
            quoteAsset: "sol".to_string(),
            launchMode: "regular".to_string(),
            selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
            execution: sample_execution(),
            tokenMayhemMode: false,
            jitoTipAccount: "tip".to_string(),
            buyTipAccount: "buy-tip".to_string(),
            sellTipAccount: "sell-tip".to_string(),
            preferPostSetupCreatorVaultForSell: false,
            mint: None,
            launchCreator: None,
            launchSignature: None,
            submitAtMs: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            reportPath: None,
            transportPlan: None,
            followLaunch: NormalizedFollowLaunch {
                enabled: true,
                source: "test".to_string(),
                schemaVersion: 1,
                snipes: vec![],
                devAutoSell: None,
                constraints: NormalizedFollowLaunchConstraints {
                    pumpOnly: false,
                    retryBudget: 0,
                    requireDaemonReadiness: false,
                    blockOnRequiredPrechecks: false,
                },
            },
            actions: vec![],
            deferredSetup: None,
            cancelRequested: false,
            lastError: None,
            timings: FollowJobTimings {
                benchmarkMode: Some(configured_benchmark_mode().as_str().to_string()),
                ..FollowJobTimings::default()
            },
        }
    }

    fn sample_follow_launch() -> NormalizedFollowLaunch {
        sample_job().followLaunch
    }

    #[test]
    fn follow_buy_transport_plan_uses_buy_provider() {
        let job = sample_job();
        let action = FollowActionRecord {
            actionId: "buy".to_string(),
            kind: FollowActionKind::SniperBuy,
            walletEnvKey: "SOLANA_PRIVATE_KEY2".to_string(),
            state: FollowActionState::Armed,
            buyAmountSol: Some("1".to_string()),
            sellPercent: None,
            submitDelayMs: Some(0),
            targetBlockOffset: None,
            delayMs: None,
            marketCap: None,
            jitterMs: Some(0),
            feeJitterBps: Some(0),
            precheckRequired: false,
            requireConfirmation: false,
            skipIfTokenBalancePositive: false,
            attemptCount: 0,
            scheduledForMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: 0,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        let plan = follow_action_transport_plan(&job, &action);
        assert_eq!(plan.resolvedProvider, "standard-rpc");
        assert_eq!(plan.transportType, "standard-rpc-fanout");
    }

    #[test]
    fn follow_sell_transport_plan_uses_sell_provider() {
        let job = sample_job();
        let action = FollowActionRecord {
            actionId: "sell".to_string(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            state: FollowActionState::Armed,
            buyAmountSol: None,
            sellPercent: Some(100),
            submitDelayMs: None,
            targetBlockOffset: Some(1),
            delayMs: Some(0),
            marketCap: None,
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: false,
            requireConfirmation: true,
            skipIfTokenBalancePositive: false,
            attemptCount: 0,
            scheduledForMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: 0,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        let plan = follow_action_transport_plan(&job, &action);
        assert_eq!(plan.resolvedProvider, "jito-bundle");
        assert_eq!(plan.transportType, "jito-bundle");
    }

    #[test]
    fn pump_sell_creator_vault_retry_detects_onchain_custom_2006() {
        let mut job = sample_job();
        job.followLaunch.constraints.retryBudget = 1;
        let action = FollowActionRecord {
            actionId: "sell".to_string(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            state: FollowActionState::Sent,
            buyAmountSol: None,
            sellPercent: Some(100),
            submitDelayMs: None,
            targetBlockOffset: Some(0),
            delayMs: None,
            marketCap: None,
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: false,
            requireConfirmation: true,
            skipIfTokenBalancePositive: false,
            attemptCount: 0,
            scheduledForMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: 0,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        assert!(should_retry_pump_sell_creator_vault_mismatch(
            &job,
            &action,
            r#"on-chain failure | Transaction abc failed on-chain: {"InstructionError":[2,{"Custom":2006}]}"#,
        ));
    }

    #[test]
    fn pump_sell_creator_vault_retry_does_not_match_buys() {
        let mut job = sample_job();
        job.followLaunch.constraints.retryBudget = 1;
        let action = FollowActionRecord {
            actionId: "buy".to_string(),
            kind: FollowActionKind::SniperBuy,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            state: FollowActionState::Sent,
            buyAmountSol: Some("0.1".to_string()),
            sellPercent: None,
            submitDelayMs: None,
            targetBlockOffset: Some(0),
            delayMs: None,
            marketCap: None,
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: false,
            requireConfirmation: true,
            skipIfTokenBalancePositive: false,
            attemptCount: 0,
            scheduledForMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: 0,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        assert!(!should_retry_pump_sell_creator_vault_mismatch(
            &job,
            &action,
            r#"on-chain failure | Transaction abc failed on-chain: {"InstructionError":[2,{"Custom":2006}]}"#,
        ));
    }

    #[test]
    fn presigned_pump_buy_creator_vault_retry_detects_onchain_custom_2006() {
        let mut job = sample_job();
        job.followLaunch.constraints.retryBudget = 1;
        let action = FollowActionRecord {
            actionId: "buy".to_string(),
            kind: FollowActionKind::SniperBuy,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            state: FollowActionState::Sent,
            buyAmountSol: Some("0.1".to_string()),
            sellPercent: None,
            submitDelayMs: None,
            targetBlockOffset: Some(0),
            delayMs: None,
            marketCap: None,
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: false,
            requireConfirmation: false,
            skipIfTokenBalancePositive: false,
            attemptCount: 0,
            scheduledForMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: 0,
            preSignedTransactions: vec![CompiledTransaction {
                label: "buy".to_string(),
                format: "v0".to_string(),
                blockhash: "hash".to_string(),
                lastValidBlockHeight: 1,
                serializedBase64: "base64".to_string(),
                signature: None,
                lookupTablesUsed: vec![],
                computeUnitLimit: None,
                computeUnitPriceMicroLamports: None,
                inlineTipLamports: None,
                inlineTipAccount: None,
            }],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        assert!(should_rebuild_presigned_pump_buy_creator_vault_mismatch(
            &job,
            &action,
            r#"on-chain failure | Transaction abc failed on-chain: {"InstructionError":[3,{"Custom":2006}]}"#,
        ));
    }

    #[test]
    fn presigned_pump_sell_slippage_retry_detects_onchain_custom_6003() {
        let mut job = sample_job();
        job.followLaunch.constraints.retryBudget = 1;
        let action = FollowActionRecord {
            actionId: "sell".to_string(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            state: FollowActionState::Sent,
            buyAmountSol: None,
            sellPercent: Some(100),
            submitDelayMs: None,
            targetBlockOffset: Some(0),
            delayMs: None,
            marketCap: None,
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: false,
            requireConfirmation: false,
            skipIfTokenBalancePositive: false,
            attemptCount: 0,
            scheduledForMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: 0,
            preSignedTransactions: vec![CompiledTransaction {
                label: "sell".to_string(),
                format: "v0".to_string(),
                blockhash: "hash".to_string(),
                lastValidBlockHeight: 1,
                serializedBase64: "base64".to_string(),
                signature: None,
                lookupTablesUsed: vec![],
                computeUnitLimit: None,
                computeUnitPriceMicroLamports: None,
                inlineTipLamports: None,
                inlineTipAccount: None,
            }],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        assert!(should_rebuild_presigned_pump_sell_onchain_slippage(
            &job,
            &action,
            r#"on-chain failure | Transaction abc failed on-chain: {"InstructionError":[2,{"Custom":6003}]}"#,
        ));
    }

    #[test]
    fn creator_vault_rebuild_retry_does_not_block_deferred_setup() {
        let action = FollowActionRecord {
            actionId: "buy".to_string(),
            kind: FollowActionKind::SniperBuy,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            state: FollowActionState::Armed,
            buyAmountSol: Some("0.1".to_string()),
            sellPercent: None,
            submitDelayMs: None,
            targetBlockOffset: Some(0),
            delayMs: None,
            marketCap: None,
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: false,
            requireConfirmation: false,
            skipIfTokenBalancePositive: false,
            attemptCount: 1,
            scheduledForMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: Some(
                "Pump buy hit creator_vault mismatch; rebuilding with refreshed creator-vault state in 300ms: seed mismatch".to_string(),
            ),
            triggerKey: None,
            orderIndex: 0,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        assert!(!should_block_deferred_setup_for_action(&action, 0));
    }

    #[test]
    fn confirmation_zero_sell_keeps_deployer_creator_vault_path() {
        let action = FollowActionRecord {
            actionId: "sell".to_string(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            state: FollowActionState::Armed,
            buyAmountSol: None,
            sellPercent: Some(100),
            submitDelayMs: None,
            targetBlockOffset: Some(0),
            delayMs: Some(0),
            marketCap: None,
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: false,
            requireConfirmation: true,
            skipIfTokenBalancePositive: false,
            attemptCount: 0,
            scheduledForMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: 0,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        assert!(!should_use_post_setup_creator_vault_for_sell(true, &action));
    }

    #[test]
    fn realtime_watchers_not_required_for_delay_only_follow_actions() {
        let mut follow = sample_follow_launch();
        follow.snipes.push(NormalizedFollowLaunchSnipe {
            actionId: "snipe-1".to_string(),
            enabled: true,
            walletEnvKey: "SOLANA_PRIVATE_KEY2".to_string(),
            buyAmountSol: "0.1".to_string(),
            submitWithLaunch: false,
            retryOnFailure: false,
            submitDelayMs: 250,
            targetBlockOffset: None,
            jitterMs: 0,
            feeJitterBps: 0,
            skipIfTokenBalancePositive: false,
            postBuySell: None,
        });
        follow.devAutoSell = Some(NormalizedFollowLaunchSell {
            actionId: "dev-auto-sell".to_string(),
            enabled: true,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            percent: 100,
            delayMs: Some(500),
            targetBlockOffset: None,
            marketCap: None,
            precheckRequired: false,
            requireConfirmation: false,
        });
        let request = FollowReadyRequest {
            followLaunch: follow,
            quoteAsset: "sol".to_string(),
            execution: sample_execution(),
            watchEndpoint: Some("wss://example.invalid".to_string()),
        };
        assert!(!requires_realtime_watchers(&request));
    }

    #[test]
    fn realtime_watchers_required_for_market_cap_follow_sell() {
        let mut follow = sample_follow_launch();
        follow.devAutoSell = Some(NormalizedFollowLaunchSell {
            actionId: "dev-auto-sell".to_string(),
            enabled: true,
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            percent: 100,
            delayMs: None,
            targetBlockOffset: None,
            marketCap: Some(NormalizedFollowLaunchMarketCapTrigger {
                direction: "gte".to_string(),
                threshold: "100000".to_string(),
                scanTimeoutSeconds: 15,
                timeoutAction: "stop".to_string(),
            }),
            precheckRequired: false,
            requireConfirmation: false,
        });
        let request = FollowReadyRequest {
            followLaunch: follow,
            quoteAsset: "sol".to_string(),
            execution: sample_execution(),
            watchEndpoint: Some("wss://example.invalid".to_string()),
        };
        assert!(requires_realtime_watchers(&request));
    }

    #[test]
    fn selected_realtime_watcher_mode_prefers_helius_transaction_subscribe() {
        unsafe {
            env::set_var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE", "true");
        }
        assert_eq!(
            selected_realtime_watcher_mode(
                "standard-rpc",
                "us",
                Some("wss://mainnet.helius-rpc.com/?api-key=test"),
            ),
            "helius-transaction-subscribe"
        );
        unsafe {
            env::remove_var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE");
        }
    }

    #[test]
    fn follow_job_capacity_is_unbounded_when_limit_is_missing() {
        let health = FollowDaemonHealth {
            running: true,
            statePath: PathBuf::from("/tmp/follow-state.json"),
            version: "test".to_string(),
            pid: None,
            startedAtMs: None,
            controlTransport: "local-http".to_string(),
            controlUrl: None,
            updatedAtMs: 0,
            queueDepth: 0,
            activeJobs: 10_000,
            maxActiveJobs: None,
            maxConcurrentCompiles: None,
            maxConcurrentSends: None,
            availableCompileSlots: None,
            availableSendSlots: None,
            slotWatcher: FollowWatcherHealth::Healthy,
            slotWatcherMode: None,
            signatureWatcher: FollowWatcherHealth::Healthy,
            signatureWatcherMode: None,
            marketWatcher: FollowWatcherHealth::Healthy,
            marketWatcherMode: None,
            lastError: None,
            watchEndpoint: None,
        };
        assert!(has_capacity_for_new_job(&health));
    }

    #[test]
    fn configured_limit_blank_means_uncapped() {
        assert_eq!(configured_limit("__LAUNCHDECK_TEST_UNSET_LIMIT__"), None);
    }

    #[test]
    fn parse_optional_limit_value_supports_uncapped_and_capped_values() {
        assert_eq!(parse_optional_limit_value(""), None);
        assert_eq!(parse_optional_limit_value("0"), None);
        assert_eq!(parse_optional_limit_value(" 0 "), None);
        assert_eq!(parse_optional_limit_value("15"), Some(15));
    }

    #[test]
    fn parse_optional_limit_value_rejects_invalid_values() {
        assert_eq!(parse_optional_limit_value("abc"), None);
        assert_eq!(parse_optional_limit_value("-1"), None);
    }
}
