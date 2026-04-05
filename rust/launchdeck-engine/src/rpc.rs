#![allow(non_snake_case, dead_code)]

use futures_util::{SinkExt, StreamExt, future::join_all, stream::FuturesUnordered};
use lunar_lander_quic_client::{ClientOptions, LunarLanderQuicClient};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use solana_sdk::transaction::VersionedTransaction;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, OnceLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::{Duration, sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::transport::{JitoBundleEndpoint, TransportPlan, configured_hellomoon_api_key};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledTransaction {
    pub label: String,
    pub format: String,
    pub blockhash: String,
    pub lastValidBlockHeight: u64,
    pub serializedBase64: String,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub lookupTablesUsed: Vec<String>,
    #[serde(default)]
    pub computeUnitLimit: Option<u64>,
    #[serde(default)]
    pub computeUnitPriceMicroLamports: Option<u64>,
    #[serde(default)]
    pub inlineTipLamports: Option<u64>,
    #[serde(default)]
    pub inlineTipAccount: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationResult {
    pub label: String,
    pub format: String,
    pub err: Option<Value>,
    pub unitsConsumed: Option<u64>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SentResult {
    pub label: String,
    pub format: String,
    pub signature: Option<String>,
    pub explorerUrl: Option<String>,
    pub transportType: String,
    pub endpoint: Option<String>,
    pub attemptedEndpoints: Vec<String>,
    pub skipPreflight: bool,
    pub maxRetries: u32,
    pub confirmationStatus: Option<String>,
    pub confirmationSource: Option<String>,
    pub submittedAtMs: Option<u128>,
    pub firstObservedStatus: Option<String>,
    pub firstObservedSlot: Option<u64>,
    pub firstObservedAtMs: Option<u128>,
    pub confirmedAtMs: Option<u128>,
    pub sendObservedBlockHeight: Option<u64>,
    pub confirmedObservedBlockHeight: Option<u64>,
    pub confirmedSlot: Option<u64>,
    pub computeUnitLimit: Option<u64>,
    pub computeUnitPriceMicroLamports: Option<u64>,
    pub inlineTipLamports: Option<u64>,
    pub inlineTipAccount: Option<String>,
    pub bundleId: Option<String>,
    pub attemptedBundleIds: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct SendTimingBreakdown {
    pub submit_ms: u128,
    pub confirm_ms: u128,
}

struct ConfirmationDetails {
    status: Value,
    confirmed_observed_block_height: Option<u64>,
    confirmed_slot: Option<u64>,
    confirmation_source: &'static str,
    first_observed_status: Option<String>,
    first_observed_slot: Option<u64>,
    first_observed_at_ms: Option<u128>,
    confirmed_at_ms: Option<u128>,
}

const BLOCKHASH_REFRESH_INTERVAL: Duration = Duration::from_secs(10);
const BLOCKHASH_MAX_AGE: Duration = Duration::from_secs(20);
const DEFAULT_BLOCK_HEIGHT_CACHE_TTL_MS: u64 = 200;
const DEFAULT_BLOCK_HEIGHT_SAMPLE_MAX_AGE_MS: u64 = 1_000;

#[derive(Clone)]
struct CachedBlockhash {
    blockhash: String,
    last_valid_block_height: u64,
    fetched_at: Instant,
}

#[derive(Clone)]
struct CachedBlockHeight {
    value: u64,
    fetched_at: Instant,
}

fn blockhash_cache() -> &'static Mutex<HashMap<String, CachedBlockhash>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedBlockhash>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn block_height_cache() -> &'static AsyncMutex<HashMap<String, CachedBlockHeight>> {
    static CACHE: OnceLock<AsyncMutex<HashMap<String, CachedBlockHeight>>> = OnceLock::new();
    CACHE.get_or_init(|| AsyncMutex::new(HashMap::new()))
}

fn block_height_refresh_inflight() -> &'static AsyncMutex<HashSet<String>> {
    static INFLIGHT: OnceLock<AsyncMutex<HashSet<String>>> = OnceLock::new();
    INFLIGHT.get_or_init(|| AsyncMutex::new(HashSet::new()))
}

fn hellomoon_quic_client_cache() -> &'static AsyncMutex<HashMap<String, Arc<LunarLanderQuicClient>>>
{
    static CACHE: OnceLock<AsyncMutex<HashMap<String, Arc<LunarLanderQuicClient>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| AsyncMutex::new(HashMap::new()))
}

fn hellomoon_quic_client_cache_key(endpoint: &str, api_key: &str, mev_protect: bool) -> String {
    format!("{endpoint}|{mev_protect}|{api_key}")
}

async fn cached_hellomoon_quic_client(
    endpoint: &str,
    api_key: &str,
    mev_protect: bool,
) -> Result<Arc<LunarLanderQuicClient>, String> {
    let key = hellomoon_quic_client_cache_key(endpoint, api_key, mev_protect);
    {
        let cache = hellomoon_quic_client_cache().lock().await;
        if let Some(client) = cache.get(&key) {
            return Ok(client.clone());
        }
    }
    let client = Arc::new(
        LunarLanderQuicClient::connect_with_options(
            endpoint.to_string(),
            api_key.to_string(),
            ClientOptions {
                mev_protect,
                ..ClientOptions::default()
            },
        )
        .await
        .map_err(|error| format!("Hello Moon QUIC connect failed for {endpoint}: {error}"))?,
    );
    let mut cache = hellomoon_quic_client_cache().lock().await;
    Ok(cache.entry(key).or_insert_with(|| client.clone()).clone())
}

async fn invalidate_hellomoon_quic_client(endpoint: &str, api_key: &str, mev_protect: bool) {
    let key = hellomoon_quic_client_cache_key(endpoint, api_key, mev_protect);
    let mut cache = hellomoon_quic_client_cache().lock().await;
    cache.remove(&key);
}

fn current_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn blockhash_cache_key(rpc_url: &str, commitment: &str) -> String {
    format!("{rpc_url}|{commitment}")
}

pub fn configured_warm_rpc_url(primary_rpc_url: &str) -> String {
    std::env::var("LAUNCHDECK_WARM_RPC_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| primary_rpc_url.to_string())
}

fn configured_block_height_cache_ttl() -> Duration {
    std::env::var("LAUNCHDECK_BLOCK_HEIGHT_CACHE_TTL_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_millis(DEFAULT_BLOCK_HEIGHT_CACHE_TTL_MS))
}

fn configured_block_height_sample_max_age() -> Duration {
    std::env::var("LAUNCHDECK_BLOCK_HEIGHT_SAMPLE_MAX_AGE_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_millis(DEFAULT_BLOCK_HEIGHT_SAMPLE_MAX_AGE_MS))
}

fn block_height_cache_lookup_key(rpc_url: &str, commitment: &str) -> String {
    blockhash_cache_key(&configured_warm_rpc_url(rpc_url), commitment)
}

async fn refresh_block_height_sample(rpc_url: &str, commitment: &str) -> Result<u64, String> {
    let block_height_rpc_url = configured_warm_rpc_url(rpc_url);
    let cache_key = blockhash_cache_key(&block_height_rpc_url, commitment);
    let result = rpc_request(
        &block_height_rpc_url,
        "getBlockHeight",
        json!([
            {
                "commitment": commitment,
            }
        ]),
    )
    .await?;
    let value = result
        .as_u64()
        .ok_or_else(|| "RPC getBlockHeight did not return a block height.".to_string())?;
    let mut cache = block_height_cache().lock().await;
    cache.insert(
        cache_key,
        CachedBlockHeight {
            value,
            fetched_at: Instant::now(),
        },
    );
    Ok(value)
}

async fn spawn_block_height_sample_refresh_if_needed(rpc_url: &str, commitment: &str) {
    let cache_key = block_height_cache_lookup_key(rpc_url, commitment);
    {
        let mut inflight = block_height_refresh_inflight().lock().await;
        if inflight.contains(&cache_key) {
            return;
        }
        inflight.insert(cache_key.clone());
    }
    let task_rpc_url = rpc_url.to_string();
    let task_commitment = commitment.to_string();
    tokio::spawn(async move {
        let _ = refresh_block_height_sample(&task_rpc_url, &task_commitment).await;
        let mut inflight = block_height_refresh_inflight().lock().await;
        inflight.remove(&cache_key);
    });
}

async fn fetch_sampled_block_height_snapshot(
    rpc_url: &str,
    commitment: &str,
) -> Result<u64, String> {
    let cache_key = block_height_cache_lookup_key(rpc_url, commitment);
    let ttl = configured_block_height_cache_ttl();
    let sample_max_age = configured_block_height_sample_max_age();
    let cached = {
        let cache = block_height_cache().lock().await;
        cache
            .get(&cache_key)
            .map(|entry| (entry.value, entry.fetched_at.elapsed()))
    };
    if let Some((value, age)) = cached {
        if age <= ttl {
            return Ok(value);
        }
        if age <= sample_max_age {
            spawn_block_height_sample_refresh_if_needed(rpc_url, commitment).await;
            return Ok(value);
        }
    }
    refresh_block_height_sample(rpc_url, commitment).await
}

fn get_cached_blockhash(rpc_url: &str, commitment: &str) -> Option<(String, u64)> {
    let cache = blockhash_cache().lock().ok()?;
    let entry = cache.get(&blockhash_cache_key(rpc_url, commitment))?;
    if entry.fetched_at.elapsed() > BLOCKHASH_MAX_AGE {
        return None;
    }
    Some((entry.blockhash.clone(), entry.last_valid_block_height))
}

fn cache_blockhash(
    rpc_url: &str,
    commitment: &str,
    blockhash: String,
    last_valid_block_height: u64,
) {
    if let Ok(mut cache) = blockhash_cache().lock() {
        cache.insert(
            blockhash_cache_key(rpc_url, commitment),
            CachedBlockhash {
                blockhash,
                last_valid_block_height,
                fetched_at: Instant::now(),
            },
        );
    }
}

async fn rpc_request(rpc_url: &str, method: &str, params: Value) -> Result<Value, String> {
    crate::observability::record_outbound_provider_http_request();
    let response = shared_http_client()
        .post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let header_names = [
        "x-request-id",
        "x-amzn-requestid",
        "cf-ray",
        "server",
        "content-type",
        "content-length",
        "date",
        "via",
    ];
    let response_headers = response.headers().clone();
    let header_summary = header_names
        .iter()
        .filter_map(|name| {
            response_headers
                .get(*name)
                .and_then(|value| value.to_str().ok())
                .map(|value| format!("{name}={value}"))
        })
        .collect::<Vec<_>>()
        .join(", ");
    let body = response.text().await.map_err(|error| error.to_string())?;
    let payload: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({ "raw": body }));
    if !status.is_success() {
        let detail = payload
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                payload
                    .get("message")
                    .and_then(Value::as_str)
                    .map(
                        |message| match payload.get("code").and_then(Value::as_i64) {
                            Some(code) => format!("code={code}, message={message}"),
                            None => message.to_string(),
                        },
                    )
            })
            .or_else(|| {
                payload
                    .get("raw")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_else(|| payload.to_string());
        let header_detail = if header_summary.is_empty() {
            String::new()
        } else {
            format!(" | headers: {header_summary}")
        };
        return Err(format!(
            "RPC {} failed with status {}: {}{}",
            method, status, detail, header_detail
        ));
    }
    if let Some(error) = payload.get("error") {
        return Err(error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("RPC request failed.")
            .to_string());
    }
    Ok(payload.get("result").cloned().unwrap_or(Value::Null))
}

pub async fn prewarm_rpc_endpoint(rpc_url: &str) -> Result<(), String> {
    rpc_request(rpc_url, "getVersion", json!([]))
        .await
        .map(|_| ())
}

pub async fn fetch_current_block_height_fresh(
    rpc_url: &str,
    commitment: &str,
) -> Result<u64, String> {
    refresh_block_height_sample(rpc_url, commitment).await
}

pub async fn fetch_latest_blockhash(
    rpc_url: &str,
    commitment: &str,
) -> Result<(String, u64), String> {
    let result = rpc_request(
        rpc_url,
        "getLatestBlockhash",
        json!([
            {
                "commitment": commitment,
            }
        ]),
    )
    .await?;
    let value = result
        .get("value")
        .cloned()
        .unwrap_or_else(|| result.clone());
    let blockhash = value
        .get("blockhash")
        .and_then(Value::as_str)
        .ok_or_else(|| "RPC getLatestBlockhash did not return a blockhash.".to_string())?
        .to_string();
    let last_valid_block_height = value
        .get("lastValidBlockHeight")
        .and_then(Value::as_u64)
        .ok_or_else(|| "RPC getLatestBlockhash did not return lastValidBlockHeight.".to_string())?;
    Ok((blockhash, last_valid_block_height))
}

pub async fn fetch_latest_blockhash_cached(
    rpc_url: &str,
    commitment: &str,
) -> Result<(String, u64), String> {
    if let Some(cached) = get_cached_blockhash(rpc_url, commitment) {
        return Ok(cached);
    }
    let (blockhash, last_valid_block_height) = fetch_latest_blockhash(rpc_url, commitment).await?;
    cache_blockhash(
        rpc_url,
        commitment,
        blockhash.clone(),
        last_valid_block_height,
    );
    Ok((blockhash, last_valid_block_height))
}

pub async fn refresh_latest_blockhash_cache(rpc_url: &str, commitment: &str) -> Result<(), String> {
    let (blockhash, last_valid_block_height) = fetch_latest_blockhash(rpc_url, commitment).await?;
    cache_blockhash(rpc_url, commitment, blockhash, last_valid_block_height);
    Ok(())
}

#[allow(dead_code)]
pub fn spawn_blockhash_refresh_task(rpc_url: String, commitment: &'static str) {
    tokio::spawn(async move {
        loop {
            if let Ok((blockhash, last_valid_block_height)) =
                fetch_latest_blockhash(&rpc_url, commitment).await
            {
                cache_blockhash(&rpc_url, commitment, blockhash, last_valid_block_height);
            }
            sleep(BLOCKHASH_REFRESH_INTERVAL).await;
        }
    });
}

pub async fn fetch_account_data(
    rpc_url: &str,
    account: &str,
    commitment: &str,
) -> Result<Vec<u8>, String> {
    let result = rpc_request(
        rpc_url,
        "getAccountInfo",
        json!([
            account,
            {
                "encoding": "base64",
                "commitment": commitment,
            }
        ]),
    )
    .await?;
    let value = result
        .get("value")
        .ok_or_else(|| format!("RPC getAccountInfo did not return a value for {account}."))?;
    if value.is_null() {
        return Err(format!("Account {account} was not found."));
    }
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(Value::as_str)
        .ok_or_else(|| format!("RPC getAccountInfo returned invalid base64 data for {account}."))?;
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    BASE64.decode(data).map_err(|error| error.to_string())
}

pub async fn fetch_multiple_account_data(
    rpc_url: &str,
    accounts: &[String],
    commitment: &str,
) -> Result<Vec<Option<Vec<u8>>>, String> {
    if accounts.is_empty() {
        return Ok(vec![]);
    }
    let result = rpc_request(
        rpc_url,
        "getMultipleAccounts",
        json!([
            accounts,
            {
                "encoding": "base64",
                "commitment": commitment,
            }
        ]),
    )
    .await?;
    let values = result
        .get("value")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| "RPC getMultipleAccounts did not return a value array.".to_string())?;
    if values.len() != accounts.len() {
        return Err(format!(
            "RPC getMultipleAccounts returned {} entries for {} requested accounts.",
            values.len(),
            accounts.len()
        ));
    }
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            if value.is_null() {
                return Ok(None);
            }
            let data = value
                .get("data")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "RPC getMultipleAccounts returned invalid base64 data for {}.",
                        accounts[index]
                    )
                })?;
            use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
            BASE64
                .decode(data)
                .map(Some)
                .map_err(|error| error.to_string())
        })
        .collect()
}

