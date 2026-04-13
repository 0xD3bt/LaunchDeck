#![allow(non_snake_case)]

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures_util::{SinkExt, StreamExt, future::join_all};
use launchdeck_engine::{
    app_logs::{record_error, record_info, record_warn},
    bags_native::{BagsFollowBuyContext, BagsMarketSnapshot, load_follow_buy_context},
    bonk_native::{
        BonkUsd1RouteSetup, NativeBonkPoolContext, load_live_follow_buy_pool_context,
        load_live_follow_buy_usd1_route_setup,
    },
    crypto::install_rustls_crypto_provider,
    follow::{
        DeferredSetupState, FOLLOW_RESPONSE_SCHEMA_VERSION, FollowActionKind, FollowActionRecord,
        FollowActionState, FollowArmRequest, FollowCancelRequest, FollowDaemonHealth,
        FollowDaemonStore, FollowJobRecord, FollowJobResponse, FollowJobState, FollowReadyRequest,
        FollowReadyResponse, FollowReserveRequest, FollowStopAllRequest, FollowWatcherHealth,
        follow_job_response, follow_ready_response, should_use_post_setup_creator_vault_for_buy,
        should_use_post_setup_creator_vault_for_sell,
    },
    launchpad_dispatch::{
        FollowBuyCompileRequest, FollowSellCompileRequest, LaunchpadMarketSnapshot,
        compile_follow_buy_for_launchpad, compile_follow_sell_for_launchpad,
        derive_follow_owner_token_account_for_launchpad,
        fetch_market_snapshot_for_launchpad,
    },
    observability::update_persisted_follow_daemon_snapshot,
    paths,
    pump_native::{
        PreparedFollowBuyRuntime, PreparedFollowBuyStatic, compile_follow_sell_transaction,
        fetch_pump_market_snapshot, finalize_follow_buy_transaction, prepare_follow_buy_runtime,
        prepare_follow_buy_static, pump_bonding_curve_address,
    },
    report::{FollowActionTimings, FollowJobTimings, configured_benchmark_mode},
    rpc::{
        CompiledTransaction, confirm_submitted_transactions_for_transport,
        fetch_current_block_height, fetch_current_block_height_fresh, fetch_current_slot,
        fetch_current_slot_fresh, fetch_latest_blockhash_fresh_or_recent,
        prewarm_watch_websocket_endpoint, spawn_blockhash_refresh_task, submit_transactions_for_transport,
    },
    transport::{
        TransportPlan, build_transport_plan, configured_enable_helius_transaction_subscribe,
        configured_helius_rpc_url_trimmed, configured_watch_endpoints_for_provider,
        prefers_helius_transaction_subscribe_path,
        resolved_helius_transaction_subscribe_ws_url,
    },
    wallet::{
        fetch_balance_lamports, fetch_token_balance, load_solana_wallet_by_env_key,
        public_key_from_secret, selected_wallet_key_or_default,
    },
};
use reqwest::Client;
use serde_json::{Value, json};
use solana_sdk::{
    hash::Hash,
    message::VersionedMessage,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex as StdMutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex, OwnedSemaphorePermit, Semaphore, watch},
    task::{JoinHandle, JoinSet},
    time::{Instant, sleep, timeout},
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const USD1_MINT: &str = "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB";
const USD_MICRO_DECIMALS: u32 = 6;
const WRAPPED_SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const SOL_USD_PRICE_HTTP_URL: &str =
    "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd";
const SOL_USD_PRICE_CACHE_TTL_MS: u128 = 30_000;
const LAMPORTS_PER_SOL: u128 = 1_000_000_000;

#[derive(Clone, Copy)]
struct CachedSolUsdPrice {
    micro_usd_per_sol: u64,
    fetched_at_ms: u128,
}

fn sol_usd_price_cache() -> &'static StdMutex<Option<CachedSolUsdPrice>> {
    static CACHE: OnceLock<StdMutex<Option<CachedSolUsdPrice>>> = OnceLock::new();
    CACHE.get_or_init(|| StdMutex::new(None))
}

fn configured_sol_usd_http_price_url() -> String {
    env::var("LAUNCHDECK_SOL_USD_HTTP_PRICE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| SOL_USD_PRICE_HTTP_URL.to_string())
}

fn resolved_helius_sol_price_rpc_url(primary_rpc_url: &str) -> Option<String> {
    if let Some(url) = configured_helius_rpc_url_trimmed() {
        return Some(url);
    }
    let trimmed = primary_rpc_url.trim();
    if trimmed.is_empty() || !trimmed.to_ascii_lowercase().contains("helius") {
        return None;
    }
    Some(trimmed.to_string())
}

fn stable_quote_asset_decimals(quote_asset: &str) -> Option<u32> {
    match quote_asset.trim().to_lowercase().as_str() {
        "usd" | "usd1" | "usdc" | "usdt" => Some(6),
        _ => None,
    }
}

fn scale_value_between_decimals(
    value: u64,
    from_decimals: u32,
    to_decimals: u32,
) -> Result<u64, String> {
    if from_decimals == to_decimals {
        return Ok(value);
    }
    let factor = 10u128.pow(from_decimals.abs_diff(to_decimals));
    let scaled = if from_decimals < to_decimals {
        u128::from(value).saturating_mul(factor)
    } else {
        u128::from(value) / factor
    };
    u64::try_from(scaled).map_err(|_| "Scaled market-cap value overflowed u64.".to_string())
}

fn decimal_price_to_micro_usd(price: f64) -> Result<u64, String> {
    if !price.is_finite() || price <= 0.0 {
        return Err(format!(
            "SOL/USD HTTP price returned invalid value {price}."
        ));
    }
    let scaled = (price * 1_000_000_f64).round();
    if !scaled.is_finite() || scaled <= 0.0 || scaled > u64::MAX as f64 {
        return Err("Scaled SOL/USD HTTP price overflowed u64.".to_string());
    }
    Ok(scaled as u64)
}

fn sol_quote_units_to_usd_micros(quote_units: u64, micro_usd_per_sol: u64) -> Result<u64, String> {
    let usd_micros = (u128::from(quote_units) * u128::from(micro_usd_per_sol)) / LAMPORTS_PER_SOL;
    u64::try_from(usd_micros).map_err(|_| "USD market-cap conversion overflowed u64.".to_string())
}

async fn fetch_sol_usd_micro_price_http() -> Result<u64, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|error| format!("Failed to build SOL/USD HTTP client: {error}"))?;
    let response = client
        .get(configured_sol_usd_http_price_url())
        .header("accept", "application/json")
        .header("user-agent", "launchdeck/sol-usd-price")
        .send()
        .await
        .map_err(|error| format!("Failed to fetch SOL/USD HTTP price: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "SOL/USD HTTP price request failed with status {}.",
            response.status()
        ));
    }
    let payload = response
        .json::<Value>()
        .await
        .map_err(|error| format!("Failed to decode SOL/USD HTTP price response: {error}"))?;
    let price = payload
        .get("solana")
        .and_then(|entry| entry.get("usd"))
        .or_else(|| payload.get("usd"))
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            "SOL/USD HTTP price response was missing a numeric usd field.".to_string()
        })?;
    decimal_price_to_micro_usd(price)
}

async fn fetch_sol_usd_micro_price_helius(primary_rpc_url: &str) -> Result<u64, String> {
    let helius_rpc_url = resolved_helius_sol_price_rpc_url(primary_rpc_url)
        .ok_or_else(|| "No Helius RPC URL is configured for SOL/USD price lookup.".to_string())?;
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|error| format!("Failed to build Helius SOL/USD client: {error}"))?;
    let response = client
        .post(helius_rpc_url)
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "launchdeck-sol-price",
            "method": "getAsset",
            "params": {
                "id": WRAPPED_SOL_MINT,
                "displayOptions": {
                    "showFungible": true
                }
            }
        }))
        .send()
        .await
        .map_err(|error| format!("Failed to fetch SOL/USD price from Helius getAsset: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Helius SOL/USD getAsset request failed with status {}.",
            response.status()
        ));
    }
    let payload = response
        .json::<Value>()
        .await
        .map_err(|error| format!("Failed to decode Helius SOL/USD response: {error}"))?;
    if let Some(error) = payload.get("error") {
        return Err(error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Helius SOL/USD getAsset request failed.")
            .to_string());
    }
    let price = payload
        .get("result")
        .and_then(|value| value.get("token_info"))
        .and_then(|value| value.get("price_info"))
        .and_then(|value| value.get("price_per_token"))
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            "Helius SOL/USD getAsset response was missing token_info.price_info.price_per_token."
                .to_string()
        })?;
    decimal_price_to_micro_usd(price)
}

async fn fetch_sol_usd_micro_price(rpc_url: &str) -> Result<u64, String> {
    let now_ms = now_ms();
    if let Ok(cache) = sol_usd_price_cache().lock()
        && let Some(entry) = *cache
        && now_ms.saturating_sub(entry.fetched_at_ms) <= SOL_USD_PRICE_CACHE_TTL_MS
    {
        return Ok(entry.micro_usd_per_sol);
    }
    let micro_usd_per_sol = match fetch_sol_usd_micro_price_helius(rpc_url).await {
        Ok(value) => value,
        Err(_error) => fetch_sol_usd_micro_price_http().await?,
    };
    if let Ok(mut cache) = sol_usd_price_cache().lock() {
        *cache = Some(CachedSolUsdPrice {
            micro_usd_per_sol,
            fetched_at_ms: now_ms,
        });
    }
    Ok(micro_usd_per_sol)
}

async fn quote_units_to_usd_micros(
    rpc_url: &str,
    quote_units: u64,
    quote_asset: &str,
) -> Result<u64, String> {
    if let Some(decimals) = stable_quote_asset_decimals(quote_asset) {
        return scale_value_between_decimals(quote_units, decimals, USD_MICRO_DECIMALS);
    }
    let micro_usd_per_sol = fetch_sol_usd_micro_price(rpc_url).await?;
    sol_quote_units_to_usd_micros(quote_units, micro_usd_per_sol)
}

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
    bonk_follow_buy_contexts: Arc<Mutex<HashMap<String, CachedBonkFollowBuyContext>>>,
    bonk_usd1_route_setups: Arc<Mutex<HashMap<String, CachedBonkUsd1RouteSetup>>>,
    bags_follow_buy_contexts: Arc<Mutex<HashMap<String, CachedBagsFollowBuyContext>>>,
    bags_market_snapshots: Arc<Mutex<HashMap<String, CachedBagsMarketSnapshot>>>,
    follow_report_flushes: Arc<Mutex<HashMap<String, PendingFollowReportFlush>>>,
    follow_report_flush_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    watch_endpoint_health: Arc<Mutex<HashMap<String, CachedWatchEndpointHealth>>>,
    wallet_precheck_cache: Arc<Mutex<HashMap<String, CachedWalletPrecheck>>>,
    #[allow(dead_code)]
    offset_worker: Arc<OffsetWorkerHub>,
}

#[derive(Clone)]
struct CachedFollowBuyRuntime {
    prepared: PreparedFollowBuyRuntime,
    refreshed_at_ms: u128,
}

#[derive(Clone)]
struct CachedBonkFollowBuyContext {
    context: NativeBonkPoolContext,
    refreshed_at_ms: u128,
}

#[derive(Clone)]
struct CachedBonkUsd1RouteSetup {
    setup: BonkUsd1RouteSetup,
    refreshed_at_ms: u128,
}

#[derive(Clone)]
struct CachedBagsFollowBuyContext {
    context: BagsFollowBuyContext,
    refreshed_at_ms: u128,
}

#[derive(Clone)]
struct CachedBagsMarketSnapshot {
    snapshot: BagsMarketSnapshot,
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
    slot_tx: watch::Sender<Option<Result<u64, String>>>,
    signature_tx: watch::Sender<Option<Result<u64, String>>>,
    market_tx: watch::Sender<Option<Result<u64, String>>>,
    started: Mutex<JobWatchStarted>,
}

#[allow(dead_code)]
struct OffsetWorkerHub {
    consumers: Mutex<HashMap<String, OffsetConsumer>>,
    running: Mutex<bool>,
    last_observed_block_height: Mutex<Option<u64>>,
    last_observed_at_ms: Mutex<Option<u128>>,
}

#[allow(dead_code)]
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
const FOLLOW_WATCHER_RPC_POLL_INTERVAL_MS: u64 = 400;
const BAGS_MARKET_SNAPSHOT_CACHE_TTL_MS: u128 = 900;
#[allow(dead_code)]
const DEFAULT_FOLLOW_OFFSET_POLL_INTERVAL_MS: u64 = 400;
#[allow(dead_code)]
const OFFSET_SAME_BLOCK_REPOLL_DELAY_MS: u64 = 25;
#[allow(dead_code)]
const OFFSET_SAME_BLOCK_REPOLL_LIMIT: usize = 3;
const FOLLOW_TRIGGER_COMPILE_BLOCKHASH_MIN_REMAINING_BLOCKS: u64 = 20;
const DEFERRED_SETUP_CONFIRMATION_TIMEOUT_SECS: u64 = 10;

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or_default()
}

fn shared_http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(Client::new)
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

