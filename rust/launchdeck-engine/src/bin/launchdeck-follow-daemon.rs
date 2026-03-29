#![allow(non_snake_case)]

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt, future::join_all};
use launchdeck_engine::{
    follow::{
        FOLLOW_RESPONSE_SCHEMA_VERSION, FOLLOW_TELEMETRY_SCHEMA_VERSION, FollowActionKind,
        FollowActionRecord, FollowActionState, FollowArmRequest, FollowCancelRequest,
        FollowDaemonHealth, FollowDaemonStore, FollowJobRecord, FollowJobResponse, FollowJobState,
        FollowReadyRequest, FollowReadyResponse, FollowReserveRequest, FollowStopAllRequest,
        FollowTelemetrySample, FollowWatcherHealth, follow_job_response, follow_ready_response,
    },
    observability::update_persisted_follow_daemon_snapshot,
    paths,
    pump_native::{
        PreparedFollowBuyRuntime, PreparedFollowBuyStatic, compile_follow_sell_transaction,
        finalize_follow_buy_transaction, prepare_follow_buy_runtime, prepare_follow_buy_static,
        fetch_pump_market_snapshot, pump_bonding_curve_address,
    },
    rpc::{
        confirm_submitted_transactions_for_transport, spawn_blockhash_refresh_task,
        submit_transactions_for_transport,
    },
    transport::configured_watch_endpoints_for_provider,
    wallet::{
        fetch_balance_lamports, fetch_token_balance, public_key_from_secret, read_keypair_bytes,
        selected_wallet_key_or_default,
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

#[derive(Clone)]
struct AppState {
    auth_token: Option<String>,
    rpc_url: String,
    store: FollowDaemonStore,
    max_active_jobs: usize,
    max_concurrent_compiles: usize,
    max_concurrent_sends: usize,
    capacity_wait_ms: u64,
    active_jobs: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    wallet_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    report_write_lock: Arc<Mutex<()>>,
    compile_slots: Arc<Semaphore>,
    send_slots: Arc<Semaphore>,
    watch_hubs: Arc<Mutex<HashMap<String, Arc<JobWatchHub>>>>,
    prepared_follow_buys: Arc<Mutex<HashMap<String, PreparedFollowBuyStatic>>>,
    hot_follow_buy_runtime: Arc<Mutex<HashMap<String, CachedFollowBuyRuntime>>>,
    hot_follow_buy_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

#[derive(Clone)]
struct CachedFollowBuyRuntime {
    prepared: PreparedFollowBuyRuntime,
    refreshed_at_ms: u128,
}

struct JobWatchHub {
    slot_tx: watch::Sender<Option<u64>>,
    signature_tx: watch::Sender<Option<Result<(), String>>>,
    market_tx: watch::Sender<Option<u64>>,
    started: Mutex<JobWatchStarted>,
}

#[derive(Default)]
struct JobWatchStarted {
    slot: bool,
    signature: bool,
    market: bool,
}

#[derive(Clone, Copy)]
enum WatcherKind {
    Slot,
    Signature,
    Market,
}

const WATCHER_MAX_RECONNECT_ATTEMPTS: u32 = 5;
const WATCHER_BACKOFF_BASE_MS: u64 = 200;
const FOLLOW_BUY_PRECHECK_BUFFER_LAMPORTS: u64 = 2_000_000;
const DEFAULT_LOCAL_AUTH_TOKEN: &str = "4815927603149027";
const HOT_FOLLOW_BUY_REFRESH_MS: u64 = 250;
const HOT_FOLLOW_BUY_MAX_AGE_MS: u128 = 900;

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

fn configured_limit(var_name: &str, fallback: usize) -> usize {
    env::var(var_name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(fallback)
}

fn configured_capacity_wait_ms() -> u64 {
    env::var("LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(5_000)
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

async fn build_health(state: &Arc<AppState>) -> FollowDaemonHealth {
    let mut health = state.store.health().await;
    health.maxActiveJobs = state.max_active_jobs;
    health.maxConcurrentCompiles = state.max_concurrent_compiles;
    health.maxConcurrentSends = state.max_concurrent_sends;
    health.availableCompileSlots = state.compile_slots.available_permits();
    health.availableSendSlots = state.send_slots.available_permits();
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
    endpoint: Option<String>,
    last_error: Option<String>,
) {
    let current = state.store.health().await;
    let slot = match kind {
        WatcherKind::Slot => status.clone(),
        _ => current.slotWatcher.clone(),
    };
    let signature = match kind {
        WatcherKind::Signature => status.clone(),
        _ => current.signatureWatcher.clone(),
    };
    let market = match kind {
        WatcherKind::Market => status,
        _ => current.marketWatcher.clone(),
    };
    let _ = state
        .store
        .update_health(
            endpoint.or(current.watchEndpoint),
            slot,
            signature,
            market,
            last_error,
        )
        .await;
}

async fn wallet_public_key(wallet_key: &str) -> Result<String, String> {
    let wallet_secret = env::var(wallet_key)
        .map_err(|_| format!("Wallet env var is missing: {wallet_key}"))
        .and_then(|value| read_keypair_bytes(&value))?;
    public_key_from_secret(&wallet_secret)
}

fn follow_buy_cache_key(trace_id: &str, action_id: &str) -> String {
    format!("{trace_id}:{action_id}")
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
    prepared: PreparedFollowBuyRuntime,
) {
    let mut cache = state.hot_follow_buy_runtime.lock().await;
    cache.insert(
        trace_id.to_string(),
        CachedFollowBuyRuntime {
            prepared,
            refreshed_at_ms: now_ms(),
        },
    );
}

async fn get_hot_follow_buy_runtime(
    state: &Arc<AppState>,
    trace_id: &str,
) -> Option<CachedFollowBuyRuntime> {
    let cache = state.hot_follow_buy_runtime.lock().await;
    cache.get(trace_id).cloned()
}

async fn clear_follow_buy_caches(state: &Arc<AppState>, trace_id: &str) {
    {
        let mut prepared = state.prepared_follow_buys.lock().await;
        prepared.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut runtime = state.hot_follow_buy_runtime.lock().await;
        runtime.remove(trace_id);
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
        runtime.remove(trace_id);
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
            if let Ok(prepared) =
                prepare_follow_buy_runtime(&task_state.rpc_url, mint, launch_creator).await
            {
                cache_hot_follow_buy_runtime(&task_state, &task_trace_id, prepared).await;
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
    if let Ok(prepared_runtime) =
        prepare_follow_buy_runtime(&state.rpc_url, mint, launch_creator).await
    {
        cache_hot_follow_buy_runtime(&state, &job.traceId, prepared_runtime).await;
    }
    let trace_id = job.traceId.clone();
    let rpc_url = state.rpc_url.clone();
    let execution = job.execution.clone();
    let jito_tip_account = job.jitoTipAccount.clone();
    let tasks = buy_actions.into_iter().map(|action| {
        let trace_id = trace_id.clone();
        let rpc_url = rpc_url.clone();
        let execution = execution.clone();
        let jito_tip_account = jito_tip_account.clone();
        let mint = mint.to_string();
        let launch_creator = launch_creator.to_string();
        async move {
            let wallet_key = selected_wallet_key_or_default(&action.walletEnvKey)
                .ok_or_else(|| format!("Wallet env key not found: {}", action.walletEnvKey))?;
            let wallet_secret = env::var(&wallet_key)
                .map_err(|_| format!("Wallet env var is missing: {wallet_key}"))
                .and_then(|value| read_keypair_bytes(&value))?;
            let buy_amount = action
                .buyAmountSol
                .as_deref()
                .ok_or_else(|| "Follow buy missing amount.".to_string())?;
            let prepared = prepare_follow_buy_static(
                &rpc_url,
                &execution,
                &jito_tip_account,
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
        &job.jitoTipAccount,
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
) -> Result<PreparedFollowBuyRuntime, String> {
    if let Some(cached) = get_hot_follow_buy_runtime(state, &job.traceId).await
        && now_ms().saturating_sub(cached.refreshed_at_ms) <= HOT_FOLLOW_BUY_MAX_AGE_MS
    {
        return Ok(cached.prepared);
    }
    let mint = job
        .mint
        .as_deref()
        .ok_or_else(|| "Follow job missing mint.".to_string())?;
    let prepared = prepare_follow_buy_runtime(&state.rpc_url, mint, launch_creator).await?;
    cache_hot_follow_buy_runtime(state, &job.traceId, prepared.clone()).await;
    Ok(prepared)
}

async fn required_action_precheck(
    rpc_url: &str,
    action: &FollowActionRecord,
) -> Result<(), String> {
    let wallet_key = selected_wallet_key_or_default(&action.walletEnvKey)
        .ok_or_else(|| format!("Wallet env key not found: {}", action.walletEnvKey))?;
    let public_key = wallet_public_key(&wallet_key).await?;
    if let Some(amount_sol) = action.buyAmountSol.as_deref() {
        let required_lamports = parse_sol_to_lamports(amount_sol, "follow buy amount")?
            .saturating_add(FOLLOW_BUY_PRECHECK_BUFFER_LAMPORTS);
        let balance_lamports = fetch_balance_lamports(rpc_url, &public_key).await?;
        if balance_lamports < required_lamports {
            return Err(format!(
                "Wallet {wallet_key} has insufficient funds for follow buy. Need at least {required_lamports} lamports, found {balance_lamports}."
            ));
        }
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
        let action = FollowActionRecord {
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
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            blocksToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
        };
        required_action_precheck(&state.rpc_url, &action).await?;
    }
    Ok(())
}

fn has_capacity_for_new_job(health: &FollowDaemonHealth) -> bool {
    health.activeJobs < health.maxActiveJobs
}

async fn acquire_capacity_slot(
    semaphore: Arc<Semaphore>,
    wait_ms: u64,
    label: &str,
) -> Result<OwnedSemaphorePermit, String> {
    timeout(Duration::from_millis(wait_ms), semaphore.acquire_owned())
        .await
        .map_err(|_| format!("Timed out waiting for follow daemon {label} capacity."))?
        .map_err(|_| format!("Follow daemon {label} capacity is unavailable."))
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
                || snipe.submitDelayMs > 0
                || snipe.postBuySell.is_some())
    }) || request.followLaunch.devAutoSell.is_some()
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

async fn validate_watch_endpoint(endpoint: &str) -> Result<(), String> {
    timeout(Duration::from_secs(3), connect_async(endpoint))
        .await
        .map_err(|_| format!("Timed out connecting to websocket endpoint: {endpoint}"))?
        .map(|_| ())
        .map_err(|error| error.to_string())
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
    if payload.followLaunch.enabled {
        if let Err(error) = run_launch_gate_prechecks(&state, &payload).await {
            let health = build_health(&state).await;
            return Ok(Json(follow_ready_response(
                health,
                watch_endpoint,
                requires_websocket,
                false,
                Some(error),
            )));
        }
    }
    if requires_websocket {
        let Some(endpoint) = watch_endpoint.as_ref() else {
            let _ = state
                .store
                .update_health(
                    None,
                    FollowWatcherHealth::Failed,
                    FollowWatcherHealth::Failed,
                    FollowWatcherHealth::Failed,
                    Some("No websocket watch endpoint configured for follow daemon. Set SOLANA_WS_URL.".to_string()),
                )
                .await;
            let health = build_health(&state).await;
            return Ok(Json(follow_ready_response(
                health,
                None,
                true,
                false,
                Some("No websocket watch endpoint configured. Set SOLANA_WS_URL.".to_string()),
            )));
        };
        if let Err(error) = validate_watch_endpoint(endpoint).await {
            let _ = state
                .store
                .update_health(
                    Some(endpoint.clone()),
                    FollowWatcherHealth::Failed,
                    FollowWatcherHealth::Failed,
                    FollowWatcherHealth::Failed,
                    Some(error.clone()),
                )
                .await;
            let health = build_health(&state).await;
            return Ok(Json(follow_ready_response(
                health,
                Some(endpoint.clone()),
                true,
                false,
                Some(error),
            )));
        }
        let _ = state
            .store
            .update_health(
                Some(endpoint.clone()),
                FollowWatcherHealth::Healthy,
                FollowWatcherHealth::Healthy,
                FollowWatcherHealth::Healthy,
                None,
            )
            .await;
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
    authorize(&headers, &state)?;
    if payload.followLaunch.enabled && phase_two_follow_actions_enabled(&payload.followLaunch) {
        return Err(internal_error(
            "Per-sniper post-buy sells are not shipped yet. Phase 1 supports multi-sniper buys plus dev auto-sell only."
                .to_string(),
        ));
    }
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
    Ok(Json(job_response(&state, Some(job)).await))
}

async fn arm_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<FollowArmRequest>,
) -> Result<Json<FollowJobResponse>, (StatusCode, Json<Value>)> {
    authorize(&headers, &state)?;
    let trace_id = payload.traceId.clone();
    let already_armed = get_job(&state, &trace_id)
        .await
        .and_then(|job| job.launchSignature)
        .is_some();
    let job = state.store.arm_job(payload).await.map_err(internal_error)?;
    sync_follow_job_report(&state, &trace_id).await;
    if !already_armed {
        record_creation_sample(&state, &job).await;
        sync_follow_job_report(&state, &trace_id).await;
    }
    prepare_follow_job_buy_caches(state.clone(), &job).await;
    spawn_job_if_needed(state.clone(), trace_id).await;
    Ok(Json(job_response(&state, Some(job)).await))
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
    sync_follow_job_report(&state, &trace_id).await;
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
        sync_follow_job_report(&state, &trace_id).await;
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

async fn job_response(state: &Arc<AppState>, job: Option<FollowJobRecord>) -> FollowJobResponse {
    let health = build_health(state).await;
    let jobs = state.store.list_jobs().await;
    let timing_profiles = state.store.inner.read().await.timingProfiles.clone();
    follow_job_response(health, job, jobs, timing_profiles)
}

async fn sync_follow_job_report(state: &Arc<AppState>, trace_id: &str) {
    let Some(job) = get_job(state, trace_id).await else {
        return;
    };
    let Some(report_path) = job.reportPath.clone() else {
        return;
    };
    let health = build_health(state).await;
    let timing_profiles = state.store.inner.read().await.timingProfiles.clone();
    let snapshot = json!({
        "schemaVersion": FOLLOW_RESPONSE_SCHEMA_VERSION,
        "transport": health.controlTransport,
        "job": job,
        "health": health,
        "timingProfiles": timing_profiles,
    });
    let _report_guard = state.report_write_lock.lock().await;
    let _ = update_persisted_follow_daemon_snapshot(&report_path, &snapshot);
}

async fn get_job_watch_hub(state: &Arc<AppState>, trace_id: &str) -> Arc<JobWatchHub> {
    let mut hubs = state.watch_hubs.lock().await;
    if let Some(existing) = hubs.get(trace_id) {
        return existing.clone();
    }
    let (slot_tx, _) = watch::channel(None);
    let (signature_tx, _) = watch::channel(None);
    let (market_tx, _) = watch::channel(None);
    let hub = Arc::new(JobWatchHub {
        slot_tx,
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
    let action_ids = job
        .actions
        .iter()
        .map(|action| action.actionId.clone())
        .collect::<Vec<_>>();
    let mut had_failure = false;
    let mut action_tasks = JoinSet::new();
    for action_id in action_ids {
        let task_state = state.clone();
        let task_trace_id = trace_id.clone();
        action_tasks
            .spawn(async move { run_action_task(task_state, task_trace_id, action_id).await });
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
    let final_state =
        if final_job.cancelRequested || matches!(final_job.state, FollowJobState::Cancelled) {
            FollowJobState::Cancelled
        } else if has_failed_actions && has_confirmed_actions {
            FollowJobState::CompletedWithFailures
        } else if had_failure || has_failed_actions {
            FollowJobState::Failed
        } else {
            FollowJobState::Completed
        };
    let _ = state
        .store
        .finalize_job_state(&trace_id, final_state, None)
        .await;
    sync_follow_job_report(&state, &trace_id).await;
    clear_follow_buy_caches(&state, &trace_id).await;
    remove_job_watch_hub(&state, &trace_id).await;
}

async fn run_action_task(
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
    if let Some(transport_plan) = job.transportPlan.as_ref() {
        let attempt_count = action.attemptCount.saturating_add(1);
        let _ = state
            .store
            .record_sample(FollowTelemetrySample {
                schemaVersion: FOLLOW_TELEMETRY_SCHEMA_VERSION,
                traceId: job.traceId.clone(),
                actionId: action.actionId.clone(),
                actionType: action_type_name(&action.kind).to_string(),
                phase: "follow".to_string(),
                provider: job.execution.provider.clone(),
                endpointProfile: job.execution.endpointProfile.clone(),
                transportType: transport_plan.transportType.clone(),
                attemptCount: attempt_count,
                triggerType: action_trigger_type(action),
                delaySettingMs: action.submitDelayMs.or(action.delayMs),
                jitterMs: action.jitterMs,
                feeJitterBps: action.feeJitterBps,
                submitLatencyMs: None,
                confirmLatencyMs: None,
                launchToActionMs: None,
                launchToActionBlocks: None,
                scheduleSlipMs: None,
                outcome: classify_action_failure(error).to_string(),
                qualityLabel: "failed".to_string(),
                qualityWeight: 0,
                detail: Some(error.to_string()),
                writtenAtMs: now_ms(),
            })
            .await;
    }
}

async fn record_creation_sample(state: &Arc<AppState>, job: &FollowJobRecord) {
    let Some(transport_plan) = job.transportPlan.as_ref() else {
        return;
    };
    let mut detail_parts = Vec::new();
    if let Some(signature) = job.launchSignature.as_deref() {
        detail_parts.push(format!("signature={signature}"));
    }
    if let Some(block_height) = job.sendObservedBlockHeight {
        detail_parts.push(format!("sendObservedBlockHeight={block_height}"));
    }
    let _ = state
        .store
        .record_sample(FollowTelemetrySample {
            schemaVersion: FOLLOW_TELEMETRY_SCHEMA_VERSION,
            traceId: job.traceId.clone(),
            actionId: "launch-create".to_string(),
            actionType: "launch-create".to_string(),
            phase: "creation".to_string(),
            provider: job.execution.provider.clone(),
            endpointProfile: job.execution.endpointProfile.clone(),
            transportType: transport_plan.transportType.clone(),
            attemptCount: 1,
            triggerType: "submit-observed".to_string(),
            delaySettingMs: None,
            jitterMs: None,
            feeJitterBps: None,
            submitLatencyMs: None,
            confirmLatencyMs: None,
            launchToActionMs: Some(0),
            launchToActionBlocks: Some(0),
            scheduleSlipMs: None,
            outcome: "success".to_string(),
            qualityLabel: "creation-observed".to_string(),
            qualityWeight: 100,
            detail: if detail_parts.is_empty() {
                None
            } else {
                Some(detail_parts.join(" | "))
            },
            writtenAtMs: now_ms(),
        })
        .await;
}

fn should_retry_action(job: &FollowJobRecord, action: &FollowActionRecord, error: &str) -> bool {
    if action.attemptCount > job.followLaunch.constraints.retryBudget {
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

async fn execute_action(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Result<(), String> {
    let trace_id = job.traceId.clone();
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
    wait_for_action_eligibility(state.clone(), job, action).await?;
    ensure_action_not_cancelled(&state, &trace_id, &action.actionId).await?;
    state
        .store
        .update_action(&trace_id, &action.actionId, |record| {
            record.state = FollowActionState::Eligible;
        })
        .await?;
    sync_follow_job_report(&state, &trace_id).await;
    let wallet_key = selected_wallet_key_or_default(&action.walletEnvKey)
        .ok_or_else(|| format!("Wallet env key not found: {}", action.walletEnvKey))?;
    let wallet_secret = env::var(&wallet_key)
        .map_err(|_| format!("Wallet env var is missing: {wallet_key}"))
        .and_then(|value| read_keypair_bytes(&value))?;
    if action.precheckRequired {
        required_action_precheck(&state.rpc_url, action).await?;
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
    let transport_plan = job
        .transportPlan
        .as_ref()
        .ok_or_else(|| "Follow job missing transport plan.".to_string())?;
    let _compile_permit = acquire_capacity_slot(
        state.compile_slots.clone(),
        state.capacity_wait_ms,
        "compile",
    )
    .await?;
    let sell_percent = match action.kind {
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => Some(
            action
                .sellPercent
                .ok_or_else(|| "Follow sell missing percent.".to_string())?,
        ),
        FollowActionKind::SniperBuy => None,
    };
    let compiled = match action.kind {
        FollowActionKind::SniperBuy => {
            let amount = action
                .buyAmountSol
                .as_deref()
                .ok_or_else(|| "Follow buy missing amount.".to_string())?;
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
                resolve_hot_follow_buy_runtime_for_job(&state, job, launch_creator).await?;
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
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => {
            compile_follow_sell_transaction(
                &state.rpc_url,
                &job.execution,
                job.tokenMayhemMode,
                &job.jitoTipAccount,
                &wallet_secret,
                mint,
                launch_creator,
                sell_percent.unwrap_or_default(),
                job.preferPostSetupCreatorVaultForSell,
            )
            .await?
        }
    };
    let Some(mut compiled) = compiled else {
        return Err("Action had nothing to send for the current wallet state.".to_string());
    };
    ensure_action_not_cancelled(&state, &trace_id, &action.actionId).await?;
    let _send_permit =
        acquire_capacity_slot(state.send_slots.clone(), state.capacity_wait_ms, "send").await?;
    let mut retried_creator_vault = false;
    let (mut submitted, mut warnings, submit_ms, submit_latency) = loop {
        let started = Instant::now();
        match submit_transactions_for_transport(
            &state.rpc_url,
            transport_plan,
            &[compiled.clone()],
            &job.execution.commitment,
            job.execution.skipPreflight,
            job.execution.trackSendBlockHeight,
        )
        .await
        {
            Ok((submitted, warnings, submit_ms)) => {
                break (submitted, warnings, submit_ms, started.elapsed().as_millis());
            }
            Err(error)
                if !retried_creator_vault
                    && sell_percent.is_some()
                    && is_creator_vault_seed_mismatch(&error) =>
            {
                retried_creator_vault = true;
                sleep(Duration::from_millis(200)).await;
                compiled = compile_follow_sell_transaction(
                    &state.rpc_url,
                    &job.execution,
                    job.tokenMayhemMode,
                    &job.jitoTipAccount,
                    &wallet_secret,
                    mint,
                    launch_creator,
                    sell_percent.unwrap_or_default(),
                    job.preferPostSetupCreatorVaultForSell,
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
    let action_send_block_height = sent.sendObservedBlockHeight;
    let mut confirm_input = vec![sent.clone()];
    let (_, confirm_ms) = confirm_submitted_transactions_for_transport(
        &state.rpc_url,
        transport_plan,
        &mut confirm_input,
        &job.execution.commitment,
        job.execution.trackSendBlockHeight,
    )
    .await?;
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
    let attempt_count = action.attemptCount.saturating_add(1);
    let submitted_at_ms = now_ms();
    let launch_to_action_ms_value = launch_to_action_ms(job, submitted_at_ms);
    let launch_to_action_blocks_value = launch_to_action_blocks(job, action_send_block_height);
    let schedule_slip_ms_value = schedule_slip_ms(action, submitted_at_ms);
    let (outcome, quality_label, quality_weight, detail) = classify_success_quality(
        job,
        action,
        attempt_count,
        &warnings,
        confirm_ms,
        submitted_at_ms,
        action_send_block_height,
    );
    state
        .store
        .record_sample(FollowTelemetrySample {
            schemaVersion: FOLLOW_TELEMETRY_SCHEMA_VERSION,
            traceId: trace_id.clone(),
            actionId: action.actionId.clone(),
            actionType: action_type_name(&action.kind).to_string(),
            phase: "follow".to_string(),
            provider: job.execution.provider.clone(),
            endpointProfile: job.execution.endpointProfile.clone(),
            transportType: transport_plan.transportType.clone(),
            attemptCount: attempt_count,
            triggerType: action_trigger_type(action),
            delaySettingMs: action.submitDelayMs.or(action.delayMs),
            jitterMs: action.jitterMs,
            feeJitterBps: action.feeJitterBps,
            submitLatencyMs: Some(submit_latency.max(submit_ms)),
            confirmLatencyMs: Some(confirm_ms),
            launchToActionMs: launch_to_action_ms_value,
            launchToActionBlocks: launch_to_action_blocks_value,
            scheduleSlipMs: schedule_slip_ms_value,
            outcome,
            qualityLabel: quality_label,
            qualityWeight: quality_weight,
            detail,
            writtenAtMs: submitted_at_ms,
        })
        .await?;
    sync_follow_job_report(&state, &trace_id).await;
    Ok(())
}

fn action_type_name(kind: &FollowActionKind) -> &'static str {
    match kind {
        FollowActionKind::SniperBuy => "sniper-buy",
        FollowActionKind::DevAutoSell => "dev-auto-sell",
        FollowActionKind::SniperSell => "sniper-sell",
    }
}

fn action_trigger_type(action: &FollowActionRecord) -> String {
    let mut triggers = Vec::new();
    if action.submitDelayMs.is_some() {
        triggers.push("submit-time");
    }
    if action.targetBlockOffset.is_some() {
        triggers.push("block");
    }
    if action.delayMs.is_some() {
        triggers.push("time");
    }
    if action.marketCap.is_some() {
        triggers.push("market-cap");
    }
    if triggers.is_empty() {
        "immediate".to_string()
    } else {
        triggers.join("+")
    }
}

fn classify_action_failure(error: &str) -> &'static str {
    let normalized = error.to_ascii_lowercase();
    if normalized.contains("insufficient funds") {
        "insufficient-funds"
    } else if normalized.contains("cancel") {
        "cancelled"
    } else if normalized.contains("provider") || normalized.contains("rpc") {
        "provider-rejected"
    } else {
        "reverted"
    }
}

fn is_creator_vault_seed_mismatch(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("creator_vault")
        && (normalized.contains("constraintseeds")
            || normalized.contains("seeds constraint was violated"))
}

fn launch_to_action_ms(job: &FollowJobRecord, submitted_at_ms: u128) -> Option<u128> {
    job.submitAtMs
        .map(|submit_at_ms| submitted_at_ms.saturating_sub(submit_at_ms))
}

fn launch_to_action_blocks(
    job: &FollowJobRecord,
    action_send_block_height: Option<u64>,
) -> Option<u64> {
    match (job.sendObservedBlockHeight, action_send_block_height) {
        (Some(job_block_height), Some(action_block_height)) => {
            Some(action_block_height.saturating_sub(job_block_height))
        }
        _ => None,
    }
}

fn schedule_slip_ms(action: &FollowActionRecord, submitted_at_ms: u128) -> Option<u64> {
    action
        .scheduledForMs
        .map(|scheduled_for_ms| submitted_at_ms.saturating_sub(scheduled_for_ms) as u64)
}

fn classify_success_quality(
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    attempt_count: u32,
    warnings: &[String],
    confirm_ms: u128,
    submitted_at_ms: u128,
    action_send_block_height: Option<u64>,
) -> (String, String, u8, Option<String>) {
    let launch_to_ms = launch_to_action_ms(job, submitted_at_ms);
    let launch_to_blocks = launch_to_action_blocks(job, action_send_block_height);
    let schedule_slip = schedule_slip_ms(action, submitted_at_ms);
    let mut quality_weight: u8 = 100;
    let mut detail_parts = Vec::new();
    if attempt_count > 1 {
        quality_weight = quality_weight.saturating_sub(((attempt_count - 1) * 12).min(36) as u8);
        detail_parts.push(format!("attempts={attempt_count}"));
    }
    if !warnings.is_empty() {
        quality_weight = quality_weight.saturating_sub(10);
        detail_parts.push(format!("warnings={}", warnings.join(" | ")));
    }
    if let Some(slip_ms) = schedule_slip {
        if slip_ms > 100 {
            quality_weight = quality_weight.saturating_sub(10);
        }
        if slip_ms > 500 {
            quality_weight = quality_weight.saturating_sub(15);
        }
        detail_parts.push(format!("scheduleSlipMs={slip_ms}"));
    }
    if let Some(blocks) = launch_to_blocks {
        if blocks > 2 {
            quality_weight = quality_weight.saturating_sub(20);
        }
        detail_parts.push(format!("launchToActionBlocks={blocks}"));
    }
    if let Some(latency_ms) = launch_to_ms {
        detail_parts.push(format!("launchToActionMs={latency_ms}"));
    }
    if confirm_ms > 4_000 {
        quality_weight = quality_weight.saturating_sub(10);
    }
    if confirm_ms > 8_000 {
        quality_weight = quality_weight.saturating_sub(15);
    }
    detail_parts.push(format!("confirmLatencyMs={confirm_ms}"));
    let missed_window = matches!(action.kind, FollowActionKind::SniperBuy)
        && action.targetBlockOffset.zip(launch_to_blocks).is_some_and(
            |(target_offset, observed_offset)| {
                observed_offset > u64::from(target_offset).saturating_add(1)
            },
        );
    let late = !missed_window
        && (schedule_slip.unwrap_or_default() > 750
            || confirm_ms > 8_000
            || launch_to_blocks.is_some_and(|blocks| blocks > 2));
    let degraded = !late && (attempt_count > 1 || !warnings.is_empty() || confirm_ms > 4_000);
    let (outcome, quality_label) = if missed_window {
        quality_weight = quality_weight.min(25);
        ("missed-window".to_string(), "missed-window".to_string())
    } else if late {
        quality_weight = quality_weight.min(55);
        ("late".to_string(), "late".to_string())
    } else if degraded {
        quality_weight = quality_weight.min(80);
        ("degraded-success".to_string(), "degraded".to_string())
    } else {
        ("success".to_string(), "clean".to_string())
    };
    let detail = if detail_parts.is_empty() {
        None
    } else {
        Some(detail_parts.join(" | "))
    };
    (outcome, quality_label, quality_weight.max(5), detail)
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
) -> Result<(), String> {
    if action.requireConfirmation {
        wait_for_signature_confirmation(state.clone(), job, &action.actionId).await?;
    }
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
            if let Some(offset) = action.targetBlockOffset {
                wait_for_slot_offset(state.clone(), job, &action.actionId, u64::from(offset))
                    .await?;
            }
        }
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => {
            let has_time = action.scheduledForMs.is_some();
            let has_market = action.marketCap.is_some();
            let has_slot = action.targetBlockOffset.is_some();
            if has_slot && has_market {
                tokio::select! {
                    result = wait_for_slot_offset(state.clone(), job, &action.actionId, u64::from(action.targetBlockOffset.unwrap())) => result?,
                    result = wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId) => result?,
                }
            } else if has_slot {
                wait_for_slot_offset(
                    state.clone(),
                    job,
                    &action.actionId,
                    u64::from(action.targetBlockOffset.unwrap()),
                )
                .await?;
            } else if has_time && has_market {
                tokio::select! {
                    result = wait_until_ms(state.clone(), &job.traceId, Some(&action.actionId), action.scheduledForMs.unwrap()) => result?,
                    result = wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId) => result?,
                }
            } else if let Some(schedule_ms) = action.scheduledForMs {
                wait_until_ms(
                    state.clone(),
                    &job.traceId,
                    Some(&action.actionId),
                    schedule_ms,
                )
                .await?;
            } else if has_market {
                wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId).await?;
            }
        }
    }
    Ok(())
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
    if attempt >= WATCHER_MAX_RECONNECT_ATTEMPTS {
        set_watcher_health(
            state,
            kind,
            FollowWatcherHealth::Failed,
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
) -> watch::Receiver<Option<Result<(), String>>> {
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

async fn ensure_slot_watcher(
    state: Arc<AppState>,
    job: &FollowJobRecord,
) -> watch::Receiver<Option<u64>> {
    let hub = get_job_watch_hub(&state, &job.traceId).await;
    let mut started = hub.started.lock().await;
    if !started.slot {
        started.slot = true;
        let task_state = state.clone();
        let task_job = job.clone();
        let task_tx = hub.slot_tx.clone();
        tokio::spawn(async move {
            run_slot_watcher(task_state, task_job, task_tx).await;
        });
    }
    drop(started);
    hub.slot_tx.subscribe()
}

async fn ensure_market_watcher(
    state: Arc<AppState>,
    job: &FollowJobRecord,
) -> watch::Receiver<Option<u64>> {
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

async fn run_signature_watcher(
    state: Arc<AppState>,
    job: FollowJobRecord,
    tx: watch::Sender<Option<Result<(), String>>>,
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
    loop {
        let session = async {
            let mut ws = open_subscription_socket(&endpoint).await?;
            subscribe(
                &mut ws,
                "signatureSubscribe",
                json!([
                    signature,
                    {
                        "commitment": "processed",
                        "enableReceivedNotification": true
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
                    return Ok(());
                }
            }
        }
        .await;
        match session {
            Ok(()) => {
                let _ = tx.send(Some(Ok(())));
                set_watcher_health(
                    &state,
                    WatcherKind::Signature,
                    FollowWatcherHealth::Healthy,
                    Some(endpoint.clone()),
                    None,
                )
                .await;
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

async fn run_slot_watcher(
    state: Arc<AppState>,
    job: FollowJobRecord,
    tx: watch::Sender<Option<u64>>,
) {
    let Ok(endpoint) = resolve_job_watch_endpoint(&job) else {
        return;
    };
    let mut attempt: u32 = 0;
    loop {
        let session = async {
            let mut ws = open_subscription_socket(&endpoint).await?;
            subscribe(&mut ws, "slotSubscribe", json!([])).await?;
            loop {
                ensure_job_not_cancelled(&state, &job.traceId).await?;
                let message = next_json_message(&mut ws).await?;
                let slot = message
                    .get("params")
                    .and_then(|params| params.get("result"))
                    .and_then(|result| result.get("slot"))
                    .and_then(Value::as_u64);
                if let Some(slot) = slot {
                    let _ = tx.send(Some(slot));
                }
            }
        }
        .await;
        match session {
            Ok(()) => return,
            Err(error) => {
                attempt = attempt.saturating_add(1);
                if handle_watcher_retry(
                    &state,
                    &job.traceId,
                    WatcherKind::Slot,
                    &endpoint,
                    attempt,
                    error,
                )
                .await
                .is_err()
                {
                    return;
                }
            }
        }
    }
}

async fn run_market_watcher(
    state: Arc<AppState>,
    job: FollowJobRecord,
    tx: watch::Sender<Option<u64>>,
) {
    let Some(mint) = job.mint.clone() else {
        return;
    };
    let Ok(endpoint) = resolve_job_watch_endpoint(&job) else {
        return;
    };
    let Ok(bonding_curve) = pump_bonding_curve_address(&mint) else {
        return;
    };
    let mut attempt: u32 = 0;
    loop {
        let session = async {
            let mut ws = open_subscription_socket(&endpoint).await?;
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
                ensure_job_not_cancelled(&state, &job.traceId).await?;
                let _ = next_json_message(&mut ws).await?;
                let snapshot = fetch_pump_market_snapshot(&state.rpc_url, &mint).await?;
                let _ = tx.send(Some(snapshot.marketCapLamports));
            }
        }
        .await;
        match session {
            Ok(()) => return,
            Err(error) => {
                attempt = attempt.saturating_add(1);
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
) -> Result<(), String> {
    let mut rx = ensure_signature_watcher(state.clone(), job).await;
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, action_id).await?;
        let current = rx.borrow().clone();
        match current {
            Some(Ok(())) => return Ok(()),
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
    target_offset: u64,
) -> Result<(), String> {
    let mut rx = ensure_slot_watcher(state.clone(), job).await;
    let mut base_slot: Option<u64> = None;
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, action_id).await?;
        let current_slot = *rx.borrow();
        if let Some(slot) = current_slot {
            let baseline = *base_slot.get_or_insert(slot);
            if slot >= baseline.saturating_add(target_offset) {
                return Ok(());
            }
        }
        rx.changed()
            .await
            .map_err(|_| "Shared slot watcher stopped unexpectedly.".to_string())?;
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
    let mut rx = ensure_market_watcher(state.clone(), job).await;
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, action_id).await?;
        let current_market_cap = *rx.borrow();
        if let Some(market_cap) = current_market_cap {
            let matches = match trigger.direction.as_str() {
                "lte" => market_cap <= threshold,
                _ => market_cap >= threshold,
            };
            if matches {
                set_watcher_health(
                    &state,
                    WatcherKind::Market,
                    FollowWatcherHealth::Healthy,
                    job.transportPlan
                        .as_ref()
                        .and_then(|plan| plan.watchEndpoint.clone()),
                    None,
                )
                .await;
                return Ok(());
            }
        }
        rx.changed()
            .await
            .map_err(|_| "Shared market watcher stopped unexpectedly.".to_string())?;
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
        .ok_or_else(|| "No websocket watch endpoint configured for follow job. Set SOLANA_WS_URL.".to_string())
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
    let max_active_jobs = configured_limit("LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS", 64);
    let max_concurrent_compiles = configured_limit("LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES", 8);
    let max_concurrent_sends = configured_limit("LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS", 8);
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
        compile_slots: Arc::new(Semaphore::new(max_concurrent_compiles)),
        send_slots: Arc::new(Semaphore::new(max_concurrent_sends)),
        watch_hubs: Arc::new(Mutex::new(HashMap::new())),
        prepared_follow_buys: Arc::new(Mutex::new(HashMap::new())),
        hot_follow_buy_runtime: Arc::new(Mutex::new(HashMap::new())),
        hot_follow_buy_tasks: Arc::new(Mutex::new(HashMap::new())),
    });
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