pub async fn fetch_account_exists(
    rpc_url: &str,
    account: &str,
    commitment: &str,
) -> Result<bool, String> {
    let result = rpc_request(
        rpc_url,
        "getAccountInfo",
        json!([
            account,
            {
                "encoding": "base64",
                "commitment": commitment,
                "dataSlice": {
                    "offset": 0,
                    "length": 0
                }
            }
        ]),
    )
    .await?;
    Ok(result.get("value").is_some_and(|value| !value.is_null()))
}

pub async fn fetch_multiple_account_exists(
    rpc_url: &str,
    accounts: &[String],
    commitment: &str,
) -> Result<Vec<bool>, String> {
    if accounts.is_empty() {
        return Ok(vec![]);
    }
    let result = rpc_request(
        rpc_url,
        "getMultipleAccounts",
        json!([
            accounts,
            {
                "encoding": "base64",
                "commitment": commitment,
                "dataSlice": {
                    "offset": 0,
                    "length": 0
                }
            }
        ]),
    )
    .await?;
    let values = result
        .get("value")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| "RPC getMultipleAccounts did not return a value array.".to_string())?;
    if values.len() != accounts.len() {
        return Err(format!(
            "RPC getMultipleAccounts returned {} entries for {} requested accounts.",
            values.len(),
            accounts.len()
        ));
    }
    Ok(values
        .into_iter()
        .map(|value| !value.is_null())
        .collect::<Vec<_>>())
}

pub async fn fetch_current_block_height(rpc_url: &str, commitment: &str) -> Result<u64, String> {
    let cache_key = block_height_cache_lookup_key(rpc_url, commitment);
    let ttl = configured_block_height_cache_ttl();
    let cache = block_height_cache().lock().await;
    if let Some(entry) = cache.get(&cache_key)
        && entry.fetched_at.elapsed() <= ttl
    {
        return Ok(entry.value);
    }
    drop(cache);
    refresh_block_height_sample(rpc_url, commitment).await
}

pub async fn simulate_transactions(
    rpc_url: &str,
    transactions: &[CompiledTransaction],
    commitment: &str,
) -> Result<(Vec<SimulationResult>, Vec<String>), String> {
    let mut results = Vec::new();
    let mut warnings = Vec::new();
    if transactions.len() > 1 {
        warnings.push(
            "Simulation runs each transaction independently; follow-up transactions may fail if they depend on earlier state changes."
                .to_string(),
        );
    }
    for transaction in transactions {
        let result = rpc_request(
            rpc_url,
            "simulateTransaction",
            json!([
                transaction.serializedBase64,
                {
                    "encoding": "base64",
                    "commitment": commitment,
                    "replaceRecentBlockhash": false,
                    "sigVerify": true,
                }
            ]),
        )
        .await?;
        let logs = result
            .get("value")
            .and_then(|value| value.get("logs"))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        results.push(SimulationResult {
            label: transaction.label.clone(),
            format: transaction.format.clone(),
            err: result
                .get("value")
                .and_then(|value| value.get("err"))
                .cloned(),
            unitsConsumed: result
                .get("value")
                .and_then(|value| value.get("unitsConsumed"))
                .and_then(Value::as_u64),
            logs,
        });
    }
    Ok((results, warnings))
}

fn commitment_rank(commitment: &str) -> u8 {
    match commitment {
        "processed" => 1,
        "confirmed" => 2,
        "finalized" => 3,
        _ => 0,
    }
}

fn commitment_satisfied(actual: &str, required: &str) -> bool {
    commitment_rank(actual) >= commitment_rank(required)
}

fn signature_from_serialized_base64(serialized_base64: &str) -> Option<String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    let bytes = BASE64.decode(serialized_base64).ok()?;
    let transaction: VersionedTransaction = bincode::deserialize(&bytes).ok()?;
    transaction
        .signatures
        .first()
        .map(|signature| signature.to_string())
}

pub(crate) fn precompute_transaction_signature(serialized_base64: &str) -> Option<String> {
    signature_from_serialized_base64(serialized_base64)
}

fn shared_http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(Client::new)
}

async fn wait_for_confirmation(
    rpc_url: &str,
    signature: &str,
    commitment: &str,
    max_attempts: u32,
    track_confirmed_block_height: bool,
) -> Result<ConfirmationDetails, String> {
    wait_for_confirmation_polling(
        rpc_url,
        signature,
        commitment,
        max_attempts,
        1500,
        track_confirmed_block_height,
    )
    .await
}