#[allow(dead_code)]
fn configured_follow_offset_poll_interval_ms() -> u64 {
    env::var("LAUNCHDECK_FOLLOW_OFFSET_POLL_INTERVAL_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_FOLLOW_OFFSET_POLL_INTERVAL_MS)
}

#[allow(dead_code)]
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

fn opt_out_flag_enabled(value: Option<&str>) -> bool {
    value
        .map(|raw| {
            let normalized = raw.trim().to_ascii_lowercase();
            !matches!(normalized.as_str(), "0" | "false" | "off" | "no")
        })
        .unwrap_or(true)
}

fn configured_enable_pump_buy_creator_vault_auto_retry() -> bool {
    opt_out_flag_enabled(
        env::var("LAUNCHDECK_ENABLE_PUMP_BUY_CREATOR_VAULT_AUTO_RETRY")
            .ok()
            .as_deref(),
    )
}

fn configured_enable_pump_sell_creator_vault_auto_retry() -> bool {
    opt_out_flag_enabled(
        env::var("LAUNCHDECK_ENABLE_PUMP_SELL_CREATOR_VAULT_AUTO_RETRY")
            .ok()
            .as_deref(),
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
        record_warn("follow-daemon", "Unauthorized follow daemon request.", None);
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

fn hot_follow_buy_runtime_cache_key(
    trace_id: &str,
    prefer_post_setup_creator_vault: bool,
) -> String {
    format!(
        "{trace_id}:{}",
        if prefer_post_setup_creator_vault {
            "post-setup"
        } else {
            "launch-creator"
        }
    )
}

fn shared_hot_follow_buy_runtime_cache_key(
    compile_context_key: &str,
    prefer_post_setup_creator_vault: bool,
) -> String {
    format!(
        "{compile_context_key}:{}",
        if prefer_post_setup_creator_vault {
            "post-setup"
        } else {
            "launch-creator"
        }
    )
}

fn shared_trigger_buy_compile_context_key(
    trace_id: &str,
    observed_slot: u64,
    launchpad: &str,
    quote_asset: &str,
    launch_mode: &str,
) -> String {
    format!(
        "{trace_id}:{observed_slot}:{launchpad}:{quote_asset}:{launch_mode}"
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

async fn cache_shared_hot_follow_buy_runtime(
    state: &Arc<AppState>,
    compile_context_key: &str,
    prefer_post_setup_creator_vault: bool,
    prepared: PreparedFollowBuyRuntime,
) {
    let mut cache = state.hot_follow_buy_runtime.lock().await;
    cache.insert(
        shared_hot_follow_buy_runtime_cache_key(
            compile_context_key,
            prefer_post_setup_creator_vault,
        ),
        CachedFollowBuyRuntime {
            prepared,
            refreshed_at_ms: now_ms(),
        },
    );
}

async fn cache_bonk_follow_buy_context(
    state: &Arc<AppState>,
    key: &str,
    context: NativeBonkPoolContext,
) {
    let mut cache = state.bonk_follow_buy_contexts.lock().await;
    cache.insert(
        key.to_string(),
        CachedBonkFollowBuyContext {
            context,
            refreshed_at_ms: now_ms(),
        },
    );
}

async fn cache_bonk_usd1_route_setup(
    state: &Arc<AppState>,
    key: &str,
    setup: BonkUsd1RouteSetup,
) {
    let mut cache = state.bonk_usd1_route_setups.lock().await;
    cache.insert(
        key.to_string(),
        CachedBonkUsd1RouteSetup {
            setup,
            refreshed_at_ms: now_ms(),
        },
    );
}

async fn cache_bags_follow_buy_context(
    state: &Arc<AppState>,
    key: &str,
    context: BagsFollowBuyContext,
) {
    let mut cache = state.bags_follow_buy_contexts.lock().await;
    cache.insert(
        key.to_string(),
        CachedBagsFollowBuyContext {
            context,
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
    cache
        .get(&hot_follow_buy_runtime_cache_key(
            trace_id,
            prefer_post_setup_creator_vault,
        ))
        .cloned()
}

async fn get_shared_hot_follow_buy_runtime(
    state: &Arc<AppState>,
    compile_context_key: &str,
    prefer_post_setup_creator_vault: bool,
) -> Option<CachedFollowBuyRuntime> {
    let cache = state.hot_follow_buy_runtime.lock().await;
    cache
        .get(&shared_hot_follow_buy_runtime_cache_key(
            compile_context_key,
            prefer_post_setup_creator_vault,
        ))
        .cloned()
}

async fn get_bonk_follow_buy_context(
    state: &Arc<AppState>,
    key: &str,
) -> Option<CachedBonkFollowBuyContext> {
    let cache = state.bonk_follow_buy_contexts.lock().await;
    cache.get(key).cloned()
}

async fn get_bonk_usd1_route_setup(
    state: &Arc<AppState>,
    key: &str,
) -> Option<CachedBonkUsd1RouteSetup> {
    let cache = state.bonk_usd1_route_setups.lock().await;
    cache.get(key).cloned()
}

async fn get_bags_follow_buy_context(
    state: &Arc<AppState>,
    key: &str,
) -> Option<CachedBagsFollowBuyContext> {
    let cache = state.bags_follow_buy_contexts.lock().await;
    cache.get(key).cloned()
}

async fn cache_bags_market_snapshot(
    state: &Arc<AppState>,
    trace_id: &str,
    snapshot: BagsMarketSnapshot,
) {
    let mut cache = state.bags_market_snapshots.lock().await;
    cache.insert(
        trace_id.to_string(),
        CachedBagsMarketSnapshot {
            snapshot,
            refreshed_at_ms: now_ms(),
        },
    );
}

async fn get_bags_market_snapshot(
    state: &Arc<AppState>,
    trace_id: &str,
) -> Option<CachedBagsMarketSnapshot> {
    let cache = state.bags_market_snapshots.lock().await;
    cache.get(trace_id).cloned()
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
    {
        let mut cache = state.bonk_follow_buy_contexts.lock().await;
        cache.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut cache = state.bonk_usd1_route_setups.lock().await;
        cache.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut cache = state.bags_follow_buy_contexts.lock().await;
        cache.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut cache = state.bags_market_snapshots.lock().await;
        cache.remove(trace_id);
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
    {
        let mut cache = state.bonk_follow_buy_contexts.lock().await;
        cache.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut cache = state.bonk_usd1_route_setups.lock().await;
        cache.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut cache = state.bags_follow_buy_contexts.lock().await;
        cache.retain(|key, _| !key.starts_with(&format!("{trace_id}:")));
    }
    {
        let mut cache = state.bags_market_snapshots.lock().await;
        cache.remove(trace_id);
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

async fn prewarm_bags_follow_job_cache(state: Arc<AppState>, job: &FollowJobRecord) {
    if job.launchpad != "bagsapp" {
        return;
    }
    let Some(mint) = job.mint.as_deref() else {
        return;
    };
    match fetch_market_snapshot_for_launchpad(
        "bagsapp",
        &state.rpc_url,
        mint,
        &job.quoteAsset,
        job.bagsLaunch.as_ref(),
    )
    .await
    {
        Ok(Some(LaunchpadMarketSnapshot::Bags(snapshot))) => {
            cache_bags_market_snapshot(&state, &job.traceId, snapshot).await;
        }
        Ok(Some(other)) => {
            record_warn(
                "follow-daemon",
                "Unexpected market snapshot variant while prewarming Bags market cache.",
                Some(json!({
                    "traceId": job.traceId,
                    "mint": mint,
                    "variant": format!("{other:?}"),
                })),
            );
        }
        Ok(None) => {}
        Err(error) => {
            record_warn(
                "follow-daemon",
                "Failed to prewarm Bags local market snapshot.",
                Some(json!({
                    "traceId": job.traceId,
                    "mint": mint,
                    "error": error,
                })),
            );
        }
    }
}

async fn prepare_follow_job_buy_caches(state: Arc<AppState>, job: &FollowJobRecord) {
    prewarm_bags_follow_job_cache(state.clone(), job).await;
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
    action: &FollowActionRecord,
    launch_creator: &str,
    prefer_post_setup_creator_vault: bool,
) -> Result<PreparedFollowBuyRuntime, String> {
    if let Some(observed_slot) = action.eligibilityObservedSlot {
        let compile_context_key = shared_trigger_buy_compile_context_key(
            &job.traceId,
            observed_slot,
            &job.launchpad,
            &job.quoteAsset,
            &job.launchMode,
        );
        if let Some(cached) = get_shared_hot_follow_buy_runtime(
            state,
            &compile_context_key,
            prefer_post_setup_creator_vault,
        )
        .await
            && now_ms().saturating_sub(cached.refreshed_at_ms) <= HOT_FOLLOW_BUY_MAX_AGE_MS
        {
            return Ok(cached.prepared);
        }
    }
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

async fn resolve_shared_bonk_follow_buy_context_for_job(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Option<NativeBonkPoolContext> {
    let observed_slot = action.eligibilityObservedSlot?;
    let key = shared_trigger_buy_compile_context_key(
        &job.traceId,
        observed_slot,
        &job.launchpad,
        &job.quoteAsset,
        &job.launchMode,
    );
    let cached = get_bonk_follow_buy_context(state, &key).await?;
    if now_ms().saturating_sub(cached.refreshed_at_ms) > HOT_FOLLOW_BUY_MAX_AGE_MS {
        return None;
    }
    Some(cached.context)
}

async fn resolve_shared_bonk_usd1_route_setup_for_job(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Option<BonkUsd1RouteSetup> {
    let observed_slot = action.eligibilityObservedSlot?;
    let key = shared_trigger_buy_compile_context_key(
        &job.traceId,
        observed_slot,
        &job.launchpad,
        &job.quoteAsset,
        &job.launchMode,
    );
    let cached = get_bonk_usd1_route_setup(state, &key).await?;
    if now_ms().saturating_sub(cached.refreshed_at_ms) > HOT_FOLLOW_BUY_MAX_AGE_MS {
        return None;
    }
    Some(cached.setup)
}

async fn resolve_shared_bags_follow_buy_context_for_job(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Option<BagsFollowBuyContext> {
    let observed_slot = action.eligibilityObservedSlot?;
    let key = shared_trigger_buy_compile_context_key(
        &job.traceId,
        observed_slot,
        &job.launchpad,
        &job.quoteAsset,
        &job.launchMode,
    );
    let cached = get_bags_follow_buy_context(state, &key).await?;
    if now_ms().saturating_sub(cached.refreshed_at_ms) > HOT_FOLLOW_BUY_MAX_AGE_MS {
        return None;
    }
    Some(cached.context)
}

fn action_needs_trigger_time_buy_compile_prep(action: &FollowActionRecord) -> bool {
    matches!(action.kind, FollowActionKind::SniperBuy) && action.preSignedTransactions.is_empty()
}

async fn prepare_trigger_time_buy_compile_batch(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    actions: &[FollowActionRecord],
    observed_slot: Option<u64>,
) {
    let Some(observed_slot) = observed_slot else {
        return;
    };
    if !actions
        .iter()
        .any(action_needs_trigger_time_buy_compile_prep)
    {
        return;
    }
    let prep_started = Instant::now();
    let compile_context_key = shared_trigger_buy_compile_context_key(
        &job.traceId,
        observed_slot,
        &job.launchpad,
        &job.quoteAsset,
        &job.launchMode,
    );
    if let Err(error) = fetch_latest_blockhash_fresh_or_recent(
        &state.rpc_url,
        &job.execution.commitment,
        FOLLOW_TRIGGER_COMPILE_BLOCKHASH_MIN_REMAINING_BLOCKS,
    )
    .await
    {
        record_warn(
            "follow-daemon",
            "Failed to prime shared trigger-time blockhash for compile batch.",
            Some(json!({
                "traceId": job.traceId,
                "compileContextKey": compile_context_key,
                "error": error,
            })),
        );
    }
    match job.launchpad.as_str() {
        "pump" => {
            let Some(mint) = job.mint.as_deref() else {
                return;
            };
            let Some(launch_creator) = job.launchCreator.as_deref() else {
                return;
            };
            for prefer_post_setup_creator_vault in [false, true] {
                if let Ok(prepared_runtime) = prepare_follow_buy_runtime(
                    &state.rpc_url,
                    mint,
                    launch_creator,
                    prefer_post_setup_creator_vault,
                )
                .await
                {
                    cache_shared_hot_follow_buy_runtime(
                        state,
                        &compile_context_key,
                        prefer_post_setup_creator_vault,
                        prepared_runtime,
                    )
                    .await;
                }
            }
        }
        "bonk" => {
            let Some(mint) = job.mint.as_deref() else {
                return;
            };
            match load_live_follow_buy_pool_context(
                &state.rpc_url,
                mint,
                &job.quoteAsset,
                &job.execution.commitment,
            )
            .await
            {
                Ok(context) => {
                    cache_bonk_follow_buy_context(state, &compile_context_key, context).await;
                    if job.quoteAsset.eq_ignore_ascii_case("usd1") {
                        match load_live_follow_buy_usd1_route_setup(&state.rpc_url).await {
                            Ok(setup) => {
                                cache_bonk_usd1_route_setup(state, &compile_context_key, setup).await;
                            }
                            Err(error) => {
                                record_warn(
                                    "follow-daemon",
                                    "Failed to prime shared Bonk USD1 route setup for compile batch.",
                                    Some(json!({
                                        "traceId": job.traceId,
                                        "compileContextKey": compile_context_key,
                                        "error": error,
                                    })),
                                );
                            }
                        }
                    }
                }
                Err(error) => {
                    record_warn(
                        "follow-daemon",
                        "Failed to prime shared Bonk follow-buy context for compile batch.",
                        Some(json!({
                            "traceId": job.traceId,
                            "compileContextKey": compile_context_key,
                            "error": error,
                        })),
                    );
                }
            }
        }
        "bagsapp" => {
            let Some(mint) = job.mint.as_deref() else {
                return;
            };
            match load_follow_buy_context(
                &state.rpc_url,
                mint,
                &job.execution.commitment,
                job.bagsLaunch.as_ref(),
            )
            .await
            {
                Ok(Some(context)) => {
                    cache_bags_follow_buy_context(state, &compile_context_key, context).await;
                }
                Ok(None) => {}
                Err(error) => {
                    record_warn(
                        "follow-daemon",
                        "Failed to prime shared Bags follow-buy context for compile batch.",
                        Some(json!({
                            "traceId": job.traceId,
                            "compileContextKey": compile_context_key,
                            "error": error,
                        })),
                    );
                }
            }
            match fetch_market_snapshot_for_launchpad(
                "bagsapp",
                &state.rpc_url,
                mint,
                &job.quoteAsset,
                job.bagsLaunch.as_ref(),
            )
            .await
            {
                Ok(Some(LaunchpadMarketSnapshot::Bags(snapshot))) => {
                    cache_bags_market_snapshot(state, &job.traceId, snapshot).await;
                }
                Ok(_) => {}
                Err(error) => {
                    record_warn(
                        "follow-daemon",
                        "Failed to prime shared Bags market snapshot for compile batch.",
                        Some(json!({
                            "traceId": job.traceId,
                            "compileContextKey": compile_context_key,
                            "error": error,
                        })),
                    );
                }
            }
        }
        _ => {}
    }
    let prep_ms = prep_started.elapsed().as_millis();
    update_job_timings(state, &job.traceId, |timings| {
        add_time(&mut timings.triggerCompilePrepMs, prep_ms);
    })
    .await;
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
        eligibleAtMs: None,
        submitStartedAtMs: None,
        submittedAtMs: None,
        confirmedAtMs: None,
        provider: None,
        endpointProfile: None,
        transportType: None,
        watcherMode: None,
        watcherFallbackReason: None,
        sendObservedSlot: None,
        confirmedObservedSlot: None,
        confirmedTokenBalanceRaw: None,
        eligibilityObservedSlot: None,
        slotsToConfirm: None,
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

fn job_uses_market_cap_trigger(job: &FollowJobRecord) -> bool {
    job.actions.iter().any(|action| action.marketCap.is_some())
}

fn job_needs_sol_usd_reference(job: &FollowJobRecord) -> bool {
    if !job_uses_market_cap_trigger(job) {
        return false;
    }
    if job.launchpad == "pump" {
        return true;
    }
    if job.launchpad == "bonk" {
        return true;
    }
    stable_quote_asset_decimals(&job.quoteAsset).is_none()
}

async fn warm_market_cap_reference_data(state: &Arc<AppState>, job: &FollowJobRecord) {
    if !job_needs_sol_usd_reference(job) {
        return;
    }
    if let Err(error) = fetch_sol_usd_micro_price(&state.rpc_url).await {
        record_warn(
            "follow-daemon",
            "Failed to warm SOL/USD price for market-cap follow job.",
            Some(json!({
                "traceId": job.traceId,
                "jobId": job.jobId,
                "launchpad": job.launchpad,
                "quoteAsset": job.quoteAsset,
                "error": error,
            })),
        );
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

fn sniper_autosell_requested(follow: &launchdeck_engine::config::NormalizedFollowLaunch) -> bool {
    follow
        .snipes
        .iter()
        .any(|snipe| snipe.postBuySell.as_ref().is_some_and(|sell| sell.enabled))
}

fn sniper_autosell_rollout_enabled() -> bool {
    opt_out_flag_enabled(std::env::var("LAUNCHDECK_ENABLE_SNIPER_AUTOSELL").ok().as_deref())
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
    let _ = provider;
    let _ = endpoint_profile;
    if prefers_helius_transaction_subscribe_path(
        configured_enable_helius_transaction_subscribe(),
        watch_endpoint,
    ) {
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

fn inferred_market_watcher_mode_for_job(
    job: &FollowJobRecord,
    watch_endpoint: Option<&str>,
) -> String {
    if market_watcher_uses_slot_subscription(job) {
        if prefers_helius_transaction_subscribe_path(
            configured_enable_helius_transaction_subscribe(),
            watch_endpoint,
        ) {
            "helius-slot-subscribe".to_string()
        } else {
            "standard-ws-slot".to_string()
        }
    } else {
        selected_market_watcher_mode(
            &job.execution.provider,
            &job.execution.endpointProfile,
            watch_endpoint,
        )
    }
}

async fn remember_watch_endpoint(state: &Arc<AppState>, endpoint: &str) {
    let key = watch_endpoint_cache_key(endpoint);
    let mut cache = state.watch_endpoint_health.lock().await;
    cache
        .entry(key)
        .or_insert_with(|| CachedWatchEndpointHealth {
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
    let endpoint_profile = endpoint_profile.map(str::to_string).or_else(|| {
        previous
            .as_ref()
            .and_then(|entry| entry.endpoint_profile.clone())
    });
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
            store_watch_endpoint_health(state, endpoint, true, None, provider, endpoint_profile)
                .await;
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
    timeout(
        Duration::from_secs(5),
        prewarm_watch_websocket_endpoint(endpoint),
    )
    .await
    .map_err(|_| format!("Timed out warming websocket endpoint: {endpoint}"))?
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
    if payload.followLaunch.enabled
        && sniper_autosell_requested(&payload.followLaunch)
        && !sniper_autosell_rollout_enabled()
    {
        let health = build_health(&state).await;
        return Ok(Json(follow_ready_response(
            health,
            watch_endpoint,
            requires_websocket,
            false,
            Some(
                "Sniper autosell is disabled by rollout gate (LAUNCHDECK_ENABLE_SNIPER_AUTOSELL)."
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
    if payload.followLaunch.enabled
        && sniper_autosell_requested(&payload.followLaunch)
        && !sniper_autosell_rollout_enabled()
    {
        return Err(internal_error(
            "Sniper autosell is disabled by rollout gate (LAUNCHDECK_ENABLE_SNIPER_AUTOSELL)."
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
    warm_market_cap_reference_data(&state, &job).await;
    record_info(
        "follow-daemon",
        "Reserved follow job.",
        Some(json!({
            "traceId": job.traceId,
            "jobId": job.jobId,
            "launchpad": job.launchpad,
            "quoteAsset": job.quoteAsset,
        })),
    );
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
    record_info(
        "follow-daemon",
        "Armed follow job.",
        Some(json!({
            "traceId": job.traceId,
            "jobId": job.jobId,
            "launchpad": job.launchpad,
        })),
    );
    update_job_timings(&state, &trace_id, |timings| {
        timings.armMs = Some(arm_started.elapsed().as_millis());
    })
    .await;
    warm_market_cap_reference_data(&state, &job).await;
    let arm_task_state = state.clone();
    let arm_task_trace_id = trace_id.clone();
    let arm_task_job = job.clone();
    tokio::spawn(async move {
        let cache_started = Instant::now();
        prepare_follow_job_buy_caches(arm_task_state.clone(), &arm_task_job).await;
        update_job_timings(&arm_task_state, &arm_task_trace_id, |timings| {
            add_time(
                &mut timings.cachePrepMs,
                cache_started.elapsed().as_millis(),
            );
        })
        .await;
        sync_follow_job_report_immediate(&arm_task_state, &arm_task_trace_id).await;
    });
    spawn_job_if_needed(state.clone(), trace_id.clone()).await;
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
    record_info(
        "follow-daemon",
        "Cancelled follow job.",
        Some(json!({
            "traceId": job.traceId,
            "jobId": job.jobId,
        })),
    );
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
    record_error(
        "follow-daemon",
        "Follow daemon request failed.",
        Some(json!({
            "message": error,
        })),
    );
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "ok": false,
            "error": error,
        })),
    )
}

async fn get_job(state: &Arc<AppState>, trace_id: &str) -> Option<FollowJobRecord> {
    state.store.get_job(trace_id).await
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
    let action_post_eligibility_to_submit = sum_follow_times(
        job.actions
            .iter()
            .map(|action| action.timings.postEligibilityToSubmitMs),
    );
    let action_presigned_expiry_check = sum_follow_times(
        job.actions
            .iter()
            .map(|action| action.timings.preSignedExpiryCheckMs),
    );
    let action_compile =
        sum_follow_times(job.actions.iter().map(|action| action.timings.compileMs));
    let action_submit = sum_follow_times(job.actions.iter().map(|action| action.timings.submitMs));
    let action_confirm =
        sum_follow_times(job.actions.iter().map(|action| action.timings.confirmMs));
    let action_execution = sum_follow_times(
        job.actions
            .iter()
            .map(|action| action.timings.executionTotalMs),
    );
    job.timings.benchmarkMode = Some(configured_benchmark_mode().as_str().to_string());
    job.timings.watcherWaitMs = action_watcher_wait;
    job.timings.eligibilityMs = action_eligibility;
    job.timings.postEligibilityToSubmitMs = action_post_eligibility_to_submit;
    job.timings.preSignedExpiryCheckMs = action_presigned_expiry_check;
    job.timings.compileMs = action_compile;
    job.timings.submitMs = action_submit;
    job.timings.confirmMs = action_confirm;
    job.timings.executionTotalMs = sum_follow_times([
        job.timings.reserveMs,
        job.timings.armMs,
        job.timings.cachePrepMs,
        action_execution,
    ]);
    job.timings.reportingOverheadMs =
        sum_follow_times([job.timings.reportSyncMs, job.timings.followSnapshotFlushMs]);
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
        add_time(
            &mut timings.reportSyncMs,
            sync_started.elapsed().as_millis(),
        );
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
    match state.store.clear_jobs_for_restart().await {
        Ok(removed) if removed > 0 => {
            record_info(
                "follow-daemon",
                format!(
                    "Cleared {} persisted follow job{} on startup",
                    removed,
                    if removed == 1 { "" } else { "s" }
                ),
                Some(json!({
                    "removedJobs": removed,
                    "reason": "startup-reset",
                })),
            );
        }
        Ok(_) => {}
        Err(error) => {
            record_error(
                "follow-daemon",
                "Failed to clear persisted follow jobs on startup".to_string(),
                Some(json!({
                    "message": error,
                })),
            );
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
        grouped_by_trigger
            .entry(key)
            .or_default()
            .push(action.clone());
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
        action_tasks
            .spawn(async move { run_deferred_setup_task(setup_state, setup_trace_id).await });
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
    let final_state = if final_job.cancelRequested
        || matches!(final_job.state, FollowJobState::Cancelled)
    {
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
        .finalize_job_state(&trace_id, final_state.clone(), None)
        .await;
    match final_state {
        FollowJobState::Completed => record_info(
            "follow-daemon",
            "Follow job completed.",
            Some(json!({ "traceId": trace_id })),
        ),
        FollowJobState::CompletedWithFailures => record_warn(
            "follow-daemon",
            "Follow job completed with failures.",
            Some(json!({ "traceId": trace_id })),
        ),
        FollowJobState::Failed => record_error(
            "follow-daemon",
            "Follow job failed.",
            Some(json!({
                "traceId": trace_id,
                "lastError": final_job.lastError,
            })),
        ),
        FollowJobState::Cancelled => record_info(
            "follow-daemon",
            "Follow job cancelled.",
            Some(json!({ "traceId": trace_id })),
        ),
        _ => {}
    }
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

fn matching_sniper_buy_action<'a>(
    job: &'a FollowJobRecord,
    action: &FollowActionRecord,
) -> Option<&'a FollowActionRecord> {
    if !matches!(action.kind, FollowActionKind::SniperSell) {
        return None;
    }
    job.actions.iter().find(|candidate| {
        matches!(candidate.kind, FollowActionKind::SniperBuy)
            && candidate.orderIndex == action.orderIndex
            && candidate.walletEnvKey == action.walletEnvKey
    })
}

fn has_matching_sniper_sell(job: &FollowJobRecord, action: &FollowActionRecord) -> bool {
    matches!(action.kind, FollowActionKind::SniperBuy)
        && job.actions.iter().any(|candidate| {
            matches!(candidate.kind, FollowActionKind::SniperSell)
                && candidate.orderIndex == action.orderIndex
                && candidate.walletEnvKey == action.walletEnvKey
        })
}

fn should_request_full_transaction_details(action: &FollowActionRecord) -> bool {
    matches!(
        action.kind,
        FollowActionKind::SniperBuy | FollowActionKind::DevAutoSell | FollowActionKind::SniperSell
    )
}

fn matching_sniper_buy_token_amount_override(
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Option<u64> {
    matching_sniper_buy_action(job, action)
        .and_then(|buy| buy.confirmedTokenBalanceRaw.as_deref())
        .and_then(|value| value.parse::<u64>().ok())
}

fn matching_confirmed_post_token_balance_raw(
    token_balances: &[launchdeck_engine::rpc::TransactionTokenBalance],
    owner: &str,
    mint: &str,
) -> Option<String> {
    token_balances
        .iter()
        .find(|balance| {
            balance.mint == mint
                && balance
                    .owner
                    .as_deref()
                    .is_some_and(|candidate| candidate == owner)
        })
        .map(|balance| balance.amount.clone())
}

fn terminal_action_state_label(state: &FollowActionState) -> &'static str {
    match state {
        FollowActionState::Queued => "queued",
        FollowActionState::Armed => "armed",
        FollowActionState::Eligible => "eligible",
        FollowActionState::Running => "running",
        FollowActionState::Sent => "sent",
        FollowActionState::Confirmed => "confirmed",
        FollowActionState::Stopped => "stopped",
        FollowActionState::Failed => "failed",
        FollowActionState::Cancelled => "cancelled",
        FollowActionState::Expired => "expired",
    }
}

fn sniper_sell_blocked_notice(sell_action_id: &str, buy_action_id: &str, state: &FollowActionState) -> String {
    format!(
        "__stopped__ sniper autosell {sell_action_id} stopped because matching sniper buy {buy_action_id} ended as {}.",
        terminal_action_state_label(state)
    )
}

async fn wait_for_matching_sniper_buy_confirmation(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Result<u64, String> {
    let Some(buy_action_id) = matching_sniper_buy_action(job, action).map(|buy| buy.actionId.clone()) else {
        return Err(format!(
            "Matching sniper buy was missing for autosell {}.",
            action.actionId
        ));
    };
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, &action.actionId).await?;
        let Some(current_job) = get_job(&state, &job.traceId).await else {
            return Err("Follow job disappeared while waiting for matching sniper buy.".to_string());
        };
        let Some(current_buy) = current_job
            .actions
            .iter()
            .find(|candidate| candidate.actionId == buy_action_id)
        else {
            return Err(format!(
                "Matching sniper buy {buy_action_id} was missing for autosell {}.",
                action.actionId
            ));
        };
        match current_buy.state {
            FollowActionState::Confirmed => {
                if let Some(confirmed_slot) = current_buy.confirmedObservedSlot {
                    return Ok(confirmed_slot);
                }
                return Err(format!(
                    "Matching sniper buy {buy_action_id} confirmed without a confirmed slot for autosell {}.",
                    action.actionId
                ));
            }
            FollowActionState::Stopped
            | FollowActionState::Failed
            | FollowActionState::Cancelled
            | FollowActionState::Expired => {
                return Err(sniper_sell_blocked_notice(
                    &action.actionId,
                    &buy_action_id,
                    &current_buy.state,
                ));
            }
            _ => sleep(Duration::from_millis(50)).await,
        }
    }
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
                | FollowActionState::Stopped
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
            if is_stopped_action_error(&error) {
                let stop_reason = normalized_stopped_action_reason(Some(&error));
                record_action_stopped(&state, &current_job, &action, stop_reason.as_deref()).await;
                return Ok(());
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
    let eligibility_timing = match wait_for_action_eligibility(state.clone(), &job, &lead_action).await {
        Ok(timing) => timing,
        Err(error) => {
            for action in &actions {
                record_action_failure(&state, &job, action, &error).await;
            }
            return Err(error);
        }
    };
    let eligible_at_ms = now_ms();
    for action in &actions {
        let transport_plan = follow_action_transport_plan(&job, action);
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
                record.eligibilityObservedSlot = eligibility_timing.observed_slot;
                record.eligibleAtMs = Some(eligible_at_ms);
            })
            .await;
        update_action_timings(&state, &trace_id, &action.actionId, |timings| {
            timings.watcherWaitMs = Some(eligibility_timing.watcher_wait_ms);
            timings.eligibilityMs = Some(eligibility_timing.total_ms);
        })
        .await;
    }
    prepare_trigger_time_buy_compile_batch(
        &state,
        &job,
        &actions,
        eligibility_timing.observed_slot,
    )
    .await;
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
        if matches!(
            setup.state,
            DeferredSetupState::Confirmed | DeferredSetupState::Failed
        ) {
            return Ok(());
        }
        wait_for_job_confirmation(state.clone(), &job).await?;
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

async fn record_action_stopped(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
    reason: Option<&str>,
) {
    let normalized_reason = normalized_stopped_action_reason(reason);
    let _ = state
        .store
        .update_action(&job.traceId, &action.actionId, |record| {
            record.state = FollowActionState::Stopped;
            record.lastError = normalized_reason.clone();
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
            | FollowActionState::Stopped
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
    if !configured_enable_pump_sell_creator_vault_auto_retry() {
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
    if !configured_enable_pump_buy_creator_vault_auto_retry() {
        return false;
    }
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

fn market_cap_scan_stopped_notice(action_id: &str, scan_timeout_seconds: u64) -> String {
    format!(
        "__stopped__ market-cap scan stopped for {action_id} after {scan_timeout_seconds} second(s)."
    )
}

fn is_stopped_action_error(error: &str) -> bool {
    error.trim().starts_with("__stopped__")
}

fn slot_anchor_unavailable_error(anchor_label: &str) -> String {
    format!("{anchor_label} was unavailable for On Confirmed Slot trigger.")
}

fn normalized_stopped_action_reason(reason: Option<&str>) -> Option<String> {
    let reason = reason?;
    let trimmed = reason.trim();
    let cleaned = trimmed
        .strip_prefix("__stopped__")
        .unwrap_or(trimmed)
        .trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
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

fn action_runs_before_deferred_setup(action: &FollowActionRecord) -> bool {
    if action.marketCap.is_some() {
        return false;
    }
    match action.targetBlockOffset {
        Some(0) => return true,
        Some(_) => return false,
        None => {}
    }
    if action.requireConfirmation {
        return false;
    }
    if action.delayMs.is_some() {
        return false;
    }
    match action.submitDelayMs {
        Some(delay_ms) => delay_ms == 0,
        None => true,
    }
}

fn reset_action_for_presigned_rebuild(record: &mut FollowActionRecord, message: String) {
    record.state = FollowActionState::Armed;
    record.preSignedTransactions.clear();
    record.eligibleAtMs = None;
    record.submitStartedAtMs = None;
    record.submittedAtMs = None;
    record.confirmedAtMs = None;
    record.sendObservedSlot = None;
    record.confirmedObservedSlot = None;
    record.slotsToConfirm = None;
    record.signature = None;
    record.explorerUrl = None;
    record.endpoint = None;
    record.bundleId = None;
    record.lastError = Some(message);
}

fn should_block_deferred_setup_for_action(action: &FollowActionRecord, now_ms: u128) -> bool {
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
    if !action_runs_before_deferred_setup(action) {
        return false;
    }
    action
        .scheduledForMs
        .is_none_or(|scheduled_for_ms| scheduled_for_ms <= now_ms.saturating_add(1_500))
}

async fn execute_action(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> Result<(), String> {
    let trace_id = job.traceId.clone();
    let mut effective_action = action.clone();
    let transport_plan = follow_action_transport_plan(job, &effective_action);
    if !matches!(effective_action.state, FollowActionState::Eligible) {
        ensure_action_not_cancelled(&state, &trace_id, &effective_action.actionId).await?;
        state
            .store
            .update_action(&trace_id, &effective_action.actionId, |record| {
                if matches!(record.state, FollowActionState::Queued) {
                    record.state = FollowActionState::Armed;
                }
            })
            .await?;
        sync_follow_job_report(&state, &trace_id).await;
        let eligibility_timing =
            wait_for_action_eligibility(state.clone(), job, &effective_action).await?;
        let eligible_at_ms = now_ms();
        update_action_timings(&state, &trace_id, &effective_action.actionId, |timings| {
            timings.watcherWaitMs = Some(eligibility_timing.watcher_wait_ms);
            timings.eligibilityMs = Some(eligibility_timing.total_ms);
        })
        .await;
        ensure_action_not_cancelled(&state, &trace_id, &effective_action.actionId).await?;
        state
            .store
            .update_action(&trace_id, &effective_action.actionId, |record| {
                record.state = FollowActionState::Eligible;
                record.provider = Some(transport_plan.resolvedProvider.clone());
                record.endpointProfile = Some(transport_plan.resolvedEndpointProfile.clone());
                record.transportType = Some(transport_plan.transportType.clone());
                record.eligibilityObservedSlot = eligibility_timing.observed_slot;
                record.eligibleAtMs = Some(eligible_at_ms);
            })
            .await?;
        effective_action.state = FollowActionState::Eligible;
        effective_action.provider = Some(transport_plan.resolvedProvider.clone());
        effective_action.endpointProfile = Some(transport_plan.resolvedEndpointProfile.clone());
        effective_action.transportType = Some(transport_plan.transportType.clone());
        effective_action.eligibilityObservedSlot = eligibility_timing.observed_slot;
        effective_action.eligibleAtMs = Some(eligible_at_ms);
        prepare_trigger_time_buy_compile_batch(
            &state,
            job,
            std::slice::from_ref(&effective_action),
            eligibility_timing.observed_slot,
        )
        .await;
        sync_follow_job_report(&state, &trace_id).await;
    }
    let wallet_key = selected_wallet_key_or_default(&effective_action.walletEnvKey)
        .ok_or_else(|| format!("Wallet env key not found: {}", effective_action.walletEnvKey))?;
    let wallet_secret = load_solana_wallet_by_env_key(&wallet_key)?;
    let wallet_owner_pubkey = public_key_from_secret(&wallet_secret)?;
    let sell_token_amount_override = if matches!(effective_action.kind, FollowActionKind::SniperSell)
    {
        get_job(&state, &trace_id)
            .await
            .as_ref()
            .and_then(|current_job| {
                matching_sniper_buy_token_amount_override(current_job, &effective_action)
            })
            .or_else(|| matching_sniper_buy_token_amount_override(job, &effective_action))
    } else {
        None
    };
    if effective_action.precheckRequired {
        required_action_precheck(&state.rpc_url, &job.quoteAsset, &effective_action).await?;
    }
    ensure_action_not_cancelled(&state, &trace_id, &effective_action.actionId).await?;
    let wallet_lock = wallet_lock(&state, &wallet_key).await;
    let _wallet_guard = wallet_lock.lock().await;
    let submit_started_at_ms = now_ms();
    if let Some(eligible_at_ms) = effective_action.eligibleAtMs {
        update_action_timings(&state, &trace_id, &effective_action.actionId, |timings| {
            timings.postEligibilityToSubmitMs =
                Some(submit_started_at_ms.saturating_sub(eligible_at_ms));
        })
        .await;
    }
    state
        .store
        .update_action(&trace_id, &effective_action.actionId, |record| {
            record.state = FollowActionState::Running;
            record.attemptCount = record.attemptCount.saturating_add(1);
            record.submitStartedAtMs = Some(submit_started_at_ms);
        })
        .await?;
    sync_follow_job_report(&state, &trace_id).await;
    let mint = job
        .mint
        .as_deref()
        .ok_or_else(|| "Follow job missing mint.".to_string())?;
    if skip_retry_if_wallet_already_holds_token(&state, job, &effective_action, &wallet_key, mint)
        .await?
    {
        return Ok(());
    }
    let launch_creator = job
        .launchCreator
        .as_deref()
        .ok_or_else(|| "Follow job missing launch creator.".to_string())?;
    let compile_started = Instant::now();
    let prefer_post_setup_creator_vault_for_buy = should_use_post_setup_creator_vault_for_buy(
        job.preferPostSetupCreatorVaultForSell,
        &effective_action,
        &job.execution.buyMevMode,
    );
    let prefer_post_setup_creator_vault_for_sell = should_use_post_setup_creator_vault_for_sell(
        job.preferPostSetupCreatorVaultForSell,
        &effective_action,
        &job.execution.sellMevMode,
    );
    let sell_percent = match effective_action.kind {
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => Some(
            effective_action
                .sellPercent
                .ok_or_else(|| "Follow sell missing percent.".to_string())?,
        ),
        FollowActionKind::SniperBuy => None,
    };
    let compiled_from_presign = !effective_action.preSignedTransactions.is_empty();
    let compiled = if let Some(tx) = effective_action.preSignedTransactions.first().cloned() {
        let presigned_expiry_started = Instant::now();
        let current_block_height = current_shared_block_height(state.clone(), &trace_id).await?;
        if current_block_height > tx.lastValidBlockHeight {
            let fresh_block_height =
                fetch_current_block_height_fresh(&state.rpc_url, "confirmed").await?;
            if fresh_block_height > tx.lastValidBlockHeight {
                return Err(format!(
                    "__expired__ pre-signed payload for {} expired at block height {} before send.",
                    effective_action.actionId, tx.lastValidBlockHeight
                ));
            }
        }
        update_action_timings(&state, &trace_id, &effective_action.actionId, |timings| {
            timings.preSignedExpiryCheckMs = Some(presigned_expiry_started.elapsed().as_millis());
        })
        .await;
        Some(tx)
    } else {
        let _compile_permit = acquire_capacity_slot(
            state.compile_slots.clone(),
            state.capacity_wait_ms,
            "compile",
        )
        .await?;
        match effective_action.kind {
            FollowActionKind::SniperBuy => {
                let amount = effective_action
                    .buyAmountSol
                    .as_deref()
                    .ok_or_else(|| "Follow buy missing amount.".to_string())?;
                if job.launchpad == "bonk" {
                    let shared_bonk_pool_context =
                        resolve_shared_bonk_follow_buy_context_for_job(&state, job, &effective_action).await;
                    let shared_bonk_usd1_route_setup =
                        resolve_shared_bonk_usd1_route_setup_for_job(&state, job, &effective_action).await;
                    Some(
                        compile_follow_buy_for_launchpad(FollowBuyCompileRequest {
                            launchpad: &job.launchpad,
                            launch_mode: &job.launchMode,
                            quote_asset: &job.quoteAsset,
                            rpc_url: &state.rpc_url,
                            execution: &job.execution,
                            token_mayhem_mode: job.tokenMayhemMode,
                            jito_tip_account: &job.buyTipAccount,
                            wallet_secret: &wallet_secret,
                            mint,
                            launch_creator,
                            buy_amount_sol: amount,
                            allow_ata_creation: true,
                            prefer_post_setup_creator_vault: prefer_post_setup_creator_vault_for_buy,
                            bonk_pool_context: shared_bonk_pool_context.as_ref(),
                            bonk_usd1_route_setup: shared_bonk_usd1_route_setup.as_ref(),
                            bags_follow_buy_context: None,
                            bags_launch: job.bagsLaunch.as_ref(),
                        })
                        .await?,
                    )
                } else if job.launchpad == "bagsapp" {
                    let shared_bags_follow_buy_context =
                        resolve_shared_bags_follow_buy_context_for_job(&state, job, &effective_action).await;
                    Some(
                        compile_follow_buy_for_launchpad(FollowBuyCompileRequest {
                            launchpad: &job.launchpad,
                            launch_mode: &job.launchMode,
                            quote_asset: &job.quoteAsset,
                            rpc_url: &state.rpc_url,
                            execution: &job.execution,
                            token_mayhem_mode: job.tokenMayhemMode,
                            jito_tip_account: &job.buyTipAccount,
                            wallet_secret: &wallet_secret,
                            mint,
                            launch_creator,
                            buy_amount_sol: amount,
                            allow_ata_creation: true,
                            prefer_post_setup_creator_vault: prefer_post_setup_creator_vault_for_buy,
                            bonk_pool_context: None,
                            bonk_usd1_route_setup: None,
                            bags_follow_buy_context: shared_bags_follow_buy_context.as_ref(),
                            bags_launch: job.bagsLaunch.as_ref(),
                        })
                        .await?,
                    )
                } else {
                    let prepared = resolve_prepared_follow_buy(
                        &state,
                        job,
                        &effective_action,
                        &wallet_secret,
                        launch_creator,
                        amount,
                    )
                    .await?;
                    let runtime = resolve_hot_follow_buy_runtime_for_job(
                        &state,
                        job,
                        &effective_action,
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
                if matches!(job.launchpad.as_str(), "pump" | "bonk" | "bagsapp") {
                    compile_follow_sell_for_launchpad(FollowSellCompileRequest {
                        launchpad: &job.launchpad,
                        quote_asset: &job.quoteAsset,
                        rpc_url: &state.rpc_url,
                        execution: &job.execution,
                        token_mayhem_mode: job.tokenMayhemMode,
                        jito_tip_account: &job.sellTipAccount,
                        wallet_secret: &wallet_secret,
                        mint,
                        launch_creator,
                        sell_percent: sell_percent.unwrap_or_default(),
                        prefer_post_setup_creator_vault: prefer_post_setup_creator_vault_for_sell,
                        token_amount_override: sell_token_amount_override,
                        bonk_pool_id: effective_action.poolId.as_deref(),
                        bonk_launch_mode: None,
                        bonk_launch_creator: None,
                        pump_cashback_enabled_override: None,
                        bags_launch: job.bagsLaunch.as_ref(),
                    })
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
    update_action_timings(&state, &trace_id, &effective_action.actionId, |timings| {
        timings.compileMs = Some(if compiled_from_presign { 0 } else { compile_ms });
    })
    .await;
    ensure_action_not_cancelled(&state, &trace_id, &effective_action.actionId).await?;
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
                    && should_retry_pump_sell_creator_vault_mismatch(job, &effective_action, &error) =>
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
    let mut sent = submitted
        .pop()
        .ok_or_else(|| "Follow daemon submit returned no transactions.".to_string())?;
    sent.capturePostTokenBalances = has_matching_sniper_sell(job, &effective_action);
    sent.requestFullTransactionDetails = should_request_full_transaction_details(&effective_action);
    sent.balanceWatchAccount = if sent.capturePostTokenBalances
        && matches!(effective_action.kind, FollowActionKind::SniperBuy)
    {
        Some(derive_follow_owner_token_account_for_launchpad(
            &job.launchpad,
            &wallet_owner_pubkey,
            mint,
        )?)
    } else {
        None
    };
    update_action_timings(&state, &trace_id, &effective_action.actionId, |timings| {
        timings.submitMs = Some(submit_latency.max(submit_ms));
    })
    .await;
    state
        .store
        .update_action(&trace_id, &effective_action.actionId, |record| {
            record.state = FollowActionState::Sent;
            record.submittedAtMs = Some(now_ms());
            record.sendObservedSlot = sent.sendObservedSlot;
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
    update_action_timings(&state, &trace_id, &effective_action.actionId, |timings| {
        timings.confirmMs = Some(confirm_ms);
        timings.executionTotalMs = Some(
            timings
                .eligibilityMs
                .unwrap_or_default()
                .saturating_add(timings.postEligibilityToSubmitMs.unwrap_or_default())
                .saturating_add(timings.preSignedExpiryCheckMs.unwrap_or_default())
                .saturating_add(timings.compileMs.unwrap_or_default())
                .saturating_add(timings.submitMs.unwrap_or_default())
                .saturating_add(confirm_ms),
        );
    })
    .await;
    let confirmed = confirm_input.pop().unwrap_or(sent);
    let confirmed_action_slot = confirmed.confirmedSlot.or(confirmed.confirmedObservedSlot);
    let confirmed_post_token_balance_raw = if matches!(effective_action.kind, FollowActionKind::SniperBuy) {
        confirmed.confirmedTokenBalanceRaw.clone().or_else(|| {
            matching_confirmed_post_token_balance_raw(
                &confirmed.postTokenBalances,
                &wallet_owner_pubkey,
                mint,
            )
        })
    } else {
        None
    };
    state
        .store
        .update_action(&trace_id, &effective_action.actionId, |record| {
            record.state = FollowActionState::Confirmed;
            record.confirmedAtMs = Some(now_ms());
            record.confirmedObservedSlot = confirmed_action_slot;
            record.confirmedTokenBalanceRaw = confirmed_post_token_balance_raw.clone();
            record.slotsToConfirm = match (record.sendObservedSlot, confirmed_action_slot) {
                (Some(send_slot), Some(confirm_slot)) if confirm_slot >= send_slot => {
                    Some(confirm_slot - send_slot)
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
    let mut setup_transactions = setup.transactions.clone();
    loop {
        let _ = state
            .store
            .update_job(&trace_id, |record| {
                if let Some(setup) = record.deferredSetup.as_mut() {
                    setup.state = DeferredSetupState::Running;
                    setup.attemptCount = setup.attemptCount.saturating_add(1);
                }
            })
            .await;
        sync_follow_job_report(&state, &trace_id).await;
        let _send_permit =
            acquire_capacity_slot(state.send_slots.clone(), state.capacity_wait_ms, "send").await?;
        match submit_transactions_for_transport(
            &state.rpc_url,
            &transport_plan,
            &setup_transactions,
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
                let _ = state
                    .store
                    .update_job(&trace_id, |record| {
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
                    })
                    .await;
                let confirmation_result = timeout(
                    Duration::from_secs(DEFERRED_SETUP_CONFIRMATION_TIMEOUT_SECS),
                    confirm_submitted_transactions_for_transport(
                        &state.rpc_url,
                        &transport_plan,
                        &mut submitted,
                        &job.execution.commitment,
                        job.execution.trackSendBlockHeight,
                    ),
                )
                .await;
                match confirmation_result {
                    Ok(Ok(_)) => {}
                    Ok(Err(error)) if attempt < 2 => {
                        attempt = attempt.saturating_add(1);
                        let retry_error = if is_terminal_onchain_confirmation_error(&error) {
                            let refreshed =
                                refresh_deferred_setup_transactions(&state, job, &setup_transactions)
                                    .await?;
                            setup_transactions = refreshed.clone();
                            format!(
                                "Deferred setup confirmation failed after send; rebuilt and requeued attempt {}: {}",
                                attempt + 1,
                                error
                            )
                        } else {
                            format!(
                                "Deferred setup confirmation failed after send; requeued attempt {}: {}",
                                attempt + 1,
                                error
                            )
                        };
                        let transactions_for_retry = setup_transactions.clone();
                        let _ = state
                            .store
                            .update_job(&trace_id, |record| {
                                if let Some(setup) = record.deferredSetup.as_mut() {
                                    setup.transactions = transactions_for_retry.clone();
                                    setup.state = DeferredSetupState::Queued;
                                    setup.signatures.clear();
                                    setup.submittedAtMs = None;
                                    setup.confirmedAtMs = None;
                                    setup.lastError = Some(retry_error.clone());
                                }
                            })
                            .await;
                        sync_follow_job_report(&state, &trace_id).await;
                        continue;
                    }
                    Ok(Err(error)) => {
                        let _ = state
                            .store
                            .update_job(&trace_id, |record| {
                                if let Some(setup) = record.deferredSetup.as_mut() {
                                    setup.state = DeferredSetupState::Failed;
                                    setup.lastError = Some(error.clone());
                                }
                            })
                            .await;
                        sync_follow_job_report(&state, &trace_id).await;
                        return Err(error);
                    }
                    Err(_) if attempt < 2 => {
                        attempt = attempt.saturating_add(1);
                        let retry_error = format!(
                            "Deferred setup was still unconfirmed {}s after send; requeued attempt {}.",
                            DEFERRED_SETUP_CONFIRMATION_TIMEOUT_SECS,
                            attempt + 1
                        );
                        let transactions_for_retry = setup_transactions.clone();
                        let _ = state
                            .store
                            .update_job(&trace_id, |record| {
                                if let Some(setup) = record.deferredSetup.as_mut() {
                                    setup.transactions = transactions_for_retry.clone();
                                    setup.state = DeferredSetupState::Queued;
                                    setup.signatures.clear();
                                    setup.submittedAtMs = None;
                                    setup.confirmedAtMs = None;
                                    setup.lastError = Some(retry_error.clone());
                                }
                            })
                            .await;
                        sync_follow_job_report(&state, &trace_id).await;
                        continue;
                    }
                    Err(_) => {
                        let error = format!(
                            "Deferred setup did not confirm within {}s after send.",
                            DEFERRED_SETUP_CONFIRMATION_TIMEOUT_SECS
                        );
                        let _ = state
                            .store
                            .update_job(&trace_id, |record| {
                                if let Some(setup) = record.deferredSetup.as_mut() {
                                    setup.state = DeferredSetupState::Failed;
                                    setup.lastError = Some(error.clone());
                                }
                            })
                            .await;
                        sync_follow_job_report(&state, &trace_id).await;
                        return Err(error);
                    }
                }
                let _ = state
                    .store
                    .update_job(&trace_id, |record| {
                        if let Some(setup) = record.deferredSetup.as_mut() {
                            setup.state = DeferredSetupState::Confirmed;
                            setup.confirmedAtMs = Some(now_ms());
                        }
                    })
                    .await;
                sync_follow_job_report(&state, &trace_id).await;
                return Ok(());
            }
            Err(error) if attempt < 2 && is_retryable_action_error(&error) => {
                attempt = attempt.saturating_add(1);
                let _ = state
                    .store
                    .update_job(&trace_id, |record| {
                        if let Some(setup) = record.deferredSetup.as_mut() {
                            setup.state = DeferredSetupState::Queued;
                            setup.lastError = Some(error.clone());
                        }
                    })
                    .await;
                sleep(Duration::from_millis(action_retry_backoff_ms(attempt))).await;
            }
            Err(error) => {
                let _ = state
                    .store
                    .update_job(&trace_id, |record| {
                        if let Some(setup) = record.deferredSetup.as_mut() {
                            setup.state = DeferredSetupState::Failed;
                            setup.lastError = Some(error.clone());
                        }
                    })
                    .await;
                sync_follow_job_report(&state, &trace_id).await;
                return Err(error);
            }
        }
    }
}

fn follow_action_execution(
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> launchdeck_engine::config::NormalizedExecution {
    let mut execution = job.execution.clone();
    match action.kind {
        FollowActionKind::SniperBuy => {
            execution.provider = job.execution.buyProvider.clone();
            execution.endpointProfile = job.execution.buyEndpointProfile.clone();
            execution.mevProtect = job.execution.buyMevProtect;
            execution.mevMode = job.execution.buyMevMode.clone();
            execution.jitodontfront = job.execution.buyJitodontfront;
        }
        FollowActionKind::DevAutoSell | FollowActionKind::SniperSell => {
            execution.provider = job.execution.sellProvider.clone();
            execution.endpointProfile = job.execution.sellEndpointProfile.clone();
            execution.mevProtect = job.execution.sellMevProtect;
            execution.mevMode = job.execution.sellMevMode.clone();
            execution.jitodontfront = job.execution.sellJitodontfront;
        }
    }
    execution
}

fn follow_action_transport_plan(
    job: &FollowJobRecord,
    action: &FollowActionRecord,
) -> TransportPlan {
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
    observed_slot: Option<u64>,
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
    let mut observed_slot = None;
    let confirmed_launch_slot = if matches!(action.kind, FollowActionKind::SniperSell) {
        None
    } else if action.requireConfirmation || action.targetBlockOffset.is_some() {
        let wait_started = Instant::now();
        let confirmed = wait_for_signature_confirmation(state.clone(), job, &action.actionId).await?;
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
                observed_slot = Some(wait_for_slot_offset(
                    state.clone(),
                    job,
                    &action.actionId,
                    confirmed_launch_slot,
                    u64::from(action.targetBlockOffset.unwrap()),
                )
                .await?);
                watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            }
        }
        FollowActionKind::DevAutoSell => {
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
                        confirmed_launch_slot,
                        u64::from(action.targetBlockOffset.unwrap()),
                    ) => {
                        let _ = result?;
                    },
                    result = wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId) => result?,
                }
                watcher_wait_ms =
                    watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            } else if has_slot {
                let wait_started = Instant::now();
                wait_for_slot_offset(
                    state.clone(),
                    job,
                    &action.actionId,
                    confirmed_launch_slot,
                    u64::from(action.targetBlockOffset.unwrap()),
                )
                .await?;
                watcher_wait_ms =
                    watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            } else if has_time && has_market {
                let wait_started = Instant::now();
                tokio::select! {
                    result = wait_until_ms(state.clone(), &job.traceId, Some(&action.actionId), action.scheduledForMs.unwrap()) => result?,
                    result = wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId) => result?,
                }
                watcher_wait_ms =
                    watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
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
                watcher_wait_ms =
                    watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            }
        }
        FollowActionKind::SniperSell => {
            let wait_started = Instant::now();
            let matching_buy_confirmed_slot =
                wait_for_matching_sniper_buy_confirmation(state.clone(), job, action).await?;
            watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            let has_time = action.scheduledForMs.is_some();
            let has_market = action.marketCap.is_some();
            if let Some(target_offset) = action.targetBlockOffset {
                let wait_started = Instant::now();
                observed_slot = Some(
                    wait_for_slot_offset_from_anchor(
                        state.clone(),
                        job,
                        &action.actionId,
                        Some(matching_buy_confirmed_slot),
                        u64::from(target_offset),
                        "Matching sniper buy confirmation slot",
                    )
                    .await?,
                );
                watcher_wait_ms =
                    watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            } else if has_time && has_market {
                let wait_started = Instant::now();
                tokio::select! {
                    result = wait_until_ms(state.clone(), &job.traceId, Some(&action.actionId), action.scheduledForMs.unwrap()) => result?,
                    result = wait_for_market_cap_trigger(state.clone(), job, action, &action.actionId) => result?,
                }
                watcher_wait_ms =
                    watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
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
                watcher_wait_ms =
                    watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
            }
        }
    }
    if job.deferredSetup.is_some() && !action_runs_before_deferred_setup(action) {
        let wait_started = Instant::now();
        wait_for_deferred_setup_confirmation(state.clone(), job, &action.actionId).await?;
        watcher_wait_ms = watcher_wait_ms.saturating_add(wait_started.elapsed().as_millis());
    }
    Ok(EligibilityTiming {
        watcher_wait_ms,
        total_ms: eligibility_started.elapsed().as_millis(),
        observed_slot,
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

async fn ensure_slot_watcher(
    state: Arc<AppState>,
    job: &FollowJobRecord,
) -> watch::Receiver<Option<Result<u64, String>>> {
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

#[allow(dead_code)]
fn offset_consumer_key(trace_id: &str, action_id: &str) -> String {
    format!("{trace_id}:{action_id}")
}

async fn current_shared_block_height(state: Arc<AppState>, _trace_id: &str) -> Result<u64, String> {
    fetch_current_block_height(&state.rpc_url, "confirmed").await
}

#[allow(dead_code)]
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

#[allow(dead_code)]
async fn unregister_offset_consumer(state: &Arc<AppState>, trace_id: &str, action_id: &str) {
    let key = offset_consumer_key(trace_id, action_id);
    let mut consumers = state.offset_worker.consumers.lock().await;
    consumers.remove(&key);
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
                (estimated_blocks_elapsed >= consumer.target_block_offset)
                    .then(|| (key.clone(), consumer.completion_tx.clone()))
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

#[allow(dead_code)]
async fn complete_offset_consumers_for_height(state: &Arc<AppState>, block_height: u64) {
    let completed = {
        let consumers = state.offset_worker.consumers.lock().await;
        consumers
            .iter()
            .filter_map(|(key, consumer)| {
                let target_height = consumer
                    .base_confirmed_block_height
                    .saturating_add(consumer.target_block_offset);
                (block_height >= target_height)
                    .then(|| (key.clone(), consumer.completion_tx.clone()))
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

#[allow(dead_code)]
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
                        let mut last_height =
                            state.offset_worker.last_observed_block_height.lock().await;
                        *last_height = Some(block_height);
                    }
                    {
                        let mut last_observed_at =
                            state.offset_worker.last_observed_at_ms.lock().await;
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
        WatcherKind::Market => Some(inferred_market_watcher_mode_for_job(
            job,
            inferred_endpoint.as_deref(),
        )),
    };
    let fallback_reason = health
        .lastError
        .clone()
        .filter(|value| !value.trim().is_empty());
    let mode = match kind {
        WatcherKind::Slot => health
            .slotWatcherMode
            .clone()
            .or_else(|| inferred_mode.clone()),
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
        let slot = match extract_slot_notification_slot(&message) {
            Some(slot) => slot,
            None => fetch_current_slot(&state.rpc_url, "confirmed").await?,
        };
        let _ = tx.send(Some(Ok(slot)));
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
    subscribe(&mut ws, "slotSubscribe", json!([])).await?;
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        let message = next_json_message(&mut ws).await?;
        if message.get("params").is_none() {
            continue;
        }
        let slot = match extract_slot_notification_slot(&message) {
            Some(slot) => slot,
            None => fetch_current_slot(&state.rpc_url, "confirmed").await?,
        };
        let _ = tx.send(Some(Ok(slot)));
        set_watcher_health(
            state,
            WatcherKind::Slot,
            FollowWatcherHealth::Healthy,
            Some("slot-subscribe".to_string()),
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
        let slot = fetch_current_slot(&state.rpc_url, "confirmed").await?;
        let _ = tx.send(Some(Ok(slot)));
        set_watcher_health(
            state,
            WatcherKind::Slot,
            FollowWatcherHealth::Healthy,
            Some("rpc-polling".to_string()),
            Some(state.rpc_url.clone()),
            note.clone(),
        )
        .await;
        sleep(Duration::from_millis(FOLLOW_WATCHER_RPC_POLL_INTERVAL_MS)).await;
    }
}

async fn recompute_market_cap_for_job(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    mint: &str,
) -> Result<u64, String> {
    if job.launchpad == "bonk" {
        let Some(LaunchpadMarketSnapshot::Bonk(snapshot)) = fetch_market_snapshot_for_launchpad(
            &job.launchpad,
            &state.rpc_url,
            mint,
            &job.quoteAsset,
            job.bagsLaunch.as_ref(),
        )
        .await?
        else {
            return Err("Bonk market snapshot was unavailable.".to_string());
        };
        let quote_units = snapshot
            .marketCapLamports
            .parse::<u64>()
            .map_err(|error| format!("Invalid Bonk market cap payload: {error}"))?;
        return quote_units_to_usd_micros(
            &state.rpc_url,
            quote_units,
            if snapshot.quoteAsset.trim().is_empty() {
                &job.quoteAsset
            } else {
                &snapshot.quoteAsset
            },
        )
        .await;
    }
    if job.launchpad == "bagsapp" {
        let snapshot = if let Some(cached) = get_bags_market_snapshot(state, &job.traceId).await {
            if now_ms().saturating_sub(cached.refreshed_at_ms) <= BAGS_MARKET_SNAPSHOT_CACHE_TTL_MS
            {
                cached.snapshot
            } else {
                let Some(LaunchpadMarketSnapshot::Bags(snapshot)) =
                    fetch_market_snapshot_for_launchpad(
                        &job.launchpad,
                        &state.rpc_url,
                        mint,
                        &job.quoteAsset,
                        job.bagsLaunch.as_ref(),
                    )
                    .await?
                else {
                    return Err("Bags market snapshot was unavailable.".to_string());
                };
                cache_bags_market_snapshot(state, &job.traceId, snapshot.clone()).await;
                snapshot
            }
        } else {
            let Some(LaunchpadMarketSnapshot::Bags(snapshot)) = fetch_market_snapshot_for_launchpad(
                &job.launchpad,
                &state.rpc_url,
                mint,
                &job.quoteAsset,
                job.bagsLaunch.as_ref(),
            )
            .await?
            else {
                return Err("Bags market snapshot was unavailable.".to_string());
            };
            cache_bags_market_snapshot(state, &job.traceId, snapshot.clone()).await;
            snapshot
        };
        let quote_units = snapshot
            .marketCapLamports
            .parse::<u64>()
            .map_err(|error| format!("Invalid Bags market cap payload: {error}"))?;
        return quote_units_to_usd_micros(
            &state.rpc_url,
            quote_units,
            if snapshot.quoteAsset.trim().is_empty() {
                &job.quoteAsset
            } else {
                &snapshot.quoteAsset
            },
        )
        .await;
    }
    let snapshot = fetch_pump_market_snapshot(&state.rpc_url, mint).await?;
    quote_units_to_usd_micros(
        &state.rpc_url,
        snapshot.marketCapLamports,
        if snapshot.quoteAsset.trim().is_empty() {
            "sol"
        } else {
            snapshot.quoteAsset.as_str()
        },
    )
    .await
}

fn market_watch_account(job: &FollowJobRecord, mint: &str) -> Result<String, String> {
    if job.launchpad == "pump" {
        return pump_bonding_curve_address(mint);
    }
    Ok(mint.to_string())
}

fn market_watcher_uses_slot_subscription(job: &FollowJobRecord) -> bool {
    matches!(job.launchpad.as_str(), "pump" | "bagsapp")
}

async fn run_slot_driven_market_watcher_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    endpoint: &str,
    mint: &str,
    watcher_mode: &str,
    note: Option<String>,
) -> Result<(), String> {
    let mut ws = open_subscription_socket(endpoint).await?;
    subscribe(&mut ws, "slotSubscribe", json!([])).await?;
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
            Some(watcher_mode.to_string()),
            Some(endpoint.to_string()),
            note.clone(),
        )
        .await;
    }
}

async fn run_standard_market_watcher_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    endpoint: &str,
    mint: &str,
    note: Option<String>,
) -> Result<(), String> {
    if market_watcher_uses_slot_subscription(job) {
        return run_slot_driven_market_watcher_session(
            state,
            job,
            tx,
            endpoint,
            mint,
            "standard-ws-slot",
            note,
        )
        .await;
    }
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
    if market_watcher_uses_slot_subscription(job) {
        return run_slot_driven_market_watcher_session(
            state,
            job,
            tx,
            endpoint,
            mint,
            "helius-slot-subscribe",
            None,
        )
        .await;
    }
    let watch_account = market_watch_account(job, mint)?;
    let mut ws = open_subscription_socket(endpoint).await?;
    let watcher_mode = if job.launchpad == "pump" {
        subscribe(
            &mut ws,
            "accountSubscribe",
            json!([
                watch_account,
                {
                    "encoding": "base64",
                    "commitment": "processed"
                }
            ]),
        )
        .await?;
        "helius-account-subscribe"
    } else {
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
        "helius-transaction-subscribe"
    };
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
            Some(watcher_mode.to_string()),
            Some(endpoint.to_string()),
            None,
        )
        .await;
    }
}

async fn run_market_watcher_polling_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    tx: &watch::Sender<Option<Result<u64, String>>>,
    mint: &str,
    note: Option<String>,
) -> Result<(), String> {
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        let market_cap = recompute_market_cap_for_job(state, job, mint).await?;
        let _ = tx.send(Some(Ok(market_cap)));
        set_watcher_health(
            state,
            WatcherKind::Market,
            FollowWatcherHealth::Healthy,
            Some("rpc-polling".to_string()),
            Some(state.rpc_url.clone()),
            note.clone(),
        )
        .await;
        sleep(Duration::from_millis(FOLLOW_WATCHER_RPC_POLL_INTERVAL_MS)).await;
    }
}

fn extract_slot_notification_slot(message: &Value) -> Option<u64> {
    [
        message.pointer("/params/result/slot"),
        message.pointer("/params/result/result/slot"),
    ]
    .into_iter()
    .flatten()
    .find_map(Value::as_u64)
}

fn extract_signature_notification_slot(message: &Value) -> Option<u64> {
    [
        message.pointer("/params/result/context/slot"),
        message.pointer("/params/result/value/context/slot"),
        message.pointer("/params/result/slot"),
    ]
    .into_iter()
    .flatten()
    .find_map(Value::as_u64)
}

fn extract_transaction_notification_slot(message: &Value) -> Option<u64> {
    [
        message.pointer("/params/result/slot"),
        message.pointer("/params/result/transaction/slot"),
        message.pointer("/params/result/transaction/transaction/slot"),
    ]
    .into_iter()
    .flatten()
    .find_map(Value::as_u64)
}

async fn run_helius_transaction_signature_watcher_session(
    state: &Arc<AppState>,
    trace_id: &str,
    endpoint: &str,
    signature: &str,
    account_required: &[String],
) -> Result<u64, String> {
    let session = async {
        let mut ws = open_subscription_socket(endpoint).await?;
        subscribe(
            &mut ws,
            "transactionSubscribe",
            json!([
                {
                    "signature": signature,
                    "accountRequired": account_required,
                    "failed": false,
                    "vote": false
                },
                {
                    "commitment": "confirmed",
                    "encoding": "jsonParsed",
                    "transactionDetails": "none",
                    "showRewards": false,
                    "maxSupportedTransactionVersion": 0
                }
            ]),
        )
        .await?;
        loop {
            ensure_job_not_cancelled(state, trace_id).await?;
            let message = next_json_message(&mut ws).await?;
            if message.get("params").is_none() {
                continue;
            }
            let confirmed_slot = match extract_transaction_notification_slot(&message) {
                Some(slot) => slot,
                None => fetch_current_slot(&state.rpc_url, "confirmed").await?,
            };
            set_watcher_health(
                state,
                WatcherKind::Signature,
                FollowWatcherHealth::Healthy,
                Some("helius-transaction-subscribe".to_string()),
                Some(endpoint.to_string()),
                None,
            )
            .await;
            return Ok(confirmed_slot);
        }
    };
    session.await
}

async fn run_standard_signature_watcher_session(
    state: &Arc<AppState>,
    trace_id: &str,
    endpoint: &str,
    signature: &str,
) -> Result<u64, String> {
    let mut ws = open_subscription_socket(endpoint).await?;
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
        ensure_job_not_cancelled(state, trace_id).await?;
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
                state,
                WatcherKind::Signature,
                FollowWatcherHealth::Healthy,
                Some("standard-ws".to_string()),
                Some(endpoint.to_string()),
                None,
            )
            .await;
            let confirmed_slot = match extract_signature_notification_slot(&message) {
                Some(slot) => slot,
                None => fetch_current_slot(&state.rpc_url, "confirmed").await?,
            };
            return Ok(confirmed_slot);
        }
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
    let watch_endpoint = resolve_job_watch_endpoint(&job).ok();
    let mut attempt: u32 = 0;
    let helius_account_required = job.launchTransactionSubscribeAccountRequired.clone();
    let prefers_helius_transaction_subscribe = prefers_helius_transaction_subscribe_path(
        configured_enable_helius_transaction_subscribe(),
        watch_endpoint.as_deref(),
    );
    loop {
        let session = if let Some(endpoint) = watch_endpoint.as_deref() {
            let websocket_result: Result<u64, String> = async {
                if prefers_helius_transaction_subscribe && !helius_account_required.is_empty() {
                    let hel_ws = resolved_helius_transaction_subscribe_ws_url(Some(endpoint))
                        .expect("Helius transactionSubscribe enabled but HELIUS_WS_URL / Helius watch endpoint is missing");
                    match run_helius_transaction_signature_watcher_session(
                        &state,
                        &job.traceId,
                        &hel_ws,
                        &signature,
                        &helius_account_required,
                    )
                    .await
                    {
                        Ok(confirmed_slot) => return Ok(confirmed_slot),
                        Err(error) => {
                            let fallback_note = format!(
                                "Helius transactionSubscribe signature watcher failed: {error}. Falling back to standard websocket."
                            );
                            set_watcher_health(
                                &state,
                                WatcherKind::Signature,
                                FollowWatcherHealth::Healthy,
                                Some("standard-ws".to_string()),
                                Some(endpoint.to_string()),
                                Some(fallback_note),
                            )
                            .await;
                        }
                    }
                } else if prefers_helius_transaction_subscribe {
                    let fallback_note = format!(
                        "UNEXPECTED: Helius transactionSubscribe signature watcher is missing derived accountRequired filters for launch {}. This should not happen for a valid compiled launch transaction. Falling back to standard websocket.",
                        signature
                    );
                    eprintln!("{fallback_note}");
                    set_watcher_health(
                        &state,
                        WatcherKind::Signature,
                        FollowWatcherHealth::Healthy,
                        Some("standard-ws".to_string()),
                        Some(endpoint.to_string()),
                        Some(fallback_note),
                    )
                    .await;
                }
                run_standard_signature_watcher_session(&state, &job.traceId, endpoint, &signature).await
            }
            .await;
            match websocket_result {
                Ok(confirmed_slot) => Ok(confirmed_slot),
                Err(error) => {
                    let fallback_note = if prefers_helius_transaction_subscribe {
                        format!(
                            "Websocket signature watcher failed after Helius transactionSubscribe preference: {error}. Falling back to RPC polling."
                        )
                    } else {
                        format!(
                            "Standard websocket signature watcher failed: {error}. Falling back to RPC polling."
                        )
                    };
                    run_signature_watcher_polling_session(
                        &state,
                        &job,
                        &signature,
                        Some(fallback_note),
                    )
                    .await
                }
            }
        } else {
            run_signature_watcher_polling_session(
                &state,
                &job,
                &signature,
                Some(
                    "No websocket watch endpoint configured for signature watcher. Falling back to RPC polling."
                        .to_string(),
                ),
            )
            .await
        };
        match session {
            Ok(confirmed_slot) => {
                let _ = tx.send(Some(Ok(confirmed_slot)));
                return;
            }
            Err(error) => {
                attempt = attempt.saturating_add(1);
                if let Err(final_error) = handle_watcher_retry(
                    &state,
                    &job.traceId,
                    WatcherKind::Signature,
                    watch_endpoint.as_deref().unwrap_or(&state.rpc_url),
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
    let prefers_helius_transaction_subscribe = prefers_helius_transaction_subscribe_path(
        configured_enable_helius_transaction_subscribe(),
        watch_endpoint.as_deref(),
    );
    loop {
        let session = if let Some(endpoint) = watch_endpoint.as_deref() {
            if prefers_helius_transaction_subscribe {
                let hel_ws = resolved_helius_transaction_subscribe_ws_url(Some(endpoint))
                    .expect("Helius transactionSubscribe enabled but HELIUS_WS_URL / Helius watch endpoint is missing");
                match run_helius_transaction_slot_watcher_session(&state, &job, &tx, &hel_ws).await
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
                        run_slot_watcher_polling_session(&state, &job, &tx, Some(fallback_note))
                            .await
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
    let watch_endpoint = resolve_job_watch_endpoint(&job).ok();
    let mut attempt: u32 = 0;
    let prefers_helius_transaction_subscribe = prefers_helius_transaction_subscribe_path(
        configured_enable_helius_transaction_subscribe(),
        watch_endpoint.as_deref(),
    );
    loop {
        let session = if let Some(endpoint) = watch_endpoint.as_deref() {
            if prefers_helius_transaction_subscribe {
                let hel_ws = resolved_helius_transaction_subscribe_ws_url(Some(endpoint))
                    .expect("Helius transactionSubscribe enabled but HELIUS_WS_URL / Helius watch endpoint is missing");
                let watcher_mode = if market_watcher_uses_slot_subscription(&job) {
                    "helius-slot-subscribe"
                } else {
                    "helius-transaction-subscribe"
                };
                set_watcher_health(
                    &state,
                    WatcherKind::Market,
                    FollowWatcherHealth::Healthy,
                    Some(watcher_mode.to_string()),
                    Some(hel_ws.clone()),
                    None,
                )
                .await;
                match run_helius_transaction_market_watcher_session(
                    &state, &job, &tx, &hel_ws, &mint,
                )
                .await
                {
                    Ok(()) => Ok(()),
                    Err(error) => {
                        let fallback_note = format!(
                            "Helius enhanced websocket market watcher failed: {error}. Falling back to standard websocket."
                        );
                        set_watcher_health(
                            &state,
                            WatcherKind::Market,
                            FollowWatcherHealth::Healthy,
                            Some("standard-ws".to_string()),
                            Some(endpoint.to_string()),
                            Some(fallback_note.clone()),
                        )
                        .await;
                        match run_standard_market_watcher_session(
                            &state,
                            &job,
                            &tx,
                            endpoint,
                            &mint,
                            Some(fallback_note.clone()),
                        )
                        .await
                        {
                            Ok(()) => Ok(()),
                            Err(error) => {
                                let fallback_note = format!(
                                    "{fallback_note} Standard websocket market watcher failed: {error}. Falling back to RPC polling."
                                );
                                run_market_watcher_polling_session(
                                    &state,
                                    &job,
                                    &tx,
                                    &mint,
                                    Some(fallback_note),
                                )
                                .await
                            }
                        }
                    }
                }
            } else {
                let watcher_mode = if market_watcher_uses_slot_subscription(&job) {
                    "standard-ws-slot"
                } else {
                    "standard-ws"
                };
                set_watcher_health(
                    &state,
                    WatcherKind::Market,
                    FollowWatcherHealth::Healthy,
                    Some(watcher_mode.to_string()),
                    Some(endpoint.to_string()),
                    None,
                )
                .await;
                match run_standard_market_watcher_session(&state, &job, &tx, endpoint, &mint, None)
                    .await
                {
                    Ok(()) => Ok(()),
                    Err(error) => {
                        let fallback_note = format!(
                            "Standard websocket market watcher failed: {error}. Falling back to RPC polling."
                        );
                        run_market_watcher_polling_session(
                            &state,
                            &job,
                            &tx,
                            &mint,
                            Some(fallback_note),
                        )
                        .await
                    }
                }
            }
        } else {
            run_market_watcher_polling_session(
                &state,
                &job,
                &tx,
                &mint,
                Some(
                    "No websocket watch endpoint configured for market watcher. Falling back to RPC polling."
                        .to_string(),
                ),
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
                    WatcherKind::Market,
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

async fn wait_for_signature_confirmation(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action_id: &str,
) -> Result<u64, String> {
    if let Some(confirmed_slot) = job.confirmedObservedSlot {
        capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Signature).await;
        return Ok(confirmed_slot);
    }
    if let Some(current_job) = get_job(&state, &job.traceId).await
        && let Some(confirmed_slot) = current_job.confirmedObservedSlot
    {
        capture_action_watcher_metadata(&state, &current_job, action_id, WatcherKind::Signature)
            .await;
        return Ok(confirmed_slot);
    }
    let mut rx = ensure_signature_watcher(state.clone(), job).await;
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, action_id).await?;
        if let Some(current_job) = get_job(&state, &job.traceId).await
            && let Some(confirmed_slot) = current_job.confirmedObservedSlot
        {
            capture_action_watcher_metadata(
                &state,
                &current_job,
                action_id,
                WatcherKind::Signature,
            )
            .await;
            return Ok(confirmed_slot);
        }
        let current = rx.borrow().clone();
        match current {
            Some(Ok(confirmed_slot)) => {
                capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Signature)
                    .await;
                return Ok(confirmed_slot);
            }
            Some(Err(error)) => return Err(error),
            None => {
                tokio::select! {
                    changed = rx.changed() => {
                        changed.map_err(|_| "Shared signature watcher stopped unexpectedly.".to_string())?;
                    }
                    _ = sleep(Duration::from_millis(50)) => {}
                }
            }
        }
    }
}

async fn wait_for_job_confirmation(
    state: Arc<AppState>,
    job: &FollowJobRecord,
) -> Result<u64, String> {
    if let Some(confirmed_slot) = job.confirmedObservedSlot {
        return Ok(confirmed_slot);
    }
    if let Some(current_job) = get_job(&state, &job.traceId).await
        && let Some(confirmed_slot) = current_job.confirmedObservedSlot
    {
        return Ok(confirmed_slot);
    }
    let mut rx = ensure_signature_watcher(state.clone(), job).await;
    loop {
        ensure_job_not_cancelled(&state, &job.traceId).await?;
        if let Some(current_job) = get_job(&state, &job.traceId).await
            && let Some(confirmed_slot) = current_job.confirmedObservedSlot
        {
            return Ok(confirmed_slot);
        }
        let current = rx.borrow().clone();
        match current {
            Some(Ok(confirmed_slot)) => return Ok(confirmed_slot),
            Some(Err(error)) => return Err(error),
            None => {
                tokio::select! {
                    changed = rx.changed() => {
                        changed.map_err(|_| "Shared signature watcher stopped unexpectedly.".to_string())?;
                    }
                    _ = sleep(Duration::from_millis(50)) => {}
                }
            }
        }
    }
}

async fn wait_for_deferred_setup_confirmation(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action_id: &str,
) -> Result<(), String> {
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, action_id).await?;
        let Some(current_job) = get_job(&state, &job.traceId).await else {
            return Err("Follow job disappeared while waiting for deferred setup.".to_string());
        };
        let Some(setup) = current_job.deferredSetup.as_ref() else {
            return Ok(());
        };
        match setup.state {
            DeferredSetupState::Confirmed => return Ok(()),
            DeferredSetupState::Failed => {
                return Err(setup
                    .lastError
                    .clone()
                    .unwrap_or_else(|| "Deferred setup failed before follow action could run.".to_string()));
            }
            DeferredSetupState::Queued | DeferredSetupState::Running | DeferredSetupState::Sent => {
                sleep(Duration::from_millis(50)).await;
            }
        }
    }
}

fn is_terminal_onchain_confirmation_error(error: &str) -> bool {
    error.contains("failed on-chain:") || error.contains("notification reported error:")
}

fn wallet_keypair_from_secret(secret: &[u8]) -> Result<Keypair, String> {
    let secret_key: [u8; 32] = match secret.len() {
        64 => secret[..32]
            .try_into()
            .map_err(|_| "Selected wallet keypair had an invalid 64-byte secret.".to_string())?,
        32 => secret
            .try_into()
            .map_err(|_| "Selected wallet keypair had an invalid 32-byte secret.".to_string())?,
        other => {
            return Err(format!(
                "Unsupported selected wallet keypair length for retry signing: {other} bytes."
            ));
        }
    };
    Ok(Keypair::new_from_array(secret_key))
}

fn refresh_owner_signed_compiled_transaction(
    transaction: &CompiledTransaction,
    owner: &Keypair,
    blockhash: &str,
    last_valid_block_height: u64,
) -> Result<CompiledTransaction, String> {
    let bytes = BASE64
        .decode(&transaction.serializedBase64)
        .map_err(|error| format!("Failed to decode deferred setup transaction: {error}"))?;
    let mut versioned: VersionedTransaction = bincode::deserialize(&bytes)
        .map_err(|error| format!("Failed to deserialize deferred setup transaction: {error}"))?;
    let required_signatures = usize::from(versioned.message.header().num_required_signatures);
    if required_signatures != 1 {
        return Err(format!(
            "Deferred setup retry only supports single-signer transactions, found {required_signatures}."
        ));
    }
    let payer = versioned
        .message
        .static_account_keys()
        .first()
        .cloned()
        .ok_or_else(|| "Deferred setup transaction had no payer.".to_string())?;
    if payer != owner.pubkey() {
        return Err(format!(
            "Deferred setup payer mismatch on retry: expected {}, found {}.",
            owner.pubkey(),
            payer
        ));
    }
    let fresh_hash = Hash::from_str(blockhash.trim())
        .map_err(|error| format!("Invalid deferred setup retry blockhash: {error}"))?;
    match &mut versioned.message {
        VersionedMessage::Legacy(message) => message.recent_blockhash = fresh_hash,
        VersionedMessage::V0(message) => message.recent_blockhash = fresh_hash,
        VersionedMessage::V1(_) => {
            return Err("Deferred setup retry does not yet support v1 versioned messages.".to_string());
        }
    }
    let rebuilt = VersionedTransaction::try_new(versioned.message.clone(), &[owner])
        .map_err(|error| format!("Failed to re-sign deferred setup transaction: {error}"))?;
    let serialized = bincode::serialize(&rebuilt)
        .map_err(|error| format!("Failed to serialize deferred setup retry transaction: {error}"))?;
    let serialized_base64 = BASE64.encode(serialized);
    Ok(CompiledTransaction {
        blockhash: blockhash.to_string(),
        lastValidBlockHeight: last_valid_block_height,
        serializedBase64: serialized_base64,
        signature: rebuilt.signatures.first().map(|signature| signature.to_string()),
        ..transaction.clone()
    })
}

async fn refresh_deferred_setup_transactions(
    state: &AppState,
    job: &FollowJobRecord,
    transactions: &[CompiledTransaction],
) -> Result<Vec<CompiledTransaction>, String> {
    let wallet_key = selected_wallet_key_or_default(&job.selectedWalletKey)
        .ok_or_else(|| format!("Wallet env key not found: {}", job.selectedWalletKey))?;
    let wallet_secret = load_solana_wallet_by_env_key(&wallet_key)?;
    let owner = wallet_keypair_from_secret(&wallet_secret)?;
    let (blockhash, last_valid_block_height) = fetch_latest_blockhash_fresh_or_recent(
        &state.rpc_url,
        &job.execution.commitment,
        FOLLOW_TRIGGER_COMPILE_BLOCKHASH_MIN_REMAINING_BLOCKS,
    )
    .await?;
    transactions
        .iter()
        .map(|transaction| {
            refresh_owner_signed_compiled_transaction(
                transaction,
                &owner,
                &blockhash,
                last_valid_block_height,
            )
        })
        .collect()
}

async fn wait_for_slot_offset(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action_id: &str,
    confirmed_launch_slot: Option<u64>,
    target_offset: u64,
) -> Result<u64, String> {
    wait_for_slot_offset_from_anchor(
        state,
        job,
        action_id,
        confirmed_launch_slot,
        target_offset,
        "Launch confirmation slot",
    )
    .await
}

async fn wait_for_slot_offset_from_anchor(
    state: Arc<AppState>,
    job: &FollowJobRecord,
    action_id: &str,
    base_slot: Option<u64>,
    target_offset: u64,
    anchor_label: &str,
) -> Result<u64, String> {
    if target_offset == 0 {
        capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Signature).await;
        return base_slot.ok_or_else(|| slot_anchor_unavailable_error(anchor_label));
    }
    let base_slot = base_slot
        .or(job.confirmedObservedSlot)
        .ok_or_else(|| slot_anchor_unavailable_error(anchor_label))?;
    let target_slot = base_slot.saturating_add(target_offset);
    let mut rx = ensure_slot_watcher(state.clone(), job).await;
    let current = rx.borrow().clone();
    if let Some(result) = current {
        match result {
            Ok(observed_slot) if observed_slot >= target_slot => {
                capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Slot).await;
                return Ok(observed_slot);
            }
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    }
    if let Ok(current_slot) = fetch_current_slot_fresh(&state.rpc_url, "confirmed").await {
        if current_slot >= target_slot {
            capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Slot).await;
            return Ok(current_slot);
        }
    }
    loop {
        ensure_action_not_cancelled(&state, &job.traceId, action_id).await?;
        let current = rx.borrow().clone();
        if let Some(result) = current {
            match result {
                Ok(observed_slot) if observed_slot >= target_slot => {
                    capture_action_watcher_metadata(&state, job, action_id, WatcherKind::Slot).await;
                    return Ok(observed_slot);
                }
                Ok(_) => {}
                Err(error) => return Err(error),
            }
        }
        tokio::select! {
            result = rx.changed() => {
                if let Err(_) = result {
                    return Err("Shared slot watcher stopped unexpectedly.".to_string());
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
    let mint = job
        .mint
        .as_deref()
        .ok_or_else(|| "Market watcher missing mint.".to_string())?;
    let current_market_cap = recompute_market_cap_for_job(&state, job, mint).await?;
    if current_market_cap >= threshold {
        return Ok(());
    }
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
            return Err(market_cap_scan_stopped_notice(
                action_id,
                trigger.scanTimeoutSeconds,
            ));
        }
        let current_market_cap = rx.borrow().clone();
        if let Some(result) = current_market_cap {
            match result {
                Ok(market_cap) => {
                    if market_cap >= threshold {
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
                        capture_action_watcher_metadata(
                            &state,
                            job,
                            action_id,
                            WatcherKind::Market,
                        )
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
                return Err(market_cap_scan_stopped_notice(
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

async fn poll_signature_confirmation_once(
    rpc_url: &str,
    signature: &str,
) -> Result<Option<u64>, String> {
    let response = shared_http_client()
        .post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignatureStatuses",
            "params": [
                [signature],
                { "searchTransactionHistory": true }
            ]
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let payload: Value = response.json().await.map_err(|error| error.to_string())?;
    if let Some(error) = payload.get("error") {
        return Err(format!("getSignatureStatuses failed: {error}"));
    }
    let status = payload
        .get("result")
        .and_then(|result| result.get("value"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or(Value::Null);
    if status.is_null() {
        return Ok(None);
    }
    if let Some(error) = status.get("err").cloned()
        && !error.is_null()
    {
        return Err(format!("Launch signature polling reported error: {error}"));
    }
    let confirmation_status = status
        .get("confirmationStatus")
        .and_then(Value::as_str)
        .unwrap_or("processed");
    if matches!(confirmation_status, "confirmed" | "finalized") {
        let confirmed_slot = if let Some(slot) = status.get("slot").and_then(Value::as_u64) {
            slot
        } else {
            fetch_current_slot(rpc_url, "confirmed")
                .await
                .map_err(|error| {
                    format!(
                        "Launch signature polling reached {confirmation_status} without a slot, and getSlot fallback failed: {error}"
                    )
                })?
        };
        return Ok(Some(confirmed_slot));
    }
    Ok(None)
}

async fn run_signature_watcher_polling_session(
    state: &Arc<AppState>,
    job: &FollowJobRecord,
    signature: &str,
    note: Option<String>,
) -> Result<u64, String> {
    loop {
        ensure_job_not_cancelled(state, &job.traceId).await?;
        if let Some(confirmed_slot) = poll_signature_confirmation_once(&state.rpc_url, signature).await?
        {
            set_watcher_health(
                state,
                WatcherKind::Signature,
                FollowWatcherHealth::Healthy,
                Some("rpc-polling".to_string()),
                Some(state.rpc_url.clone()),
                note,
            )
            .await;
            return Ok(confirmed_slot);
        }
        sleep(Duration::from_millis(400)).await;
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
    install_rustls_crypto_provider();
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
        bonk_follow_buy_contexts: Arc::new(Mutex::new(HashMap::new())),
        bonk_usd1_route_setups: Arc::new(Mutex::new(HashMap::new())),
        bags_follow_buy_contexts: Arc::new(Mutex::new(HashMap::new())),
        bags_market_snapshots: Arc::new(Mutex::new(HashMap::new())),
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
    spawn_blockhash_refresh_task(state.rpc_url.clone(), "confirmed");
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
    record_info(
        "follow-daemon",
        format!("LaunchDeck follow daemon listening at {}", base_url),
        Some(json!({
            "address": addr.to_string(),
            "baseUrl": base_url,
        })),
    );
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
    use axum::{Json, Router, extract::State, routing::post};
    use launchdeck_engine::config::{
        NormalizedExecution, NormalizedFollowLaunch, NormalizedFollowLaunchConstraints,
        NormalizedFollowLaunchMarketCapTrigger, NormalizedFollowLaunchSell,
        NormalizedFollowLaunchSnipe,
    };
    use launchdeck_engine::rpc::CompiledTransaction;
    use serde_json::json;
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tokio::sync::{Mutex, Semaphore};

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
            "mevProtect": false,
            "autoGas": false,
            "autoMode": "off",
            "priorityFeeSol": "0.001",
            "tipSol": "0.01",
            "maxPriorityFeeSol": "",
            "maxTipSol": "",
            "buyProvider": "standard-rpc",
            "buyEndpointProfile": "",
            "buyMevProtect": false,
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
            "sellMevProtect": false,
            "sellPriorityFeeSol": "0.001",
            "sellTipSol": "0.01",
            "sellSlippagePercent": "90",
            "sellMaxPriorityFeeSol": "",
            "sellMaxTipSol": ""
        }))
        .expect("sample execution should deserialize")
    }

    fn sample_transport_plan() -> TransportPlan {
        TransportPlan {
            requestedProvider: "standard-rpc".to_string(),
            resolvedProvider: "standard-rpc".to_string(),
            requestedEndpointProfile: "default".to_string(),
            resolvedEndpointProfile: "default".to_string(),
            executionClass: "direct".to_string(),
            transportType: "standard-rpc".to_string(),
            ordering: "sequential".to_string(),
            verified: true,
            supportsBundle: false,
            requiresInlineTip: false,
            requiresPriorityFee: false,
            separateTipTransaction: false,
            skipPreflight: false,
            maxRetries: 0,
            standardRpcSubmitEndpoints: vec![],
            helloMoonApiKeyConfigured: false,
            helloMoonMevProtect: false,
            helloMoonQuicEndpoint: None,
            helloMoonQuicEndpoints: vec![],
            helloMoonBundleEndpoint: None,
            helloMoonBundleEndpoints: vec![],
            heliusSenderEndpoint: None,
            heliusSenderEndpoints: vec![],
            watchEndpoint: None,
            watchEndpoints: vec![],
            jitoBundleEndpoints: vec![],
            warnings: vec![],
        }
    }

    fn test_state_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "launchdeck-follow-daemon-test-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
    }

    fn test_app_state(rpc_url: String, state_path: PathBuf) -> Arc<AppState> {
        Arc::new(AppState {
            auth_token: None,
            rpc_url,
            store: FollowDaemonStore::load_or_default(state_path),
            max_active_jobs: None,
            max_concurrent_compiles: None,
            max_concurrent_sends: None,
            capacity_wait_ms: 1_000,
            active_jobs: Arc::new(Mutex::new(HashMap::new())),
            wallet_locks: Arc::new(Mutex::new(HashMap::new())),
            report_write_lock: Arc::new(Mutex::new(())),
            compile_slots: Some(Arc::new(Semaphore::new(1))),
            send_slots: Some(Arc::new(Semaphore::new(1))),
            watch_hubs: Arc::new(Mutex::new(HashMap::new())),
            prepared_follow_buys: Arc::new(Mutex::new(HashMap::new())),
            hot_follow_buy_runtime: Arc::new(Mutex::new(HashMap::new())),
            hot_follow_buy_tasks: Arc::new(Mutex::new(HashMap::new())),
            bonk_follow_buy_contexts: Arc::new(Mutex::new(HashMap::new())),
            bonk_usd1_route_setups: Arc::new(Mutex::new(HashMap::new())),
            bags_follow_buy_contexts: Arc::new(Mutex::new(HashMap::new())),
            bags_market_snapshots: Arc::new(Mutex::new(HashMap::new())),
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
        })
    }

    async fn start_slot_jsonrpc_server(slot: u64) -> std::net::SocketAddr {
        async fn handler(State(slot): State<u64>, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
            let method = payload
                .get("method")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let response = match method {
                "getSlot" => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": slot
                }),
                _ => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": serde_json::Value::Null
                }),
            };
            Json(response)
        }

        let app = Router::new().route("/", post(handler)).with_state(slot);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind slot jsonrpc listener");
        let addr = listener.local_addr().expect("read slot jsonrpc addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve slot jsonrpc app");
        });
        addr
    }

    #[derive(Clone)]
    struct SignatureStatusJsonRpcState {
        status: Value,
        slot_result: Option<u64>,
    }

    async fn start_signature_status_jsonrpc_server(
        status: Value,
        slot_result: Option<u64>,
    ) -> std::net::SocketAddr {
        async fn handler(
            State(state): State<SignatureStatusJsonRpcState>,
            Json(payload): Json<serde_json::Value>,
        ) -> Json<serde_json::Value> {
            let method = payload
                .get("method")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let response = match method {
                "getSignatureStatuses" => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "value": [state.status]
                    }
                }),
                "getSlot" => match state.slot_result {
                    Some(slot) => json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": slot
                    }),
                    None => json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "error": {
                            "code": -32000,
                            "message": "slot unavailable"
                        }
                    }),
                },
                _ => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": serde_json::Value::Null
                }),
            };
            Json(response)
        }

        let app = Router::new()
            .route("/", post(handler))
            .with_state(SignatureStatusJsonRpcState {
                status,
                slot_result,
            });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind signature-status jsonrpc listener");
        let addr = listener.local_addr().expect("read signature-status addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve signature-status jsonrpc app");
        });
        addr
    }

    #[test]
    fn market_cap_threshold_scaling_uses_usd_micro_units() {
        assert_eq!(
            scale_value_between_decimals(100_000_000_000, 6, 6).expect("same scale"),
            100_000_000_000
        );
        assert_eq!(
            sol_quote_units_to_usd_micros(1_000_000_000, 150_000_000).expect("1 SOL in micros"),
            150_000_000
        );
    }

    fn env_lock() -> &'static StdMutex<()> {
        static LOCK: OnceLock<StdMutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| StdMutex::new(()))
    }

    #[test]
    fn resolved_helius_sol_price_rpc_url_prefers_override_then_helius_primary() {
        let _guard = env_lock().lock().expect("env lock");
        unsafe {
            std::env::remove_var("HELIUS_RPC_URL");
        }
        assert_eq!(
            resolved_helius_sol_price_rpc_url("https://beta.helius-rpc.com/?api-key=test"),
            Some("https://beta.helius-rpc.com/?api-key=test".to_string())
        );
        assert_eq!(
            resolved_helius_sol_price_rpc_url("https://rpc.shyft.to?api_key=test"),
            None
        );
        unsafe {
            std::env::set_var("HELIUS_RPC_URL", "https://mainnet.helius-rpc.com/?api-key=override");
        }
        assert_eq!(
            resolved_helius_sol_price_rpc_url("https://rpc.shyft.to?api_key=test"),
            Some("https://mainnet.helius-rpc.com/?api-key=override".to_string())
        );
        unsafe {
            std::env::remove_var("HELIUS_RPC_URL");
        }
    }

    #[test]
    fn http_price_scaling_matches_micro_usd_target() {
        assert_eq!(
            decimal_price_to_micro_usd(79.36).expect("scaled http price"),
            79_360_000
        );
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
            launchTransactionSubscribeAccountRequired: vec![],
            submitAtMs: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            reportPath: None,
            transportPlan: None,
            bagsLaunch: None,
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
            reservedPayloadFingerprint: String::new(),
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

    fn sample_sniper_buy_action(action_id: &str, wallet_env_key: &str, order_index: u32) -> FollowActionRecord {
        FollowActionRecord {
            actionId: action_id.to_string(),
            kind: FollowActionKind::SniperBuy,
            walletEnvKey: wallet_env_key.to_string(),
            state: FollowActionState::Queued,
            buyAmountSol: Some("0.1".to_string()),
            sellPercent: None,
            submitDelayMs: Some(0),
            targetBlockOffset: Some(0),
            delayMs: None,
            marketCap: None,
            jitterMs: Some(0),
            feeJitterBps: Some(0),
            precheckRequired: false,
            requireConfirmation: false,
            skipIfTokenBalancePositive: false,
            attemptCount: 0,
            scheduledForMs: None,
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: order_index,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        }
    }

    fn sample_sniper_sell_action(action_id: &str, wallet_env_key: &str, order_index: u32) -> FollowActionRecord {
        FollowActionRecord {
            actionId: action_id.to_string(),
            kind: FollowActionKind::SniperSell,
            walletEnvKey: wallet_env_key.to_string(),
            state: FollowActionState::Queued,
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
            signature: None,
            explorerUrl: None,
            endpoint: None,
            bundleId: None,
            lastError: None,
            triggerKey: None,
            orderIndex: order_index,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        }
    }

    #[test]
    fn full_transaction_details_are_requested_for_buys_and_follow_sells() {
        let buy = sample_sniper_buy_action("buy-1", "SOLANA_PRIVATE_KEY2", 0);
        let sniper_sell = sample_sniper_sell_action("sell-1", "SOLANA_PRIVATE_KEY2", 0);
        let mut dev_auto_sell = sample_sniper_sell_action("dev-sell", "SOLANA_PRIVATE_KEY", 0);
        dev_auto_sell.kind = FollowActionKind::DevAutoSell;

        assert!(should_request_full_transaction_details(&buy));
        assert!(should_request_full_transaction_details(&sniper_sell));
        assert!(should_request_full_transaction_details(&dev_auto_sell));
    }

    async fn set_test_slot_watch_value(
        state: &Arc<AppState>,
        trace_id: &str,
        value: Option<Result<u64, String>>,
    ) {
        let hub = get_job_watch_hub(state, trace_id).await;
        let mut started = hub.started.lock().await;
        started.slot = true;
        drop(started);
        let _ = hub.slot_tx.send(value);
    }

    #[tokio::test]
    async fn sniper_sell_waits_for_matching_buy_confirmation() {
        let state_path = test_state_path();
        let state = test_app_state("http://127.0.0.1:9/".to_string(), state_path.clone());
        let trace_id = "trace-sniper-sell-wait".to_string();
        state
            .store
            .reserve_job(FollowReserveRequest {
                traceId: trace_id.clone(),
                launchpad: "pump".to_string(),
                quoteAsset: "sol".to_string(),
                launchMode: "regular".to_string(),
                selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
                followLaunch: sample_follow_launch(),
                execution: sample_execution(),
                tokenMayhemMode: false,
                jitoTipAccount: String::new(),
                buyTipAccount: String::new(),
                sellTipAccount: String::new(),
                preferPostSetupCreatorVaultForSell: false,
                bagsLaunch: None,
                prebuiltActions: vec![
                    sample_sniper_buy_action("buy-a", "SOLANA_PRIVATE_KEY2", 0),
                    sample_sniper_sell_action("sell-a", "SOLANA_PRIVATE_KEY2", 0),
                ],
                deferredSetupTransactions: vec![],
            })
            .await
            .expect("reserve");
        let job = get_job(&state, &trace_id).await.expect("job");
        let sell_action = job
            .actions
            .iter()
            .find(|action| action.actionId == "sell-a")
            .cloned()
            .expect("sell action");
        let wait_state = state.clone();
        let wait_job = job.clone();
        let waiter = tokio::spawn(async move {
            wait_for_matching_sniper_buy_confirmation(wait_state, &wait_job, &sell_action).await
        });
        sleep(Duration::from_millis(75)).await;
        state
            .store
            .update_action(&trace_id, "buy-a", |record| {
                record.state = FollowActionState::Confirmed;
                record.confirmedObservedSlot = Some(123);
            })
            .await
            .expect("confirm matching buy");
        waiter
            .await
            .expect("wait task should join")
            .map(|confirmed_slot| assert_eq!(confirmed_slot, 123))
            .expect("sell should unblock after matching buy confirms");
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn sniper_sell_stops_when_matching_buy_fails() {
        let state_path = test_state_path();
        let state = test_app_state("http://127.0.0.1:9/".to_string(), state_path.clone());
        let trace_id = "trace-sniper-sell-stop".to_string();
        state
            .store
            .reserve_job(FollowReserveRequest {
                traceId: trace_id.clone(),
                launchpad: "pump".to_string(),
                quoteAsset: "sol".to_string(),
                launchMode: "regular".to_string(),
                selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
                followLaunch: sample_follow_launch(),
                execution: sample_execution(),
                tokenMayhemMode: false,
                jitoTipAccount: String::new(),
                buyTipAccount: String::new(),
                sellTipAccount: String::new(),
                preferPostSetupCreatorVaultForSell: false,
                bagsLaunch: None,
                prebuiltActions: vec![
                    sample_sniper_buy_action("buy-a", "SOLANA_PRIVATE_KEY2", 0),
                    sample_sniper_sell_action("sell-a", "SOLANA_PRIVATE_KEY2", 0),
                ],
                deferredSetupTransactions: vec![],
            })
            .await
            .expect("reserve");
        let job = get_job(&state, &trace_id).await.expect("job");
        let sell_action = job
            .actions
            .iter()
            .find(|action| action.actionId == "sell-a")
            .cloned()
            .expect("sell action");
        let wait_state = state.clone();
        let wait_job = job.clone();
        let waiter = tokio::spawn(async move {
            wait_for_matching_sniper_buy_confirmation(wait_state, &wait_job, &sell_action).await
        });
        sleep(Duration::from_millis(75)).await;
        state
            .store
            .update_action(&trace_id, "buy-a", |record| {
                record.state = FollowActionState::Failed;
            })
            .await
            .expect("fail matching buy");
        let error = waiter
            .await
            .expect("wait task should join")
            .expect_err("sell should stop when matching buy fails");
        assert!(error.starts_with("__stopped__"));
        assert!(error.contains("matching sniper buy buy-a ended as failed"));
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn sniper_sell_requires_matching_buy_confirmed_slot() {
        let state_path = test_state_path();
        let state = test_app_state("http://127.0.0.1:9/".to_string(), state_path.clone());
        let trace_id = "trace-sniper-sell-missing-confirm-slot".to_string();
        state
            .store
            .reserve_job(FollowReserveRequest {
                traceId: trace_id.clone(),
                launchpad: "pump".to_string(),
                quoteAsset: "sol".to_string(),
                launchMode: "regular".to_string(),
                selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
                followLaunch: sample_follow_launch(),
                execution: sample_execution(),
                tokenMayhemMode: false,
                jitoTipAccount: String::new(),
                buyTipAccount: String::new(),
                sellTipAccount: String::new(),
                preferPostSetupCreatorVaultForSell: false,
                bagsLaunch: None,
                prebuiltActions: vec![
                    sample_sniper_buy_action("buy-a", "SOLANA_PRIVATE_KEY2", 0),
                    sample_sniper_sell_action("sell-a", "SOLANA_PRIVATE_KEY2", 0),
                ],
                deferredSetupTransactions: vec![],
            })
            .await
            .expect("reserve");
        let job = get_job(&state, &trace_id).await.expect("job");
        let sell_action = job
            .actions
            .iter()
            .find(|action| action.actionId == "sell-a")
            .cloned()
            .expect("sell action");
        state
            .store
            .update_action(&trace_id, "buy-a", |record| {
                record.state = FollowActionState::Confirmed;
                record.sendObservedSlot = Some(123);
                record.confirmedObservedSlot = None;
            })
            .await
            .expect("confirm matching buy without confirm slot");
        let error = wait_for_matching_sniper_buy_confirmation(state.clone(), &job, &sell_action)
            .await
            .expect_err("autosell should require a confirmed slot anchor");
        assert!(error.contains("confirmed without a confirmed slot"));
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn sniper_sell_plus_zero_anchors_to_matching_buy_confirmation() {
        let state_path = test_state_path();
        let state = test_app_state("http://127.0.0.1:9/".to_string(), state_path.clone());
        let trace_id = "trace-sniper-sell-plus-zero".to_string();
        state
            .store
            .reserve_job(FollowReserveRequest {
                traceId: trace_id.clone(),
                launchpad: "pump".to_string(),
                quoteAsset: "sol".to_string(),
                launchMode: "regular".to_string(),
                selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
                followLaunch: sample_follow_launch(),
                execution: sample_execution(),
                tokenMayhemMode: false,
                jitoTipAccount: String::new(),
                buyTipAccount: String::new(),
                sellTipAccount: String::new(),
                preferPostSetupCreatorVaultForSell: false,
                bagsLaunch: None,
                prebuiltActions: vec![
                    sample_sniper_buy_action("buy-a", "SOLANA_PRIVATE_KEY2", 0),
                    sample_sniper_sell_action("sell-a", "SOLANA_PRIVATE_KEY2", 0),
                ],
                deferredSetupTransactions: vec![],
            })
            .await
            .expect("reserve");
        let job = get_job(&state, &trace_id).await.expect("job");
        let sell_action = job
            .actions
            .iter()
            .find(|action| action.actionId == "sell-a")
            .cloned()
            .expect("sell action");
        let wait_state = state.clone();
        let wait_job = job.clone();
        let waiter = tokio::spawn(async move {
            wait_for_action_eligibility(wait_state, &wait_job, &sell_action).await
        });
        sleep(Duration::from_millis(75)).await;
        state
            .store
            .update_action(&trace_id, "buy-a", |record| {
                record.state = FollowActionState::Confirmed;
                record.confirmedObservedSlot = Some(222);
            })
            .await
            .expect("confirm matching buy");
        let timing = waiter
            .await
            .expect("wait task should join")
            .expect("sniper sell +0 should become eligible after buy confirmation");
        assert_eq!(timing.observed_slot, Some(222));
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn sniper_sell_slot_offset_uses_matching_buy_confirmation_as_anchor() {
        let state_path = test_state_path();
        let state = test_app_state("http://127.0.0.1:9/".to_string(), state_path.clone());
        let trace_id = "trace-sniper-sell-buy-anchor".to_string();
        let mut sell_action = sample_sniper_sell_action("sell-a", "SOLANA_PRIVATE_KEY2", 0);
        sell_action.targetBlockOffset = Some(1);
        state
            .store
            .reserve_job(FollowReserveRequest {
                traceId: trace_id.clone(),
                launchpad: "pump".to_string(),
                quoteAsset: "sol".to_string(),
                launchMode: "regular".to_string(),
                selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
                followLaunch: sample_follow_launch(),
                execution: sample_execution(),
                tokenMayhemMode: false,
                jitoTipAccount: String::new(),
                buyTipAccount: String::new(),
                sellTipAccount: String::new(),
                preferPostSetupCreatorVaultForSell: false,
                bagsLaunch: None,
                prebuiltActions: vec![
                    sample_sniper_buy_action("buy-a", "SOLANA_PRIVATE_KEY2", 0),
                    sell_action,
                ],
                deferredSetupTransactions: vec![],
            })
            .await
            .expect("reserve");
        let job = get_job(&state, &trace_id).await.expect("job");
        let sell_action = job
            .actions
            .iter()
            .find(|action| action.actionId == "sell-a")
            .cloned()
            .expect("sell action");
        let wait_state = state.clone();
        let wait_job = job.clone();
        let waiter = tokio::spawn(async move {
            wait_for_action_eligibility(wait_state, &wait_job, &sell_action).await
        });
        sleep(Duration::from_millis(75)).await;
        state
            .store
            .update_action(&trace_id, "buy-a", |record| {
                record.state = FollowActionState::Confirmed;
                record.confirmedObservedSlot = Some(300);
            })
            .await
            .expect("confirm matching buy");
        set_test_slot_watch_value(&state, &trace_id, Some(Ok(300))).await;
        sleep(Duration::from_millis(75)).await;
        assert!(!waiter.is_finished());
        set_test_slot_watch_value(&state, &trace_id, Some(Ok(301))).await;
        let timing = waiter
            .await
            .expect("wait task should join")
            .expect("sniper sell +1 should wait for buy-confirmed-slot + 1");
        assert_eq!(timing.observed_slot, Some(301));
        let _ = std::fs::remove_file(state_path);
    }

    #[test]
    fn multi_wallet_sniper_sells_anchor_to_their_own_matching_buys() {
        let mut buy_a = sample_sniper_buy_action("buy-a", "SOLANA_PRIVATE_KEY2", 0);
        buy_a.confirmedObservedSlot = Some(700);
        let sell_a = sample_sniper_sell_action("sell-a", "SOLANA_PRIVATE_KEY2", 0);
        let mut buy_b = sample_sniper_buy_action("buy-b", "SOLANA_PRIVATE_KEY3", 1);
        buy_b.confirmedObservedSlot = Some(500);
        let sell_b = sample_sniper_sell_action("sell-b", "SOLANA_PRIVATE_KEY3", 1);
        let mut job = sample_job();
        job.actions = vec![buy_a, sell_a.clone(), buy_b, sell_b.clone()];

        let matched_a = matching_sniper_buy_action(&job, &sell_a).expect("sell a should match buy a");
        let matched_b = matching_sniper_buy_action(&job, &sell_b).expect("sell b should match buy b");

        assert_eq!(matched_a.actionId, "buy-a");
        assert_eq!(matched_a.confirmedObservedSlot, Some(700));
        assert_eq!(matched_b.actionId, "buy-b");
        assert_eq!(matched_b.confirmedObservedSlot, Some(500));
    }

    #[tokio::test]
    async fn signature_polling_requires_real_slot_anchor() {
        let addr = start_signature_status_jsonrpc_server(
            json!({
                "confirmationStatus": "confirmed",
                "slot": serde_json::Value::Null,
                "err": serde_json::Value::Null
            }),
            None,
        )
        .await;
        let rpc_url = format!("http://{addr}/");
        let error = poll_signature_confirmation_once(&rpc_url, "sig-test-123")
            .await
            .expect_err("missing slot fallback should fail");
        assert!(error.contains("without a slot"));
    }

    #[test]
    fn normalized_stopped_action_reason_strips_internal_marker() {
        assert_eq!(
            normalized_stopped_action_reason(Some(
                "__stopped__ sniper autosell sell-a stopped because matching sniper buy buy-a ended as failed."
            )),
            Some(
                "sniper autosell sell-a stopped because matching sniper buy buy-a ended as failed."
                    .to_string()
            )
        );
        assert_eq!(
            normalized_stopped_action_reason(Some("market-cap scan stopped for sell-a after 30 second(s).")),
            Some("market-cap scan stopped for sell-a after 30 second(s).".to_string())
        );
    }

    #[test]
    fn market_watchers_use_slot_subscription_for_migrating_launchpads() {
        let pump = sample_job();
        assert!(market_watcher_uses_slot_subscription(&pump));

        let mut bags = sample_job();
        bags.launchpad = "bagsapp".to_string();
        assert!(market_watcher_uses_slot_subscription(&bags));

        let mut bonk = sample_job();
        bonk.launchpad = "bonk".to_string();
        assert!(!market_watcher_uses_slot_subscription(&bonk));
    }

    #[tokio::test]
    async fn record_action_stopped_persists_clean_reason() {
        let state_path = test_state_path();
        let state = test_app_state("http://127.0.0.1:9/".to_string(), state_path.clone());
        let trace_id = "trace-stopped-reason".to_string();
        state
            .store
            .reserve_job(FollowReserveRequest {
                traceId: trace_id.clone(),
                launchpad: "pump".to_string(),
                quoteAsset: "sol".to_string(),
                launchMode: "regular".to_string(),
                selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
                followLaunch: sample_follow_launch(),
                execution: sample_execution(),
                tokenMayhemMode: false,
                jitoTipAccount: String::new(),
                buyTipAccount: String::new(),
                sellTipAccount: String::new(),
                preferPostSetupCreatorVaultForSell: false,
                bagsLaunch: None,
                prebuiltActions: vec![sample_sniper_sell_action("sell-a", "SOLANA_PRIVATE_KEY2", 0)],
                deferredSetupTransactions: vec![],
            })
            .await
            .expect("reserve");
        let job = get_job(&state, &trace_id).await.expect("job");
        let action = job
            .actions
            .iter()
            .find(|candidate| candidate.actionId == "sell-a")
            .cloned()
            .expect("sell action");
        record_action_stopped(
            &state,
            &job,
            &action,
            Some(
                "__stopped__ sniper autosell sell-a stopped because matching sniper buy buy-a ended as failed.",
            ),
        )
        .await;
        let updated = get_job(&state, &trace_id).await.expect("updated job");
        let updated_action = updated
            .actions
            .iter()
            .find(|candidate| candidate.actionId == "sell-a")
            .expect("updated sell action");
        assert_eq!(updated_action.state, FollowActionState::Stopped);
        assert_eq!(
            updated_action.lastError.as_deref(),
            Some("sniper autosell sell-a stopped because matching sniper buy buy-a ended as failed.")
        );
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn wait_for_slot_offset_returns_immediately_when_current_slot_already_reached() {
        let slot_addr = start_slot_jsonrpc_server(105).await;
        let state_path = test_state_path();
        let state = test_app_state(format!("http://{slot_addr}/"), state_path.clone());
        let follow_launch = NormalizedFollowLaunch {
            enabled: true,
            source: "test".to_string(),
            schemaVersion: 1,
            snipes: vec![NormalizedFollowLaunchSnipe {
                actionId: "snipe-a".to_string(),
                enabled: true,
                walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
                buyAmountSol: "1".to_string(),
                submitWithLaunch: false,
                retryOnFailure: false,
                submitDelayMs: 0,
                targetBlockOffset: Some(1),
                jitterMs: 0,
                feeJitterBps: 0,
                skipIfTokenBalancePositive: false,
                postBuySell: None,
            }],
            devAutoSell: None,
            constraints: NormalizedFollowLaunchConstraints {
                pumpOnly: false,
                retryBudget: 0,
                requireDaemonReadiness: false,
                blockOnRequiredPrechecks: false,
            },
        };
        let reserved = state
            .store
            .reserve_job(FollowReserveRequest {
                traceId: "trace-fast-slot".to_string(),
                launchpad: "pump".to_string(),
                quoteAsset: "sol".to_string(),
                launchMode: "regular".to_string(),
                selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
                followLaunch: follow_launch,
                execution: sample_execution(),
                tokenMayhemMode: false,
                jitoTipAccount: String::new(),
                buyTipAccount: String::new(),
                sellTipAccount: String::new(),
                preferPostSetupCreatorVaultForSell: false,
                bagsLaunch: None,
                prebuiltActions: vec![],
                deferredSetupTransactions: vec![],
            })
            .await
            .expect("reserve");
        let armed = state
            .store
            .arm_job(FollowArmRequest {
                traceId: reserved.traceId.clone(),
                mint: "mint".to_string(),
                launchCreator: "creator".to_string(),
                launchSignature: "sig".to_string(),
                launchTransactionSubscribeAccountRequired: vec!["payer".to_string()],
                submitAtMs: 1,
                sendObservedSlot: Some(100),
                confirmedObservedSlot: Some(100),
                reportPath: None,
                transportPlan: sample_transport_plan(),
            })
            .await
            .expect("arm");
        let observed_slot =
            wait_for_slot_offset(state.clone(), &armed, "snipe-a", Some(100), 1)
                .await
                .expect("slot wait should complete immediately");
        assert_eq!(observed_slot, 105);
        let _ = std::fs::remove_file(state_path);
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
    fn pump_sell_creator_vault_retry_ignores_retry_budget() {
        let mut job = sample_job();
        job.followLaunch.constraints.retryBudget = 0;
        let action = FollowActionRecord {
            actionId: "sell".to_string(),
            kind: FollowActionKind::SniperSell,
            walletEnvKey: "SOLANA_PRIVATE_KEY2".to_string(),
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
            attemptCount: 9,
            scheduledForMs: None,
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
    fn pump_sell_creator_vault_retry_respects_env_opt_out() {
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
        unsafe {
            env::set_var("LAUNCHDECK_ENABLE_PUMP_SELL_CREATOR_VAULT_AUTO_RETRY", "false");
        }
        assert!(!should_retry_pump_sell_creator_vault_mismatch(
            &job,
            &action,
            r#"on-chain failure | Transaction abc failed on-chain: {"InstructionError":[2,{"Custom":2006}]}"#,
        ));
        unsafe {
            env::remove_var("LAUNCHDECK_ENABLE_PUMP_SELL_CREATOR_VAULT_AUTO_RETRY");
        }
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
    fn presigned_pump_buy_creator_vault_retry_respects_env_opt_out() {
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
        unsafe {
            env::set_var("LAUNCHDECK_ENABLE_PUMP_BUY_CREATOR_VAULT_AUTO_RETRY", "false");
        }
        assert!(!should_rebuild_presigned_pump_buy_creator_vault_mismatch(
            &job,
            &action,
            r#"on-chain failure | Transaction abc failed on-chain: {"InstructionError":[3,{"Custom":2006}]}"#,
        ));
        unsafe {
            env::remove_var("LAUNCHDECK_ENABLE_PUMP_BUY_CREATOR_VAULT_AUTO_RETRY");
        }
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
    fn plus_one_buy_does_not_block_deferred_setup_phase() {
        let mut action = sample_sniper_buy_action("buy", "SOLANA_PRIVATE_KEY", 0);
        action.targetBlockOffset = Some(1);
        assert!(!action_runs_before_deferred_setup(&action));
        assert!(!should_block_deferred_setup_for_action(&action, 0));
    }

    #[test]
    fn plus_zero_buy_still_blocks_deferred_setup_phase() {
        let action = sample_sniper_buy_action("buy", "SOLANA_PRIVATE_KEY", 0);
        assert!(action_runs_before_deferred_setup(&action));
        assert!(should_block_deferred_setup_for_action(&action, 0));
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
            eligibleAtMs: None,
            submitStartedAtMs: None,
            submittedAtMs: None,
            confirmedAtMs: None,
            provider: None,
            endpointProfile: None,
            transportType: None,
            watcherMode: None,
            watcherFallbackReason: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            confirmedTokenBalanceRaw: None,
            eligibilityObservedSlot: None,
            slotsToConfirm: None,
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
        assert!(!should_use_post_setup_creator_vault_for_sell(
            true, &action, "reduced"
        ));
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
    fn sniper_autosell_rollout_gate_defaults_enabled_and_respects_env() {
        unsafe {
            env::remove_var("LAUNCHDECK_ENABLE_SNIPER_AUTOSELL");
        }
        assert!(sniper_autosell_rollout_enabled());
        unsafe {
            env::set_var("LAUNCHDECK_ENABLE_SNIPER_AUTOSELL", "false");
        }
        assert!(!sniper_autosell_rollout_enabled());
        unsafe {
            env::set_var("LAUNCHDECK_ENABLE_SNIPER_AUTOSELL", "true");
        }
        assert!(sniper_autosell_rollout_enabled());
        unsafe {
            env::remove_var("LAUNCHDECK_ENABLE_SNIPER_AUTOSELL");
        }
    }

    #[test]
    fn opt_out_flag_enabled_defaults_true_and_respects_falseish_values() {
        assert!(opt_out_flag_enabled(None));
        assert!(opt_out_flag_enabled(Some("true")));
        assert!(opt_out_flag_enabled(Some("yes")));
        assert!(!opt_out_flag_enabled(Some("false")));
        assert!(!opt_out_flag_enabled(Some("0")));
        assert!(!opt_out_flag_enabled(Some("off")));
        assert!(!opt_out_flag_enabled(Some("no")));
    }

    #[test]
    fn sniper_autosell_requested_ignores_disabled_post_buy_sell() {
        let mut follow = sample_follow_launch();
        follow.snipes.push(NormalizedFollowLaunchSnipe {
            actionId: "buy-a".to_string(),
            enabled: true,
            walletEnvKey: "WALLET_A".to_string(),
            buyAmountSol: "0.1".to_string(),
            submitWithLaunch: false,
            retryOnFailure: false,
            submitDelayMs: 0,
            targetBlockOffset: Some(0),
            jitterMs: 0,
            feeJitterBps: 0,
            skipIfTokenBalancePositive: false,
            postBuySell: Some(NormalizedFollowLaunchSell {
                actionId: "sell-a".to_string(),
                enabled: false,
                walletEnvKey: "WALLET_A".to_string(),
                percent: 100,
                delayMs: None,
                targetBlockOffset: Some(0),
                marketCap: None,
                precheckRequired: false,
                requireConfirmation: false,
            }),
        });
        assert!(!sniper_autosell_requested(&follow));
        follow.snipes[0].postBuySell = Some(NormalizedFollowLaunchSell {
            actionId: "sell-a".to_string(),
            enabled: true,
            walletEnvKey: "WALLET_A".to_string(),
            percent: 100,
            delayMs: None,
            targetBlockOffset: Some(0),
            marketCap: None,
            precheckRequired: false,
            requireConfirmation: false,
        });
        assert!(sniper_autosell_requested(&follow));
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
    fn selected_realtime_watcher_mode_uses_helius_ws_env_with_non_helius_watch() {
        unsafe {
            env::set_var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE", "true");
            env::set_var(
                "HELIUS_WS_URL",
                "wss://mainnet.helius-rpc.com/?api-key=test-env",
            );
        }
        assert_eq!(
            selected_realtime_watcher_mode("standard-rpc", "us", Some("wss://rpc.shyft.to/ws"),),
            "helius-transaction-subscribe"
        );
        unsafe {
            env::remove_var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE");
            env::remove_var("HELIUS_WS_URL");
        }
    }

    #[test]
    fn inferred_market_watcher_mode_uses_slot_labels_for_migrating_launchpads() {
        unsafe {
            env::set_var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE", "true");
        }
        let pump = sample_job();
        assert_eq!(
            inferred_market_watcher_mode_for_job(
                &pump,
                Some("wss://mainnet.helius-rpc.com/?api-key=test"),
            ),
            "helius-slot-subscribe"
        );

        let mut bags = sample_job();
        bags.launchpad = "bagsapp".to_string();
        assert_eq!(
            inferred_market_watcher_mode_for_job(&bags, Some("wss://rpc.shyft.to/ws")),
            "standard-ws-slot"
        );

        let mut bonk = sample_job();
        bonk.launchpad = "bonk".to_string();
        assert_eq!(
            inferred_market_watcher_mode_for_job(
                &bonk,
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

    #[test]
    fn deferred_setup_retry_refresh_re_signs_single_signer_transaction() {
        let owner = Keypair::new();
        let original_hash = Hash::new_unique();
        let fresh_hash = Hash::new_unique();
        let message = VersionedMessage::Legacy(solana_sdk::message::Message::new_with_blockhash(
            &[],
            Some(&owner.pubkey()),
            &original_hash,
        ));
        let transaction = VersionedTransaction::try_new(message, &[&owner]).unwrap();
        let serialized_base64 = BASE64.encode(bincode::serialize(&transaction).unwrap());
        let compiled = CompiledTransaction {
            label: "agent-setup".to_string(),
            format: "legacy".to_string(),
            blockhash: original_hash.to_string(),
            lastValidBlockHeight: 10,
            serializedBase64: serialized_base64,
            signature: transaction.signatures.first().map(|signature| signature.to_string()),
            lookupTablesUsed: vec![],
            computeUnitLimit: None,
            computeUnitPriceMicroLamports: None,
            inlineTipLamports: None,
            inlineTipAccount: None,
        };

        let refreshed = refresh_owner_signed_compiled_transaction(
            &compiled,
            &owner,
            &fresh_hash.to_string(),
            25,
        )
        .unwrap();
        let refreshed_bytes = BASE64.decode(&refreshed.serializedBase64).unwrap();
        let refreshed_tx: VersionedTransaction = bincode::deserialize(&refreshed_bytes).unwrap();

        assert_eq!(refreshed.blockhash, fresh_hash.to_string());
        assert_eq!(refreshed.lastValidBlockHeight, 25);
        assert_eq!(
            refreshed_tx.message.recent_blockhash().to_string(),
            fresh_hash.to_string()
        );
        assert_ne!(refreshed.signature, compiled.signature);
    }

    #[test]
    fn deferred_setup_retry_refresh_rejects_multi_signer_transaction() {
        let payer = Keypair::new();
        let extra_signer = Keypair::new();
        let original_hash = Hash::new_unique();
        let instruction = solana_sdk::instruction::Instruction {
            program_id: solana_sdk::pubkey::Pubkey::new_unique(),
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(payer.pubkey(), true),
                solana_sdk::instruction::AccountMeta::new_readonly(extra_signer.pubkey(), true),
            ],
            data: vec![],
        };
        let message = VersionedMessage::Legacy(solana_sdk::message::Message::new_with_blockhash(
            &[instruction],
            Some(&payer.pubkey()),
            &original_hash,
        ));
        let transaction = VersionedTransaction::try_new(message, &[&payer, &extra_signer]).unwrap();
        let compiled = CompiledTransaction {
            label: "agent-setup".to_string(),
            format: "legacy".to_string(),
            blockhash: original_hash.to_string(),
            lastValidBlockHeight: 10,
            serializedBase64: BASE64.encode(bincode::serialize(&transaction).unwrap()),
            signature: transaction.signatures.first().map(|signature| signature.to_string()),
            lookupTablesUsed: vec![],
            computeUnitLimit: None,
            computeUnitPriceMicroLamports: None,
            inlineTipLamports: None,
            inlineTipAccount: None,
        };

        let error = refresh_owner_signed_compiled_transaction(
            &compiled,
            &payer,
            &Hash::new_unique().to_string(),
            25,
        )
        .unwrap_err();
        assert!(error.contains("single-signer"));
    }
}