async fn wait_for_confirmation_polling(
    rpc_url: &str,
    signature: &str,
    commitment: &str,
    max_attempts: u32,
    poll_interval_ms: u64,
    track_confirmed_block_height: bool,
) -> Result<ConfirmationDetails, String> {
    let mut first_observed_status = None;
    let mut first_observed_slot = None;
    let mut first_observed_at_ms = None;
    for _ in 0..max_attempts {
        let result = rpc_request(
            rpc_url,
            "getSignatureStatuses",
            json!([
                [signature],
                { "searchTransactionHistory": true }
            ]),
        )
        .await?;
        if let Some(status) = result
            .get("value")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .cloned()
        {
            if status.is_null() {
                sleep(Duration::from_millis(poll_interval_ms)).await;
                continue;
            }
            let actual_commitment = status
                .get("confirmationStatus")
                .and_then(Value::as_str)
                .unwrap_or("processed");
            if first_observed_status.is_none() {
                first_observed_status = Some(actual_commitment.to_string());
                first_observed_slot = status.get("slot").and_then(Value::as_u64);
                first_observed_at_ms = Some(current_time_ms());
            }
            if status.get("err").is_some() && !status.get("err").unwrap_or(&Value::Null).is_null() {
                return Err(format!(
                    "Transaction {} failed on-chain: {}",
                    signature,
                    status.get("err").cloned().unwrap_or(Value::Null)
                ));
            }
            if commitment_satisfied(actual_commitment, commitment) {
                let confirmed_observed_block_height = if track_confirmed_block_height {
                    fetch_sampled_block_height_snapshot(rpc_url, commitment)
                        .await
                        .ok()
                } else {
                    None
                };
                let confirmed_slot = status.get("slot").and_then(Value::as_u64);
                return Ok(ConfirmationDetails {
                    status,
                    confirmed_observed_block_height,
                    confirmed_slot,
                    confirmation_source: "rpc-polling",
                    first_observed_status,
                    first_observed_slot,
                    first_observed_at_ms,
                    confirmed_at_ms: Some(current_time_ms()),
                });
            }
        }
        sleep(Duration::from_millis(poll_interval_ms)).await;
    }
    Err(format!(
        "Timed out waiting for transaction {} to reach {}.",
        signature, commitment
    ))
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

async fn send_jsonrpc_request(
    ws: &mut WsStream,
    request_id: i64,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    ws.send(Message::Text(
        json!({
            "jsonrpc": "2.0",
            "id": request_id,
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
        if payload.get("id").and_then(Value::as_i64) == Some(request_id) {
            if payload.get("error").is_some() {
                return Err(format!("JSON-RPC request failed: {payload}"));
            }
            return Ok(payload);
        }
    }
}

async fn subscribe(ws: &mut WsStream, method: &str, params: Value) -> Result<(), String> {
    send_jsonrpc_request(ws, 1, method, params).await.map(|_| ())
}

fn jsonrpc_subscription_id(payload: &Value, method: &str) -> Result<i64, String> {
    payload
        .get("result")
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{method} ack missing subscription id: {payload}"))
}

pub async fn prewarm_watch_websocket_endpoint(endpoint: &str) -> Result<(), String> {
    let mut ws = open_subscription_socket(endpoint).await?;
    let subscribe_payload =
        send_jsonrpc_request(&mut ws, 70_001, "slotSubscribe", json!([])).await?;
    let subscription_id = jsonrpc_subscription_id(&subscribe_payload, "slotSubscribe")?;
    let _ = send_jsonrpc_request(&mut ws, 70_002, "slotUnsubscribe", json!([subscription_id]))
        .await;
    Ok(())
}

pub async fn prewarm_helius_transaction_subscribe_endpoint(endpoint: &str) -> Result<(), String> {
    let mut ws = open_subscription_socket(endpoint).await?;
    let subscribe_payload = send_jsonrpc_request(
        &mut ws,
        71_001,
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
    let subscription_id = jsonrpc_subscription_id(&subscribe_payload, "transactionSubscribe")?;
    let _ = send_jsonrpc_request(
        &mut ws,
        71_002,
        "transactionUnsubscribe",
        json!([subscription_id]),
    )
    .await;
    Ok(())
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

async fn wait_for_confirmation_websocket(
    endpoint: &str,
    rpc_url: &str,
    signature: &str,
    commitment: &str,
    track_confirmed_block_height: bool,
) -> Result<ConfirmationDetails, String> {
    let session = async {
        let mut ws = open_subscription_socket(endpoint).await?;
        subscribe(
            &mut ws,
            "signatureSubscribe",
            json!([
                signature,
                {
                    "commitment": commitment
                }
            ]),
        )
        .await?;
        loop {
            let message = next_json_message(&mut ws).await?;
            if let Some(params) = message.get("params") {
                let context_slot = params
                    .get("result")
                    .and_then(|result| result.get("context"))
                    .and_then(|context| context.get("slot"))
                    .and_then(Value::as_u64);
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
                let confirmed_observed_block_height = if track_confirmed_block_height {
                    fetch_sampled_block_height_snapshot(rpc_url, commitment)
                        .await
                        .ok()
                } else {
                    None
                };
                let observed_at_ms = current_time_ms();
                return Ok(ConfirmationDetails {
                    status: json!({
                        "confirmationStatus": commitment,
                        "slot": context_slot,
                    }),
                    confirmed_observed_block_height,
                    confirmed_slot: context_slot,
                    confirmation_source: "websocket",
                    first_observed_status: Some(commitment.to_string()),
                    first_observed_slot: context_slot,
                    first_observed_at_ms: Some(observed_at_ms),
                    confirmed_at_ms: Some(observed_at_ms),
                });
            }
        }
    };
    timeout(Duration::from_secs(35), session)
        .await
        .map_err(|_| {
            format!(
                "Timed out waiting for websocket confirmation for transaction {}.",
                signature
            )
        })?
}

async fn wait_for_confirmations_polling_batch(
    rpc_url: &str,
    signatures: &[(usize, String)],
    commitment: &str,
    max_attempts: u32,
    poll_interval_ms: u64,
) -> Result<Vec<(usize, ConfirmationDetails)>, String> {
    let mut pending = signatures.to_vec();
    let mut completed = Vec::with_capacity(signatures.len());
    let mut first_observed_status: HashMap<usize, String> = HashMap::new();
    let mut first_observed_slot: HashMap<usize, u64> = HashMap::new();
    let mut first_observed_at_ms: HashMap<usize, u128> = HashMap::new();

    for attempt in 0..max_attempts {
        if pending.is_empty() {
            return Ok(completed);
        }
        let search_transaction_history = attempt + 1 >= std::cmp::max(2, max_attempts / 2);
        let signature_batch = pending
            .iter()
            .map(|(_, signature)| Value::String(signature.clone()))
            .collect::<Vec<_>>();
        let result = rpc_request(
            rpc_url,
            "getSignatureStatuses",
            json!([
                signature_batch,
                { "searchTransactionHistory": search_transaction_history }
            ]),
        )
        .await?;
        let values = result
            .get("value")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut next_pending = Vec::new();

        for (position, (index, signature)) in pending.into_iter().enumerate() {
            let status = values.get(position).cloned().unwrap_or(Value::Null);
            if status.is_null() {
                next_pending.push((index, signature));
                continue;
            }
            let actual_commitment = status
                .get("confirmationStatus")
                .and_then(Value::as_str)
                .unwrap_or("processed");
            first_observed_status
                .entry(index)
                .or_insert_with(|| actual_commitment.to_string());
            if let Some(slot) = status.get("slot").and_then(Value::as_u64) {
                first_observed_slot.entry(index).or_insert(slot);
            }
            first_observed_at_ms
                .entry(index)
                .or_insert_with(current_time_ms);
            if status.get("err").is_some() && !status.get("err").unwrap_or(&Value::Null).is_null() {
                return Err(format!(
                    "Transaction {} failed on-chain: {}",
                    signature,
                    status.get("err").cloned().unwrap_or(Value::Null)
                ));
            }
            if commitment_satisfied(actual_commitment, commitment) {
                completed.push((
                    index,
                    ConfirmationDetails {
                        status,
                        confirmed_observed_block_height: None,
                        confirmed_slot: first_observed_slot.get(&index).copied(),
                        confirmation_source: "rpc-polling-batch",
                        first_observed_status: first_observed_status.get(&index).cloned(),
                        first_observed_slot: first_observed_slot.get(&index).copied(),
                        first_observed_at_ms: first_observed_at_ms.get(&index).copied(),
                        confirmed_at_ms: Some(current_time_ms()),
                    },
                ));
            } else {
                next_pending.push((index, signature));
            }
        }

        pending = next_pending;
        if pending.is_empty() {
            return Ok(completed);
        }
        sleep(Duration::from_millis(poll_interval_ms)).await;
    }

    Err(format!(
        "Timed out waiting for transactions {} to reach {}.",
        pending
            .iter()
            .map(|(_, signature)| signature.clone())
            .collect::<Vec<_>>()
            .join(", "),
        commitment
    ))
}

async fn subscribe_signature_batch(
    ws: &mut WsStream,
    signatures: &[(usize, String)],
    commitment: &str,
) -> Result<HashMap<u64, usize>, String> {
    let mut subscription_map = HashMap::new();
    for (request_offset, (index, signature)) in signatures.iter().enumerate() {
        let request_id = request_offset as u64 + 1;
        ws.send(Message::Text(
            json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "signatureSubscribe",
                "params": [
                    signature,
                    {
                        "commitment": commitment
                    }
                ],
            })
            .to_string()
            .into(),
        ))
        .await
        .map_err(|error| error.to_string())?;
        loop {
            let payload = next_json_message(ws).await?;
            if payload.get("id").and_then(Value::as_u64) == Some(request_id) {
                if payload.get("error").is_some() {
                    return Err(format!("Subscription failed: {payload}"));
                }
                let subscription_id = payload
                    .get("result")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| format!("Subscription ack missing id: {payload}"))?;
                subscription_map.insert(subscription_id, *index);
                break;
            }
        }
    }
    Ok(subscription_map)
}

async fn wait_for_confirmations_websocket_batch(
    endpoint: &str,
    rpc_url: &str,
    signatures: &[(usize, String)],
    commitment: &str,
    track_confirmed_block_height: bool,
) -> Result<Vec<(usize, ConfirmationDetails)>, String> {
    let session = async {
        let mut ws = open_subscription_socket(endpoint).await?;
        let subscription_map = subscribe_signature_batch(&mut ws, signatures, commitment).await?;
        let mut pending = signatures
            .iter()
            .map(|(index, _)| *index)
            .collect::<HashSet<_>>();
        let mut confirmations = Vec::with_capacity(signatures.len());
        while !pending.is_empty() {
            let message = next_json_message(&mut ws).await?;
            let Some(params) = message.get("params") else {
                continue;
            };
            let Some(subscription_id) = params.get("subscription").and_then(Value::as_u64) else {
                continue;
            };
            let Some(index) = subscription_map.get(&subscription_id).copied() else {
                continue;
            };
            if !pending.remove(&index) {
                continue;
            }
            let context_slot = params
                .get("result")
                .and_then(|result| result.get("context"))
                .and_then(|context| context.get("slot"))
                .and_then(Value::as_u64);
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
            let confirmed_observed_block_height = if track_confirmed_block_height {
                fetch_sampled_block_height_snapshot(rpc_url, commitment)
                    .await
                    .ok()
            } else {
                None
            };
            let observed_at_ms = current_time_ms();
            confirmations.push((
                index,
                ConfirmationDetails {
                    status: json!({
                        "confirmationStatus": commitment,
                        "slot": context_slot,
                    }),
                    confirmed_observed_block_height,
                    confirmed_slot: context_slot,
                    confirmation_source: "websocket",
                    first_observed_status: Some(commitment.to_string()),
                    first_observed_slot: context_slot,
                    first_observed_at_ms: Some(observed_at_ms),
                    confirmed_at_ms: Some(observed_at_ms),
                },
            ));
        }
        Ok(confirmations)
    };
    timeout(Duration::from_secs(35), session)
        .await
        .map_err(|_| {
            format!(
                "Timed out waiting for websocket confirmation for {} transaction(s).",
                signatures.len()
            )
        })?
}

fn transport_watch_endpoint(transport_plan: &TransportPlan) -> Option<&str> {
    transport_plan
        .watchEndpoint
        .as_deref()
        .or_else(|| transport_plan.watchEndpoints.first().map(String::as_str))
}

pub async fn confirm_transactions_with_websocket_fallback(
    rpc_url: &str,
    watch_endpoint: Option<&str>,
    submitted: &mut [SentResult],
    commitment: &str,
    track_send_block_height: bool,
    poll_max_attempts: u32,
    poll_interval_ms: u64,
) -> Result<(Vec<String>, u128), String> {
    let confirm_started = std::time::Instant::now();
    let signatures = submitted
        .iter()
        .enumerate()
        .map(|(index, result)| {
            result
                .signature
                .clone()
                .map(|signature| (index, signature))
                .ok_or_else(|| {
                    format!(
                        "Submitted transaction {} is missing a signature.",
                        result.label
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut warnings = Vec::new();
    let confirmations = if let Some(endpoint) = watch_endpoint {
        match wait_for_confirmations_websocket_batch(
            endpoint,
            rpc_url,
            &signatures,
            commitment,
            false,
        )
        .await
        {
            Ok(confirmations) => confirmations,
            Err(error) => {
                warnings.push(format!(
                    "Websocket batch confirmation failed: {}. Falling back to batched RPC polling.",
                    error
                ));
                wait_for_confirmations_polling_batch(
                    rpc_url,
                    &signatures,
                    commitment,
                    poll_max_attempts,
                    poll_interval_ms,
                )
                .await?
            }
        }
    } else {
        wait_for_confirmations_polling_batch(
            rpc_url,
            &signatures,
            commitment,
            poll_max_attempts,
            poll_interval_ms,
        )
        .await?
    };
    let confirmed_observed_block_height = if track_send_block_height {
        fetch_sampled_block_height_snapshot(rpc_url, commitment)
            .await
            .ok()
    } else {
        None
    };
    for (index, confirmation) in confirmations {
        let result = &mut submitted[index];
        result.confirmationStatus = confirmation
            .status
            .get("confirmationStatus")
            .and_then(Value::as_str)
            .map(str::to_string);
        result.confirmationSource = Some(confirmation.confirmation_source.to_string());
        result.firstObservedStatus = confirmation.first_observed_status;
        result.firstObservedSlot = confirmation.first_observed_slot;
        result.firstObservedAtMs = confirmation.first_observed_at_ms;
        result.confirmedAtMs = confirmation.confirmed_at_ms;
        result.confirmedObservedBlockHeight =
            confirmed_observed_block_height.or(confirmation.confirmed_observed_block_height);
        result.confirmedSlot = confirmation.confirmed_slot;
    }
    Ok((warnings, confirm_started.elapsed().as_millis()))
}

pub async fn submit_transactions_sequential(
    rpc_url: &str,
    transactions: &[CompiledTransaction],
    commitment: &str,
    skip_preflight: bool,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    let mut results = Vec::new();
    let warnings = Vec::new();
    let submit_started = std::time::Instant::now();
    for transaction in transactions {
        let max_retries = 3;
        let signature = rpc_request(
            rpc_url,
            "sendTransaction",
            json!([
                transaction.serializedBase64,
                {
                    "encoding": "base64",
                    "skipPreflight": skip_preflight,
                    "preflightCommitment": commitment,
                    "maxRetries": max_retries,
                }
            ]),
        )
        .await?
        .as_str()
        .map(str::to_string)
        .or_else(|| transaction.signature.clone())
        .or_else(|| signature_from_serialized_base64(&transaction.serializedBase64))
        .ok_or_else(|| "RPC sendTransaction did not return a signature.".to_string())?;
        let send_observed_block_height = if track_send_block_height {
            fetch_sampled_block_height_snapshot(rpc_url, commitment)
                .await
                .ok()
        } else {
            None
        };
        results.push(SentResult {
            label: transaction.label.clone(),
            format: transaction.format.clone(),
            signature: Some(signature.clone()),
            explorerUrl: Some(format!("https://solscan.io/tx/{signature}")),
            transportType: "standard-rpc-sequential".to_string(),
            endpoint: Some(rpc_url.to_string()),
            attemptedEndpoints: vec![rpc_url.to_string()],
            skipPreflight: skip_preflight,
            maxRetries: max_retries,
            confirmationStatus: None,
            confirmationSource: None,
            submittedAtMs: Some(current_time_ms()),
            firstObservedStatus: None,
            firstObservedSlot: None,
            firstObservedAtMs: None,
            confirmedAtMs: None,
            sendObservedBlockHeight: send_observed_block_height,
            confirmedObservedBlockHeight: None,
            confirmedSlot: None,
            computeUnitLimit: transaction.computeUnitLimit,
            computeUnitPriceMicroLamports: transaction.computeUnitPriceMicroLamports,
            inlineTipLamports: transaction.inlineTipLamports,
            inlineTipAccount: transaction.inlineTipAccount.clone(),
            bundleId: None,
            attemptedBundleIds: vec![],
        });
    }
    Ok((results, warnings, submit_started.elapsed().as_millis()))
}

async fn submit_single_transaction_rpc(
    rpc_url: &str,
    transaction: &CompiledTransaction,
    commitment: &str,
    skip_preflight: bool,
    track_send_block_height: bool,
) -> Result<SentResult, String> {
    let max_retries = 3;
    let signature = rpc_request(
        rpc_url,
        "sendTransaction",
        json!([
            transaction.serializedBase64,
            {
                "encoding": "base64",
                "skipPreflight": skip_preflight,
                "preflightCommitment": commitment,
                "maxRetries": max_retries,
            }
        ]),
    )
    .await?
    .as_str()
    .map(str::to_string)
    .or_else(|| transaction.signature.clone())
    .or_else(|| signature_from_serialized_base64(&transaction.serializedBase64))
    .ok_or_else(|| "RPC sendTransaction did not return a signature.".to_string())?;
    let send_observed_block_height = if track_send_block_height {
        fetch_sampled_block_height_snapshot(rpc_url, commitment)
            .await
            .ok()
    } else {
        None
    };
    Ok(SentResult {
        label: transaction.label.clone(),
        format: transaction.format.clone(),
        signature: Some(signature.clone()),
        explorerUrl: Some(format!("https://solscan.io/tx/{signature}")),
        transportType: "standard-rpc-sequential".to_string(),
        endpoint: Some(rpc_url.to_string()),
        attemptedEndpoints: vec![rpc_url.to_string()],
        skipPreflight: skip_preflight,
        maxRetries: max_retries,
        confirmationStatus: None,
        confirmationSource: None,
        submittedAtMs: Some(current_time_ms()),
        firstObservedStatus: None,
        firstObservedSlot: None,
        firstObservedAtMs: None,
        confirmedAtMs: None,
        sendObservedBlockHeight: send_observed_block_height,
        confirmedObservedBlockHeight: None,
        confirmedSlot: None,
        computeUnitLimit: transaction.computeUnitLimit,
        computeUnitPriceMicroLamports: transaction.computeUnitPriceMicroLamports,
        inlineTipLamports: transaction.inlineTipLamports,
        inlineTipAccount: transaction.inlineTipAccount.clone(),
        bundleId: None,
        attemptedBundleIds: vec![],
    })
}

pub async fn submit_transactions_parallel(
    rpc_url: &str,
    transactions: &[CompiledTransaction],
    commitment: &str,
    skip_preflight: bool,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    let submit_started = std::time::Instant::now();
    let results = join_all(transactions.iter().map(|transaction| {
        submit_single_transaction_rpc(
            rpc_url,
            transaction,
            commitment,
            skip_preflight,
            track_send_block_height,
        )
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    Ok((results, vec![], submit_started.elapsed().as_millis()))
}

fn standard_rpc_submit_endpoints(primary_rpc_url: &str, extra_endpoints: &[String]) -> Vec<String> {
    let mut endpoints = vec![primary_rpc_url.to_string()];
    for endpoint in extra_endpoints {
        let trimmed = endpoint.trim();
        if trimmed.is_empty() || endpoints.iter().any(|existing| existing == trimmed) {
            continue;
        }
        endpoints.push(trimmed.to_string());
    }
    endpoints
}

async fn submit_single_transaction_standard_rpc_fanout(
    endpoints: &[String],
    transaction: &CompiledTransaction,
    commitment: &str,
    skip_preflight: bool,
    max_retries: u32,
) -> Result<(SentResult, Vec<String>), String> {
    if endpoints.is_empty() {
        return Err(format!(
            "Standard RPC submission failed for transaction {} because no endpoints were configured.",
            transaction.label
        ));
    }
    let mut endpoint_results = endpoints
        .iter()
        .cloned()
        .map(|endpoint| {
            let serialized = transaction.serializedBase64.clone();
            let commitment = commitment.to_string();
            async move {
                (
                    endpoint.clone(),
                    rpc_request(
                        &endpoint,
                        "sendTransaction",
                        json!([
                            serialized,
                            {
                                "encoding": "base64",
                                "skipPreflight": skip_preflight,
                                "preflightCommitment": commitment,
                                "maxRetries": max_retries,
                            }
                        ]),
                    )
                    .await,
                )
            }
        })
        .collect::<FuturesUnordered<_>>();
    let mut first_successful_endpoint = None;
    let mut successful_endpoints = Vec::new();
    let mut returned_signatures = Vec::new();
    let mut errors = Vec::new();
    while let Some((endpoint, result)) = endpoint_results.next().await {
        match result {
            Ok(value) => {
                if first_successful_endpoint.is_none() {
                    first_successful_endpoint = Some(endpoint.clone());
                }
                if let Some(signature) = value.as_str() {
                    returned_signatures.push(signature.to_string());
                }
                successful_endpoints.push(endpoint);
            }
            Err(error) => errors.push(format!("{endpoint}: {error}")),
        }
    }
    if successful_endpoints.is_empty() {
        return Err(format!(
            "Standard RPC fanout failed for transaction {} on all attempted endpoints: {}",
            transaction.label,
            errors.join(" | ")
        ));
    }
    let mut warnings = Vec::new();
    if !errors.is_empty() {
        warnings.push(format!(
            "Standard RPC fanout had partial failures for {}: {}",
            transaction.label,
            errors.join(" | ")
        ));
    }
    let signature = transaction
        .signature
        .clone()
        .or_else(|| signature_from_serialized_base64(&transaction.serializedBase64))
        .or_else(|| returned_signatures.first().cloned())
        .ok_or_else(|| {
            format!(
                "Standard RPC fanout did not return a signature for {}.",
                transaction.label
            )
        })?;
    Ok((
        SentResult {
            label: transaction.label.clone(),
            format: transaction.format.clone(),
            signature: Some(signature.clone()),
            explorerUrl: Some(format!("https://solscan.io/tx/{signature}")),
            transportType: "standard-rpc-fanout".to_string(),
            endpoint: first_successful_endpoint,
            attemptedEndpoints: endpoints.to_vec(),
            skipPreflight: skip_preflight,
            maxRetries: max_retries,
            confirmationStatus: None,
            confirmationSource: None,
            submittedAtMs: Some(current_time_ms()),
            firstObservedStatus: None,
            firstObservedSlot: None,
            firstObservedAtMs: None,
            confirmedAtMs: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            confirmedSlot: None,
            computeUnitLimit: transaction.computeUnitLimit,
            computeUnitPriceMicroLamports: transaction.computeUnitPriceMicroLamports,
            inlineTipLamports: transaction.inlineTipLamports,
            inlineTipAccount: transaction.inlineTipAccount.clone(),
            bundleId: None,
            attemptedBundleIds: vec![],
        },
        warnings,
    ))
}

pub async fn submit_transactions_standard_rpc_fanout(
    primary_rpc_url: &str,
    extra_endpoints: &[String],
    transactions: &[CompiledTransaction],
    commitment: &str,
    skip_preflight: bool,
    max_retries: u32,
    track_send_block_height: bool,
    parallelize_transactions: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    let endpoints = standard_rpc_submit_endpoints(primary_rpc_url, extra_endpoints);
    let submit_started = std::time::Instant::now();
    let mut warnings = Vec::new();
    let mut results = if parallelize_transactions {
        let entries = join_all(transactions.iter().map(|transaction| {
            submit_single_transaction_standard_rpc_fanout(
                &endpoints,
                transaction,
                commitment,
                skip_preflight,
                max_retries,
            )
        }))
        .await;
        let mut sent = Vec::with_capacity(entries.len());
        for entry in entries {
            let (result, entry_warnings) = entry?;
            sent.push(result);
            warnings.extend(entry_warnings);
        }
        sent
    } else {
        let mut sent = Vec::with_capacity(transactions.len());
        for transaction in transactions {
            let (result, entry_warnings) = submit_single_transaction_standard_rpc_fanout(
                &endpoints,
                transaction,
                commitment,
                skip_preflight,
                max_retries,
            )
            .await?;
            sent.push(result);
            warnings.extend(entry_warnings);
        }
        sent
    };
    if track_send_block_height {
        let send_observed_block_height =
            fetch_sampled_block_height_snapshot(primary_rpc_url, commitment)
                .await
                .ok();
        for result in &mut results {
            result.sendObservedBlockHeight = send_observed_block_height;
        }
    }
    Ok((results, warnings, submit_started.elapsed().as_millis()))
}

pub async fn confirm_transactions_sequential(
    rpc_url: &str,
    submitted: &mut [SentResult],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<u128, String> {
    confirm_transactions_sequential_with_attempts(
        rpc_url,
        submitted,
        commitment,
        track_send_block_height,
        20,
    )
    .await
}

pub async fn confirm_transactions_sequential_with_attempts(
    rpc_url: &str,
    submitted: &mut [SentResult],
    commitment: &str,
    track_send_block_height: bool,
    max_attempts: u32,
) -> Result<u128, String> {
    let confirm_started = std::time::Instant::now();
    for result in submitted {
        let signature = result.signature.clone().ok_or_else(|| {
            format!(
                "Submitted transaction {} is missing a signature.",
                result.label
            )
        })?;
        let confirmation = wait_for_confirmation(
            rpc_url,
            &signature,
            commitment,
            max_attempts,
            track_send_block_height,
        )
        .await?;
        result.confirmationStatus = confirmation
            .status
            .get("confirmationStatus")
            .and_then(Value::as_str)
            .map(str::to_string);
        result.confirmationSource = Some(confirmation.confirmation_source.to_string());
        result.firstObservedStatus = confirmation.first_observed_status;
        result.firstObservedSlot = confirmation.first_observed_slot;
        result.firstObservedAtMs = confirmation.first_observed_at_ms;
        result.confirmedAtMs = confirmation.confirmed_at_ms;
        result.confirmedObservedBlockHeight = confirmation.confirmed_observed_block_height;
        result.confirmedSlot = confirmation.confirmed_slot;
    }
    Ok(confirm_started.elapsed().as_millis())
}

pub async fn submit_transactions_helius_sender(
    rpc_url: &str,
    endpoints: &[String],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    let mut results = Vec::new();
    let mut warnings = Vec::new();
    if endpoints.is_empty() {
        return Err("Helius Sender endpoint is not configured.".to_string());
    }
    let submit_started = std::time::Instant::now();
    for transaction in transactions {
        let (entry, entry_warnings) =
            submit_single_transaction_helius_sender(endpoints, transaction).await?;
        results.push(entry);
        warnings.extend(entry_warnings);
    }
    if track_send_block_height {
        let send_observed_block_height = fetch_sampled_block_height_snapshot(rpc_url, commitment)
            .await
            .ok();
        for result in &mut results {
            result.sendObservedBlockHeight = send_observed_block_height;
        }
    }
    Ok((results, warnings, submit_started.elapsed().as_millis()))
}

async fn submit_single_transaction_helius_sender(
    endpoints: &[String],
    transaction: &CompiledTransaction,
) -> Result<(SentResult, Vec<String>), String> {
    validate_helius_sender_transaction(transaction)?;
    let mut endpoint_results = endpoints
        .iter()
        .cloned()
        .map(|endpoint| {
            let serialized = transaction.serializedBase64.clone();
            async move {
                (
                    endpoint.clone(),
                    rpc_request(
                        &endpoint,
                        "sendTransaction",
                        json!([
                            serialized,
                            {
                                "encoding": "base64",
                                "skipPreflight": true,
                                "maxRetries": 0,
                            }
                        ]),
                    )
                    .await,
                )
            }
        })
        .collect::<FuturesUnordered<_>>();
    let mut first_successful_endpoint = None;
    let mut successful_endpoints = Vec::new();
    let mut returned_signatures = Vec::new();
    let mut errors = Vec::new();
    while let Some((endpoint, result)) = endpoint_results.next().await {
        match result {
            Ok(value) => {
                if first_successful_endpoint.is_none() {
                    first_successful_endpoint = Some(endpoint.clone());
                }
                if let Some(signature) = value.as_str() {
                    returned_signatures.push(signature.to_string());
                }
                successful_endpoints.push(endpoint);
            }
            Err(error) => errors.push(format!("{endpoint}: {error}")),
        }
    }
    if successful_endpoints.is_empty() {
        return Err(format!(
            "Helius Sender failed for transaction {} on all attempted endpoints: {}",
            transaction.label,
            errors.join(" | ")
        ));
    }
    let mut warnings = Vec::new();
    if !errors.is_empty() {
        warnings.push(format!(
            "Helius Sender fanout had partial failures for {}: {}",
            transaction.label,
            errors.join(" | ")
        ));
    }
    let local_signature = transaction
        .signature
        .clone()
        .or_else(|| signature_from_serialized_base64(&transaction.serializedBase64));
    let signature = local_signature
        .or_else(|| returned_signatures.first().cloned())
        .ok_or_else(|| {
            format!(
                "Helius Sender did not return a signature for {}.",
                transaction.label
            )
        })?;
    Ok((
        SentResult {
            label: transaction.label.clone(),
            format: transaction.format.clone(),
            signature: Some(signature.clone()),
            explorerUrl: Some(format!("https://solscan.io/tx/{signature}")),
            transportType: "helius-sender".to_string(),
            endpoint: first_successful_endpoint,
            attemptedEndpoints: endpoints.to_vec(),
            skipPreflight: true,
            maxRetries: 0,
            confirmationStatus: None,
            confirmationSource: None,
            submittedAtMs: Some(current_time_ms()),
            firstObservedStatus: None,
            firstObservedSlot: None,
            firstObservedAtMs: None,
            confirmedAtMs: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            confirmedSlot: None,
            computeUnitLimit: transaction.computeUnitLimit,
            computeUnitPriceMicroLamports: transaction.computeUnitPriceMicroLamports,
            inlineTipLamports: transaction.inlineTipLamports,
            inlineTipAccount: transaction.inlineTipAccount.clone(),
            bundleId: None,
            attemptedBundleIds: vec![],
        },
        warnings,
    ))
}

pub async fn submit_transactions_helius_sender_parallel(
    rpc_url: &str,
    endpoints: &[String],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    if endpoints.is_empty() {
        return Err("Helius Sender endpoint is not configured.".to_string());
    }
    let submit_started = std::time::Instant::now();
    let results = join_all(
        transactions
            .iter()
            .map(|transaction| submit_single_transaction_helius_sender(endpoints, transaction)),
    )
    .await;
    let mut sent = Vec::with_capacity(results.len());
    let mut warnings = Vec::new();
    for result in results {
        let (entry, entry_warnings) = result?;
        sent.push(entry);
        warnings.extend(entry_warnings);
    }
    if track_send_block_height {
        let send_observed_block_height = fetch_sampled_block_height_snapshot(rpc_url, commitment)
            .await
            .ok();
        for result in &mut sent {
            result.sendObservedBlockHeight = send_observed_block_height;
        }
    }
    Ok((sent, warnings, submit_started.elapsed().as_millis()))
}

pub async fn submit_transactions_bundle(
    rpc_url: &str,
    endpoints: &[JitoBundleEndpoint],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    if transactions.is_empty() {
        return Ok((vec![], vec![], 0));
    }
    if transactions.len() > 5 {
        return Err(format!(
            "Jito bundles support at most 5 transactions. Got: {}",
            transactions.len()
        ));
    }
    if endpoints.is_empty() {
        return Err("No Jito bundle endpoints configured.".to_string());
    }
    let encoded: Vec<String> = transactions
        .iter()
        .map(|entry| entry.serializedBase64.clone())
        .collect();
    let local_signatures = transactions
        .iter()
        .map(|transaction| {
            transaction
                .signature
                .clone()
                .or_else(|| signature_from_serialized_base64(&transaction.serializedBase64))
        })
        .collect::<Vec<_>>();
    let mut attempts = Vec::new();
    let mut send_errors = Vec::new();
    let submit_started = std::time::Instant::now();
    for endpoint in endpoints {
        match jito_request(
            &endpoint.send,
            "sendBundle",
            json!([encoded, { "encoding": "base64" }]),
        )
        .await
        {
            Ok(result) => {
                let bundle_id = result
                    .as_str()
                    .ok_or_else(|| "Jito sendBundle did not return a bundle id.".to_string())?
                    .to_string();
                attempts.push((endpoint.clone(), bundle_id));
            }
            Err(error) => send_errors.push(format!("{}: {}", endpoint.name, error)),
        }
    }
    if attempts.is_empty() {
        return Err(format!(
            "Jito bundle submission failed for all endpoints in the selected profile: {}",
            send_errors.join(" | ")
        ));
    }
    let mut warnings = Vec::new();
    if !send_errors.is_empty() {
        warnings.push(format!(
            "Jito fanout had partial submission failures: {}",
            send_errors.join(" | ")
        ));
    }
    warnings.push(format!(
        "Jito fanout accepted bundle submissions: {}",
        attempts
            .iter()
            .map(|(endpoint, bundle_id)| format!("{}={}", endpoint.name, bundle_id))
            .collect::<Vec<_>>()
            .join(" | ")
    ));
    let attempted_endpoints = attempts
        .iter()
        .map(|(attempt_endpoint, _)| attempt_endpoint.send.clone())
        .collect::<Vec<_>>();
    let attempted_bundle_ids = attempts
        .iter()
        .map(|(_, attempt_bundle_id)| attempt_bundle_id.clone())
        .collect::<Vec<_>>();
    let send_observed_block_height = if track_send_block_height {
        fetch_sampled_block_height_snapshot(rpc_url, commitment)
            .await
            .ok()
    } else {
        None
    };
    let results = transactions
        .iter()
        .enumerate()
        .map(|(index, transaction)| SentResult {
            label: transaction.label.clone(),
            format: transaction.format.clone(),
            signature: local_signatures.get(index).cloned().flatten(),
            explorerUrl: local_signatures
                .get(index)
                .and_then(|signature| signature.as_ref())
                .map(|signature| format!("https://solscan.io/tx/{signature}")),
            transportType: "jito-bundle".to_string(),
            endpoint: attempts.first().map(|(endpoint, _)| endpoint.send.clone()),
            attemptedEndpoints: attempted_endpoints.clone(),
            skipPreflight: true,
            maxRetries: 0,
            confirmationStatus: None,
            confirmationSource: None,
            submittedAtMs: Some(current_time_ms()),
            firstObservedStatus: None,
            firstObservedSlot: None,
            firstObservedAtMs: None,
            confirmedAtMs: None,
            sendObservedBlockHeight: send_observed_block_height,
            confirmedObservedBlockHeight: None,
            confirmedSlot: None,
            computeUnitLimit: transaction.computeUnitLimit,
            computeUnitPriceMicroLamports: transaction.computeUnitPriceMicroLamports,
            inlineTipLamports: transaction.inlineTipLamports,
            inlineTipAccount: transaction.inlineTipAccount.clone(),
            bundleId: attempts.first().map(|(_, bundle_id)| bundle_id.clone()),
            attemptedBundleIds: attempted_bundle_ids.clone(),
        })
        .collect::<Vec<_>>();
    Ok((results, warnings, submit_started.elapsed().as_millis()))
}

pub async fn confirm_transactions_bundle(
    rpc_url: &str,
    endpoints: &[JitoBundleEndpoint],
    submitted: &mut [SentResult],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<(Vec<String>, u128), String> {
    if submitted.is_empty() {
        return Ok((vec![], 0));
    }
    let attempted_bundle_ids = submitted[0].attemptedBundleIds.clone();
    if attempted_bundle_ids.is_empty() {
        return Err("Submitted Jito bundle results are missing bundle ids.".to_string());
    }
    let attempts = endpoints
        .iter()
        .filter_map(|endpoint| {
            let bundle_id = submitted[0]
                .attemptedEndpoints
                .iter()
                .position(|attempted| attempted == &endpoint.send)
                .and_then(|index| attempted_bundle_ids.get(index))
                .cloned()?;
            Some((endpoint.clone(), bundle_id))
        })
        .collect::<Vec<_>>();
    let confirm_started = std::time::Instant::now();
    let accepted_attempts = attempts
        .iter()
        .map(|(endpoint, bundle_id)| format!("{}={}", endpoint.name, bundle_id))
        .collect::<Vec<_>>();
    let mut last_observed_bundle_statuses = Vec::new();
    for _ in 0..20 {
        let mut observed_errors = Vec::new();
        let mut observed_bundle_statuses = Vec::new();
        for (endpoint, bundle_id) in &attempts {
            let status_payload = match jito_request(
                &endpoint.status,
                "getBundleStatuses",
                json!([[bundle_id.clone()]]),
            )
            .await
            {
                Ok(payload) => payload,
                Err(error) => {
                    observed_errors.push(format!(
                        "{}:{} status-request={}",
                        endpoint.name, bundle_id, error
                    ));
                    observed_bundle_statuses.push(format!(
                        "{}:{}=status-request-failed",
                        endpoint.name, bundle_id
                    ));
                    continue;
                }
            };
            let maybe_status = status_payload
                .get("value")
                .and_then(Value::as_array)
                .and_then(|items| {
                    items.iter().find(|entry| {
                        entry.get("bundle_id").and_then(Value::as_str) == Some(bundle_id.as_str())
                    })
                })
                .cloned();
            if let Some(status) = maybe_status {
                if let Some(err) = status.get("err") {
                    if !err.is_null() {
                        observed_errors.push(format!("{}:{}={}", endpoint.name, bundle_id, err));
                        observed_bundle_statuses
                            .push(format!("{}:{}=err:{}", endpoint.name, bundle_id, err));
                        continue;
                    }
                }
                let actual = status
                    .get("confirmation_status")
                    .and_then(Value::as_str)
                    .unwrap_or("processed");
                observed_bundle_statuses
                    .push(format!("{}:{}={}", endpoint.name, bundle_id, actual));
                if !commitment_satisfied(actual, commitment) {
                    continue;
                }
                let signatures = status
                    .get("transactions")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let confirmed_observed_block_height = if track_send_block_height {
                    fetch_sampled_block_height_snapshot(rpc_url, commitment)
                        .await
                        .ok()
                } else {
                    None
                };
                let confirmed_slot = status.get("slot").and_then(Value::as_u64);
                for (index, result) in submitted.iter_mut().enumerate() {
                    let signature = signatures
                        .get(index)
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or_else(|| result.signature.clone());
                    result.signature = signature.clone();
                    result.explorerUrl = signature
                        .as_ref()
                        .map(|value| format!("https://solscan.io/tx/{value}"));
                    result.confirmationStatus = Some(actual.to_string());
                    result.confirmationSource = Some("jito-bundle-status".to_string());
                    result.firstObservedStatus = Some(actual.to_string());
                    result.firstObservedSlot = confirmed_slot;
                    result.firstObservedAtMs = Some(current_time_ms());
                    result.confirmedAtMs = result.firstObservedAtMs;
                    result.confirmedObservedBlockHeight = confirmed_observed_block_height;
                    result.confirmedSlot = confirmed_slot;
                    result.bundleId = Some(bundle_id.clone());
                    result.endpoint = Some(endpoint.send.clone());
                }
                let mut warnings = vec![format!(
                    "Sent via Jito bundle {} using {} after fanout to {} endpoints.",
                    bundle_id,
                    endpoint.name,
                    endpoints.len()
                )];
                if !observed_errors.is_empty() {
                    warnings.push(format!(
                        "Jito fanout observed non-winning endpoint errors: {}",
                        observed_errors.join(" | ")
                    ));
                }
                return Ok((warnings, confirm_started.elapsed().as_millis()));
            } else {
                observed_bundle_statuses.push(format!("{}:{}=not-found", endpoint.name, bundle_id));
            }
        }
        if !observed_bundle_statuses.is_empty() {
            last_observed_bundle_statuses = observed_bundle_statuses;
        }
        sleep(Duration::from_millis(1500)).await;
    }
    Err(format!(
        "Timed out waiting for fanout Jito bundle submissions to reach {}. Accepted endpoints: {}. Bundle ids: {}. Last observed bundle statuses: {}",
        commitment,
        accepted_attempts.join(" | "),
        attempted_bundle_ids.join(", "),
        if last_observed_bundle_statuses.is_empty() {
            "none".to_string()
        } else {
            last_observed_bundle_statuses.join(" | ")
        }
    ))
}

pub async fn confirm_transactions_helius_sender(
    rpc_url: &str,
    submitted: &mut [SentResult],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<u128, String> {
    confirm_transactions_sequential(rpc_url, submitted, commitment, track_send_block_height).await
}

pub async fn send_transactions_sequential(
    rpc_url: &str,
    transactions: &[CompiledTransaction],
    commitment: &str,
    skip_preflight: bool,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, SendTimingBreakdown), String> {
    let (mut results, warnings, submit_ms) = submit_transactions_sequential(
        rpc_url,
        transactions,
        commitment,
        skip_preflight,
        track_send_block_height,
    )
    .await?;
    let confirm_ms =
        confirm_transactions_sequential(rpc_url, &mut results, commitment, track_send_block_height)
            .await?;
    Ok((
        results,
        warnings,
        SendTimingBreakdown {
            submit_ms,
            confirm_ms,
        },
    ))
}

fn validate_helius_sender_transaction(transaction: &CompiledTransaction) -> Result<(), String> {
    if transaction.inlineTipLamports.unwrap_or(0) < 200_000 {
        return Err(format!(
            "Transaction {} is missing the required inline Helius Sender tip.",
            transaction.label
        ));
    }
    if transaction.computeUnitPriceMicroLamports.unwrap_or(0) == 0 {
        return Err(format!(
            "Transaction {} is missing the required compute unit price for Helius Sender.",
            transaction.label
        ));
    }
    Ok(())
}

fn validate_hellomoon_transaction(transaction: &CompiledTransaction) -> Result<(), String> {
    if transaction.inlineTipLamports.unwrap_or(0) < 1_000_000 {
        return Err(format!(
            "Transaction {} is missing the required inline Hello Moon tip (minimum 0.001 SOL).",
            transaction.label
        ));
    }
    if transaction.computeUnitPriceMicroLamports.unwrap_or(0) == 0 {
        return Err(format!(
            "Transaction {} is missing the required compute unit price for Hello Moon QUIC.",
            transaction.label
        ));
    }
    Ok(())
}

fn validate_hellomoon_bundle_transactions(
    transactions: &[CompiledTransaction],
) -> Result<(), String> {
    if transactions.is_empty() {
        return Ok(());
    }
    for transaction in transactions {
        if transaction.computeUnitPriceMicroLamports.unwrap_or(0) == 0 {
            return Err(format!(
                "Transaction {} is missing the required compute unit price for Hello Moon bundles.",
                transaction.label
            ));
        }
    }
    let mut has_valid_tip = false;
    for transaction in transactions {
        let tip_lamports = transaction.inlineTipLamports.unwrap_or(0);
        if tip_lamports >= 1_000_000 {
            has_valid_tip = true;
        }
    }
    if !has_valid_tip {
        return Err(
            "Hello Moon bundles require at least one inline Hello Moon tip (minimum 0.001 SOL) somewhere in the bundle."
                .to_string(),
        );
    }
    Ok(())
}

fn normalize_hellomoon_bundle_error(status: StatusCode, detail: &str) -> String {
    let normalized = detail.trim();
    if (status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN)
        && normalized
            .to_ascii_lowercase()
            .contains("api key missing required scope")
    {
        return format!(
            "status {}: Hello Moon recognized the API key, but sendBundle is not enabled for it (api key missing required scope).",
            status
        );
    }
    format!("status {}: {}", status, normalized)
}

async fn hellomoon_bundle_request(
    endpoint: &str,
    api_key: &str,
    transactions: &[String],
) -> Result<Vec<String>, String> {
    crate::observability::record_outbound_provider_http_request();
    let response = shared_http_client()
        .post(format!("{endpoint}?api-key={api_key}"))
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [
                transactions,
                {
                    "encoding": "base64",
                }
            ],
        }))
        .send()
        .await
        .map_err(|error| format!("Hello Moon bundle request failed for {endpoint}: {error}"))?;
    let status = response.status();
    let payload: Value = response.json().await.map_err(|error| {
        format!("Hello Moon bundle response decode failed for {endpoint}: {error}")
    })?;
    if !status.is_success() {
        let detail = payload
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .or_else(|| payload.get("message").and_then(Value::as_str))
            .unwrap_or("Hello Moon bundle request failed.");
        return Err(normalize_hellomoon_bundle_error(status, detail));
    }
    if let Some(error) = payload.get("error") {
        return Err(error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Hello Moon bundle request failed.")
            .to_string());
    }
    payload
        .get("result")
        .and_then(Value::as_array)
        .ok_or_else(|| "Hello Moon sendBundle did not return a signature array.".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "Hello Moon sendBundle returned a non-string signature.".to_string())
        })
        .collect()
}

pub async fn submit_transactions_hellomoon_bundle(
    rpc_url: &str,
    endpoints: &[String],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    if transactions.is_empty() {
        return Ok((vec![], vec![], 0));
    }
    if transactions.len() > 4 {
        return Err(format!(
            "Hello Moon bundles support at most 4 transactions. Got: {}",
            transactions.len()
        ));
    }
    if endpoints.is_empty() {
        return Err("Hello Moon bundle endpoint is not configured.".to_string());
    }
    validate_hellomoon_bundle_transactions(transactions)?;
    let api_key = configured_hellomoon_api_key();
    if api_key.trim().is_empty() {
        return Err(
            "Hello Moon bundle transport requires HELLOMOON_API_KEY.".to_string(),
        );
    }
    let encoded = transactions
        .iter()
        .map(|entry| entry.serializedBase64.clone())
        .collect::<Vec<_>>();
    let local_signatures = transactions
        .iter()
        .map(|transaction| {
            transaction
                .signature
                .clone()
                .or_else(|| signature_from_serialized_base64(&transaction.serializedBase64))
        })
        .collect::<Vec<_>>();
    let submit_started = std::time::Instant::now();
    let mut first_successful_endpoint = None;
    let mut successful_endpoints = Vec::new();
    let mut returned_signatures = Vec::new();
    let mut errors = Vec::new();
    let mut endpoint_results = endpoints
        .iter()
        .cloned()
        .map(|endpoint| {
            let api_key = api_key.clone();
            let encoded = encoded.clone();
            async move {
                (
                    endpoint.clone(),
                    hellomoon_bundle_request(&endpoint, &api_key, &encoded).await,
                )
            }
        })
        .collect::<FuturesUnordered<_>>();
    while let Some((endpoint, result)) = endpoint_results.next().await {
        match result {
            Ok(signatures) => {
                if first_successful_endpoint.is_none() {
                    first_successful_endpoint = Some(endpoint.clone());
                    returned_signatures = signatures;
                }
                successful_endpoints.push(endpoint);
            }
            Err(error) => errors.push(format!("{endpoint}: {error}")),
        }
    }
    if successful_endpoints.is_empty() {
        return Err(format!(
            "Hello Moon bundle submission failed on all attempted endpoints: {}",
            errors.join(" | ")
        ));
    }
    let mut warnings = Vec::new();
    if !errors.is_empty() {
        warnings.push(format!(
            "Hello Moon bundle fanout had partial submission failures: {}",
            errors.join(" | ")
        ));
    }
    let send_observed_block_height = if track_send_block_height {
        fetch_sampled_block_height_snapshot(rpc_url, commitment)
            .await
            .ok()
    } else {
        None
    };
    let results = transactions
        .iter()
        .enumerate()
        .map(|(index, transaction)| {
            let signature = local_signatures
                .get(index)
                .cloned()
                .flatten()
                .or_else(|| returned_signatures.get(index).cloned());
            SentResult {
                label: transaction.label.clone(),
                format: transaction.format.clone(),
                explorerUrl: signature
                    .as_ref()
                    .map(|value| format!("https://solscan.io/tx/{value}")),
                signature,
                transportType: "hellomoon-bundle".to_string(),
                endpoint: first_successful_endpoint.clone(),
                attemptedEndpoints: successful_endpoints.clone(),
                skipPreflight: true,
                maxRetries: 0,
                confirmationStatus: None,
                confirmationSource: None,
                submittedAtMs: Some(current_time_ms()),
                firstObservedStatus: None,
                firstObservedSlot: None,
                firstObservedAtMs: None,
                confirmedAtMs: None,
                sendObservedBlockHeight: send_observed_block_height,
                confirmedObservedBlockHeight: None,
                confirmedSlot: None,
                computeUnitLimit: transaction.computeUnitLimit,
                computeUnitPriceMicroLamports: transaction.computeUnitPriceMicroLamports,
                inlineTipLamports: transaction.inlineTipLamports,
                inlineTipAccount: transaction.inlineTipAccount.clone(),
                bundleId: None,
                attemptedBundleIds: vec![],
            }
        })
        .collect::<Vec<_>>();
    Ok((results, warnings, submit_started.elapsed().as_millis()))
}

pub async fn prewarm_hellomoon_quic_endpoint(
    endpoint: &str,
    mev_protect: bool,
) -> Result<(), String> {
    let api_key = configured_hellomoon_api_key();
    if api_key.trim().is_empty() {
        return Err(
            "Hello Moon QUIC prewarm requires HELLOMOON_API_KEY.".to_string(),
        );
    }
    cached_hellomoon_quic_client(endpoint, &api_key, mev_protect)
        .await
        .map(|_| ())
}

pub async fn prewarm_hellomoon_bundle_endpoint(endpoint: &str) -> Result<(), String> {
    let api_key = configured_hellomoon_api_key();
    if api_key.trim().is_empty() {
        return Err("Hello Moon bundle prewarm requires HELLOMOON_API_KEY.".to_string());
    }
    crate::observability::record_outbound_provider_http_request();
    let response = shared_http_client()
        .get(format!("{endpoint}?api-key={api_key}"))
        .send()
        .await
        .map_err(|error| format!("Hello Moon bundle prewarm failed for {endpoint}: {error}"))?;
    let status = response.status();
    let _ = response.bytes().await.map_err(|error| {
        format!("Hello Moon bundle prewarm body read failed for {endpoint}: {error}")
    })?;
    if status.is_server_error() {
        return Err(format!(
            "Hello Moon bundle prewarm failed for {endpoint}: status {}",
            status
        ));
    }
    Ok(())
}

async fn send_transaction_hellomoon_quic_endpoint(
    endpoint: &str,
    api_key: &str,
    mev_protect: bool,
    payload: &[u8],
) -> Result<(), String> {
    let client = cached_hellomoon_quic_client(endpoint, api_key, mev_protect).await?;
    match client.send_transaction(payload).await {
        Ok(()) => Ok(()),
        Err(first_error) => {
            invalidate_hellomoon_quic_client(endpoint, api_key, mev_protect).await;
            let client = cached_hellomoon_quic_client(endpoint, api_key, mev_protect).await?;
            client.send_transaction(payload).await.map_err(|second_error| {
                format!(
                    "Hello Moon QUIC send failed on {endpoint}: {first_error}; reconnect retry failed: {second_error}"
                )
            })
        }
    }
}

async fn submit_single_transaction_hellomoon_quic(
    endpoints: &[String],
    transaction: &CompiledTransaction,
    mev_protect: bool,
) -> Result<(SentResult, Vec<String>), String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

    validate_hellomoon_transaction(transaction)?;
    let api_key = configured_hellomoon_api_key();
    if api_key.trim().is_empty() {
        return Err(
            "Hello Moon QUIC requires HELLOMOON_API_KEY.".to_string(),
        );
    }
    let payload = BASE64
        .decode(&transaction.serializedBase64)
        .map_err(|error| {
            format!(
                "Failed to decode {} for Hello Moon QUIC: {error}",
                transaction.label
            )
        })?;
    let mut endpoint_results = endpoints
        .iter()
        .cloned()
        .map(|endpoint| {
            let api_key = api_key.clone();
            let payload = payload.clone();
            async move {
                (
                    endpoint.clone(),
                    send_transaction_hellomoon_quic_endpoint(
                        &endpoint,
                        &api_key,
                        mev_protect,
                        &payload,
                    )
                    .await,
                )
            }
        })
        .collect::<FuturesUnordered<_>>();
    let mut first_successful_endpoint = None;
    let mut successful_endpoints = Vec::new();
    let mut errors = Vec::new();
    while let Some((endpoint, result)) = endpoint_results.next().await {
        match result {
            Ok(()) => {
                if first_successful_endpoint.is_none() {
                    first_successful_endpoint = Some(endpoint.clone());
                }
                successful_endpoints.push(endpoint);
            }
            Err(error) => errors.push(format!("{endpoint}: {error}")),
        }
    }
    if successful_endpoints.is_empty() {
        return Err(format!(
            "Hello Moon QUIC failed for transaction {} on all attempted endpoints: {}",
            transaction.label,
            errors.join(" | ")
        ));
    }
    let mut warnings = Vec::new();
    if !errors.is_empty() {
        warnings.push(format!(
            "Hello Moon QUIC fanout had partial failures for {}: {}",
            transaction.label,
            errors.join(" | ")
        ));
    }
    let signature = transaction
        .signature
        .clone()
        .or_else(|| signature_from_serialized_base64(&transaction.serializedBase64))
        .ok_or_else(|| {
            format!(
                "Hello Moon QUIC could not derive a signature for {}.",
                transaction.label
            )
        })?;
    Ok((
        SentResult {
            label: transaction.label.clone(),
            format: transaction.format.clone(),
            signature: Some(signature.clone()),
            explorerUrl: Some(format!("https://solscan.io/tx/{signature}")),
            transportType: "hellomoon-quic".to_string(),
            endpoint: first_successful_endpoint,
            attemptedEndpoints: endpoints.to_vec(),
            skipPreflight: true,
            maxRetries: 0,
            confirmationStatus: None,
            confirmationSource: None,
            submittedAtMs: Some(current_time_ms()),
            firstObservedStatus: None,
            firstObservedSlot: None,
            firstObservedAtMs: None,
            confirmedAtMs: None,
            sendObservedBlockHeight: None,
            confirmedObservedBlockHeight: None,
            confirmedSlot: None,
            computeUnitLimit: transaction.computeUnitLimit,
            computeUnitPriceMicroLamports: transaction.computeUnitPriceMicroLamports,
            inlineTipLamports: transaction.inlineTipLamports,
            inlineTipAccount: transaction.inlineTipAccount.clone(),
            bundleId: None,
            attemptedBundleIds: vec![],
        },
        warnings,
    ))
}

pub async fn submit_transactions_hellomoon_quic(
    rpc_url: &str,
    endpoints: &[String],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
    mev_protect: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    if endpoints.is_empty() {
        return Err("Hello Moon QUIC endpoint is not configured.".to_string());
    }
    let submit_started = std::time::Instant::now();
    let mut results = Vec::with_capacity(transactions.len());
    let mut warnings = Vec::new();
    for transaction in transactions {
        let (entry, entry_warnings) =
            submit_single_transaction_hellomoon_quic(endpoints, transaction, mev_protect).await?;
        results.push(entry);
        warnings.extend(entry_warnings);
    }
    if track_send_block_height {
        let send_observed_block_height = fetch_sampled_block_height_snapshot(rpc_url, commitment)
            .await
            .ok();
        for result in &mut results {
            result.sendObservedBlockHeight = send_observed_block_height;
        }
    }
    Ok((results, warnings, submit_started.elapsed().as_millis()))
}

pub async fn submit_transactions_hellomoon_quic_parallel(
    rpc_url: &str,
    endpoints: &[String],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
    mev_protect: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    if endpoints.is_empty() {
        return Err("Hello Moon QUIC endpoint is not configured.".to_string());
    }
    let submit_started = std::time::Instant::now();
    let results = join_all(transactions.iter().map(|transaction| {
        submit_single_transaction_hellomoon_quic(endpoints, transaction, mev_protect)
    }))
    .await;
    let mut sent = Vec::with_capacity(results.len());
    let mut warnings = Vec::new();
    for result in results {
        let (entry, entry_warnings) = result?;
        sent.push(entry);
        warnings.extend(entry_warnings);
    }
    if track_send_block_height {
        let send_observed_block_height = fetch_sampled_block_height_snapshot(rpc_url, commitment)
            .await
            .ok();
        for result in &mut sent {
            result.sendObservedBlockHeight = send_observed_block_height;
        }
    }
    Ok((sent, warnings, submit_started.elapsed().as_millis()))
}

pub async fn send_transactions_hellomoon_quic(
    rpc_url: &str,
    endpoints: &[String],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
    mev_protect: bool,
) -> Result<(Vec<SentResult>, Vec<String>, SendTimingBreakdown), String> {
    if transactions.len() > 1 {
        let started = std::time::Instant::now();
        let mut results = Vec::with_capacity(transactions.len());
        let mut warnings = Vec::new();
        let mut submit_ms = 0u128;
        let mut confirm_ms = 0u128;
        for transaction in transactions {
            let submit_started = std::time::Instant::now();
            let (mut sent, entry_warnings) =
                submit_single_transaction_hellomoon_quic(endpoints, transaction, mev_protect)
                    .await?;
            if track_send_block_height {
                sent.sendObservedBlockHeight =
                    fetch_sampled_block_height_snapshot(rpc_url, commitment)
                        .await
                        .ok();
            }
            submit_ms += submit_started.elapsed().as_millis();
            warnings.extend(entry_warnings);
            let mut submitted = vec![sent];
            let confirm_started = std::time::Instant::now();
            confirm_transactions_with_websocket_fallback(
                rpc_url,
                None,
                &mut submitted,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await?;
            confirm_ms += confirm_started.elapsed().as_millis();
            results.extend(submitted);
        }
        let elapsed_ms = started.elapsed().as_millis();
        if submit_ms + confirm_ms < elapsed_ms {
            confirm_ms += elapsed_ms - (submit_ms + confirm_ms);
        }
        return Ok((
            results,
            warnings,
            SendTimingBreakdown {
                submit_ms,
                confirm_ms,
            },
        ));
    }
    let (mut results, warnings, submit_ms) = submit_transactions_hellomoon_quic(
        rpc_url,
        endpoints,
        transactions,
        commitment,
        track_send_block_height,
        mev_protect,
    )
    .await?;
    let (confirm_warnings, confirm_ms) = confirm_transactions_with_websocket_fallback(
        rpc_url,
        None,
        &mut results,
        commitment,
        track_send_block_height,
        75,
        400,
    )
    .await?;
    let mut combined_warnings = warnings;
    combined_warnings.extend(confirm_warnings);
    Ok((
        results,
        combined_warnings,
        SendTimingBreakdown {
            submit_ms,
            confirm_ms,
        },
    ))
}

pub async fn send_transactions_helius_sender(
    rpc_url: &str,
    endpoints: &[String],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, SendTimingBreakdown), String> {
    if transactions.len() > 1 {
        let started = std::time::Instant::now();
        let mut results = Vec::with_capacity(transactions.len());
        let mut warnings = Vec::new();
        let mut submit_ms = 0u128;
        let mut confirm_ms = 0u128;
        for transaction in transactions {
            let submit_started = std::time::Instant::now();
            let (mut sent, entry_warnings) =
                submit_single_transaction_helius_sender(endpoints, transaction).await?;
            if track_send_block_height {
                sent.sendObservedBlockHeight =
                    fetch_sampled_block_height_snapshot(rpc_url, commitment)
                        .await
                        .ok();
            }
            submit_ms += submit_started.elapsed().as_millis();
            warnings.extend(entry_warnings);
            let mut submitted = vec![sent];
            let confirm_started = std::time::Instant::now();
            confirm_transactions_helius_sender(
                rpc_url,
                &mut submitted,
                commitment,
                track_send_block_height,
            )
            .await?;
            confirm_ms += confirm_started.elapsed().as_millis();
            results.extend(submitted);
        }
        let elapsed_ms = started.elapsed().as_millis();
        if submit_ms + confirm_ms < elapsed_ms {
            confirm_ms += elapsed_ms - (submit_ms + confirm_ms);
        }
        return Ok((
            results,
            warnings,
            SendTimingBreakdown {
                submit_ms,
                confirm_ms,
            },
        ));
    }
    let (mut results, warnings, submit_ms) = submit_transactions_helius_sender(
        rpc_url,
        endpoints,
        transactions,
        commitment,
        track_send_block_height,
    )
    .await?;
    let confirm_ms = confirm_transactions_helius_sender(
        rpc_url,
        &mut results,
        commitment,
        track_send_block_height,
    )
    .await?;
    Ok((
        results,
        warnings,
        SendTimingBreakdown {
            submit_ms,
            confirm_ms,
        },
    ))
}

async fn jito_request(endpoint: &str, method: &str, params: Value) -> Result<Value, String> {
    let response = shared_http_client()
        .post(endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let payload: Value = response.json().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(payload
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .unwrap_or(&format!(
                "Jito bundle request failed with status {}.",
                status
            ))
            .to_string());
    }
    if let Some(error) = payload.get("error") {
        return Err(error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Jito request failed.")
            .to_string());
    }
    Ok(payload.get("result").cloned().unwrap_or(Value::Null))
}

fn jito_tip_accounts_endpoint(endpoint: &JitoBundleEndpoint) -> String {
    fn derive_base(url: &str) -> Option<String> {
        let trimmed = url.trim().trim_end_matches('/');
        for suffix in ["/api/v1/bundles", "/api/v1/getBundleStatuses"] {
            if let Some(prefix) = trimmed.strip_suffix(suffix) {
                return Some(prefix.to_string());
            }
        }
        None
    }

    let base = derive_base(&endpoint.send)
        .or_else(|| derive_base(&endpoint.status))
        .unwrap_or_else(|| endpoint.send.trim().trim_end_matches('/').to_string());
    format!("{base}/api/v1/getTipAccounts")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JitoWarmResult {
    Warmed,
    RateLimited(String),
}

fn jito_error_message_from_body(body: &str, fallback: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|payload| {
            payload
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .or_else(|| {
            let trimmed = body.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn jito_error_is_rate_limited(status: StatusCode, body: &str) -> bool {
    if status == StatusCode::TOO_MANY_REQUESTS {
        return true;
    }
    let lower = body.to_ascii_lowercase();
    lower.contains("rate limit") || lower.contains("rate-limited")
}

pub async fn prewarm_jito_bundle_endpoint(
    endpoint: &JitoBundleEndpoint,
) -> Result<JitoWarmResult, String> {
    let tip_accounts_endpoint = jito_tip_accounts_endpoint(endpoint);
    let response = shared_http_client()
        .post(&tip_accounts_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTipAccounts",
            "params": [],
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let body = response.text().await.map_err(|error| error.to_string())?;
    if jito_error_is_rate_limited(status, &body) {
        return Ok(JitoWarmResult::RateLimited(jito_error_message_from_body(
            &body,
            "Jito endpoint is reachable but rate-limited.",
        )));
    }
    if !status.is_success() {
        return Err(jito_error_message_from_body(
            &body,
            &format!("Jito bundle request failed with status {}.", status),
        ));
    }
    let payload: Value = serde_json::from_str(&body).map_err(|error| error.to_string())?;
    if let Some(error) = payload.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Jito request failed.")
            .to_string();
        if jito_error_is_rate_limited(status, &message) {
            return Ok(JitoWarmResult::RateLimited(message));
        }
        return Err(message);
    }
    Ok(JitoWarmResult::Warmed)
}

pub async fn send_transactions_bundle(
    rpc_url: &str,
    endpoints: &[JitoBundleEndpoint],
    transactions: &[CompiledTransaction],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, SendTimingBreakdown), String> {
    let (mut results, mut warnings, submit_ms) = submit_transactions_bundle(
        rpc_url,
        endpoints,
        transactions,
        commitment,
        track_send_block_height,
    )
    .await?;
    let (confirm_warnings, confirm_ms) = confirm_transactions_bundle(
        rpc_url,
        endpoints,
        &mut results,
        commitment,
        track_send_block_height,
    )
    .await?;
    warnings.extend(confirm_warnings);
    Ok((
        results,
        warnings,
        SendTimingBreakdown {
            submit_ms,
            confirm_ms,
        },
    ))
}

pub async fn send_transactions_for_transport(
    rpc_url: &str,
    transport_plan: &TransportPlan,
    transactions: &[CompiledTransaction],
    commitment: &str,
    skip_preflight: bool,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, SendTimingBreakdown), String> {
    match transport_plan.transportType.as_str() {
        "jito-bundle" => {
            send_transactions_bundle(
                rpc_url,
                &transport_plan.jitoBundleEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await
        }
        "hellomoon-bundle" => {
            let (mut results, mut warnings, submit_ms) = submit_transactions_hellomoon_bundle(
                rpc_url,
                &transport_plan.helloMoonBundleEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await?;
            let (confirm_warnings, confirm_ms) = confirm_transactions_with_websocket_fallback(
                rpc_url,
                transport_watch_endpoint(transport_plan),
                &mut results,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await?;
            warnings.extend(confirm_warnings);
            Ok((
                results,
                warnings,
                SendTimingBreakdown {
                    submit_ms,
                    confirm_ms,
                },
            ))
        }
        "hellomoon-quic" => {
            let (mut results, mut warnings, submit_ms) = submit_transactions_hellomoon_quic(
                rpc_url,
                &transport_plan.helloMoonQuicEndpoints,
                transactions,
                commitment,
                track_send_block_height,
                transport_plan.helloMoonMevProtect,
            )
            .await?;
            let (confirm_warnings, confirm_ms) = confirm_transactions_with_websocket_fallback(
                rpc_url,
                transport_watch_endpoint(transport_plan),
                &mut results,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await?;
            warnings.extend(confirm_warnings);
            Ok((
                results,
                warnings,
                SendTimingBreakdown {
                    submit_ms,
                    confirm_ms,
                },
            ))
        }
        "helius-sender" => {
            let (mut results, mut warnings, submit_ms) = submit_transactions_helius_sender(
                rpc_url,
                &transport_plan.heliusSenderEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await?;
            let (confirm_warnings, confirm_ms) = confirm_transactions_with_websocket_fallback(
                rpc_url,
                transport_watch_endpoint(transport_plan),
                &mut results,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await?;
            warnings.extend(confirm_warnings);
            Ok((
                results,
                warnings,
                SendTimingBreakdown {
                    submit_ms,
                    confirm_ms,
                },
            ))
        }
        _ => {
            let (mut results, warnings, submit_ms) = submit_transactions_standard_rpc_fanout(
                rpc_url,
                &transport_plan.standardRpcSubmitEndpoints,
                transactions,
                commitment,
                transport_plan.skipPreflight || skip_preflight,
                transport_plan.maxRetries,
                track_send_block_height,
                false,
            )
            .await?;
            let (confirm_warnings, confirm_ms) = confirm_transactions_with_websocket_fallback(
                rpc_url,
                transport_watch_endpoint(transport_plan),
                &mut results,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await?;
            let mut combined_warnings = warnings;
            combined_warnings.extend(confirm_warnings);
            Ok((
                results,
                combined_warnings,
                SendTimingBreakdown {
                    submit_ms,
                    confirm_ms,
                },
            ))
        }
    }
}

pub async fn submit_transactions_for_transport(
    rpc_url: &str,
    transport_plan: &TransportPlan,
    transactions: &[CompiledTransaction],
    commitment: &str,
    skip_preflight: bool,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    match transport_plan.transportType.as_str() {
        "jito-bundle" => {
            submit_transactions_bundle(
                rpc_url,
                &transport_plan.jitoBundleEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await
        }
        "hellomoon-bundle" => {
            submit_transactions_hellomoon_bundle(
                rpc_url,
                &transport_plan.helloMoonBundleEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await
        }
        "hellomoon-quic" => {
            submit_transactions_hellomoon_quic(
                rpc_url,
                &transport_plan.helloMoonQuicEndpoints,
                transactions,
                commitment,
                track_send_block_height,
                transport_plan.helloMoonMevProtect,
            )
            .await
        }
        "helius-sender" => {
            submit_transactions_helius_sender(
                rpc_url,
                &transport_plan.heliusSenderEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await
        }
        _ => {
            submit_transactions_standard_rpc_fanout(
                rpc_url,
                &transport_plan.standardRpcSubmitEndpoints,
                transactions,
                commitment,
                transport_plan.skipPreflight || skip_preflight,
                transport_plan.maxRetries,
                track_send_block_height,
                false,
            )
            .await
        }
    }
}

pub async fn submit_independent_transactions_for_transport(
    rpc_url: &str,
    transport_plan: &TransportPlan,
    transactions: &[CompiledTransaction],
    commitment: &str,
    skip_preflight: bool,
    track_send_block_height: bool,
) -> Result<(Vec<SentResult>, Vec<String>, u128), String> {
    match transport_plan.transportType.as_str() {
        "jito-bundle" => {
            submit_transactions_bundle(
                rpc_url,
                &transport_plan.jitoBundleEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await
        }
        "hellomoon-bundle" => {
            submit_transactions_hellomoon_bundle(
                rpc_url,
                &transport_plan.helloMoonBundleEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await
        }
        "hellomoon-quic" => {
            submit_transactions_hellomoon_quic_parallel(
                rpc_url,
                &transport_plan.helloMoonQuicEndpoints,
                transactions,
                commitment,
                track_send_block_height,
                transport_plan.helloMoonMevProtect,
            )
            .await
        }
        "helius-sender" => {
            submit_transactions_helius_sender_parallel(
                rpc_url,
                &transport_plan.heliusSenderEndpoints,
                transactions,
                commitment,
                track_send_block_height,
            )
            .await
        }
        _ => {
            submit_transactions_standard_rpc_fanout(
                rpc_url,
                &transport_plan.standardRpcSubmitEndpoints,
                transactions,
                commitment,
                transport_plan.skipPreflight || skip_preflight,
                transport_plan.maxRetries,
                track_send_block_height,
                true,
            )
            .await
        }
    }
}

pub async fn confirm_submitted_transactions_for_transport(
    rpc_url: &str,
    transport_plan: &TransportPlan,
    submitted: &mut [SentResult],
    commitment: &str,
    track_send_block_height: bool,
) -> Result<(Vec<String>, u128), String> {
    match transport_plan.transportType.as_str() {
        "jito-bundle" => {
            confirm_transactions_bundle(
                rpc_url,
                &transport_plan.jitoBundleEndpoints,
                submitted,
                commitment,
                track_send_block_height,
            )
            .await
        }
        "hellomoon-bundle" => {
            confirm_transactions_with_websocket_fallback(
                rpc_url,
                transport_watch_endpoint(transport_plan),
                submitted,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await
        }
        "hellomoon-quic" => {
            confirm_transactions_with_websocket_fallback(
                rpc_url,
                transport_watch_endpoint(transport_plan),
                submitted,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await
        }
        "helius-sender" => {
            confirm_transactions_with_websocket_fallback(
                rpc_url,
                transport_watch_endpoint(transport_plan),
                submitted,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await
        }
        _ => {
            confirm_transactions_with_websocket_fallback(
                rpc_url,
                transport_watch_endpoint(transport_plan),
                submitted,
                commitment,
                track_send_block_height,
                75,
                400,
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
    use serde_json::json;
    use std::{net::SocketAddr, sync::Arc};
    use tokio::sync::Mutex;

    fn compiled_tx(label: &str) -> CompiledTransaction {
        CompiledTransaction {
            label: label.to_string(),
            format: "legacy".to_string(),
            blockhash: "test-blockhash".to_string(),
            lastValidBlockHeight: 123,
            serializedBase64: "AQID".to_string(),
            signature: None,
            lookupTablesUsed: vec![],
            computeUnitLimit: Some(1_000_000),
            computeUnitPriceMicroLamports: Some(1),
            inlineTipLamports: Some(200_000),
            inlineTipAccount: Some("tip-account".to_string()),
        }
    }

    async fn start_jsonrpc_server() -> SocketAddr {
        async fn handler(Json(payload): Json<Value>) -> Json<Value> {
            let method = payload
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let response = match method {
                "getBlockHeight" => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": 345678
                }),
                "simulateTransaction" => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "value": {
                            "err": null,
                            "unitsConsumed": 4242,
                            "logs": ["program log: ok"]
                        }
                    }
                }),
                "sendTransaction" => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "sig-test-123"
                }),
                "getSignatureStatuses" => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "value": [{
                            "confirmationStatus": "confirmed",
                            "err": null,
                            "slot": 456789
                        }]
                    }
                }),
                _ => json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "error": { "message": format!("unsupported method: {method}") }
                }),
            };
            Json(response)
        }

        let app = Router::new().route("/", post(handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test rpc listener");
        let addr = listener.local_addr().expect("read local addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve rpc test app");
        });
        addr
    }

    async fn start_jito_server(
        send_calls: Arc<Mutex<Vec<Value>>>,
        status_calls: Arc<Mutex<Vec<Value>>>,
        bundle_id: &'static str,
    ) -> (SocketAddr, SocketAddr) {
        async fn send_handler(
            State((send_calls, bundle_id)): State<(Arc<Mutex<Vec<Value>>>, &'static str)>,
            Json(payload): Json<Value>,
        ) -> Json<Value> {
            send_calls.lock().await.push(payload);
            Json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": bundle_id
            }))
        }

        async fn status_handler(
            State((status_calls, bundle_id)): State<(Arc<Mutex<Vec<Value>>>, &'static str)>,
            Json(payload): Json<Value>,
        ) -> Json<Value> {
            status_calls.lock().await.push(payload);
            Json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "value": [{
                        "bundle_id": bundle_id,
                        "confirmation_status": "confirmed",
                        "err": null,
                        "slot": 567890,
                        "transactions": ["sig-1", "sig-2"]
                    }]
                }
            }))
        }

        let send_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind jito send listener");
        let send_addr = send_listener.local_addr().expect("read send addr");
        tokio::spawn(async move {
            let app = Router::new()
                .route("/api/v1/bundles", post(send_handler))
                .with_state((send_calls, bundle_id));
            axum::serve(send_listener, app)
                .await
                .expect("serve jito send app");
        });

        let status_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind jito status listener");
        let status_addr = status_listener.local_addr().expect("read status addr");
        tokio::spawn(async move {
            let app = Router::new()
                .route("/api/v1/getBundleStatuses", post(status_handler))
                .with_state((status_calls, bundle_id));
            axum::serve(status_listener, app)
                .await
                .expect("serve jito status app");
        });

        (send_addr, status_addr)
    }

    async fn start_sender_server(calls: Arc<Mutex<Vec<Value>>>) -> SocketAddr {
        async fn handler(
            State(calls): State<Arc<Mutex<Vec<Value>>>>,
            Json(payload): Json<Value>,
        ) -> Json<Value> {
            calls.lock().await.push(payload);
            Json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "sender-sig-123"
            }))
        }

        let app = Router::new().route("/", post(handler)).with_state(calls);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind sender listener");
        let addr = listener.local_addr().expect("read sender addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve sender app");
        });
        addr
    }

    async fn start_jito_rate_limited_server(message: &'static str) -> SocketAddr {
        async fn handler(State(message): State<&'static str>) -> (StatusCode, Json<Value>) {
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "error": { "message": message }
                })),
            )
        }

        let app = Router::new()
            .route("/api/v1/getTipAccounts", post(handler))
            .with_state(message);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind jito rate-limited listener");
        let addr = listener.local_addr().expect("read local addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve jito rate-limited app");
        });
        addr
    }

    #[test]
    fn jito_tip_accounts_endpoint_uses_documented_path() {
        let endpoint = JitoBundleEndpoint {
            name: "frankfurt.mainnet.block-engine.jito.wtf".to_string(),
            send: "https://frankfurt.mainnet.block-engine.jito.wtf/api/v1/bundles".to_string(),
            status: "https://frankfurt.mainnet.block-engine.jito.wtf/api/v1/getBundleStatuses"
                .to_string(),
        };
        assert_eq!(
            jito_tip_accounts_endpoint(&endpoint),
            "https://frankfurt.mainnet.block-engine.jito.wtf/api/v1/getTipAccounts"
        );
    }

    #[tokio::test]
    async fn jito_prewarm_classifies_rate_limited_endpoint_as_reachable() {
        let addr = start_jito_rate_limited_server("rate limit exceeded").await;
        let endpoint = JitoBundleEndpoint {
            name: "local-jito".to_string(),
            send: format!("http://{addr}/api/v1/bundles"),
            status: format!("http://{addr}/api/v1/getBundleStatuses"),
        };
        assert_eq!(
            prewarm_jito_bundle_endpoint(&endpoint)
                .await
                .expect("rate-limited warm should not hard fail"),
            JitoWarmResult::RateLimited("rate limit exceeded".to_string())
        );
    }

    #[tokio::test]
    async fn simulates_transactions_via_rpc() {
        let addr = start_jsonrpc_server().await;
        let rpc_url = format!("http://{addr}/");
        let (results, warnings) =
            simulate_transactions(&rpc_url, &[compiled_tx("launch")], "confirmed")
                .await
                .expect("simulate should succeed");
        assert!(warnings.is_empty());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "launch");
        assert_eq!(results[0].unitsConsumed, Some(4242));
        assert_eq!(results[0].logs, vec!["program log: ok".to_string()]);
    }

    #[tokio::test]
    async fn sends_transactions_sequentially_via_rpc() {
        let addr = start_jsonrpc_server().await;
        let rpc_url = format!("http://{addr}/");
        let (results, warnings, timing) = send_transactions_sequential(
            &rpc_url,
            &[compiled_tx("launch")],
            "confirmed",
            false,
            true,
        )
        .await
        .expect("send should succeed");
        assert!(warnings.is_empty());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].signature.as_deref(), Some("sig-test-123"));
        assert_eq!(results[0].sendObservedBlockHeight, Some(345678));
        assert_eq!(results[0].confirmedObservedBlockHeight, Some(345678));
        assert_eq!(results[0].confirmedSlot, Some(456789));
        assert_eq!(
            results[0].explorerUrl.as_deref(),
            Some("https://solscan.io/tx/sig-test-123")
        );
        assert_eq!(
            timing.submit_ms.saturating_add(timing.confirm_ms) >= timing.confirm_ms,
            true
        );
    }

    #[tokio::test]
    async fn sends_bundle_transactions_via_jito_endpoints() {
        let send_calls_one = Arc::new(Mutex::new(Vec::new()));
        let status_calls_one = Arc::new(Mutex::new(Vec::new()));
        let send_calls_two = Arc::new(Mutex::new(Vec::new()));
        let status_calls_two = Arc::new(Mutex::new(Vec::new()));
        let (send_addr_one, status_addr_one) = start_jito_server(
            send_calls_one.clone(),
            status_calls_one.clone(),
            "bundle-123",
        )
        .await;
        let (send_addr_two, status_addr_two) = start_jito_server(
            send_calls_two.clone(),
            status_calls_two.clone(),
            "bundle-456",
        )
        .await;
        let endpoints = vec![
            JitoBundleEndpoint {
                name: "local-one".to_string(),
                send: format!("http://{send_addr_one}/api/v1/bundles"),
                status: format!("http://{status_addr_one}/api/v1/getBundleStatuses"),
            },
            JitoBundleEndpoint {
                name: "local-two".to_string(),
                send: format!("http://{send_addr_two}/api/v1/bundles"),
                status: format!("http://{status_addr_two}/api/v1/getBundleStatuses"),
            },
        ];
        let transactions = vec![compiled_tx("launch"), compiled_tx("jito-tip")];
        let rpc_addr = start_jsonrpc_server().await;
        let rpc_url = format!("http://{rpc_addr}/");
        let (results, warnings, timing) =
            send_transactions_bundle(&rpc_url, &endpoints, &transactions, "confirmed", true)
                .await
                .expect("bundle send should succeed");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].signature.as_deref(), Some("sig-1"));
        assert_eq!(results[1].signature.as_deref(), Some("sig-2"));
        assert_eq!(results[0].sendObservedBlockHeight, Some(345678));
        assert_eq!(results[0].confirmedObservedBlockHeight, Some(345678));
        assert_eq!(results[0].confirmedSlot, Some(567890));
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("fanout to 2 endpoints"))
        );
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("Sent via Jito bundle"))
        );
        assert_eq!(results[0].attemptedEndpoints.len(), 2);
        assert_eq!(results[0].attemptedBundleIds.len(), 2);
        assert_eq!(send_calls_one.lock().await.len(), 1);
        assert_eq!(send_calls_two.lock().await.len(), 1);
        assert!(
            !status_calls_one.lock().await.is_empty() || !status_calls_two.lock().await.is_empty()
        );
        assert_eq!(
            timing.submit_ms.saturating_add(timing.confirm_ms) >= timing.confirm_ms,
            true
        );
    }

    #[tokio::test]
    async fn sends_transactions_via_helius_sender_with_required_flags() {
        let rpc_addr = start_jsonrpc_server().await;
        let calls_one = Arc::new(Mutex::new(Vec::new()));
        let calls_two = Arc::new(Mutex::new(Vec::new()));
        let sender_addr_one = start_sender_server(calls_one.clone()).await;
        let sender_addr_two = start_sender_server(calls_two.clone()).await;
        let rpc_url = format!("http://{rpc_addr}/");
        let endpoint_one = format!("http://{sender_addr_one}/");
        let endpoint_two = format!("http://{sender_addr_two}/");
        let (results, warnings, timing) = send_transactions_helius_sender(
            &rpc_url,
            &[endpoint_one.clone(), endpoint_two.clone()],
            &[compiled_tx("launch")],
            "confirmed",
            true,
        )
        .await
        .expect("sender send should succeed");
        assert!(warnings.is_empty());
        assert_eq!(results[0].signature.as_deref(), Some("sender-sig-123"));
        assert_eq!(results[0].sendObservedBlockHeight, Some(345678));
        assert_eq!(results[0].confirmedObservedBlockHeight, Some(345678));
        assert_eq!(results[0].confirmedSlot, Some(456789));
        assert_eq!(results[0].attemptedEndpoints.len(), 2);
        assert_eq!(
            timing.submit_ms.saturating_add(timing.confirm_ms) >= timing.confirm_ms,
            true
        );

        let recorded = calls_one.lock().await;
        assert_eq!(recorded.len(), 1);
        let params = recorded[0]
            .get("params")
            .and_then(Value::as_array)
            .expect("sender params");
        let options = params[1].as_object().expect("sender options");
        assert_eq!(options.get("skipPreflight"), Some(&Value::Bool(true)));
        assert_eq!(options.get("maxRetries"), Some(&Value::from(0)));
        assert_eq!(calls_two.lock().await.len(), 1);
    }

    #[test]
    fn hellomoon_bundle_accepts_tip_on_last_transaction() {
        let mut first = compiled_tx("setup");
        first.computeUnitPriceMicroLamports = Some(5);
        first.inlineTipLamports = None;
        first.inlineTipAccount = None;

        let mut last = compiled_tx("launch");
        last.computeUnitPriceMicroLamports = Some(5);
        last.inlineTipLamports = Some(1_000_000);

        assert!(validate_hellomoon_bundle_transactions(&[first, last]).is_ok());
    }

    #[test]
    fn hellomoon_bundle_accepts_non_last_inline_tip() {
        let mut first = compiled_tx("setup");
        first.computeUnitPriceMicroLamports = Some(5);
        first.inlineTipLamports = Some(1_000_000);

        let mut last = compiled_tx("launch");
        last.computeUnitPriceMicroLamports = Some(5);
        last.inlineTipLamports = None;

        assert!(validate_hellomoon_bundle_transactions(&[first, last]).is_ok());
    }

    #[test]
    fn hellomoon_bundle_requires_at_least_one_valid_tip() {
        let mut first = compiled_tx("setup");
        first.computeUnitPriceMicroLamports = Some(5);
        first.inlineTipLamports = None;

        let mut last = compiled_tx("launch");
        last.computeUnitPriceMicroLamports = Some(5);
        last.inlineTipLamports = Some(500_000);

        let error = validate_hellomoon_bundle_transactions(&[first, last])
            .expect_err("bundle should require a valid Hello Moon tip");
        assert!(error.contains("at least one inline Hello Moon tip"));
    }

    #[tokio::test]
    async fn helius_sender_rejects_transactions_without_inline_tip() {
        let rpc_addr = start_jsonrpc_server().await;
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sender_addr = start_sender_server(calls).await;
        let rpc_url = format!("http://{rpc_addr}/");
        let endpoint = format!("http://{sender_addr}/");
        let mut transaction = compiled_tx("launch");
        transaction.inlineTipLamports = None;
        let error = send_transactions_helius_sender(
            &rpc_url,
            &[endpoint.clone()],
            &[transaction],
            "confirmed",
            false,
        )
        .await
        .expect_err("sender should reject missing inline tip");
        assert!(error.contains("required inline Helius Sender tip"));
    }
}
