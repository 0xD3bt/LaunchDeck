#![allow(non_snake_case, dead_code)]

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize, de::DeserializeOwned, Deserializer};
use serde_json::{Value, json};
use solana_address_lookup_table_interface::state::AddressLookupTable;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    process::Stdio,
    str::FromStr,
    sync::{Arc, Mutex, OnceLock},
};
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::{AddressLookupTableAccount, VersionedMessage, v0},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Semaphore,
    time::{Duration, timeout},
};

use crate::{
    config::{
        NormalizedConfig, NormalizedExecution, configured_default_dev_auto_sell_compute_unit_limit,
        configured_default_follow_up_compute_unit_limit,
        configured_default_launch_compute_unit_limit,
        configured_default_launch_usd1_topup_compute_unit_limit,
        configured_default_sniper_buy_compute_unit_limit, validate_launchpad_support,
    },
    helper_worker::{
        HelperWorkerClient, HelperWorkerConfig, HelperWorkerError, helper_worker_enabled,
    },
    launchpad_dispatch::{launchpad_action_backend, launchpad_action_rollout_state},
    paths,
    report::{
        BonkUsd1LaunchSummary, FeeSettings, InstructionSummary, TransactionSummary, build_report,
        render_report,
    },
    rpc::{CompiledTransaction, fetch_account_data, fetch_latest_blockhash_cached},
    transport::TransportPlan,
    wallet::read_keypair_bytes,
};

use crate::pump_native::{LaunchQuote, NativeCompileTimings};

const PACKET_LIMIT_BYTES: usize = 1232;
const PRIORITY_FEE_PRICE_BASE_COMPUTE_UNIT_LIMIT: u64 = 1_000_000;
const COMPUTE_BUDGET_PROGRAM_ID: &str = "ComputeBudget111111111111111111111111111111";
const JITODONTFRONT_ACCOUNT: &str = "jitodontfront111111111111111111111111111111";
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const MEMO_PROGRAM_ID: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";
const MPL_TOKEN_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
const BONK_LAUNCHPAD_PROGRAM_ID: &str = "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj";
const BONK_LETSBONK_PLATFORM_ID: &str = "FfYek5vEz23cMkWsdJwG2oa6EphsvXSHrGpdALN4g6W1";
const BONK_BONKERS_PLATFORM_ID: &str = "82NMHVCKwehXgbXMyzL41mvv3sdkypaMCtTxvJ4CtTzm";
const BONK_SOL_QUOTE_MINT: &str = "So11111111111111111111111111111111111111112";
const BONK_USD1_QUOTE_MINT: &str = "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB";
const BONK_USD1_SUPER_LOOKUP_TABLE: &str = "GHVFasDr4sFtF2fMNBLnaRUKeSxX77DgK5SsThB3Ro7U";
const BONK_PINNED_USD1_ROUTE_POOL_ID: &str = "AQAGYQsdU853WAKhXM79CgNdoyhrRwXvYHX6qrDyC1FS";
const BONK_PREFERRED_USD1_ROUTE_CONFIG_ID: &str = "E64NGkDLLCdQ2yFNPcavaKptrEgmiQaNykUuLC1Qgwyp";
const BONK_CLMM_PROGRAM_ID: &str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";
const RAYDIUM_POOL_SEARCH_MINT_ENDPOINT: &str = "https://api-v3.raydium.io/pools/info/mint";
const RAYDIUM_MAINNET_LAUNCH_CONFIGS_ENDPOINT: &str = "https://launch-mint-v1.raydium.io/main/configs";
const RAYDIUM_DEVNET_LAUNCH_CONFIGS_ENDPOINT: &str =
    "https://launch-mint-v1-devnet.raydium.io/main/configs";
const BONK_DEFAULT_LAUNCH_DEFAULTS_CACHE_TTL_MS: u64 = 30 * 60 * 1000;
const BONK_DEFAULT_USD1_ROUTE_SETUP_CACHE_TTL_MS: u64 = 10 * 60 * 1000;
const BONK_DEFAULT_LOOKUP_TABLE_CACHE_TTL_MS: u64 = 30 * 60 * 1000;
const BONK_TOKEN_DECIMALS: u32 = 6;
const BONK_FEE_RATE_DENOMINATOR: u64 = 1_000_000;
const BONK_SPL_TOKEN_ACCOUNT_LEN: u64 = 165;
const BONK_CLMM_TICK_ARRAY_SIZE: i32 = 60;
const BONK_CLMM_DEFAULT_BITMAP_OFFSET: i32 = 512;
const BONK_USD1_QUOTE_MAX_INPUT_LAMPORTS: u64 = 100_000 * 1_000_000_000;
const BONK_CLMM_MIN_SQRT_PRICE_X64_PLUS_ONE: u128 = 4_295_048_017;
const BONK_CLMM_SWAP_DISCRIMINATOR: [u8; 8] = [43, 4, 237, 11, 26, 201, 30, 98];
const BONK_INITIALIZE_V2_DISCRIMINATOR: [u8; 8] = [67, 153, 175, 39, 218, 16, 38, 32];
const DEFAULT_HELPER_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_HELPER_MAX_CONCURRENCY: usize = 4;
const BONK_BUY_EXACT_IN_DISCRIMINATOR: [u8; 8] = [250, 234, 13, 123, 213, 156, 19, 236];
const BONK_SELL_EXACT_IN_DISCRIMINATOR: [u8; 8] = [149, 39, 222, 155, 211, 124, 152, 26];

fn helper_timeout_ms() -> u64 {
    std::env::var("LAUNCHDECK_LAUNCHPAD_HELPER_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_HELPER_TIMEOUT_MS)
}

fn helper_semaphore() -> Arc<Semaphore> {
    static SEMAPHORE: OnceLock<Arc<Semaphore>> = OnceLock::new();
    SEMAPHORE
        .get_or_init(|| {
            let limit = std::env::var("LAUNCHDECK_LAUNCHPAD_HELPER_MAX_CONCURRENCY")
                .ok()
                .and_then(|value| value.trim().parse::<usize>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(DEFAULT_HELPER_MAX_CONCURRENCY);
            Arc::new(Semaphore::new(limit))
        })
        .clone()
}

fn bonk_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("bonk http client")
    })
}

#[derive(Debug, Clone)]
pub struct NativeBonkArtifacts {
    pub compiled_transactions: Vec<CompiledTransaction>,
    pub creation_transactions: Vec<CompiledTransaction>,
    pub deferred_setup_transactions: Vec<CompiledTransaction>,
    pub report: Value,
    pub text: String,
    pub compile_timings: NativeCompileTimings,
    pub mint: String,
    pub launch_creator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkMarketSnapshot {
    pub mint: String,
    pub creator: String,
    pub virtualTokenReserves: String,
    pub virtualSolReserves: String,
    pub realTokenReserves: String,
    pub realSolReserves: String,
    pub tokenTotalSupply: String,
    pub complete: bool,
    pub marketCapLamports: String,
    pub marketCapSol: String,
    #[serde(default)]
    pub quoteAsset: String,
    #[serde(default)]
    pub quoteAssetLabel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkImportContext {
    pub launchpad: String,
    pub mode: String,
    pub quoteAsset: String,
    #[serde(default)]
    pub creator: String,
    #[serde(default)]
    pub platformId: String,
    #[serde(default)]
    pub configId: String,
    #[serde(default)]
    pub poolId: String,
    #[serde(default)]
    pub detectionSource: String,
}

#[derive(Debug, Serialize)]
struct HelperTxConfig<'a> {
    computeUnitLimit: u64,
    computeUnitPriceMicroLamports: u64,
    tipLamports: u64,
    tipAccount: &'a str,
    jitodontfront: bool,
    singleBundleTipLastTx: bool,
}

#[derive(Debug, Deserialize)]
struct HelperCompiledTransaction {
    label: String,
    format: String,
    blockhash: String,
    lastValidBlockHeight: u64,
    serializedBase64: String,
    #[serde(default)]
    lookupTablesUsed: Vec<String>,
    #[serde(default)]
    computeUnitLimit: Option<u64>,
    #[serde(default)]
    computeUnitPriceMicroLamports: Option<u64>,
    #[serde(default)]
    inlineTipLamports: Option<u64>,
    #[serde(default)]
    inlineTipAccount: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct HelperUsd1QuoteMetrics {
    #[serde(default)]
    quoteCalls: u64,
    #[serde(default)]
    quoteTotalMs: u64,
    #[serde(default)]
    averageQuoteMs: f64,
    #[serde(default)]
    quoteCacheHits: u64,
    #[serde(default)]
    routeSetupLocalHits: u64,
    #[serde(default)]
    routeSetupCacheHits: u64,
    #[serde(default)]
    routeSetupCacheMisses: u64,
    #[serde(default)]
    routeSetupFetchMs: u64,
    #[serde(default)]
    superAltLocalSnapshotHits: u64,
    #[serde(default)]
    superAltRpcRefreshes: u64,
    #[serde(default)]
    expansionQuoteCalls: u64,
    #[serde(default)]
    binarySearchQuoteCalls: u64,
    #[serde(default)]
    bufferQuoteCalls: u64,
    #[serde(default)]
    searchIterations: u64,
}

#[derive(Debug, Deserialize)]
struct HelperLaunchResponse {
    mint: String,
    launchCreator: String,
    compiledTransactions: Vec<HelperCompiledTransaction>,
    #[serde(default)]
    predictedDevBuyTokenAmountRaw: Option<String>,
    #[serde(default)]
    atomicCombined: bool,
    #[serde(default)]
    atomicFallbackReason: Option<String>,
    #[serde(default)]
    usd1LaunchDetails: Option<HelperUsd1LaunchDetails>,
    #[serde(default)]
    usd1QuoteMetrics: Option<HelperUsd1QuoteMetrics>,
}

#[derive(Debug, Deserialize)]
struct HelperUsd1LaunchDetails {
    compilePath: String,
    requiredQuoteAmount: String,
    currentQuoteAmount: String,
    shortfallQuoteAmount: String,
    #[serde(default)]
    inputSol: Option<String>,
    #[serde(default)]
    expectedQuoteOut: Option<String>,
    #[serde(default)]
    minQuoteOut: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HelperFollowBuyResponse {
    compiledTransaction: HelperCompiledTransaction,
}

#[derive(Debug, Deserialize)]
struct HelperFollowSellResponse {
    compiledTransaction: Option<HelperCompiledTransaction>,
}

#[derive(Debug, Deserialize)]
struct HelperPredictDevBuyResponse {
    #[serde(default)]
    predictedDevBuyTokenAmountRaw: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HelperDerivePoolIdResponse {
    poolId: String,
}

#[derive(Debug, Deserialize)]
struct HelperUsd1TopupResponse {
    compiledTransaction: Option<HelperCompiledTransaction>,
    #[serde(default)]
    usd1QuoteMetrics: Option<HelperUsd1QuoteMetrics>,
}

#[derive(Debug, Clone)]
struct BonkQuoteAssetConfig {
    asset: &'static str,
    label: &'static str,
    mint: &'static str,
    decimals: u32,
}

#[derive(Debug, Clone)]
struct DecodedBonkLaunchpadPool {
    creator: Pubkey,
    status: u8,
    supply: u64,
    config_id: Pubkey,
    total_sell_a: u64,
    virtual_a: u64,
    virtual_b: u64,
    real_a: u64,
    real_b: u64,
    platform_id: Pubkey,
    mint_a: Pubkey,
}

#[derive(Debug, Clone)]
struct BonkMarketCandidate {
    mode: String,
    quote_asset: String,
    quote_asset_label: String,
    creator: String,
    platform_id: String,
    config_id: String,
    pool_id: String,
    real_quote_reserves: u64,
    complete: bool,
    detection_source: String,
    launch_migrate_pool: bool,
    tvl: f64,
    pool_type: String,
    launchpad_pool: Option<DecodedBonkLaunchpadPool>,
    raydium_pool: Option<RaydiumPoolInfo>,
}

#[derive(Debug, Clone)]
struct DecodedBonkLaunchpadConfig {
    curve_type: u8,
    migrate_fee: u64,
    trade_fee_rate: u64,
}

#[derive(Debug, Clone)]
struct DecodedBonkPlatformConfig {
    fee_rate: u64,
    creator_fee_rate: u64,
}

#[derive(Debug, Clone)]
struct DecodedBonkClmmConfig {
    trade_fee_rate: u32,
    tick_spacing: u16,
}

#[derive(Debug, Clone)]
struct DecodedBonkClmmPool {
    amm_config: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    mint_decimals_a: u8,
    mint_decimals_b: u8,
    tick_spacing: u16,
    liquidity: BigUint,
    sqrt_price_x64: BigUint,
    tick_current: i32,
    tick_array_bitmap: [u64; 16],
}

#[derive(Debug, Clone)]
struct BonkClmmTick {
    tick: i32,
    liquidity_net: i128,
    liquidity_gross: BigUint,
}

#[derive(Debug, Clone)]
struct BonkClmmTickArray {
    start_tick_index: i32,
    ticks: Vec<BonkClmmTick>,
}

#[derive(Debug, Clone)]
pub struct BonkUsd1RouteSetup {
    pool_id: Pubkey,
    program_id: Pubkey,
    tick_spacing: i32,
    trade_fee_rate: u32,
    sqrt_price_x64: BigUint,
    liquidity: BigUint,
    tick_current: i32,
    mint_a_decimals: u32,
    mint_b_decimals: u32,
    current_price: f64,
    tick_arrays_desc: Vec<i32>,
    tick_arrays: HashMap<i32, BonkClmmTickArray>,
}

#[derive(Debug, Clone)]
struct BonkUsd1RouteSetupCacheEntry {
    fetched_at: std::time::Instant,
    setup: BonkUsd1RouteSetup,
}

#[derive(Debug, Clone)]
struct BonkLookupTableCacheEntry {
    fetched_at: std::time::Instant,
    table: AddressLookupTableAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedBonkLookupTableCache {
    tables: HashMap<String, PersistedBonkLookupTableEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedBonkLookupTableEntry {
    addresses: Vec<String>,
}

#[derive(Debug, Clone)]
struct BonkUsd1DirectQuote {
    expected_out: BigUint,
    min_out: BigUint,
    price_impact_pct: f64,
    traversed_tick_array_starts: Vec<i32>,
}

#[derive(Debug, Clone)]
struct NativeBonkUsd1LaunchDetails {
    compile_path: String,
    required_quote_amount: String,
    current_quote_amount: String,
    shortfall_quote_amount: String,
    input_sol: Option<String>,
    expected_quote_out: Option<String>,
    min_quote_out: Option<String>,
}

#[derive(Debug, Clone)]
struct NativeBonkLaunchResult {
    mint: String,
    launch_creator: String,
    compiled_transactions: Vec<CompiledTransaction>,
    predicted_dev_buy_token_amount_raw: Option<String>,
    atomic_combined: bool,
    atomic_fallback_reason: Option<String>,
    usd1_launch_details: Option<NativeBonkUsd1LaunchDetails>,
    usd1_quote_metrics: Option<HelperUsd1QuoteMetrics>,
    compiled_via_native: bool,
}

#[derive(Debug, Clone)]
struct NativeBonkPreparedUsd1Topup {
    required_quote_amount: BigUint,
    current_quote_amount: BigUint,
    shortfall_quote_amount: BigUint,
    input_lamports: Option<BigUint>,
    expected_quote_out: Option<BigUint>,
    min_quote_out: Option<BigUint>,
    traversed_tick_array_starts: Vec<i32>,
}

#[derive(Debug, Clone)]
struct BonkCurvePoolState {
    total_sell_a: BigUint,
    virtual_a: BigUint,
    virtual_b: BigUint,
    real_a: BigUint,
    real_b: BigUint,
}

#[derive(Debug, Clone)]
struct BonkLaunchDefaults {
    supply: BigUint,
    total_fund_raising_b: BigUint,
    quote: BonkQuoteAssetConfig,
    trade_fee_rate: BigUint,
    platform_fee_rate: BigUint,
    creator_fee_rate: BigUint,
    curve_type: u8,
    pool: BonkCurvePoolState,
}

#[derive(Debug, Clone)]
struct BonkLaunchDefaultsCacheEntry {
    fetched_at: std::time::Instant,
    defaults: BonkLaunchDefaults,
}

#[derive(Debug, Clone)]
struct NativeBonkTxConfig {
    compute_unit_limit: u32,
    compute_unit_price_micro_lamports: u64,
    tip_lamports: u64,
    tip_account: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeBonkTxFormat {
    Legacy,
    V0,
}

#[derive(Debug, Clone)]
pub struct NativeBonkPoolContext {
    pool_id: Pubkey,
    pool: DecodedBonkLaunchpadPool,
    config: DecodedBonkLaunchpadConfig,
    platform: DecodedBonkPlatformConfig,
    quote: BonkQuoteAssetConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BonkPredictedDevBuyEffect {
    pub requested_quote_amount_b: u64,
    pub token_amount: u64,
}

#[derive(Debug, Clone)]
struct DecomposedBonkVersionedTransaction {
    instructions: Vec<Instruction>,
    lookup_tables: Vec<AddressLookupTableAccount>,
}

#[derive(Debug, Clone)]
struct RaydiumLaunchConfigCacheEntry {
    fetched_at: std::time::Instant,
    configs: Vec<RaydiumLaunchConfigEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct RaydiumLaunchConfigEntry {
    key: RaydiumLaunchConfigKey,
    #[serde(default, rename = "defaultParams")]
    default_params: RaydiumLaunchConfigDefaultParams,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RaydiumLaunchConfigKey {
    #[serde(default, rename = "pubKey")]
    pubkey: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RaydiumLaunchConfigDefaultParams {
    #[serde(default, rename = "supplyInit")]
    supply_init: String,
    #[serde(default, rename = "totalFundRaisingB")]
    total_fund_raising_b: String,
    #[serde(default, rename = "totalSellA")]
    total_sell_a: String,
}

fn parse_raydium_launch_configs_payload(payload: Value) -> Result<Vec<RaydiumLaunchConfigEntry>, String> {
    let configs_value = payload
        .get("data")
        .and_then(|value| {
            if value.is_array() {
                Some(value.clone())
            } else {
                value.get("data").cloned()
            }
        })
        .ok_or_else(|| "Raydium launch configs payload did not include a data array.".to_string())?;
    serde_json::from_value::<Vec<RaydiumLaunchConfigEntry>>(configs_value)
        .map_err(|error| format!("Failed to decode Raydium launch configs payload: {error}"))
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RaydiumTokenAddress {
    #[serde(default)]
    address: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RaydiumConfigRef {
    #[serde(default)]
    id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RaydiumPoolInfo {
    #[serde(default)]
    id: String,
    #[serde(default)]
    price: f64,
    #[serde(default)]
    tvl: f64,
    #[serde(default, rename = "type")]
    pool_type: String,
    #[serde(default, rename = "launchMigratePool")]
    launch_migrate_pool: bool,
    #[serde(default, rename = "mintA")]
    mint_a: RaydiumTokenAddress,
    #[serde(default, rename = "mintB")]
    mint_b: RaydiumTokenAddress,
    #[serde(default)]
    config: Option<RaydiumConfigRef>,
}

#[derive(Debug, Clone, Deserialize)]
struct RaydiumPoolsResponse {
    #[serde(default, deserialize_with = "deserialize_raydium_pools_response_data")]
    data: Vec<RaydiumPoolInfo>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RaydiumPoolsResponsePage {
    #[serde(default)]
    data: Vec<RaydiumPoolInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RaydiumPoolsResponseData {
    Direct(Vec<RaydiumPoolInfo>),
    Nested(RaydiumPoolsResponsePage),
}

fn deserialize_raydium_pools_response_data<'de, D>(
    deserializer: D,
) -> Result<Vec<RaydiumPoolInfo>, D::Error>
where
    D: Deserializer<'de>,
{
    let payload = RaydiumPoolsResponseData::deserialize(deserializer)?;
    Ok(match payload {
        RaydiumPoolsResponseData::Direct(entries) => entries,
        RaydiumPoolsResponseData::Nested(page) => page.data,
    })
}

#[derive(Debug, Deserialize)]
struct RpcMultipleAccountsResponse {
    result: RpcMultipleAccountsResult,
}

#[derive(Debug, Deserialize)]
struct RpcMultipleAccountsResult {
    value: Vec<Option<RpcMultipleAccountsValue>>,
}

#[derive(Debug, Deserialize)]
struct RpcMultipleAccountsValue {
    data: (String, String),
}

#[derive(Debug, Clone, Deserialize)]
struct RpcResponse<T> {
    result: T,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcTokenSupplyValue {
    #[serde(default)]
    amount: String,
    #[serde(default)]
    decimals: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcTokenSupplyResult {
    value: RpcTokenSupplyValue,
}

#[derive(Debug, Deserialize)]
struct HelperWarmLaunchDefault {
    mode: String,
    quoteAsset: String,
    platformId: String,
    configId: String,
    quoteMint: String,
}

#[derive(Debug, Deserialize)]
struct HelperWarmStateResponse {
    #[serde(default)]
    warmedLaunchDefaults: Vec<HelperWarmLaunchDefault>,
    #[serde(default)]
    usd1RoutePoolId: String,
    #[serde(default)]
    usd1RouteConfigId: String,
    #[serde(default)]
    usd1QuoteMetrics: Option<HelperUsd1QuoteMetrics>,
}

fn project_root() -> Result<PathBuf, String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|path| path.parent())
        .map(PathBuf::from)
        .ok_or_else(|| "Failed to resolve LaunchDeck project root.".to_string())
}

fn helper_script_path() -> Result<PathBuf, String> {
    let script_path = project_root()?
        .join("scripts")
        .join("bonk-launchpad.js");
    if script_path.exists() {
        Ok(script_path)
    } else {
        Err(format!(
            "Bonk helper script was not found at {}.",
            script_path.display()
        ))
    }
}

fn bonk_worker_enabled() -> bool {
    helper_worker_enabled("LAUNCHDECK_ENABLE_BONK_HELPER_WORKER")
}

fn worker_client() -> Result<Arc<HelperWorkerClient>, String> {
    static CLIENT: OnceLock<Result<Arc<HelperWorkerClient>, String>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            let project_root = project_root()?;
            let script_path = helper_script_path()?;
            Ok(Arc::new(HelperWorkerClient::new(HelperWorkerConfig {
                helper_name: "Bonk",
                project_root,
                script_path,
                timeout_ms: helper_timeout_ms(),
            })))
        })
        .clone()
}

fn render_usd1_quote_metrics_note(metrics: &HelperUsd1QuoteMetrics) -> Option<String> {
    if metrics.quoteCalls == 0
        && metrics.routeSetupLocalHits == 0
        && metrics.routeSetupCacheHits == 0
        && metrics.routeSetupCacheMisses == 0
        && metrics.superAltLocalSnapshotHits == 0
        && metrics.superAltRpcRefreshes == 0
    {
        return None;
    }
    Some(format!(
        "USD1 quote metrics: calls={} total={}ms avg={:.1}ms quote-cache-hits={} route-setup(local/ttl/miss)={}/{}/{} route-setup-fetch={}ms super-alt(local/rpc-refresh)={}/{} search(expansion/binary/buffer iters)={}/{}/{}/{}",
        metrics.quoteCalls,
        metrics.quoteTotalMs,
        metrics.averageQuoteMs,
        metrics.quoteCacheHits,
        metrics.routeSetupLocalHits,
        metrics.routeSetupCacheHits,
        metrics.routeSetupCacheMisses,
        metrics.routeSetupFetchMs,
        metrics.superAltLocalSnapshotHits,
        metrics.superAltRpcRefreshes,
        metrics.expansionQuoteCalls,
        metrics.binarySearchQuoteCalls,
        metrics.bufferQuoteCalls,
        metrics.searchIterations,
    ))
}

fn bonk_launchpad_program_id() -> Result<Pubkey, String> {
    Pubkey::from_str(BONK_LAUNCHPAD_PROGRAM_ID)
        .map_err(|error| format!("Invalid Bonk launchpad program id: {error}"))
}

fn bonk_quote_mint(quote_asset: &str) -> Result<Pubkey, String> {
    let quote_mint = match quote_asset.trim().to_ascii_lowercase().as_str() {
        "usd1" => BONK_USD1_QUOTE_MINT,
        _ => BONK_SOL_QUOTE_MINT,
    };
    Pubkey::from_str(quote_mint).map_err(|error| format!("Invalid Bonk quote mint address: {error}"))
}

fn bonk_platform_id(mode: &str) -> &'static str {
    match mode.trim().to_ascii_lowercase().as_str() {
        "bonkers" => BONK_BONKERS_PLATFORM_ID,
        _ => BONK_LETSBONK_PLATFORM_ID,
    }
}

fn bonk_u16_be_bytes(value: u16) -> [u8; 2] {
    value.to_be_bytes()
}

fn bonk_launch_config_id(quote_asset: &str) -> Result<String, String> {
    let quote_mint = bonk_quote_mint(quote_asset)?;
    let (config_id, _) = Pubkey::find_program_address(
        &[
            b"global_config",
            quote_mint.as_ref(),
            &[0],
            &bonk_u16_be_bytes(0),
        ],
        &bonk_launchpad_program_id()?,
    );
    Ok(config_id.to_string())
}

fn bonk_quote_asset_config(asset: &str) -> BonkQuoteAssetConfig {
    match asset.trim().to_ascii_lowercase().as_str() {
        "usd1" => BonkQuoteAssetConfig {
            asset: "usd1",
            label: "USD1",
            mint: BONK_USD1_QUOTE_MINT,
            decimals: 6,
        },
        _ => BonkQuoteAssetConfig {
            asset: "sol",
            label: "SOL",
            mint: BONK_SOL_QUOTE_MINT,
            decimals: 9,
        },
    }
}

fn bonk_quote_asset_from_mint_address(address: &str) -> Option<BonkQuoteAssetConfig> {
    let normalized = address.trim();
    if normalized == BONK_SOL_QUOTE_MINT {
        return Some(bonk_quote_asset_config("sol"));
    }
    if normalized == BONK_USD1_QUOTE_MINT {
        return Some(bonk_quote_asset_config("usd1"));
    }
    None
}

fn pool_type_priority(pool_type: &str) -> u8 {
    match pool_type.trim() {
        "Standard" => 0,
        "Concentrated" => 1,
        _ => 2,
    }
}

fn bonk_launch_defaults_cache_ttl() -> Duration {
    Duration::from_millis(
        std::env::var("BONK_LAUNCH_DEFAULTS_CACHE_TTL_MS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(BONK_DEFAULT_LAUNCH_DEFAULTS_CACHE_TTL_MS),
    )
}

fn bonk_launch_defaults_cache() -> &'static Mutex<HashMap<String, BonkLaunchDefaultsCacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, BonkLaunchDefaultsCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn bonk_launch_configs_cache() -> &'static Mutex<HashMap<String, RaydiumLaunchConfigCacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, RaydiumLaunchConfigCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn bonk_usd1_route_setup_cache_ttl() -> Duration {
    Duration::from_millis(
        std::env::var("BONK_USD1_ROUTE_SETUP_CACHE_TTL_MS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(BONK_DEFAULT_USD1_ROUTE_SETUP_CACHE_TTL_MS),
    )
}

fn bonk_lookup_table_cache_ttl() -> Duration {
    Duration::from_millis(
        std::env::var("BONK_LOOKUP_TABLE_CACHE_TTL_MS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(BONK_DEFAULT_LOOKUP_TABLE_CACHE_TTL_MS),
    )
}

fn bonk_usd1_route_setup_cache() -> &'static Mutex<HashMap<String, BonkUsd1RouteSetupCacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, BonkUsd1RouteSetupCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn normalize_bonk_launch_mode(mode: &str) -> &'static str {
    match mode.trim().to_ascii_lowercase().as_str() {
        "bonkers" => "bonkers",
        _ => "regular",
    }
}

fn bonk_launch_configs_endpoint(rpc_url: &str) -> &'static str {
    if rpc_url.to_ascii_lowercase().contains("devnet") {
        RAYDIUM_DEVNET_LAUNCH_CONFIGS_ENDPOINT
    } else {
        RAYDIUM_MAINNET_LAUNCH_CONFIGS_ENDPOINT
    }
}

fn bonk_biguint_from_u64(value: u64) -> BigUint {
    BigUint::from(value)
}

fn bonk_q64() -> BigUint {
    BigUint::from(1u8) << 64usize
}

fn parse_decimal_biguint(value: &str, decimals: u32, label: &str) -> Result<BigUint, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} was empty."));
    }
    if trimmed.starts_with('-') {
        return Err(format!("{label} must be non-negative."));
    }
    let mut parts = trimmed.split('.');
    let whole_raw = parts.next().unwrap_or_default();
    let fraction_raw = parts.next().unwrap_or_default();
    if parts.next().is_some() {
        return Err(format!("Invalid {label}: {trimmed}"));
    }
    if !whole_raw.chars().all(|ch| ch.is_ascii_digit())
        || !fraction_raw.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(format!("Invalid {label}: {trimmed}"));
    }
    let whole = if whole_raw.is_empty() { "0" } else { whole_raw };
    let mut padded_fraction = fraction_raw.to_string();
    if padded_fraction.len() > decimals as usize {
        padded_fraction.truncate(decimals as usize);
    } else if padded_fraction.len() < decimals as usize {
        padded_fraction.push_str(&"0".repeat(decimals as usize - padded_fraction.len()));
    }
    let whole_value = BigUint::parse_bytes(whole.as_bytes(), 10)
        .ok_or_else(|| format!("Invalid {label}: {trimmed}"))?;
    let factor = BigUint::from(10u8).pow(decimals);
    let fraction_value = if padded_fraction.is_empty() {
        BigUint::ZERO
    } else {
        BigUint::parse_bytes(padded_fraction.as_bytes(), 10)
            .ok_or_else(|| format!("Invalid {label}: {trimmed}"))?
    };
    Ok(whole_value * factor + fraction_value)
}

fn parse_biguint_integer(value: &str, label: &str) -> Result<BigUint, String> {
    BigUint::parse_bytes(value.trim().as_bytes(), 10)
        .ok_or_else(|| format!("Invalid {label}: {value}"))
}

fn format_biguint_decimal(value: &BigUint, decimals: u32, max_fraction_digits: u32) -> String {
    if decimals == 0 {
        return value.to_string();
    }
    let raw = value.to_string();
    let width = decimals as usize;
    let (whole, mut fraction) = if raw.len() <= width {
        ("0".to_string(), format!("{raw:0>width$}", width = width))
    } else {
        let split = raw.len() - width;
        (raw[..split].to_string(), raw[split..].to_string())
    };
    fraction.truncate(max_fraction_digits.min(decimals) as usize);
    while fraction.ends_with('0') {
        fraction.pop();
    }
    if fraction.is_empty() {
        whole
    } else {
        format!("{whole}.{fraction}")
    }
}

fn bonk_estimate_supply_percent(amount: &BigUint, supply: &BigUint) -> String {
    if supply == &BigUint::ZERO {
        return "0".to_string();
    }
    let scaled = (amount * BigUint::from(100_000_000u64)) / supply;
    format_biguint_decimal(&scaled, 6, 4)
}

fn bonk_big_sub(left: &BigUint, right: &BigUint, label: &str) -> Result<BigUint, String> {
    if left < right {
        return Err(format!("Bonk {label} underflow."));
    }
    Ok(left - right)
}

fn bonk_ceil_div(amount_a: &BigUint, amount_b: &BigUint) -> BigUint {
    if amount_a == &BigUint::ZERO {
        BigUint::ZERO
    } else {
        (amount_a + amount_b - BigUint::from(1u8)) / amount_b
    }
}

fn bonk_biguint_sqrt_floor(value: &BigUint) -> BigUint {
    if value <= &BigUint::from(1u8) {
        return value.clone();
    }
    let mut current = BigUint::from(1u8) << (value.bits() as usize).div_ceil(2);
    loop {
        let next = (&current + (value / &current)) >> 1usize;
        if next >= current {
            return current;
        }
        current = next;
    }
}

fn bonk_biguint_sqrt_round(value: &BigUint) -> BigUint {
    let floor = bonk_biguint_sqrt_floor(value);
    let floor_squared = &floor * &floor;
    let remainder = value - &floor_squared;
    if remainder > floor {
        floor + BigUint::from(1u8)
    } else {
        floor
    }
}

fn decode_bonk_launchpad_config(data: &[u8]) -> Result<DecodedBonkLaunchpadConfig, String> {
    let mut offset = 0usize;
    let _discriminator = read_bonk_u64(data, &mut offset)?;
    let _epoch = read_bonk_u64(data, &mut offset)?;
    let curve_type = read_bonk_u8(data, &mut offset)?;
    offset += 2;
    let migrate_fee = read_bonk_u64(data, &mut offset)?;
    let trade_fee_rate = read_bonk_u64(data, &mut offset)?;
    Ok(DecodedBonkLaunchpadConfig {
        curve_type,
        migrate_fee,
        trade_fee_rate,
    })
}

fn decode_bonk_platform_config(data: &[u8]) -> Result<DecodedBonkPlatformConfig, String> {
    let mut offset = 0usize;
    let _discriminator = read_bonk_u64(data, &mut offset)?;
    let _epoch = read_bonk_u64(data, &mut offset)?;
    let _platform_claim_fee_wallet = read_bonk_pubkey(data, &mut offset)?;
    let _platform_lock_nft_wallet = read_bonk_pubkey(data, &mut offset)?;
    let _platform_scale = read_bonk_u64(data, &mut offset)?;
    let _creator_scale = read_bonk_u64(data, &mut offset)?;
    let _burn_scale = read_bonk_u64(data, &mut offset)?;
    let fee_rate = read_bonk_u64(data, &mut offset)?;
    offset += 64 + 256 + 256;
    let _cp_config_id = read_bonk_pubkey(data, &mut offset)?;
    let creator_fee_rate = read_bonk_u64(data, &mut offset)?;
    Ok(DecodedBonkPlatformConfig {
        fee_rate,
        creator_fee_rate,
    })
}

fn bonk_total_fee_rate(
    trade_fee_rate: &BigUint,
    platform_fee_rate: &BigUint,
    creator_fee_rate: &BigUint,
) -> Result<BigUint, String> {
    let total = trade_fee_rate + platform_fee_rate + creator_fee_rate;
    if total > bonk_biguint_from_u64(BONK_FEE_RATE_DENOMINATOR) {
        return Err("total fee rate gt 1_000_000".to_string());
    }
    Ok(total)
}

fn bonk_calculate_fee(amount: &BigUint, fee_rate: &BigUint) -> BigUint {
    let numerator = amount * fee_rate;
    bonk_ceil_div(&numerator, &bonk_biguint_from_u64(BONK_FEE_RATE_DENOMINATOR))
}

fn bonk_calculate_pre_fee(post_fee_amount: &BigUint, fee_rate: &BigUint) -> Result<BigUint, String> {
    if fee_rate == &BigUint::ZERO {
        return Ok(post_fee_amount.clone());
    }
    let denominator = bonk_big_sub(
        &bonk_biguint_from_u64(BONK_FEE_RATE_DENOMINATOR),
        fee_rate,
        "fee denominator",
    )?;
    if denominator == BigUint::ZERO {
        return Err("Bonk fee denominator was zero.".to_string());
    }
    let numerator = post_fee_amount * bonk_biguint_from_u64(BONK_FEE_RATE_DENOMINATOR);
    Ok((numerator + &denominator - BigUint::from(1u8)) / denominator)
}

fn bonk_curve_init_virtuals(
    curve_type: u8,
    supply: &BigUint,
    total_fund_raising: &BigUint,
    total_sell: &BigUint,
    total_locked_amount: &BigUint,
    migrate_fee: &BigUint,
) -> Result<(BigUint, BigUint), String> {
    match curve_type {
        0 => {
            if supply <= total_sell {
                return Err("supply need gt total sell".to_string());
            }
            let supply_minus_sell_locked =
                bonk_big_sub(&bonk_big_sub(supply, total_sell, "supply minus total sell")?, total_locked_amount, "supply minus locked amount")?;
            if supply_minus_sell_locked == BigUint::ZERO {
                return Err("supplyMinusSellLocked <= 0".to_string());
            }
            let tf_minus_mf =
                bonk_big_sub(total_fund_raising, migrate_fee, "total fund raising minus migrate fee")?;
            if tf_minus_mf == BigUint::ZERO {
                return Err("tfMinusMf <= 0".to_string());
            }
            let numerator = ((&tf_minus_mf * total_sell) * total_sell) / &supply_minus_sell_locked;
            let denominator_base = (&tf_minus_mf * total_sell) / &supply_minus_sell_locked;
            let denominator =
                bonk_big_sub(&denominator_base, total_fund_raising, "constant-product denominator")?;
            if denominator == BigUint::ZERO {
                return Err("invalid input 0".to_string());
            }
            Ok((numerator / &denominator, (total_fund_raising * total_fund_raising) / denominator))
        }
        1 => {
            let supply_minus_locked =
                bonk_big_sub(supply, total_locked_amount, "supply minus locked amount")?;
            if supply_minus_locked == BigUint::ZERO {
                return Err("invalid input 1".to_string());
            }
            let denominator = bonk_big_sub(
                &(BigUint::from(2u8) * total_fund_raising),
                migrate_fee,
                "fixed-price denominator",
            )?;
            if denominator == BigUint::ZERO {
                return Err("invalid input 0".to_string());
            }
            let total_sell_expect = (total_fund_raising * supply_minus_locked) / &denominator;
            Ok((total_sell_expect, total_fund_raising.clone()))
        }
        2 => {
            let supply_minus_locked =
                bonk_big_sub(supply, total_locked_amount, "supply minus locked amount")?;
            if supply_minus_locked == BigUint::ZERO {
                return Err("supplyMinusLocked need gt 0".to_string());
            }
            let denominator = bonk_big_sub(
                &(BigUint::from(3u8) * total_fund_raising),
                migrate_fee,
                "linear-price denominator",
            )?;
            if denominator == BigUint::ZERO {
                return Err("invalid input 0".to_string());
            }
            let numerator = (BigUint::from(2u8) * total_fund_raising) * supply_minus_locked;
            let total_sell_expect = numerator / &denominator;
            let total_sell_squared = &total_sell_expect * &total_sell_expect;
            if total_sell_squared == BigUint::ZERO {
                return Err("a need gt 0".to_string());
            }
            let a = ((BigUint::from(2u8) * total_fund_raising) * bonk_q64()) / total_sell_squared;
            if a == BigUint::ZERO {
                return Err("a need gt 0".to_string());
            }
            Ok((a, BigUint::ZERO))
        }
        _ => Err("find curve error".to_string()),
    }
}

fn bonk_curve_buy_exact_in(pool: &BonkCurvePoolState, curve_type: u8, amount: &BigUint) -> Result<BigUint, String> {
    match curve_type {
        0 => {
            let input_reserve = &pool.virtual_b + &pool.real_b;
            let output_reserve = bonk_big_sub(&pool.virtual_a, &pool.real_a, "launch output reserve")?;
            Ok((amount * output_reserve) / (input_reserve + amount))
        }
        1 => {
            if pool.virtual_b == BigUint::ZERO {
                return Err("Bonk fixed-price virtual quote reserve was zero.".to_string());
            }
            Ok((&pool.virtual_a * amount) / &pool.virtual_b)
        }
        2 => {
            if pool.virtual_a == BigUint::ZERO {
                return Err("Bonk linear-price virtual coefficient was zero.".to_string());
            }
            let new_quote = &pool.real_b + amount;
            let term_inside_sqrt = (BigUint::from(2u8) * new_quote * bonk_q64()) / &pool.virtual_a;
            let sqrt_term = bonk_biguint_sqrt_round(&term_inside_sqrt);
            bonk_big_sub(&sqrt_term, &pool.real_a, "linear-price amount out")
        }
        _ => Err("find curve error".to_string()),
    }
}

fn bonk_curve_buy_exact_out(pool: &BonkCurvePoolState, curve_type: u8, amount: &BigUint) -> Result<BigUint, String> {
    match curve_type {
        0 => {
            let input_reserve = &pool.virtual_b + &pool.real_b;
            let output_reserve = bonk_big_sub(&pool.virtual_a, &pool.real_a, "launch output reserve")?;
            let denominator = bonk_big_sub(&output_reserve, amount, "launch remaining output reserve")?;
            if denominator == BigUint::ZERO {
                return Err("Bonk constant-product buyExactOut denominator was zero.".to_string());
            }
            Ok(bonk_ceil_div(&(input_reserve * amount), &denominator))
        }
        1 => {
            if pool.virtual_a == BigUint::ZERO {
                return Err("Bonk fixed-price virtual token reserve was zero.".to_string());
            }
            Ok(bonk_ceil_div(&(&pool.virtual_b * amount), &pool.virtual_a))
        }
        2 => {
            let new_base = &pool.real_a + amount;
            let new_base_squared = &new_base * &new_base;
            let denominator = BigUint::from(2u8) * bonk_q64();
            let new_quote = bonk_ceil_div(&(&pool.virtual_a * new_base_squared), &denominator);
            bonk_big_sub(&new_quote, &pool.real_b, "linear-price amount in")
        }
        _ => Err("find curve error".to_string()),
    }
}

fn bonk_quote_buy_exact_in_amount_a(
    defaults: &BonkLaunchDefaults,
    amount_b: &BigUint,
) -> Result<BigUint, String> {
    let fee_rate = bonk_total_fee_rate(
        &defaults.trade_fee_rate,
        &defaults.platform_fee_rate,
        &defaults.creator_fee_rate,
    )?;
    let total_fee = bonk_calculate_fee(amount_b, &fee_rate);
    let amount_less_fee_b = bonk_big_sub(amount_b, &total_fee, "buy input after fee")?;
    let quoted_amount_a =
        bonk_curve_buy_exact_in(&defaults.pool, defaults.curve_type, &amount_less_fee_b)?;
    let remaining_amount_a =
        bonk_big_sub(&defaults.pool.total_sell_a, &defaults.pool.real_a, "remaining sell amount")?;
    if quoted_amount_a > remaining_amount_a {
        Ok(remaining_amount_a)
    } else {
        Ok(quoted_amount_a)
    }
}

fn bonk_quote_buy_exact_out_amount_b(
    defaults: &BonkLaunchDefaults,
    requested_amount_a: &BigUint,
) -> Result<BigUint, String> {
    let remaining_amount_a =
        bonk_big_sub(&defaults.pool.total_sell_a, &defaults.pool.real_a, "remaining sell amount")?;
    let real_amount_a = if requested_amount_a > &remaining_amount_a {
        remaining_amount_a
    } else {
        requested_amount_a.clone()
    };
    let amount_in_less_fee_b =
        bonk_curve_buy_exact_out(&defaults.pool, defaults.curve_type, &real_amount_a)?;
    let fee_rate = bonk_total_fee_rate(
        &defaults.trade_fee_rate,
        &defaults.platform_fee_rate,
        &defaults.creator_fee_rate,
    )?;
    bonk_calculate_pre_fee(&amount_in_less_fee_b, &fee_rate)
}

fn bonk_curve_sell_exact_in(pool: &BonkCurvePoolState, curve_type: u8, amount: &BigUint) -> Result<BigUint, String> {
    match curve_type {
        0 => {
            let input_reserve = bonk_big_sub(&pool.virtual_a, &pool.real_a, "launch input reserve")?;
            let output_reserve = &pool.virtual_b + &pool.real_b;
            Ok((amount * output_reserve) / (input_reserve + amount))
        }
        1 => {
            if pool.virtual_a == BigUint::ZERO {
                return Err("Bonk fixed-price virtual token reserve was zero.".to_string());
            }
            Ok((&pool.virtual_b * amount) / &pool.virtual_a)
        }
        2 => {
            let new_base = bonk_big_sub(&pool.real_a, amount, "linear-price new base")?;
            let new_base_squared = &new_base * &new_base;
            let denominator = BigUint::from(2u8) * bonk_q64();
            let new_quote = bonk_ceil_div(&(&pool.virtual_a * new_base_squared), &denominator);
            bonk_big_sub(&pool.real_b, &new_quote, "linear-price sell output")
        }
        _ => Err("find curve error".to_string()),
    }
}

fn bonk_curve_sell_exact_out(pool: &BonkCurvePoolState, curve_type: u8, amount: &BigUint) -> Result<BigUint, String> {
    match curve_type {
        0 => {
            let input_reserve = bonk_big_sub(&pool.virtual_a, &pool.real_a, "launch input reserve")?;
            let output_reserve = &pool.virtual_b + &pool.real_b;
            let denominator =
                bonk_big_sub(&output_reserve, amount, "launch remaining output reserve")?;
            if denominator == BigUint::ZERO {
                return Err("Bonk constant-product sellExactOut denominator was zero.".to_string());
            }
            Ok(bonk_ceil_div(&(input_reserve * amount), &denominator))
        }
        1 => {
            if pool.virtual_b == BigUint::ZERO {
                return Err("Bonk fixed-price virtual quote reserve was zero.".to_string());
            }
            Ok(bonk_ceil_div(&(&pool.virtual_a * amount), &pool.virtual_b))
        }
        2 => {
            let new_quote = bonk_big_sub(&pool.real_b, amount, "linear-price new quote")?;
            if pool.virtual_a == BigUint::ZERO {
                return Err("Bonk linear-price virtual coefficient was zero.".to_string());
            }
            let term_inside_sqrt = (BigUint::from(2u8) * new_quote * bonk_q64()) / &pool.virtual_a;
            let sqrt_term = bonk_biguint_sqrt_round(&term_inside_sqrt);
            bonk_big_sub(&pool.real_a, &sqrt_term, "linear-price sell input")
        }
        _ => Err("find curve error".to_string()),
    }
}

fn bonk_quote_sell_exact_in_amount_b(
    pool: &BonkCurvePoolState,
    curve_type: u8,
    trade_fee_rate: &BigUint,
    platform_fee_rate: &BigUint,
    creator_fee_rate: &BigUint,
    amount_a: &BigUint,
) -> Result<BigUint, String> {
    let quoted_amount_b = bonk_curve_sell_exact_in(pool, curve_type, amount_a)?;
    let fee_rate = bonk_total_fee_rate(trade_fee_rate, platform_fee_rate, creator_fee_rate)?;
    let total_fee = bonk_calculate_fee(&quoted_amount_b, &fee_rate);
    bonk_big_sub(&quoted_amount_b, &total_fee, "sell output after fee")
}

fn bonk_quote_sell_exact_out_amount_a(
    pool: &BonkCurvePoolState,
    curve_type: u8,
    trade_fee_rate: &BigUint,
    platform_fee_rate: &BigUint,
    creator_fee_rate: &BigUint,
    amount_b: &BigUint,
) -> Result<BigUint, String> {
    let fee_rate = bonk_total_fee_rate(trade_fee_rate, platform_fee_rate, creator_fee_rate)?;
    let amount_out_with_fee_b = bonk_calculate_pre_fee(amount_b, &fee_rate)?;
    if pool.real_b < amount_out_with_fee_b {
        return Err("Insufficient liquidity".to_string());
    }
    let amount_a = bonk_curve_sell_exact_out(pool, curve_type, &amount_out_with_fee_b)?;
    if amount_a > pool.real_a {
        return Err("Insufficient launch token liquidity".to_string());
    }
    Ok(amount_a)
}

fn bonk_build_min_amount_from_bps(amount: &BigUint, slippage_bps: u64) -> BigUint {
    let safe_bps = slippage_bps.min(10_000);
    (amount * BigUint::from(10_000u64 - safe_bps)) / BigUint::from(10_000u64)
}

fn biguint_to_u64(value: &BigUint, label: &str) -> Result<u64, String> {
    value
        .to_string()
        .parse::<u64>()
        .map_err(|error| format!("Invalid Bonk {label}: {error}"))
}

fn biguint_to_u128(value: &BigUint, label: &str) -> Result<u128, String> {
    value
        .to_u128()
        .ok_or_else(|| format!("Invalid Bonk {label}: value exceeds u128"))
}

fn bonk_clmm_program_id() -> Result<Pubkey, String> {
    Pubkey::from_str(BONK_CLMM_PROGRAM_ID).map_err(|error| format!("Invalid Bonk CLMM program id: {error}"))
}

fn bonk_clmm_q64() -> BigUint {
    BigUint::from(1u8) << 64usize
}

fn bonk_clmm_q128() -> BigUint {
    BigUint::from(1u8) << 128usize
}

fn bonk_biguint_from_u128(value: u128) -> BigUint {
    BigUint::from(value)
}

fn bonk_pow10_biguint(decimals: u32) -> BigUint {
    BigUint::from(10u8).pow(decimals)
}

fn bonk_get_tick_array_start_index_by_tick(tick_index: i32, tick_spacing: i32) -> i32 {
    let tick_count = BONK_CLMM_TICK_ARRAY_SIZE * tick_spacing;
    tick_index.div_euclid(tick_count) * tick_count
}

fn bonk_tick_array_bit_position(start_index: i32, tick_spacing: i32) -> Result<usize, String> {
    let tick_count = BONK_CLMM_TICK_ARRAY_SIZE * tick_spacing;
    if tick_count <= 0 || start_index % tick_count != 0 {
        return Err("Invalid Bonk CLMM tick array start index.".to_string());
    }
    let bit_position = start_index.div_euclid(tick_count) + BONK_CLMM_DEFAULT_BITMAP_OFFSET;
    if !(0..1024).contains(&bit_position) {
        return Err("Bonk USD1 CLMM quote exceeded default bitmap coverage.".to_string());
    }
    usize::try_from(bit_position).map_err(|error| format!("Invalid Bonk CLMM bitmap index: {error}"))
}

fn bonk_bitmap_is_initialized(bitmap_words: &[u64; 16], bit_position: usize) -> bool {
    let word = bitmap_words[bit_position / 64];
    (word & (1u64 << (bit_position % 64))) != 0
}

fn bonk_derive_clmm_tick_array_address(
    program_id: &Pubkey,
    pool_id: &Pubkey,
    start_index: i32,
) -> Pubkey {
    let (address, _) = Pubkey::find_program_address(
        &[b"tick_array", pool_id.as_ref(), &start_index.to_be_bytes()],
        program_id,
    );
    address
}

fn bonk_mul_div_floor(left: &BigUint, right: &BigUint, denominator: &BigUint) -> Result<BigUint, String> {
    if denominator == &BigUint::ZERO {
        return Err("Bonk CLMM division by zero.".to_string());
    }
    Ok((left * right) / denominator)
}

fn bonk_mul_div_ceil(left: &BigUint, right: &BigUint, denominator: &BigUint) -> Result<BigUint, String> {
    if denominator == &BigUint::ZERO {
        return Err("Bonk CLMM division by zero.".to_string());
    }
    Ok(((left * right) + denominator - BigUint::from(1u8)) / denominator)
}

fn bonk_get_token_amount_a_from_liquidity(
    mut sqrt_price_a_x64: BigUint,
    mut sqrt_price_b_x64: BigUint,
    liquidity: &BigUint,
    round_up: bool,
) -> Result<BigUint, String> {
    if sqrt_price_a_x64 > sqrt_price_b_x64 {
        std::mem::swap(&mut sqrt_price_a_x64, &mut sqrt_price_b_x64);
    }
    if sqrt_price_a_x64 == BigUint::ZERO {
        return Err("Bonk CLMM sqrt price must be greater than zero.".to_string());
    }
    let numerator1 = liquidity << 64usize;
    let numerator2 = &sqrt_price_b_x64 - &sqrt_price_a_x64;
    if round_up {
        let intermediate = bonk_mul_div_ceil(&numerator1, &numerator2, &sqrt_price_b_x64)?;
        Ok(bonk_ceil_div(&intermediate, &sqrt_price_a_x64))
    } else {
        Ok(bonk_mul_div_floor(&numerator1, &numerator2, &sqrt_price_b_x64)? / &sqrt_price_a_x64)
    }
}

fn bonk_get_token_amount_b_from_liquidity(
    mut sqrt_price_a_x64: BigUint,
    mut sqrt_price_b_x64: BigUint,
    liquidity: &BigUint,
    round_up: bool,
) -> Result<BigUint, String> {
    if sqrt_price_a_x64 > sqrt_price_b_x64 {
        std::mem::swap(&mut sqrt_price_a_x64, &mut sqrt_price_b_x64);
    }
    if sqrt_price_a_x64 == BigUint::ZERO {
        return Err("Bonk CLMM sqrt price must be greater than zero.".to_string());
    }
    if round_up {
        bonk_mul_div_ceil(liquidity, &(&sqrt_price_b_x64 - &sqrt_price_a_x64), &bonk_clmm_q64())
    } else {
        bonk_mul_div_floor(liquidity, &(&sqrt_price_b_x64 - &sqrt_price_a_x64), &bonk_clmm_q64())
    }
}

fn bonk_get_next_sqrt_price_from_token_amount_a_rounding_up(
    sqrt_price_x64: &BigUint,
    liquidity: &BigUint,
    amount: &BigUint,
    add: bool,
) -> Result<BigUint, String> {
    if amount == &BigUint::ZERO {
        return Ok(sqrt_price_x64.clone());
    }
    let liquidity_left_shift = liquidity << 64usize;
    if add {
        let denominator = &liquidity_left_shift + (amount * sqrt_price_x64);
        if denominator >= liquidity_left_shift {
            bonk_mul_div_ceil(&liquidity_left_shift, sqrt_price_x64, &denominator)
        } else {
            let fallback_denominator = (&liquidity_left_shift / sqrt_price_x64) + amount;
            bonk_mul_div_ceil(&liquidity_left_shift, &BigUint::from(1u8), &fallback_denominator)
        }
    } else {
        let amount_mul_sqrt_price = amount * sqrt_price_x64;
        if liquidity_left_shift <= amount_mul_sqrt_price {
            return Err(
                "Bonk CLMM liquidity shift must exceed amount * sqrt price for output quotes."
                    .to_string(),
            );
        }
        let denominator = &liquidity_left_shift - amount_mul_sqrt_price;
        bonk_mul_div_ceil(&liquidity_left_shift, sqrt_price_x64, &denominator)
    }
}

fn bonk_get_next_sqrt_price_from_input_zero_for_one(
    sqrt_price_x64: &BigUint,
    liquidity: &BigUint,
    amount_in: &BigUint,
) -> Result<BigUint, String> {
    bonk_get_next_sqrt_price_from_token_amount_a_rounding_up(sqrt_price_x64, liquidity, amount_in, true)
}

fn bonk_sqrt_price_from_tick(tick: i32) -> Result<BigUint, String> {
    const FACTORS: &[(u32, u64)] = &[
        (0x2, 18_444_899_583_751_176_192),
        (0x4, 18_443_055_278_223_355_904),
        (0x8, 18_439_367_220_385_607_680),
        (0x10, 18_431_993_317_065_453_568),
        (0x20, 18_417_254_355_718_170_624),
        (0x40, 18_387_811_781_193_609_216),
        (0x80, 18_329_067_761_203_558_400),
        (0x100, 18_212_142_134_806_163_456),
        (0x200, 17_980_523_815_641_700_352),
        (0x400, 17_526_086_738_831_433_728),
        (0x800, 16_651_378_430_235_570_176),
        (0x1000, 15_030_750_278_694_412_288),
        (0x2000, 12_247_334_978_884_435_968),
        (0x4000, 8_131_365_268_886_854_656),
        (0x8000, 3_584_323_654_725_218_816),
        (0x10000, 696_457_651_848_324_352),
        (0x20000, 26_294_789_957_507_116),
        (0x40000, 37_481_735_321_082),
    ];

    let tick_abs = tick.unsigned_abs();
    let mut ratio = if (tick_abs & 0x1) != 0 {
        BigUint::from(18_445_821_805_675_395_072u64)
    } else {
        BigUint::from(1u8) << 64usize
    };
    for (mask, factor) in FACTORS {
        if (tick_abs & mask) != 0 {
            ratio = (&ratio * BigUint::from(*factor)) >> 64usize;
        }
    }
    if tick > 0 {
        ratio = (bonk_clmm_q128() - BigUint::from(1u8)) / ratio;
    }
    Ok(ratio)
}

fn bonk_sqrt_price_x64_to_price(
    sqrt_price_x64: &BigUint,
    decimals_a: u32,
    decimals_b: u32,
) -> Result<f64, String> {
    let numerator = sqrt_price_x64 * sqrt_price_x64 * bonk_pow10_biguint(decimals_a);
    let denominator = bonk_clmm_q128() * bonk_pow10_biguint(decimals_b);
    let numerator_f64 = numerator
        .to_f64()
        .ok_or_else(|| "Bonk CLMM numerator was too large to format.".to_string())?;
    let denominator_f64 = denominator
        .to_f64()
        .ok_or_else(|| "Bonk CLMM denominator was too large to format.".to_string())?;
    Ok(numerator_f64 / denominator_f64)
}

fn bonk_apply_liquidity_delta(liquidity: &BigUint, liquidity_net: i128) -> Result<BigUint, String> {
    if liquidity_net >= 0 {
        bonk_big_sub(
            liquidity,
            &bonk_biguint_from_u128(liquidity_net as u128),
            "CLMM liquidity delta",
        )
    } else {
        Ok(liquidity + bonk_biguint_from_u128(liquidity_net.unsigned_abs()))
    }
}

fn bonk_usd1_search_tolerance_lamports(high: &BigUint) -> BigUint {
    let search_tolerance_bps = std::env::var("BONK_USD1_SEARCH_TOLERANCE_BPS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(50);
    let search_min_lamports = std::env::var("BONK_USD1_SEARCH_MIN_LAMPORTS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(50_000);
    let bps_lamports = (high * BigUint::from(search_tolerance_bps)) / BigUint::from(10_000u64);
    bonk_biguint_from_u64(search_min_lamports).max(bps_lamports)
}

fn bonk_add_usd1_search_buffer_lamports(high: &BigUint, max_input_lamports: &BigUint) -> BigUint {
    let search_buffer_bps = std::env::var("BONK_USD1_SEARCH_BUFFER_BPS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(25);
    let search_buffer_min_lamports = std::env::var("BONK_USD1_SEARCH_BUFFER_MIN_LAMPORTS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(25_000);
    let bps_buffer = (high * BigUint::from(search_buffer_bps)) / BigUint::from(10_000u64);
    let buffer = bonk_biguint_from_u64(search_buffer_min_lamports).max(bps_buffer);
    std::cmp::min(high + buffer, max_input_lamports.clone())
}

fn bonk_usd1_min_remaining_lamports() -> Result<u64, String> {
    parse_decimal_u64(
        &std::env::var("BONK_USD1_MIN_REMAINING_SOL").unwrap_or_else(|_| "0.02".to_string()),
        9,
        "BONK_USD1_MIN_REMAINING_SOL",
    )
}

async fn bonk_rpc_get_balance_lamports(rpc_url: &str, owner: &Pubkey) -> Result<u64, String> {
    let response = bonk_http_client()
        .post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [owner.to_string(), "processed"],
        }))
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bonk owner SOL balance: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bonk owner SOL balance: status {}.",
            response.status()
        ));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bonk owner SOL balance response: {error}"))?;
    payload
        .get("result")
        .and_then(|result| result.get("value"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "Bonk owner SOL balance response did not include a numeric value.".to_string())
}

async fn native_prepare_bonk_usd1_topup(
    rpc_url: &str,
    commitment: &str,
    owner: &Pubkey,
    required_quote_amount: &BigUint,
    slippage_bps: u64,
    mut metrics: Option<&mut HelperUsd1QuoteMetrics>,
    route_setup_override: Option<&BonkUsd1RouteSetup>,
) -> Result<NativeBonkPreparedUsd1Topup, String> {
    let usd1_mint = bonk_quote_mint("usd1")?;
    let current_quote_amount = bonk_biguint_from_u64(
        fetch_bonk_owner_token_balance(rpc_url, "processed", owner, &usd1_mint)
            .await?
            .unwrap_or(0),
    );
    if current_quote_amount >= *required_quote_amount {
        return Ok(NativeBonkPreparedUsd1Topup {
            required_quote_amount: required_quote_amount.clone(),
            current_quote_amount,
            shortfall_quote_amount: BigUint::ZERO,
            input_lamports: None,
            expected_quote_out: None,
            min_quote_out: None,
            traversed_tick_array_starts: vec![],
        });
    }
    let shortfall_quote_amount = bonk_big_sub(
        required_quote_amount,
        &current_quote_amount,
        "Bonk USD1 shortfall amount",
    )?;
    let balance_lamports = bonk_rpc_get_balance_lamports(rpc_url, owner).await?;
    let min_remaining_lamports = bonk_usd1_min_remaining_lamports()?;
    let max_spendable_lamports = balance_lamports.saturating_sub(min_remaining_lamports);
    if max_spendable_lamports == 0 {
        return Err(format!(
            "Insufficient SOL headroom for USD1 top-up. Need at least {} SOL reserved after swap.",
            std::env::var("BONK_USD1_MIN_REMAINING_SOL").unwrap_or_else(|_| "0.02".to_string())
        ));
    }
    let input_lamports = native_quote_sol_input_for_usd1_output_with_max_and_metrics(
        rpc_url,
        &shortfall_quote_amount,
        slippage_bps,
        Some(BigUint::from(max_spendable_lamports)),
        metrics.as_deref_mut(),
        route_setup_override,
    )
    .await?;
    let quote = native_quote_usd1_output_from_sol_input_with_metrics(
        rpc_url,
        &input_lamports,
        slippage_bps,
        metrics.as_deref_mut(),
        route_setup_override,
    )
    .await?;
    if quote.min_out < shortfall_quote_amount {
        return Err("Native Bonk USD1 top-up quote could not satisfy required output.".to_string());
    }
    let _ = commitment;
    Ok(NativeBonkPreparedUsd1Topup {
        required_quote_amount: required_quote_amount.clone(),
        current_quote_amount,
        shortfall_quote_amount,
        input_lamports: Some(input_lamports),
        expected_quote_out: Some(quote.expected_out),
        min_quote_out: Some(quote.min_out),
        traversed_tick_array_starts: quote.traversed_tick_array_starts,
    })
}

fn bonk_build_usd1_search_guess_lamports(
    required_quote_amount: &BigUint,
    reference_price: f64,
    max_input_lamports: &BigUint,
) -> Result<BigUint, String> {
    let guess = if reference_price.is_finite() && reference_price > 0.0 {
        let required_quote = required_quote_amount
            .to_f64()
            .ok_or_else(|| "Bonk USD1 quote amount was too large to estimate.".to_string())?
            / 1_000_000f64;
        let guess_sol = ((required_quote / reference_price) * 1.05f64).max(0.01f64);
        parse_decimal_biguint(&format!("{guess_sol:.9}"), 9, "top-up search guess")?
    } else {
        parse_decimal_biguint("0.01", 9, "top-up search floor")?
    };
    Ok(std::cmp::min(guess, max_input_lamports.clone()))
}

async fn rpc_get_multiple_accounts_data(
    rpc_url: &str,
    addresses: &[String],
    commitment: &str,
) -> Result<Vec<Vec<u8>>, String> {
    if addresses.is_empty() {
        return Ok(Vec::new());
    }
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "launchdeck-bonk-multiple-accounts",
        "method": "getMultipleAccounts",
        "params": [
            addresses,
            {
                "encoding": "base64",
                "commitment": commitment,
            }
        ]
    });
    let response = bonk_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bonk multiple accounts: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bonk multiple accounts: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RpcMultipleAccountsResponse = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bonk multiple accounts response: {error}"))?;
    parsed
        .result
        .value
        .into_iter()
        .enumerate()
        .map(|(index, maybe_value)| {
            let value = maybe_value.ok_or_else(|| {
                format!(
                    "Bonk RPC did not return account data for {}.",
                    addresses.get(index).cloned().unwrap_or_default()
                )
            })?;
            BASE64
                .decode(value.data.0.trim())
                .map_err(|error| format!("Failed to decode Bonk account {}: {error}", addresses[index]))
        })
        .collect()
}

async fn load_bonk_usd1_route_setup_with_metrics(
    rpc_url: &str,
    mut metrics: Option<&mut HelperUsd1QuoteMetrics>,
    force_refresh: bool,
) -> Result<BonkUsd1RouteSetup, String> {
    let cache_key = BONK_PINNED_USD1_ROUTE_POOL_ID.to_string();
    let ttl = bonk_usd1_route_setup_cache_ttl();
    if !force_refresh {
        if let Some(entry) = bonk_usd1_route_setup_cache()
            .lock()
            .expect("bonk usd1 route setup cache")
            .get(&cache_key)
            .filter(|entry| entry.fetched_at.elapsed() <= ttl)
            .cloned()
        {
            if let Some(metrics) = metrics.as_deref_mut() {
                metrics.routeSetupCacheHits = metrics.routeSetupCacheHits.saturating_add(1);
            }
            return Ok(entry.setup);
        }
    }
    if let Some(metrics) = metrics.as_deref_mut() {
        metrics.routeSetupCacheMisses = metrics.routeSetupCacheMisses.saturating_add(1);
    }
    let route_fetch_started = std::time::Instant::now();

    let pool_id = Pubkey::from_str(BONK_PINNED_USD1_ROUTE_POOL_ID)
        .map_err(|error| format!("Invalid Bonk USD1 route pool id: {error}"))?;
    let config_id = Pubkey::from_str(BONK_PREFERRED_USD1_ROUTE_CONFIG_ID)
        .map_err(|error| format!("Invalid Bonk USD1 route config id: {error}"))?;
    let program_id = bonk_clmm_program_id()?;

    let pool_data = fetch_account_data(rpc_url, BONK_PINNED_USD1_ROUTE_POOL_ID, "confirmed").await?;
    let pool = decode_bonk_clmm_pool(&pool_data)?;
    if pool.amm_config != config_id {
        return Err(format!(
            "Pinned USD1 route pool config changed: {BONK_PINNED_USD1_ROUTE_POOL_ID}"
        ));
    }
    let mint_a = pool.mint_a.to_string();
    let mint_b = pool.mint_b.to_string();
    let expected_pair = (mint_a == BONK_SOL_QUOTE_MINT && mint_b == BONK_USD1_QUOTE_MINT)
        || (mint_a == BONK_USD1_QUOTE_MINT && mint_b == BONK_SOL_QUOTE_MINT);
    if !expected_pair {
        return Err(format!(
            "Pinned USD1 route pool no longer matches SOL/USD1: {BONK_PINNED_USD1_ROUTE_POOL_ID}"
        ));
    }
    if mint_a != BONK_SOL_QUOTE_MINT || mint_b != BONK_USD1_QUOTE_MINT {
        return Err("Native Bonk USD1 quote currently only supports SOL as CLMM mintA.".to_string());
    }
    let current_array_start = bonk_get_tick_array_start_index_by_tick(pool.tick_current, i32::from(pool.tick_spacing));
    let current_bit_position = bonk_tick_array_bit_position(current_array_start, i32::from(pool.tick_spacing))?;
    if !bonk_bitmap_is_initialized(&pool.tick_array_bitmap, current_bit_position) {
        return Err("Pinned Bonk USD1 CLMM current tick array is not initialized.".to_string());
    }

    let tick_count = BONK_CLMM_TICK_ARRAY_SIZE * i32::from(pool.tick_spacing);
    let tick_array_starts_desc = (0..=current_bit_position)
        .rev()
        .filter(|bit_position| bonk_bitmap_is_initialized(&pool.tick_array_bitmap, *bit_position))
        .map(|bit_position| ((bit_position as i32) - BONK_CLMM_DEFAULT_BITMAP_OFFSET) * tick_count)
        .collect::<Vec<_>>();
    if tick_array_starts_desc.is_empty() {
        return Err("Pinned Bonk USD1 CLMM had no initialized tick arrays.".to_string());
    }

    let tick_array_addresses = tick_array_starts_desc
        .iter()
        .map(|start_index| {
            bonk_derive_clmm_tick_array_address(&program_id, &pool_id, *start_index).to_string()
        })
        .collect::<Vec<_>>();
    let tick_array_account_datas =
        rpc_get_multiple_accounts_data(rpc_url, &tick_array_addresses, "confirmed").await?;
    let tick_arrays = tick_array_account_datas
        .into_iter()
        .map(|data| decode_bonk_clmm_tick_array(&data))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|tick_array| (tick_array.start_tick_index, tick_array))
        .collect::<HashMap<_, _>>();
    if !tick_arrays.contains_key(&current_array_start) {
        return Err("Pinned Bonk USD1 CLMM current tick array could not be decoded.".to_string());
    }

    let config_data = fetch_account_data(rpc_url, BONK_PREFERRED_USD1_ROUTE_CONFIG_ID, "confirmed").await?;
    let config = decode_bonk_clmm_config(&config_data)?;
    if config.tick_spacing != pool.tick_spacing {
        return Err("Pinned Bonk USD1 CLMM tick spacing no longer matches its config.".to_string());
    }

    let setup = BonkUsd1RouteSetup {
        pool_id,
        program_id,
        tick_spacing: i32::from(pool.tick_spacing),
        trade_fee_rate: config.trade_fee_rate,
        sqrt_price_x64: pool.sqrt_price_x64.clone(),
        liquidity: pool.liquidity.clone(),
        tick_current: pool.tick_current,
        mint_a_decimals: u32::from(pool.mint_decimals_a),
        mint_b_decimals: u32::from(pool.mint_decimals_b),
        current_price: bonk_sqrt_price_x64_to_price(
            &pool.sqrt_price_x64,
            u32::from(pool.mint_decimals_a),
            u32::from(pool.mint_decimals_b),
        )?,
        tick_arrays_desc: tick_array_starts_desc,
        tick_arrays,
    };
    bonk_usd1_route_setup_cache()
        .lock()
        .expect("bonk usd1 route setup cache")
        .insert(
            cache_key,
            BonkUsd1RouteSetupCacheEntry {
                fetched_at: std::time::Instant::now(),
                setup: setup.clone(),
            },
        );
    if let Some(metrics) = metrics.as_deref_mut() {
        metrics.routeSetupFetchMs = metrics
            .routeSetupFetchMs
            .saturating_add(route_fetch_started.elapsed().as_millis() as u64);
    }
    Ok(setup)
}

async fn load_bonk_usd1_route_setup(rpc_url: &str) -> Result<BonkUsd1RouteSetup, String> {
    load_bonk_usd1_route_setup_with_metrics(rpc_url, None, false).await
}

async fn load_bonk_usd1_route_setup_fresh(rpc_url: &str) -> Result<BonkUsd1RouteSetup, String> {
    load_bonk_usd1_route_setup_with_metrics(rpc_url, None, true).await
}

fn bonk_find_next_initialized_tick_zero_for_one(
    setup: &BonkUsd1RouteSetup,
    current_tick: i32,
) -> Result<BonkClmmTick, String> {
    let current_array_start = bonk_get_tick_array_start_index_by_tick(current_tick, setup.tick_spacing);
    let current_array = setup
        .tick_arrays
        .get(&current_array_start)
        .ok_or_else(|| format!("Missing Bonk CLMM tick array for start index {current_array_start}."))?;
    let current_tick_position = (current_tick - current_array_start).div_euclid(setup.tick_spacing);
    for tick_index in (0..=current_tick_position).rev() {
        let tick = current_array
            .ticks
            .get(usize::try_from(tick_index).map_err(|error| format!("Invalid Bonk tick index: {error}"))?)
            .ok_or_else(|| "Bonk CLMM current tick array index overflowed.".to_string())?;
        if tick.liquidity_gross > BigUint::ZERO {
            return Ok(tick.clone());
        }
    }
    let current_array_position = setup
        .tick_arrays_desc
        .iter()
        .position(|start_index| *start_index == current_array_start)
        .ok_or_else(|| "Bonk CLMM current tick array was not present in the route setup.".to_string())?;
    setup
        .tick_arrays_desc
        .iter()
        .skip(current_array_position + 1)
        .find_map(|start_index| {
            let tick_array = setup.tick_arrays.get(start_index)?;
            tick_array
                .ticks
                .iter()
                .rev()
                .find(|tick| tick.liquidity_gross > BigUint::ZERO)
                .cloned()
        })
        .ok_or_else(|| "swapCompute LiquidityInsufficient".to_string())
}

fn bonk_clmm_swap_step_exact_in_zero_for_one(
    sqrt_price_current_x64: &BigUint,
    sqrt_price_target_x64: &BigUint,
    liquidity: &BigUint,
    amount_remaining: &BigUint,
    fee_rate: u32,
) -> Result<(BigUint, BigUint, BigUint, BigUint), String> {
    let fee_denominator = bonk_biguint_from_u64(BONK_FEE_RATE_DENOMINATOR);
    let fee_rate_big = bonk_biguint_from_u64(u64::from(fee_rate));
    let amount_remaining_less_fee = (amount_remaining * (&fee_denominator - &fee_rate_big)) / &fee_denominator;
    let amount_in_to_target = bonk_get_token_amount_a_from_liquidity(
        sqrt_price_target_x64.clone(),
        sqrt_price_current_x64.clone(),
        liquidity,
        true,
    )?;
    let next_sqrt_price_x64 = if amount_remaining_less_fee >= amount_in_to_target {
        sqrt_price_target_x64.clone()
    } else {
        bonk_get_next_sqrt_price_from_input_zero_for_one(
            sqrt_price_current_x64,
            liquidity,
            &amount_remaining_less_fee,
        )?
    };
    let reach_target_price = next_sqrt_price_x64 == *sqrt_price_target_x64;
    let amount_in = if reach_target_price {
        amount_in_to_target
    } else {
        bonk_get_token_amount_a_from_liquidity(
            next_sqrt_price_x64.clone(),
            sqrt_price_current_x64.clone(),
            liquidity,
            true,
        )?
    };
    let amount_out = bonk_get_token_amount_b_from_liquidity(
        next_sqrt_price_x64.clone(),
        sqrt_price_current_x64.clone(),
        liquidity,
        false,
    )?;
    let fee_amount = if !reach_target_price {
        bonk_big_sub(amount_remaining, &amount_in, "CLMM swap fee amount")?
    } else {
        bonk_mul_div_ceil(
            &amount_in,
            &fee_rate_big,
            &(&fee_denominator - &fee_rate_big),
        )?
    };
    Ok((next_sqrt_price_x64, amount_in, amount_out, fee_amount))
}

fn bonk_quote_usd1_from_exact_sol_input(
    setup: &BonkUsd1RouteSetup,
    input_lamports: &BigUint,
    slippage_bps: u64,
) -> Result<BonkUsd1DirectQuote, String> {
    if input_lamports == &BigUint::ZERO {
        return Ok(BonkUsd1DirectQuote {
            expected_out: BigUint::ZERO,
            min_out: BigUint::ZERO,
            price_impact_pct: 0.0,
            traversed_tick_array_starts: vec![],
        });
    }
    let mut amount_remaining = input_lamports.clone();
    let mut amount_out_total = BigUint::ZERO;
    let mut sqrt_price_x64 = setup.sqrt_price_x64.clone();
    let mut liquidity = setup.liquidity.clone();
    let mut current_tick = setup.tick_current;
    let mut traversed_tick_array_starts = Vec::new();
    let min_sqrt_price = bonk_biguint_from_u128(BONK_CLMM_MIN_SQRT_PRICE_X64_PLUS_ONE);

    while amount_remaining > BigUint::ZERO && sqrt_price_x64 > min_sqrt_price {
        let current_array_start =
            bonk_get_tick_array_start_index_by_tick(current_tick, setup.tick_spacing);
        if traversed_tick_array_starts
            .last()
            .copied()
            .map(|value| value != current_array_start)
            .unwrap_or(true)
        {
            traversed_tick_array_starts.push(current_array_start);
        }
        let next_tick = bonk_find_next_initialized_tick_zero_for_one(setup, current_tick)?;
        let next_tick_sqrt_price = bonk_sqrt_price_from_tick(next_tick.tick)?;
        let target_sqrt_price = if next_tick_sqrt_price < min_sqrt_price {
            min_sqrt_price.clone()
        } else {
            next_tick_sqrt_price
        };
        let (step_next_sqrt_price, step_amount_in, step_amount_out, step_fee_amount) =
            bonk_clmm_swap_step_exact_in_zero_for_one(
                &sqrt_price_x64,
                &target_sqrt_price,
                &liquidity,
                &amount_remaining,
                setup.trade_fee_rate,
            )?;
        amount_remaining =
            bonk_big_sub(&amount_remaining, &(step_amount_in.clone() + &step_fee_amount), "CLMM remaining input")?;
        amount_out_total += &step_amount_out;
        sqrt_price_x64 = step_next_sqrt_price;
        if sqrt_price_x64 == target_sqrt_price {
            liquidity = bonk_apply_liquidity_delta(&liquidity, next_tick.liquidity_net)?;
            current_tick = next_tick.tick.saturating_sub(1);
        }
    }

    let execution_price = bonk_sqrt_price_x64_to_price(
        &sqrt_price_x64,
        setup.mint_a_decimals,
        setup.mint_b_decimals,
    )?;
    let price_impact_pct = if !setup.current_price.is_finite() || setup.current_price <= 0.0 {
        0.0
    } else {
        ((execution_price - setup.current_price).abs() / setup.current_price) * 100.0
    };
    Ok(BonkUsd1DirectQuote {
        min_out: bonk_build_min_amount_from_bps(&amount_out_total, slippage_bps),
        expected_out: amount_out_total,
        price_impact_pct,
        traversed_tick_array_starts,
    })
}

async fn native_quote_usd1_output_from_sol_input_with_metrics(
    rpc_url: &str,
    input_lamports: &BigUint,
    slippage_bps: u64,
    mut metrics: Option<&mut HelperUsd1QuoteMetrics>,
    route_setup_override: Option<&BonkUsd1RouteSetup>,
) -> Result<BonkUsd1DirectQuote, String> {
    let setup = if let Some(setup) = route_setup_override {
        setup.clone()
    } else {
        load_bonk_usd1_route_setup_with_metrics(rpc_url, metrics.as_deref_mut(), false).await?
    };
    let quote_started = std::time::Instant::now();
    let quote = bonk_quote_usd1_from_exact_sol_input(&setup, input_lamports, slippage_bps)?;
    if let Some(metrics) = metrics.as_deref_mut() {
        metrics.quoteCalls = metrics.quoteCalls.saturating_add(1);
        metrics.quoteTotalMs = metrics
            .quoteTotalMs
            .saturating_add(quote_started.elapsed().as_millis() as u64);
        metrics.averageQuoteMs = if metrics.quoteCalls == 0 {
            0.0
        } else {
            metrics.quoteTotalMs as f64 / metrics.quoteCalls as f64
        };
    }
    Ok(quote)
}

async fn native_quote_usd1_output_from_sol_input(
    rpc_url: &str,
    input_lamports: &BigUint,
    slippage_bps: u64,
) -> Result<BonkUsd1DirectQuote, String> {
    native_quote_usd1_output_from_sol_input_with_metrics(
        rpc_url,
        input_lamports,
        slippage_bps,
        None,
        None,
    )
    .await
}

async fn native_quote_sol_input_for_usd1_output(
    rpc_url: &str,
    required_quote_amount: &BigUint,
    slippage_bps: u64,
) -> Result<BigUint, String> {
    native_quote_sol_input_for_usd1_output_with_max_and_metrics(
        rpc_url,
        required_quote_amount,
        slippage_bps,
        None,
        None,
        None,
    )
    .await
}

async fn native_quote_sol_input_for_usd1_output_with_max_and_metrics(
    rpc_url: &str,
    required_quote_amount: &BigUint,
    slippage_bps: u64,
    max_input_lamports_override: Option<BigUint>,
    mut metrics: Option<&mut HelperUsd1QuoteMetrics>,
    route_setup_override: Option<&BonkUsd1RouteSetup>,
) -> Result<BigUint, String> {
    let quote_started = std::time::Instant::now();
    let setup = if let Some(setup) = route_setup_override {
        setup.clone()
    } else {
        load_bonk_usd1_route_setup_with_metrics(rpc_url, metrics.as_deref_mut(), false).await?
    };
    if !setup.current_price.is_finite() || setup.current_price <= 0.0 {
        return Err(format!(
            "Pinned USD1 route pool has invalid price metadata: {BONK_PINNED_USD1_ROUTE_POOL_ID}"
        ));
    }
    let max_input_lamports = max_input_lamports_override
        .unwrap_or_else(|| bonk_biguint_from_u64(BONK_USD1_QUOTE_MAX_INPUT_LAMPORTS));
    let mut low = BigUint::from(1u8);
    let mut high = bonk_build_usd1_search_guess_lamports(
        required_quote_amount,
        setup.current_price,
        &max_input_lamports,
    )?;
    let mut quote = bonk_quote_usd1_from_exact_sol_input(&setup, &high, slippage_bps)?;
    if let Some(metrics) = metrics.as_deref_mut() {
        metrics.quoteCalls = metrics.quoteCalls.saturating_add(1);
        metrics.expansionQuoteCalls = metrics.expansionQuoteCalls.saturating_add(1);
    }
    while quote.min_out < *required_quote_amount && high < max_input_lamports {
        low = &high + BigUint::from(1u8);
        high = std::cmp::min(high * BigUint::from(2u8), max_input_lamports.clone());
        quote = bonk_quote_usd1_from_exact_sol_input(&setup, &high, slippage_bps)?;
        if let Some(metrics) = metrics.as_deref_mut() {
            metrics.quoteCalls = metrics.quoteCalls.saturating_add(1);
            metrics.expansionQuoteCalls = metrics.expansionQuoteCalls.saturating_add(1);
        }
        if high == max_input_lamports {
            break;
        }
    }
    if quote.min_out < *required_quote_amount {
        return Err(format!(
            "Pinned USD1 route pool could not satisfy required USD1 output: {BONK_PINNED_USD1_ROUTE_POOL_ID}."
        ));
    }
    let max_search_iterations = std::env::var("BONK_USD1_MAX_INPUT_SEARCH_ITERATIONS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(10);
    for _ in 0..max_search_iterations {
        if low >= high || (&high - &low) <= bonk_usd1_search_tolerance_lamports(&high) {
            break;
        }
        let mid = (&low + &high) / BigUint::from(2u8);
        let mid_quote = bonk_quote_usd1_from_exact_sol_input(&setup, &mid, slippage_bps)?;
        if let Some(metrics) = metrics.as_deref_mut() {
            metrics.quoteCalls = metrics.quoteCalls.saturating_add(1);
            metrics.binarySearchQuoteCalls = metrics.binarySearchQuoteCalls.saturating_add(1);
            metrics.searchIterations = metrics.searchIterations.saturating_add(1);
        }
        if mid_quote.min_out >= *required_quote_amount {
            high = mid;
            quote = mid_quote;
        } else {
            low = mid + BigUint::from(1u8);
        }
    }
    let buffered_input = bonk_add_usd1_search_buffer_lamports(&high, &max_input_lamports);
    if buffered_input > high {
        high = buffered_input;
        quote = bonk_quote_usd1_from_exact_sol_input(&setup, &high, slippage_bps)?;
        if let Some(metrics) = metrics.as_deref_mut() {
            metrics.quoteCalls = metrics.quoteCalls.saturating_add(1);
            metrics.bufferQuoteCalls = metrics.bufferQuoteCalls.saturating_add(1);
        }
    }
    if quote.min_out < *required_quote_amount {
        return Err(format!(
            "Pinned USD1 route pool could not satisfy required USD1 output after search buffering: {BONK_PINNED_USD1_ROUTE_POOL_ID}."
        ));
    }
    if let Some(metrics) = metrics.as_deref_mut() {
        metrics.quoteTotalMs = metrics
            .quoteTotalMs
            .saturating_add(quote_started.elapsed().as_millis() as u64);
        metrics.averageQuoteMs = if metrics.quoteCalls == 0 {
            0.0
        } else {
            metrics.quoteTotalMs as f64 / metrics.quoteCalls as f64
        };
    }
    Ok(high)
}

async fn native_quote_sol_input_for_usd1_output_with_max(
    rpc_url: &str,
    required_quote_amount: &BigUint,
    slippage_bps: u64,
    max_input_lamports_override: Option<BigUint>,
    route_setup_override: Option<&BonkUsd1RouteSetup>,
) -> Result<BigUint, String> {
    native_quote_sol_input_for_usd1_output_with_max_and_metrics(
        rpc_url,
        required_quote_amount,
        slippage_bps,
        max_input_lamports_override,
        None,
        route_setup_override,
    )
    .await
}

async fn fetch_raydium_launch_configs(rpc_url: &str) -> Result<Vec<RaydiumLaunchConfigEntry>, String> {
    let endpoint = bonk_launch_configs_endpoint(rpc_url).to_string();
    let ttl = bonk_launch_defaults_cache_ttl();
    if let Some(entry) = bonk_launch_configs_cache()
        .lock()
        .expect("bonk launch config cache")
        .get(&endpoint)
        .filter(|entry| entry.fetched_at.elapsed() <= ttl)
        .cloned()
    {
        return Ok(entry.configs);
    }
    let response = bonk_http_client()
        .get(&endpoint)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Raydium launch configs: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Raydium launch configs: status {}.",
            response.status()
        ));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Raydium launch configs: {error}"))?;
    let configs = parse_raydium_launch_configs_payload(payload)?;
    bonk_launch_configs_cache()
        .lock()
        .expect("bonk launch config cache")
        .insert(
            endpoint,
            RaydiumLaunchConfigCacheEntry {
                fetched_at: std::time::Instant::now(),
                configs: configs.clone(),
            },
        );
    Ok(configs)
}

async fn load_bonk_launch_defaults(
    rpc_url: &str,
    launch_mode: &str,
    quote_asset: &str,
) -> Result<BonkLaunchDefaults, String> {
    let normalized_mode = normalize_bonk_launch_mode(launch_mode);
    let quote = bonk_quote_asset_config(quote_asset);
    let cache_key = format!("{normalized_mode}:{}", quote.asset);
    let ttl = bonk_launch_defaults_cache_ttl();
    if let Some(entry) = bonk_launch_defaults_cache()
        .lock()
        .expect("bonk launch defaults cache")
        .get(&cache_key)
        .filter(|entry| entry.fetched_at.elapsed() <= ttl)
        .cloned()
    {
        return Ok(entry.defaults);
    }
    let config_id = bonk_launch_config_id(quote.asset)?;
    let platform_id = bonk_platform_id(normalized_mode);
    let (config_data, platform_data, launch_configs) = tokio::try_join!(
        fetch_account_data(rpc_url, &config_id, "confirmed"),
        fetch_account_data(rpc_url, platform_id, "confirmed"),
        fetch_raydium_launch_configs(rpc_url),
    )?;
    let config_info = decode_bonk_launchpad_config(&config_data)?;
    let platform_info = decode_bonk_platform_config(&platform_data)?;
    let api_config = launch_configs
        .into_iter()
        .find(|entry| entry.key.pubkey == config_id)
        .ok_or_else(|| format!("Raydium launch config defaults not found for {config_id}"))?;
    let supply = parse_biguint_integer(&api_config.default_params.supply_init, "Bonk launch supply")?;
    let total_sell_a =
        parse_biguint_integer(&api_config.default_params.total_sell_a, "Bonk launch total sell")?;
    let total_fund_raising_b = parse_biguint_integer(
        &api_config.default_params.total_fund_raising_b,
        "Bonk launch total fund raising",
    )?;
    let (virtual_a, virtual_b) = bonk_curve_init_virtuals(
        config_info.curve_type,
        &supply,
        &total_fund_raising_b,
        &total_sell_a,
        &BigUint::ZERO,
        &bonk_biguint_from_u64(config_info.migrate_fee),
    )?;
    let defaults = BonkLaunchDefaults {
        supply,
        total_fund_raising_b: total_fund_raising_b.clone(),
        quote: quote.clone(),
        trade_fee_rate: bonk_biguint_from_u64(config_info.trade_fee_rate),
        platform_fee_rate: bonk_biguint_from_u64(platform_info.fee_rate),
        creator_fee_rate: bonk_biguint_from_u64(platform_info.creator_fee_rate),
        curve_type: config_info.curve_type,
        pool: BonkCurvePoolState {
            total_sell_a,
            virtual_a,
            virtual_b,
            real_a: BigUint::ZERO,
            real_b: BigUint::ZERO,
        },
    };
    bonk_launch_defaults_cache()
        .lock()
        .expect("bonk launch defaults cache")
        .insert(
            cache_key,
            BonkLaunchDefaultsCacheEntry {
                fetched_at: std::time::Instant::now(),
                defaults: defaults.clone(),
            },
        );
    Ok(defaults)
}

fn build_native_bonk_quote_from_defaults(
    defaults: &BonkLaunchDefaults,
    mode: &str,
    amount: &str,
) -> Result<LaunchQuote, String> {
    let normalized_mode = mode.trim().to_ascii_lowercase();
    if normalized_mode == "tokens" {
        let token_amount = parse_decimal_biguint(amount, BONK_TOKEN_DECIMALS, "buy amount")?;
        let quote_amount = bonk_quote_buy_exact_out_amount_b(defaults, &token_amount)?;
        return Ok(LaunchQuote {
            mode: normalized_mode,
            input: amount.to_string(),
            estimatedTokens: format_biguint_decimal(&token_amount, BONK_TOKEN_DECIMALS, 6),
            estimatedSol: format_biguint_decimal(&quote_amount, defaults.quote.decimals, 6),
            estimatedQuoteAmount: format_biguint_decimal(&quote_amount, defaults.quote.decimals, 6),
            quoteAsset: defaults.quote.asset.to_string(),
            quoteAssetLabel: defaults.quote.label.to_string(),
            estimatedSupplyPercent: bonk_estimate_supply_percent(&token_amount, &defaults.supply),
        });
    }
    let buy_amount = parse_decimal_biguint(
        amount,
        defaults.quote.decimals,
        &format!("buy amount {}", defaults.quote.label),
    )?;
    let token_amount = bonk_quote_buy_exact_in_amount_a(defaults, &buy_amount)?;
    Ok(LaunchQuote {
        mode: normalized_mode,
        input: amount.to_string(),
        estimatedTokens: format_biguint_decimal(&token_amount, BONK_TOKEN_DECIMALS, 6),
        estimatedSol: format_biguint_decimal(&buy_amount, defaults.quote.decimals, 6),
        estimatedQuoteAmount: format_biguint_decimal(&buy_amount, defaults.quote.decimals, 6),
        quoteAsset: defaults.quote.asset.to_string(),
        quoteAssetLabel: defaults.quote.label.to_string(),
        estimatedSupplyPercent: bonk_estimate_supply_percent(&token_amount, &defaults.supply),
    })
}

async fn native_quote_launch(
    rpc_url: &str,
    quote_asset: &str,
    launch_mode: &str,
    mode: &str,
    amount: &str,
) -> Result<LaunchQuote, String> {
    let defaults = load_bonk_launch_defaults(rpc_url, launch_mode, quote_asset).await?;
    if defaults.quote.asset == "usd1" {
        let slippage_bps = 0u64;
        let normalized_mode = mode.trim().to_ascii_lowercase();
        if normalized_mode == "tokens" {
            let token_amount = parse_decimal_biguint(amount, BONK_TOKEN_DECIMALS, "buy amount")?;
            let required_quote_amount = bonk_quote_buy_exact_out_amount_b(&defaults, &token_amount)?;
            let quoted_sol_input =
                native_quote_sol_input_for_usd1_output(rpc_url, &required_quote_amount, slippage_bps)
                    .await?;
            return Ok(LaunchQuote {
                mode: normalized_mode,
                input: amount.to_string(),
                estimatedTokens: format_biguint_decimal(&token_amount, BONK_TOKEN_DECIMALS, 6),
                estimatedSol: format_biguint_decimal(&quoted_sol_input, 9, 6),
                estimatedQuoteAmount: format_biguint_decimal(&quoted_sol_input, 9, 6),
                quoteAsset: "sol".to_string(),
                quoteAssetLabel: "SOL".to_string(),
                estimatedSupplyPercent: bonk_estimate_supply_percent(&token_amount, &defaults.supply),
            });
        }
        let input_sol = parse_decimal_biguint(amount, 9, "buy amount SOL")?;
        let usd1_route_quote =
            native_quote_usd1_output_from_sol_input(rpc_url, &input_sol, slippage_bps).await?;
        let token_amount = bonk_quote_buy_exact_in_amount_a(&defaults, &usd1_route_quote.min_out)?;
        return Ok(LaunchQuote {
            mode: normalized_mode,
            input: amount.to_string(),
            estimatedTokens: format_biguint_decimal(&token_amount, BONK_TOKEN_DECIMALS, 6),
            estimatedSol: format_biguint_decimal(&input_sol, 9, 6),
            estimatedQuoteAmount: format_biguint_decimal(&input_sol, 9, 6),
            quoteAsset: "sol".to_string(),
            quoteAssetLabel: "SOL".to_string(),
            estimatedSupplyPercent: bonk_estimate_supply_percent(&token_amount, &defaults.supply),
        });
    }
    build_native_bonk_quote_from_defaults(&defaults, mode, amount)
}

async fn native_predict_dev_buy_effect(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<Option<BonkPredictedDevBuyEffect>, String> {
    let Some(dev_buy) = config.devBuy.as_ref() else {
        return Ok(None);
    };
    let dev_buy_mode = dev_buy.mode.trim().to_ascii_lowercase();
    if dev_buy_mode.is_empty() || dev_buy.amount.trim().is_empty() {
        return Ok(None);
    }
    let defaults = load_bonk_launch_defaults(rpc_url, &config.mode, &config.quoteAsset).await?;
    let requested_amount_b = if dev_buy_mode == "tokens" {
        let requested_tokens =
            parse_decimal_biguint(&dev_buy.amount, BONK_TOKEN_DECIMALS, "dev buy tokens")?;
        bonk_quote_buy_exact_out_amount_b(&defaults, &requested_tokens)?
    } else if defaults.quote.asset == "usd1" {
        let input_sol = parse_decimal_biguint(&dev_buy.amount, 9, "dev buy SOL")?;
        native_quote_usd1_output_from_sol_input(
            rpc_url,
            &input_sol,
            slippage_bps_from_percent(&config.execution.buySlippagePercent)?,
        )
        .await?
        .min_out
    } else {
        parse_decimal_biguint(
            &dev_buy.amount,
            defaults.quote.decimals,
            &format!("dev buy {}", defaults.quote.label),
        )?
    };
    let mint = Pubkey::new_unique();
    let creator = Pubkey::new_unique();
    let pool_context = build_prelaunch_bonk_pool_context(&defaults, &mint, &creator, &config.mode)?;
    let details = bonk_follow_buy_quote_details(
        &pool_context,
        biguint_to_u64(&requested_amount_b, "predicted dev buy quote amount")?,
        slippage_bps_from_percent(&config.execution.buySlippagePercent)?,
    )?;
    Ok(Some(BonkPredictedDevBuyEffect {
        requested_quote_amount_b: details.gross_input_b,
        token_amount: details.amount_a,
    }))
}

async fn native_predict_dev_buy_token_amount(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<Option<u64>, String> {
    Ok(native_predict_dev_buy_effect(rpc_url, config)
        .await?
        .map(|effect| effect.token_amount))
}

fn read_bonk_u8(data: &[u8], offset: &mut usize) -> Result<u8, String> {
    let value = data
        .get(*offset)
        .copied()
        .ok_or_else(|| "Bonk launchpad account was too short.".to_string())?;
    *offset += 1;
    Ok(value)
}

fn read_bonk_u64(data: &[u8], offset: &mut usize) -> Result<u64, String> {
    let bytes = data
        .get(*offset..(*offset + 8))
        .ok_or_else(|| "Bonk launchpad account was too short.".to_string())?;
    *offset += 8;
    let array: [u8; 8] = bytes
        .try_into()
        .map_err(|_| "Bonk launchpad account returned an invalid u64 field.".to_string())?;
    Ok(u64::from_le_bytes(array))
}

fn read_bonk_u16(data: &[u8], offset: &mut usize) -> Result<u16, String> {
    let bytes = data
        .get(*offset..(*offset + 2))
        .ok_or_else(|| "Bonk account was too short.".to_string())?;
    *offset += 2;
    let array: [u8; 2] = bytes
        .try_into()
        .map_err(|_| "Bonk account returned an invalid u16 field.".to_string())?;
    Ok(u16::from_le_bytes(array))
}

fn read_bonk_u32(data: &[u8], offset: &mut usize) -> Result<u32, String> {
    let bytes = data
        .get(*offset..(*offset + 4))
        .ok_or_else(|| "Bonk account was too short.".to_string())?;
    *offset += 4;
    let array: [u8; 4] = bytes
        .try_into()
        .map_err(|_| "Bonk account returned an invalid u32 field.".to_string())?;
    Ok(u32::from_le_bytes(array))
}

fn read_bonk_i32(data: &[u8], offset: &mut usize) -> Result<i32, String> {
    let bytes = data
        .get(*offset..(*offset + 4))
        .ok_or_else(|| "Bonk account was too short.".to_string())?;
    *offset += 4;
    let array: [u8; 4] = bytes
        .try_into()
        .map_err(|_| "Bonk account returned an invalid i32 field.".to_string())?;
    Ok(i32::from_le_bytes(array))
}

fn read_bonk_u128(data: &[u8], offset: &mut usize) -> Result<BigUint, String> {
    let bytes = data
        .get(*offset..(*offset + 16))
        .ok_or_else(|| "Bonk account was too short.".to_string())?;
    *offset += 16;
    Ok(BigUint::from_bytes_le(bytes))
}

fn read_bonk_i128(data: &[u8], offset: &mut usize) -> Result<i128, String> {
    let bytes = data
        .get(*offset..(*offset + 16))
        .ok_or_else(|| "Bonk account was too short.".to_string())?;
    *offset += 16;
    let array: [u8; 16] = bytes
        .try_into()
        .map_err(|_| "Bonk account returned an invalid i128 field.".to_string())?;
    Ok(i128::from_le_bytes(array))
}

fn read_bonk_pubkey(data: &[u8], offset: &mut usize) -> Result<Pubkey, String> {
    let bytes = data
        .get(*offset..(*offset + 32))
        .ok_or_else(|| "Bonk launchpad account was too short.".to_string())?;
    *offset += 32;
    let array: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "Bonk launchpad account returned an invalid pubkey field.".to_string())?;
    Ok(Pubkey::new_from_array(array))
}

fn decode_bonk_clmm_config(data: &[u8]) -> Result<DecodedBonkClmmConfig, String> {
    let mut offset = 0usize;
    offset += 8;
    let _bump = read_bonk_u8(data, &mut offset)?;
    let _index = read_bonk_u16(data, &mut offset)?;
    let _fund_owner = read_bonk_pubkey(data, &mut offset)?;
    let _protocol_fee_rate = read_bonk_u32(data, &mut offset)?;
    let trade_fee_rate = read_bonk_u32(data, &mut offset)?;
    let tick_spacing = read_bonk_u16(data, &mut offset)?;
    Ok(DecodedBonkClmmConfig {
        trade_fee_rate,
        tick_spacing,
    })
}

fn decode_bonk_clmm_pool(data: &[u8]) -> Result<DecodedBonkClmmPool, String> {
    let mut offset = 0usize;
    offset += 8;
    let _bump = read_bonk_u8(data, &mut offset)?;
    let amm_config = read_bonk_pubkey(data, &mut offset)?;
    let _creator = read_bonk_pubkey(data, &mut offset)?;
    let mint_a = read_bonk_pubkey(data, &mut offset)?;
    let mint_b = read_bonk_pubkey(data, &mut offset)?;
    let _vault_a = read_bonk_pubkey(data, &mut offset)?;
    let _vault_b = read_bonk_pubkey(data, &mut offset)?;
    let _observation_id = read_bonk_pubkey(data, &mut offset)?;
    let mint_decimals_a = read_bonk_u8(data, &mut offset)?;
    let mint_decimals_b = read_bonk_u8(data, &mut offset)?;
    let tick_spacing = read_bonk_u16(data, &mut offset)?;
    let liquidity = read_bonk_u128(data, &mut offset)?;
    let sqrt_price_x64 = read_bonk_u128(data, &mut offset)?;
    let tick_current = read_bonk_i32(data, &mut offset)?;
    let _padding = read_bonk_u32(data, &mut offset)?;
    offset += 16 + 16;
    offset += 8 + 8;
    offset += 16 + 16 + 16 + 16;
    let _status = read_bonk_u8(data, &mut offset)?;
    offset += 7;
    offset += 3 * 169;
    let mut tick_array_bitmap = [0u64; 16];
    for word in &mut tick_array_bitmap {
        *word = read_bonk_u64(data, &mut offset)?;
    }
    Ok(DecodedBonkClmmPool {
        amm_config,
        mint_a,
        mint_b,
        mint_decimals_a,
        mint_decimals_b,
        tick_spacing,
        liquidity,
        sqrt_price_x64,
        tick_current,
        tick_array_bitmap,
    })
}

fn decode_bonk_clmm_tick_array(data: &[u8]) -> Result<BonkClmmTickArray, String> {
    let mut offset = 0usize;
    offset += 8;
    let _pool_id = read_bonk_pubkey(data, &mut offset)?;
    let start_tick_index = read_bonk_i32(data, &mut offset)?;
    let mut ticks = Vec::with_capacity(usize::try_from(BONK_CLMM_TICK_ARRAY_SIZE).unwrap_or(60));
    for _ in 0..BONK_CLMM_TICK_ARRAY_SIZE {
        let tick = read_bonk_i32(data, &mut offset)?;
        let liquidity_net = read_bonk_i128(data, &mut offset)?;
        let liquidity_gross = read_bonk_u128(data, &mut offset)?;
        offset += 16 + 16 + (3 * 16) + (13 * 4);
        ticks.push(BonkClmmTick {
            tick,
            liquidity_net,
            liquidity_gross,
        });
    }
    let _initialized_tick_count = read_bonk_u8(data, &mut offset)?;
    Ok(BonkClmmTickArray {
        start_tick_index,
        ticks,
    })
}

fn decode_bonk_launchpad_pool(data: &[u8]) -> Result<DecodedBonkLaunchpadPool, String> {
    let mut offset = 0usize;
    let _discriminator = read_bonk_u64(data, &mut offset)?;
    let _epoch = read_bonk_u64(data, &mut offset)?;
    let _bump = read_bonk_u8(data, &mut offset)?;
    let status = read_bonk_u8(data, &mut offset)?;
    let _mint_decimals_a = read_bonk_u8(data, &mut offset)?;
    let _mint_decimals_b = read_bonk_u8(data, &mut offset)?;
    let _migrate_type = read_bonk_u8(data, &mut offset)?;
    let supply = read_bonk_u64(data, &mut offset)?;
    let total_sell_a = read_bonk_u64(data, &mut offset)?;
    let virtual_a = read_bonk_u64(data, &mut offset)?;
    let virtual_b = read_bonk_u64(data, &mut offset)?;
    let real_a = read_bonk_u64(data, &mut offset)?;
    let real_b = read_bonk_u64(data, &mut offset)?;
    let _total_fund_raising_b = read_bonk_u64(data, &mut offset)?;
    let _protocol_fee = read_bonk_u64(data, &mut offset)?;
    let _platform_fee = read_bonk_u64(data, &mut offset)?;
    let _migrate_fee = read_bonk_u64(data, &mut offset)?;
    for _ in 0..5 {
        let _ = read_bonk_u64(data, &mut offset)?;
    }
    let config_id = read_bonk_pubkey(data, &mut offset)?;
    let platform_id = read_bonk_pubkey(data, &mut offset)?;
    let mint_a = read_bonk_pubkey(data, &mut offset)?;
    let _mint_b = read_bonk_pubkey(data, &mut offset)?;
    let _vault_a = read_bonk_pubkey(data, &mut offset)?;
    let _vault_b = read_bonk_pubkey(data, &mut offset)?;
    let creator = read_bonk_pubkey(data, &mut offset)?;
    Ok(DecodedBonkLaunchpadPool {
        creator,
        status,
        supply,
        config_id,
        total_sell_a,
        virtual_a,
        virtual_b,
        real_a,
        real_b,
        platform_id,
        mint_a,
    })
}

async fn fetch_launchpad_pool_candidate(
    rpc_url: &str,
    mint: &Pubkey,
    asset: &str,
) -> Result<Option<BonkMarketCandidate>, String> {
    let quote = bonk_quote_asset_config(asset);
    let pool_id = derive_canonical_pool_id(quote.asset, &mint.to_string()).await?;
    let account_data = match fetch_account_data(rpc_url, &pool_id, "processed").await {
        Ok(data) => data,
        Err(error) if error.contains("was not found.") => return Ok(None),
        Err(error) => return Err(error),
    };
    let pool = decode_bonk_launchpad_pool(&account_data)?;
    Ok(Some(BonkMarketCandidate {
        mode: if pool.platform_id.to_string() == BONK_BONKERS_PLATFORM_ID {
            "bonkers".to_string()
        } else {
            "regular".to_string()
        },
        quote_asset: quote.asset.to_string(),
        quote_asset_label: quote.label.to_string(),
        creator: pool.creator.to_string(),
        platform_id: pool.platform_id.to_string(),
        config_id: pool.config_id.to_string(),
        pool_id,
        real_quote_reserves: pool.real_b,
        complete: pool.status != 0,
        detection_source: "raydium-launchpad".to_string(),
        launch_migrate_pool: false,
        tvl: 0.0,
        pool_type: "LaunchLab".to_string(),
        launchpad_pool: Some(pool),
        raydium_pool: None,
    }))
}

async fn fetch_migrated_raydium_candidates(
    mint: &Pubkey,
) -> Result<Vec<BonkMarketCandidate>, String> {
    let client = bonk_http_client();
    let mint_string = mint.to_string();
    let mut candidates = Vec::new();
    for asset in ["sol", "usd1"] {
        let quote = bonk_quote_asset_config(asset);
        let (mint1, mint2) = if mint_string.as_str() > quote.mint {
            (quote.mint.to_string(), mint_string.clone())
        } else {
            (mint_string.clone(), quote.mint.to_string())
        };
        let response = client
            .get(RAYDIUM_POOL_SEARCH_MINT_ENDPOINT)
            .query(&[
                ("mint1", mint1.as_str()),
                ("mint2", mint2.as_str()),
                ("poolType", "all"),
                ("poolSortField", "default"),
                ("sortType", "desc"),
                ("pageSize", "100"),
                ("page", "1"),
            ])
            .send()
            .await
            .map_err(|error| format!("Failed to query Raydium migrated pools: {error}"))?;
        if !response.status().is_success() {
            continue;
        }
        let payload: RaydiumPoolsResponse = response
            .json()
            .await
            .map_err(|error| format!("Failed to parse Raydium migrated pool response: {error}"))?;
        for pool in payload.data {
            let quote_meta = bonk_quote_asset_from_mint_address(if !pool.mint_a.address.is_empty() {
                &pool.mint_a.address
            } else {
                &pool.mint_b.address
            });
            let Some(quote_meta) = quote_meta else {
                continue;
            };
            let lowered_pool_type = pool.pool_type.trim().to_ascii_lowercase();
            candidates.push(BonkMarketCandidate {
                mode: "regular".to_string(),
                quote_asset: quote_meta.asset.to_string(),
                quote_asset_label: quote_meta.label.to_string(),
                creator: String::new(),
                platform_id: String::new(),
                config_id: pool
                    .config
                    .as_ref()
                    .map(|config| config.id.clone())
                    .unwrap_or_default(),
                pool_id: pool.id.clone(),
                real_quote_reserves: pool.tvl.max(0.0).round() as u64 * 1_000_000,
                complete: true,
                detection_source: format!(
                    "raydium-{}",
                    if lowered_pool_type.is_empty() {
                        "migrated"
                    } else {
                        lowered_pool_type.as_str()
                    }
                ),
                launch_migrate_pool: pool.launch_migrate_pool,
                tvl: pool.tvl,
                pool_type: pool.pool_type.clone(),
                launchpad_pool: None,
                raydium_pool: Some(pool),
            });
        }
    }
    Ok(candidates)
}

async fn detect_bonk_market_candidates(
    rpc_url: &str,
    mint: &Pubkey,
) -> Result<Vec<BonkMarketCandidate>, String> {
    let mut launchpad_candidates = Vec::new();
    for asset in ["sol", "usd1"] {
        if let Some(candidate) = fetch_launchpad_pool_candidate(rpc_url, mint, asset).await? {
            launchpad_candidates.push(candidate);
        }
    }
    if !launchpad_candidates.iter().any(|candidate| candidate.complete) {
        return Ok(launchpad_candidates);
    }
    let migrated_candidates = fetch_migrated_raydium_candidates(mint).await?;
    if migrated_candidates.is_empty() {
        Ok(launchpad_candidates)
    } else {
        Ok(migrated_candidates)
    }
}

fn compare_bonk_market_candidates(
    left: &BonkMarketCandidate,
    right: &BonkMarketCandidate,
    preferred_quote_asset: &str,
) -> Ordering {
    let left_canonical = if left.launch_migrate_pool { 1 } else { 0 };
    let right_canonical = if right.launch_migrate_pool { 1 } else { 0 };
    right_canonical
        .cmp(&left_canonical)
        .then_with(|| {
            let left_liquidity = if left.tvl > 0.0 {
                left.tvl
            } else {
                left.real_quote_reserves as f64
            };
            let right_liquidity = if right.tvl > 0.0 {
                right.tvl
            } else {
                right.real_quote_reserves as f64
            };
            right_liquidity
                .partial_cmp(&left_liquidity)
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| {
            let left_requested =
                (!preferred_quote_asset.is_empty() && left.quote_asset == preferred_quote_asset)
                    as u8;
            let right_requested =
                (!preferred_quote_asset.is_empty() && right.quote_asset == preferred_quote_asset)
                    as u8;
            right_requested.cmp(&left_requested)
        })
        .then_with(|| pool_type_priority(&left.pool_type).cmp(&pool_type_priority(&right.pool_type)))
        .then_with(|| {
            if left.quote_asset == right.quote_asset {
                Ordering::Equal
            } else if left.quote_asset == "sol" {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
}

fn select_preferred_bonk_market_candidate<'a>(
    candidates: &'a [BonkMarketCandidate],
    preferred_quote_asset: &str,
) -> Option<&'a BonkMarketCandidate> {
    let normalized_preferred = preferred_quote_asset.trim().to_ascii_lowercase();
    candidates.iter().min_by(|left, right| {
        compare_bonk_market_candidates(left, right, normalized_preferred.as_str())
    })
}

async fn fetch_token_supply_value(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
) -> Result<RpcTokenSupplyValue, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "launchdeck-bonk-token-supply",
        "method": "getTokenSupply",
        "params": [
            mint.to_string(),
            {
                "commitment": commitment,
            }
        ]
    });
    let response = bonk_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bonk token supply: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bonk token supply: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RpcResponse<RpcTokenSupplyResult> = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bonk token supply response: {error}"))?;
    Ok(parsed.result.value)
}

fn format_decimal_u128(value: u128, decimals: u32, max_fraction_digits: u32) -> String {
    let base = 10u128.pow(decimals);
    let whole = value / base;
    let fraction = value % base;
    if fraction == 0 {
        return whole.to_string();
    }
    let width = decimals as usize;
    let mut fraction_text = format!("{fraction:0width$}");
    fraction_text.truncate(max_fraction_digits.min(decimals) as usize);
    while fraction_text.ends_with('0') {
        fraction_text.pop();
    }
    if fraction_text.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{fraction_text}")
    }
}

fn build_launchpad_market_snapshot(candidate: &BonkMarketCandidate) -> Result<BonkMarketSnapshot, String> {
    let pool = candidate
        .launchpad_pool
        .as_ref()
        .ok_or_else(|| "Missing Bonk launchpad pool candidate.".to_string())?;
    let market_cap_lamports = if pool.virtual_a == 0 {
        0
    } else {
        (u128::from(pool.supply) * u128::from(pool.virtual_b)) / u128::from(pool.virtual_a)
    };
    Ok(BonkMarketSnapshot {
        mint: pool.mint_a.to_string(),
        creator: candidate.creator.clone(),
        virtualTokenReserves: pool.virtual_a.to_string(),
        virtualSolReserves: pool.virtual_b.to_string(),
        realTokenReserves: pool.total_sell_a.saturating_sub(pool.real_a).to_string(),
        realSolReserves: pool.real_b.to_string(),
        tokenTotalSupply: pool.supply.to_string(),
        complete: candidate.complete,
        marketCapLamports: market_cap_lamports.to_string(),
        marketCapSol: format_decimal_u128(
            market_cap_lamports,
            bonk_quote_asset_config(&candidate.quote_asset).decimals,
            6,
        ),
        quoteAsset: candidate.quote_asset.clone(),
        quoteAssetLabel: candidate.quote_asset_label.clone(),
    })
}

fn market_cap_from_raydium_pool_price(
    pool: &RaydiumPoolInfo,
    token_supply: u128,
    token_decimals: u32,
    quote: &BonkQuoteAssetConfig,
) -> Result<u128, String> {
    let price = pool.price;
    if !price.is_finite() || price <= 0.0 {
        return Err(format!(
            "Invalid Raydium migrated pool price for {}: {}",
            pool.id, pool.price
        ));
    }
    let scale = 10f64.powi(18);
    let scaled_price = (price * scale).round();
    if !scaled_price.is_finite() || scaled_price <= 0.0 {
        return Err(format!(
            "Invalid Raydium migrated pool price for {}: {}",
            pool.id, pool.price
        ));
    }
    let scaled_price = scaled_price as u128;
    let token_supply_big = bonk_biguint_from_u128(token_supply);
    let scaled_price_big = bonk_biguint_from_u128(scaled_price);
    let token_scale_big = bonk_pow10_biguint(token_decimals);
    let quote_scale_big = bonk_pow10_biguint(quote.decimals);
    let price_scale_big = bonk_pow10_biguint(18);
    if pool.mint_a.address == quote.mint {
        let market_cap = (((&token_supply_big * &price_scale_big) * &quote_scale_big)
            / &scaled_price_big)
            / &token_scale_big;
        return biguint_to_u128(&market_cap, &format!("migrated market cap for {}", pool.id));
    }
    if pool.mint_b.address == quote.mint {
        let market_cap = (((&token_supply_big * &scaled_price_big) * &quote_scale_big)
            / &price_scale_big)
            / &token_scale_big;
        return biguint_to_u128(&market_cap, &format!("migrated market cap for {}", pool.id));
    }
    Err(format!(
        "Migrated Raydium pool {} does not match requested quote asset {}.",
        pool.id, quote.asset
    ))
}

async fn build_migrated_raydium_market_snapshot(
    rpc_url: &str,
    mint: &Pubkey,
    candidate: &BonkMarketCandidate,
) -> Result<BonkMarketSnapshot, String> {
    let pool = candidate
        .raydium_pool
        .as_ref()
        .ok_or_else(|| "Missing Raydium migrated pool candidate.".to_string())?;
    let supply = fetch_token_supply_value(rpc_url, mint, "processed").await?;
    let token_supply = supply.amount.trim().parse::<u128>().map_err(|error| {
        format!(
            "Invalid Bonk token supply amount for {}: {error}",
            mint
        )
    })?;
    let quote = bonk_quote_asset_config(&candidate.quote_asset);
    let market_cap_lamports =
        market_cap_from_raydium_pool_price(pool, token_supply, supply.decimals, &quote)?;
    Ok(BonkMarketSnapshot {
        mint: mint.to_string(),
        creator: candidate.creator.clone(),
        virtualTokenReserves: "0".to_string(),
        virtualSolReserves: "0".to_string(),
        realTokenReserves: "0".to_string(),
        realSolReserves: "0".to_string(),
        tokenTotalSupply: token_supply.to_string(),
        complete: true,
        marketCapLamports: market_cap_lamports.to_string(),
        marketCapSol: format_decimal_u128(market_cap_lamports, quote.decimals, 6),
        quoteAsset: candidate.quote_asset.clone(),
        quoteAssetLabel: candidate.quote_asset_label.clone(),
    })
}

async fn native_fetch_bonk_market_snapshot(
    rpc_url: &str,
    mint: &str,
    quote_asset: &str,
) -> Result<BonkMarketSnapshot, String> {
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bonk mint address: {error}"))?;
    let candidates = detect_bonk_market_candidates(rpc_url, &mint_pubkey).await?;
    let preferred = select_preferred_bonk_market_candidate(&candidates, quote_asset)
        .ok_or_else(|| format!("No Bonk market candidate found for {mint}."))?;
    if preferred.raydium_pool.is_some() {
        build_migrated_raydium_market_snapshot(rpc_url, &mint_pubkey, preferred).await
    } else {
        build_launchpad_market_snapshot(preferred)
    }
}

async fn native_detect_bonk_import_context_with_quote_asset(
    rpc_url: &str,
    mint: &str,
    quote_asset: &str,
) -> Result<Option<BonkImportContext>, String> {
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bonk mint address: {error}"))?;
    let candidates = detect_bonk_market_candidates(rpc_url, &mint_pubkey).await?;
    let Some(preferred) = select_preferred_bonk_market_candidate(&candidates, quote_asset) else {
        return Ok(None);
    };
    Ok(Some(BonkImportContext {
        launchpad: "bonk".to_string(),
        mode: preferred.mode.clone(),
        quoteAsset: preferred.quote_asset.clone(),
        creator: preferred.creator.clone(),
        platformId: preferred.platform_id.clone(),
        configId: preferred.config_id.clone(),
        poolId: preferred.pool_id.clone(),
        detectionSource: preferred.detection_source.clone(),
    }))
}

async fn native_detect_bonk_import_context(
    rpc_url: &str,
    mint: &str,
) -> Result<Option<BonkImportContext>, String> {
    native_detect_bonk_import_context_with_quote_asset(rpc_url, mint, "").await
}

fn parse_helper_output<R: DeserializeOwned>(
    output_stdout: &[u8],
    helper_name: &str,
) -> Result<R, String> {
    let stdout = String::from_utf8_lossy(output_stdout);
    let trimmed = stdout.trim();
    let mut candidates = Vec::new();
    if !trimmed.is_empty() {
        candidates.push(trimmed.to_string());
        if let Some(last_line) = trimmed.lines().rev().find(|line| !line.trim().is_empty()) {
            let last_line = last_line.trim();
            if !last_line.is_empty() && !candidates.iter().any(|entry| entry == last_line) {
                candidates.push(last_line.to_string());
            }
        }
        for (index, ch) in trimmed.char_indices().rev() {
            if ch != '{' && ch != '[' {
                continue;
            }
            let candidate = trimmed[index..].trim();
            if candidate.is_empty() || candidates.iter().any(|entry| entry == candidate) {
                continue;
            }
            candidates.push(candidate.to_string());
            if candidates.len() >= 12 {
                break;
            }
        }
    }
    for candidate in &candidates {
        if let Ok(parsed) = serde_json::from_str::<R>(candidate) {
            return Ok(parsed);
        }
    }
    let preview = trimmed.chars().take(240).collect::<String>();
    Err(format!(
        "Failed to parse {helper_name} helper output. stdout preview: {}",
        if preview.is_empty() {
            "(empty)".to_string()
        } else {
            preview.replace('\n', "\\n")
        }
    ))
}

async fn run_helper_once<T: Serialize, R: DeserializeOwned>(request: &T) -> Result<R, String> {
    let _permit = helper_semaphore()
        .acquire_owned()
        .await
        .map_err(|_| "Bonk helper semaphore closed unexpectedly.".to_string())?;
    let script_path = helper_script_path()?;
    let request_bytes = serde_json::to_vec(request).map_err(|error| error.to_string())?;
    let mut child = Command::new("node")
        .arg(script_path)
        .current_dir(project_root()?)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start Bonk helper: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&request_bytes)
            .await
            .map_err(|error| format!("Failed to send Bonk helper request: {error}"))?;
    }
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Bonk helper stdout was unavailable.".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Bonk helper stderr was unavailable.".to_string())?;
    let stdout_task = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stdout
            .read_to_end(&mut bytes)
            .await
            .map(|_| bytes)
            .map_err(|error| error.to_string())
    });
    let stderr_task = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stderr
            .read_to_end(&mut bytes)
            .await
            .map(|_| bytes)
            .map_err(|error| error.to_string())
    });
    let status = match timeout(Duration::from_millis(helper_timeout_ms()), child.wait()).await {
        Ok(result) => result.map_err(|error| format!("Bonk helper failed to complete: {error}"))?,
        Err(_) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err(format!(
                "Bonk helper timed out after {}ms.",
                helper_timeout_ms()
            ));
        }
    };
    let output_stdout = stdout_task
        .await
        .map_err(|error| format!("Bonk helper stdout task failed: {error}"))?
        .map_err(|error| format!("Failed to read Bonk helper stdout: {error}"))?;
    let output_stderr = stderr_task
        .await
        .map_err(|error| format!("Bonk helper stderr task failed: {error}"))?
        .map_err(|error| format!("Failed to read Bonk helper stderr: {error}"))?;
    if !status.success() {
        let stderr = String::from_utf8_lossy(&output_stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Bonk helper exited with a non-zero status.".to_string()
        } else {
            format!("Bonk helper error: {stderr}")
        });
    }
    parse_helper_output(&output_stdout, "Bonk")
}

async fn run_helper<T: Serialize, R: DeserializeOwned>(request: &T) -> Result<R, String> {
    if bonk_worker_enabled() {
        match worker_client()?.request::<T, R>(request).await {
            Ok(response) => return Ok(response),
            Err(HelperWorkerError::Request(error)) => return Err(error),
            Err(HelperWorkerError::Transport(error)) => {
                eprintln!("Bonk worker transport failed, falling back to one-shot helper: {error}");
            }
        }
    }
    run_helper_once(request).await
}

fn helper_tx_config(
    compute_unit_limit: Option<u64>,
    compute_unit_price_micro_lamports: u64,
    tip_lamports: u64,
    tip_account: &str,
    jitodontfront: bool,
    single_bundle_tip_last_tx: bool,
) -> HelperTxConfig<'_> {
    HelperTxConfig {
        computeUnitLimit: compute_unit_limit
            .unwrap_or_else(configured_default_launch_compute_unit_limit),
        computeUnitPriceMicroLamports: compute_unit_price_micro_lamports,
        tipLamports: tip_lamports,
        tipAccount: tip_account,
        jitodontfront,
        singleBundleTipLastTx: single_bundle_tip_last_tx,
    }
}

fn uses_single_bundle_tip_last_tx(provider: &str, mev_mode: &str) -> bool {
    provider.trim().eq_ignore_ascii_case("hellomoon")
        && mev_mode.trim().eq_ignore_ascii_case("secure")
}

fn provider_uses_follow_tip(provider: &str) -> bool {
    matches!(
        provider.trim().to_ascii_lowercase().as_str(),
        "helius-sender" | "hellomoon" | "jito-bundle"
    )
}

const HELLOMOON_MIN_FOLLOW_TIP_LAMPORTS: u64 = 1_000_000;

fn resolve_follow_tip_lamports(provider: &str, tip_sol: &str, label: &str) -> Result<u64, String> {
    if !provider_uses_follow_tip(provider) {
        return Ok(0);
    }
    if provider.trim().eq_ignore_ascii_case("hellomoon") && tip_sol.trim().is_empty() {
        return Err(format!(
            "{label} cannot be empty when using Hello Moon for follow / snipe / auto-sell."
        ));
    }
    let tip_lamports = parse_decimal_u64(tip_sol, 9, label)?;
    if provider.trim().eq_ignore_ascii_case("hellomoon")
        && tip_lamports < HELLOMOON_MIN_FOLLOW_TIP_LAMPORTS
    {
        return Err(format!(
            "{label} must be at least 0.001 SOL when using Hello Moon for follow / snipe / auto-sell."
        ));
    }
    Ok(tip_lamports)
}

fn parse_decimal_u64(value: &str, decimals: u32, label: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }
    let parsed = trimmed
        .parse::<f64>()
        .map_err(|error| format!("Invalid {label}: {error}"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(format!("Invalid {label}: expected a non-negative decimal."));
    }
    let scale = 10u64.saturating_pow(decimals);
    let scaled = parsed * scale as f64;
    if scaled > u64::MAX as f64 {
        return Err(format!("{label} is too large."));
    }
    Ok(scaled.round() as u64)
}

fn priority_fee_sol_to_micro_lamports(priority_fee_sol: &str) -> Result<u64, String> {
    let lamports = parse_decimal_u64(priority_fee_sol, 9, "priority fee")?;
    if lamports == 0 {
        Ok(0)
    } else {
        Ok((lamports.saturating_mul(1_000_000)) / PRIORITY_FEE_PRICE_BASE_COMPUTE_UNIT_LIMIT)
    }
}

fn slippage_bps_from_percent(slippage_percent: &str) -> Result<u64, String> {
    let percent = parse_decimal_u64(slippage_percent, 2, "slippage percent")?;
    Ok(percent.min(10_000))
}

fn decode_secret_base64(secret: &[u8]) -> String {
    format!("base64:{}", BASE64.encode(secret))
}

async fn normalize_vanity_secret_for_helper(
    rpc_url: &str,
    raw_secret: &str,
) -> Result<Option<String>, String> {
    let trimmed = raw_secret.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let bytes =
        read_keypair_bytes(trimmed).map_err(|error| format!("Invalid vanity private key: {error}"))?;
    let keypair = solana_sdk::signature::Keypair::try_from(bytes.as_slice())
        .map_err(|error| format!("Invalid vanity private key: {error}"))?;
    let public_key = bs58::encode(&keypair.to_bytes()[32..]).into_string();
    match fetch_account_data(rpc_url, &public_key, "confirmed").await {
        Ok(_) => {
            return Err(format!(
                "This vanity address has already been used on-chain. Generate a fresh one. ({})",
                public_key
            ));
        }
        Err(error) if error.contains("was not found.") => {}
        Err(error) => {
            return Err(format!(
                "Failed to verify vanity private key availability: {error}"
            ));
        }
    }
    Ok(Some(format!(
        "base58:{}",
        bs58::encode(keypair.to_bytes()).into_string()
    )))
}

fn convert_compiled_transaction(source: HelperCompiledTransaction) -> CompiledTransaction {
    let signature = crate::rpc::precompute_transaction_signature(&source.serializedBase64);
    CompiledTransaction {
        label: source.label,
        format: source.format,
        blockhash: source.blockhash,
        lastValidBlockHeight: source.lastValidBlockHeight,
        serializedBase64: source.serializedBase64,
        signature,
        lookupTablesUsed: source.lookupTablesUsed,
        computeUnitLimit: source.computeUnitLimit,
        computeUnitPriceMicroLamports: source.computeUnitPriceMicroLamports,
        inlineTipLamports: source.inlineTipLamports,
        inlineTipAccount: source.inlineTipAccount,
    }
}

fn parse_owner_keypair(secret: &[u8]) -> Result<Keypair, String> {
    Keypair::try_from(secret).map_err(|error| format!("Invalid owner secret key: {error}"))
}

fn compute_budget_program_id() -> Result<Pubkey, String> {
    Pubkey::from_str(COMPUTE_BUDGET_PROGRAM_ID)
        .map_err(|error| format!("Invalid Compute Budget program id: {error}"))
}

fn bonk_follow_tx_config(
    compute_unit_limit: u64,
    compute_unit_price_micro_lamports: u64,
    tip_lamports: u64,
    tip_account: &str,
) -> Result<NativeBonkTxConfig, String> {
    Ok(NativeBonkTxConfig {
        compute_unit_limit: u32::try_from(compute_unit_limit)
            .map_err(|_| "Bonk compute unit limit exceeded u32.".to_string())?,
        compute_unit_price_micro_lamports,
        tip_lamports,
        tip_account: tip_account.to_string(),
    })
}

fn bonk_launch_tx_config(config: &NormalizedConfig) -> Result<NativeBonkTxConfig, String> {
    Ok(NativeBonkTxConfig {
        compute_unit_limit: u32::try_from(
            config
                .tx
                .computeUnitLimit
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or_else(configured_default_launch_compute_unit_limit),
        )
        .map_err(|error| format!("Invalid Bonk launch compute unit limit: {error}"))?,
        compute_unit_price_micro_lamports: u64::try_from(
            config.tx.computeUnitPriceMicroLamports.unwrap_or_default().max(0),
        )
        .unwrap_or_default(),
        tip_lamports: u64::try_from(config.tx.jitoTipLamports.max(0)).unwrap_or_default(),
        tip_account: config.tx.jitoTipAccount.clone(),
    })
}

fn select_bonk_native_tx_format(requested: &str) -> NativeBonkTxFormat {
    if requested.trim().eq_ignore_ascii_case("legacy") {
        NativeBonkTxFormat::Legacy
    } else {
        NativeBonkTxFormat::V0
    }
}

fn bonk_bundle_tx_config_for_index(
    tx_config: &NativeBonkTxConfig,
    index: usize,
    total: usize,
    single_bundle_tip_last_tx: bool,
) -> NativeBonkTxConfig {
    if !single_bundle_tip_last_tx || total <= 1 || index + 1 == total {
        return tx_config.clone();
    }
    let mut adjusted = tx_config.clone();
    adjusted.tip_lamports = 0;
    adjusted.tip_account.clear();
    adjusted
}

fn bonk_label_for_bundle_index(label_prefix: &str, index: usize, total: usize) -> String {
    if total <= 1 {
        label_prefix.to_string()
    } else {
        format!("{label_prefix}-{}", index + 1)
    }
}

fn build_compute_unit_limit_instruction(compute_unit_limit: u32) -> Result<Instruction, String> {
    let mut data = vec![2];
    data.extend_from_slice(&compute_unit_limit.to_le_bytes());
    Ok(Instruction {
        program_id: compute_budget_program_id()?,
        accounts: vec![],
        data,
    })
}

fn build_compute_unit_price_instruction(micro_lamports: u64) -> Result<Instruction, String> {
    let mut data = vec![3];
    data.extend_from_slice(&micro_lamports.to_le_bytes());
    Ok(Instruction {
        program_id: compute_budget_program_id()?,
        accounts: vec![],
        data,
    })
}

fn apply_jitodontfront(mut instructions: Vec<Instruction>, enabled: bool) -> Result<Vec<Instruction>, String> {
    if !enabled {
        return Ok(instructions);
    }
    let dontfront = Pubkey::from_str(JITODONTFRONT_ACCOUNT)
        .map_err(|error| format!("Invalid jitodontfront account: {error}"))?;
    for instruction in &mut instructions {
        if instruction.accounts.iter().any(|meta| meta.pubkey == dontfront) {
            continue;
        }
        instruction
            .accounts
            .push(AccountMeta::new_readonly(dontfront, false));
    }
    Ok(instructions)
}

fn with_bonk_tx_settings(
    core_instructions: Vec<Instruction>,
    tx_config: &NativeBonkTxConfig,
    payer: &Pubkey,
    jitodontfront_enabled: bool,
) -> Result<Vec<Instruction>, String> {
    let mut instructions = vec![build_compute_unit_limit_instruction(
        tx_config.compute_unit_limit,
    )?];
    if tx_config.compute_unit_price_micro_lamports > 0 {
        instructions.push(build_compute_unit_price_instruction(
            tx_config.compute_unit_price_micro_lamports,
        )?);
    }
    instructions.extend(apply_jitodontfront(
        core_instructions,
        jitodontfront_enabled,
    )?);
    if tx_config.tip_lamports > 0 && !tx_config.tip_account.trim().is_empty() {
        let tip_account = Pubkey::from_str(tx_config.tip_account.trim())
            .map_err(|error| format!("Invalid Jito tip account: {error}"))?;
        instructions.push(solana_system_interface::instruction::transfer(
            payer,
            &tip_account,
            tx_config.tip_lamports,
        ));
    }
    Ok(instructions)
}

fn build_bonk_compiled_transaction(
    label: &str,
    tx_format: NativeBonkTxFormat,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instructions: Vec<Instruction>,
    tx_config: &NativeBonkTxConfig,
) -> Result<CompiledTransaction, String> {
    let hash = Hash::from_str(blockhash).map_err(|error| error.to_string())?;
    let mut signers = Vec::with_capacity(1 + extra_signers.len());
    signers.push(payer);
    signers.extend(extra_signers.iter().copied());
    let serialized = if tx_format == NativeBonkTxFormat::Legacy {
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &signers,
            hash,
        );
        bincode::serialize(&transaction).map_err(|error| error.to_string())?
    } else {
        let message = v0::Message::try_compile(&payer.pubkey(), &instructions, &[], hash)
            .map_err(|error| error.to_string())?;
        let transaction = VersionedTransaction::try_new(VersionedMessage::V0(message), &signers)
            .map_err(|error| error.to_string())?;
        bincode::serialize(&transaction).map_err(|error| error.to_string())?
    };
    let serialized_base64 = BASE64.encode(serialized);
    let signature = crate::rpc::precompute_transaction_signature(&serialized_base64);
    Ok(CompiledTransaction {
        label: label.to_string(),
        format: if tx_format == NativeBonkTxFormat::Legacy {
            "legacy".to_string()
        } else {
            "v0".to_string()
        },
        blockhash: blockhash.to_string(),
        lastValidBlockHeight: last_valid_block_height,
        serializedBase64: serialized_base64,
        signature,
        lookupTablesUsed: vec![],
        computeUnitLimit: Some(u64::from(tx_config.compute_unit_limit)),
        computeUnitPriceMicroLamports: if tx_config.compute_unit_price_micro_lamports > 0 {
            Some(tx_config.compute_unit_price_micro_lamports)
        } else {
            None
        },
        inlineTipLamports: if tx_config.tip_lamports > 0 {
            Some(tx_config.tip_lamports)
        } else {
            None
        },
        inlineTipAccount: if tx_config.tip_lamports > 0 && !tx_config.tip_account.trim().is_empty() {
            Some(tx_config.tip_account.clone())
        } else {
            None
        },
    })
}

fn bonk_instruction_required_extra_signers<'a>(
    payer: &Keypair,
    instructions: &[Instruction],
    extra_signers: &'a [&'a Keypair],
) -> Vec<&'a Keypair> {
    let mut required = Vec::new();
    for signer in extra_signers {
        if signer.pubkey() == payer.pubkey()
            || required
                .iter()
                .any(|entry: &&Keypair| entry.pubkey() == signer.pubkey())
        {
            continue;
        }
        if instructions.iter().any(|instruction| {
            instruction
                .accounts
                .iter()
                .any(|meta| meta.is_signer && meta.pubkey == signer.pubkey())
        }) {
            required.push(*signer);
        }
    }
    required
}

fn bonk_compiled_transaction_fits(
    tx_format: NativeBonkTxFormat,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instructions: Vec<Instruction>,
    tx_config: &NativeBonkTxConfig,
) -> Result<bool, String> {
    match build_bonk_compiled_transaction(
        "__size-check__",
        tx_format,
        blockhash,
        last_valid_block_height,
        payer,
        extra_signers,
        instructions,
        tx_config,
    ) {
        Ok(compiled) => {
            let raw = BASE64
                .decode(compiled.serializedBase64.as_bytes())
                .map_err(|error| format!("Failed to decode Bonk compiled transaction: {error}"))?;
            Ok(raw.len() <= PACKET_LIMIT_BYTES)
        }
        Err(error) => Err(error),
    }
}

fn compile_bonk_instruction_bundle(
    label_prefix: &str,
    tx_format: NativeBonkTxFormat,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instruction_groups: Vec<Vec<Instruction>>,
    tx_config: &NativeBonkTxConfig,
    jitodontfront_enabled: bool,
    single_bundle_tip_last_tx: bool,
    preferred_lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<CompiledTransaction>, String> {
    let total = instruction_groups.len();
    instruction_groups
        .into_iter()
        .enumerate()
        .map(|(index, instructions)| {
            let group_tx_config =
                bonk_bundle_tx_config_for_index(tx_config, index, total, single_bundle_tip_last_tx);
            let tx_instructions = with_bonk_tx_settings(
                instructions.clone(),
                &group_tx_config,
                &payer.pubkey(),
                jitodontfront_enabled,
            )?;
            let required_signers =
                bonk_instruction_required_extra_signers(payer, &instructions, extra_signers);
            build_bonk_compiled_transaction_with_lookup_preference(
                &bonk_label_for_bundle_index(label_prefix, index, total),
                tx_format,
                blockhash,
                last_valid_block_height,
                payer,
                &required_signers,
                tx_instructions,
                &group_tx_config,
                &[],
                preferred_lookup_tables,
            )
        })
        .collect()
}

fn split_bonk_instruction_bundle(
    label_prefix: &str,
    tx_format: NativeBonkTxFormat,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instructions: Vec<Instruction>,
    tx_config: &NativeBonkTxConfig,
    jitodontfront_enabled: bool,
    single_bundle_tip_last_tx: bool,
    preferred_lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<CompiledTransaction>, String> {
    let mut groups: Vec<Vec<Instruction>> = Vec::new();
    let mut queue: Vec<Instruction> = Vec::new();
    for instruction in instructions {
        if queue.is_empty() {
            queue.push(instruction);
            continue;
        }
        let mut candidate = queue.clone();
        candidate.push(instruction.clone());
        let preview_instructions = with_bonk_tx_settings(
            candidate.clone(),
            tx_config,
            &payer.pubkey(),
            jitodontfront_enabled,
        )?;
        let preview_signers =
            bonk_instruction_required_extra_signers(payer, &candidate, extra_signers);
        let fits = preview_instructions.len() <= 12
            && bonk_compiled_transaction_fits_with_lookup_preference(
                "__size-check__",
                tx_format,
                blockhash,
                last_valid_block_height,
                payer,
                &preview_signers,
                preview_instructions,
                tx_config,
                &[],
                preferred_lookup_tables,
            )?;
        if fits {
            queue = candidate;
        } else {
            if queue.is_empty() {
                return Err("Bonk launch instruction bundle contained an oversized instruction.".to_string());
            }
            groups.push(queue);
            queue = vec![instruction];
        }
    }
    if !queue.is_empty() {
        groups.push(queue);
    }
    compile_bonk_instruction_bundle(
        label_prefix,
        tx_format,
        blockhash,
        last_valid_block_height,
        payer,
        extra_signers,
        groups,
        tx_config,
        jitodontfront_enabled,
        single_bundle_tip_last_tx,
        preferred_lookup_tables,
    )
}

fn build_bonk_v0_compiled_transaction_with_lookup_tables(
    label: &str,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instructions: Vec<Instruction>,
    tx_config: &NativeBonkTxConfig,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<CompiledTransaction, String> {
    let hash = Hash::from_str(blockhash).map_err(|error| error.to_string())?;
    let message = v0::Message::try_compile(&payer.pubkey(), &instructions, lookup_tables, hash)
        .map_err(|error| error.to_string())?;
    let lookup_tables_used = message
        .address_table_lookups
        .iter()
        .map(|lookup| lookup.account_key.to_string())
        .collect::<Vec<_>>();
    let mut signers = Vec::with_capacity(1 + extra_signers.len());
    signers.push(payer);
    signers.extend(extra_signers.iter().copied());
    let transaction =
        VersionedTransaction::try_new(VersionedMessage::V0(message), &signers).map_err(|error| error.to_string())?;
    let serialized = bincode::serialize(&transaction).map_err(|error| error.to_string())?;
    if serialized.len() > PACKET_LIMIT_BYTES {
        return Err(format!(
            "Atomic USD1 action exceeded packet limits after serialize: raw {} > {} bytes",
            serialized.len(),
            PACKET_LIMIT_BYTES
        ));
    }
    let serialized_base64 = BASE64.encode(serialized);
    let signature = crate::rpc::precompute_transaction_signature(&serialized_base64);
    Ok(CompiledTransaction {
        label: label.to_string(),
        format: "v0".to_string(),
        blockhash: blockhash.to_string(),
        lastValidBlockHeight: last_valid_block_height,
        serializedBase64: serialized_base64,
        signature,
        lookupTablesUsed: lookup_tables_used,
        computeUnitLimit: Some(u64::from(tx_config.compute_unit_limit)),
        computeUnitPriceMicroLamports: if tx_config.compute_unit_price_micro_lamports > 0 {
            Some(tx_config.compute_unit_price_micro_lamports)
        } else {
            None
        },
        inlineTipLamports: if tx_config.tip_lamports > 0 {
            Some(tx_config.tip_lamports)
        } else {
            None
        },
        inlineTipAccount: if tx_config.tip_lamports > 0 && !tx_config.tip_account.trim().is_empty() {
            Some(tx_config.tip_account.clone())
        } else {
            None
        },
    })
}

fn is_compute_budget_instruction(instruction: &Instruction) -> bool {
    instruction.program_id == compute_budget_program_id().unwrap_or_default()
}

fn is_inline_tip_instruction(
    instruction: &Instruction,
    owner_pubkey: &Pubkey,
    tip_account: &str,
    tip_lamports: u64,
) -> bool {
    if tip_account.trim().is_empty() || tip_lamports == 0 {
        return false;
    }
    if instruction.program_id != solana_system_interface::program::ID || instruction.accounts.len() < 2 {
        return false;
    }
    let Ok(system_instruction) =
        bincode::deserialize::<solana_system_interface::instruction::SystemInstruction>(
            &instruction.data,
        )
    else {
        return false;
    };
    match system_instruction {
        solana_system_interface::instruction::SystemInstruction::Transfer { lamports } => {
            instruction.accounts[0].pubkey == *owner_pubkey
                && instruction.accounts[0].is_signer
                && instruction.accounts[1].pubkey
                    == match Pubkey::from_str(tip_account.trim()) {
                        Ok(value) => value,
                        Err(_) => return false,
                    }
                && lamports == tip_lamports
        }
        _ => false,
    }
}

async fn load_lookup_table_account_for_bonk_transaction(
    rpc_url: &str,
    address: &Pubkey,
    commitment: &str,
) -> Result<AddressLookupTableAccount, String> {
    let data = fetch_account_data(rpc_url, &address.to_string(), commitment).await?;
    let table = AddressLookupTable::deserialize(&data)
        .map_err(|error| format!("Failed to decode address lookup table {address}: {error}"))?;
    Ok(AddressLookupTableAccount {
        key: *address,
        addresses: table.addresses.to_vec(),
    })
}

async fn resolve_lookup_table_accounts_for_bonk_transaction(
    rpc_url: &str,
    transaction: &VersionedTransaction,
    commitment: &str,
) -> Result<Vec<AddressLookupTableAccount>, String> {
    let Some(lookups) = transaction.message.address_table_lookups() else {
        return Ok(vec![]);
    };
    let mut resolved = Vec::with_capacity(lookups.len());
    for lookup in lookups {
        resolved.push(
            load_lookup_table_account_for_bonk_transaction(rpc_url, &lookup.account_key, commitment)
                .await?,
        );
    }
    Ok(resolved)
}

fn resolve_bonk_transaction_account_keys(
    transaction: &VersionedTransaction,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<Pubkey>, String> {
    let mut account_keys = transaction.message.static_account_keys().to_vec();
    let Some(lookups) = transaction.message.address_table_lookups() else {
        return Ok(account_keys);
    };
    let mut writable = Vec::new();
    let mut readonly = Vec::new();
    for lookup in lookups {
        let table = lookup_tables
            .iter()
            .find(|table| table.key == lookup.account_key)
            .ok_or_else(|| format!("Address lookup table not found: {}", lookup.account_key))?;
        for index in &lookup.writable_indexes {
            let address = table
                .addresses
                .get(usize::from(*index))
                .ok_or_else(|| format!("Writable ALT index {index} was out of bounds for {}", table.key))?;
            writable.push(*address);
        }
        for index in &lookup.readonly_indexes {
            let address = table
                .addresses
                .get(usize::from(*index))
                .ok_or_else(|| format!("Readonly ALT index {index} was out of bounds for {}", table.key))?;
            readonly.push(*address);
        }
    }
    account_keys.extend(writable);
    account_keys.extend(readonly);
    Ok(account_keys)
}

fn decompile_bonk_versioned_transaction_instructions(
    transaction: &VersionedTransaction,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<Instruction>, String> {
    let account_keys = resolve_bonk_transaction_account_keys(transaction, lookup_tables)?;
    let mut instructions = Vec::new();
    for compiled in transaction.message.instructions() {
        let program_id = account_keys
            .get(usize::from(compiled.program_id_index))
            .copied()
            .ok_or_else(|| "Bonk transaction referenced a missing program account.".to_string())?;
        let mut accounts = Vec::with_capacity(compiled.accounts.len());
        for account_index in &compiled.accounts {
            let index = usize::from(*account_index);
            let pubkey = account_keys
                .get(index)
                .copied()
                .ok_or_else(|| "Bonk transaction referenced a missing account meta.".to_string())?;
            accounts.push(AccountMeta {
                pubkey,
                is_signer: transaction.message.is_signer(index),
                is_writable: transaction.message.is_maybe_writable(index, None),
            });
        }
        instructions.push(Instruction {
            program_id,
            accounts,
            data: compiled.data.clone(),
        });
    }
    Ok(instructions)
}

fn decode_bonk_versioned_transaction(encoded: &str) -> Result<VersionedTransaction, String> {
    let bytes = BASE64
        .decode(encoded.trim())
        .map_err(|error| format!("Failed to decode Bonk transaction payload: {error}"))?;
    bincode::deserialize::<VersionedTransaction>(&bytes)
        .map_err(|error| format!("Failed to deserialize Bonk versioned transaction: {error}"))
}

async fn decompose_bonk_compiled_v0_transaction(
    rpc_url: &str,
    transaction: &CompiledTransaction,
    commitment: &str,
) -> Result<DecomposedBonkVersionedTransaction, String> {
    let decoded = decode_bonk_versioned_transaction(&transaction.serializedBase64)?;
    let lookup_tables =
        resolve_lookup_table_accounts_for_bonk_transaction(rpc_url, &decoded, commitment).await?;
    let instructions = decompile_bonk_versioned_transaction_instructions(&decoded, &lookup_tables)?;
    Ok(DecomposedBonkVersionedTransaction {
        instructions,
        lookup_tables,
    })
}

fn merge_bonk_lookup_tables(
    lists: &[Vec<AddressLookupTableAccount>],
) -> Vec<AddressLookupTableAccount> {
    let mut merged = Vec::new();
    for list in lists {
        for table in list {
            if merged.iter().any(|existing: &AddressLookupTableAccount| existing.key == table.key) {
                continue;
            }
            merged.push(table.clone());
        }
    }
    merged
}

fn rewrite_missing_bonk_instruction_signers(
    owner: &Pubkey,
    instructions: &mut [Instruction],
    extra_signers: &[&Keypair],
) -> Vec<Keypair> {
    let known_signers = extra_signers
        .iter()
        .map(|signer| signer.pubkey())
        .collect::<Vec<_>>();
    let mut missing_signers = Vec::<Pubkey>::new();
    for instruction in instructions.iter() {
        for meta in &instruction.accounts {
            if !meta.is_signer || meta.pubkey == *owner || known_signers.contains(&meta.pubkey) {
                continue;
            }
            if !missing_signers.contains(&meta.pubkey) {
                missing_signers.push(meta.pubkey);
            }
        }
    }
    let replacements = missing_signers
        .into_iter()
        .map(|original| (original, Keypair::new()))
        .collect::<Vec<_>>();
    for instruction in instructions.iter_mut() {
        for meta in &mut instruction.accounts {
            if let Some((_, replacement)) =
                replacements.iter().find(|(original, _)| *original == meta.pubkey)
            {
                meta.pubkey = replacement.pubkey();
            }
        }
    }
    replacements
        .into_iter()
        .map(|(_, replacement)| replacement)
        .collect()
}

fn bonk_lookup_table_cache() -> &'static Mutex<HashMap<String, BonkLookupTableCacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, BonkLookupTableCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn persisted_bonk_lookup_table_cache() -> &'static Mutex<PersistedBonkLookupTableCache> {
    static CACHE: OnceLock<Mutex<PersistedBonkLookupTableCache>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let cache = fs::read_to_string(paths::bonk_lookup_table_cache_path())
            .ok()
            .and_then(|raw| serde_json::from_str::<PersistedBonkLookupTableCache>(&raw).ok())
            .unwrap_or_default();
        Mutex::new(cache)
    })
}

fn is_persisted_bonk_lookup_table_address(address: &str) -> bool {
    address == BONK_USD1_SUPER_LOOKUP_TABLE
}

fn persist_bonk_lookup_table_account(
    address: &str,
    table: &AddressLookupTableAccount,
) -> Result<(), String> {
    if !is_persisted_bonk_lookup_table_address(address) {
        return Ok(());
    }
    let mut cache = persisted_bonk_lookup_table_cache()
        .lock()
        .map_err(|error| error.to_string())?;
    cache.tables.insert(
        address.to_string(),
        PersistedBonkLookupTableEntry {
            addresses: table.addresses.iter().map(|entry| entry.to_string()).collect(),
        },
    );
    let serialized = serde_json::to_string_pretty(&*cache).map_err(|error| error.to_string())?;
    let path = paths::bonk_lookup_table_cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, serialized).map_err(|error| error.to_string())?;
    Ok(())
}

fn load_persisted_bonk_lookup_table_account(address: &str) -> Option<AddressLookupTableAccount> {
    if !is_persisted_bonk_lookup_table_address(address) {
        return None;
    }
    let cache = persisted_bonk_lookup_table_cache().lock().ok()?;
    let entry = cache.tables.get(address)?;
    let key = Pubkey::from_str(address).ok()?;
    let addresses = entry
        .addresses
        .iter()
        .map(|entry| Pubkey::from_str(entry))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    Some(AddressLookupTableAccount { key, addresses })
}

async fn load_bonk_preferred_usd1_lookup_tables_with_metrics(
    rpc_url: &str,
    commitment: &str,
    mut metrics: Option<&mut HelperUsd1QuoteMetrics>,
) -> Vec<AddressLookupTableAccount> {
    let ttl = bonk_lookup_table_cache_ttl();
    if let Ok(cache) = bonk_lookup_table_cache().lock() {
        if let Some(entry) = cache
            .get(BONK_USD1_SUPER_LOOKUP_TABLE)
            .filter(|entry| entry.fetched_at.elapsed() <= ttl)
        {
            if let Some(metrics) = metrics.as_deref_mut() {
                metrics.superAltLocalSnapshotHits = metrics.superAltLocalSnapshotHits.saturating_add(1);
            }
            return vec![entry.table.clone()];
        }
    }
    let Ok(address) = Pubkey::from_str(BONK_USD1_SUPER_LOOKUP_TABLE) else {
        return vec![];
    };
    if let Some(table) = load_persisted_bonk_lookup_table_account(BONK_USD1_SUPER_LOOKUP_TABLE) {
        if let Some(metrics) = metrics.as_deref_mut() {
            metrics.superAltLocalSnapshotHits = metrics.superAltLocalSnapshotHits.saturating_add(1);
        }
        if let Ok(mut cache) = bonk_lookup_table_cache().lock() {
            cache.insert(
                BONK_USD1_SUPER_LOOKUP_TABLE.to_string(),
                BonkLookupTableCacheEntry {
                    fetched_at: std::time::Instant::now(),
                    table: table.clone(),
                },
            );
        }
        return vec![table];
    }
    let Ok(table) = load_lookup_table_account_for_bonk_transaction(rpc_url, &address, commitment).await else {
        return vec![];
    };
    if let Ok(mut cache) = bonk_lookup_table_cache().lock() {
        cache.insert(
            BONK_USD1_SUPER_LOOKUP_TABLE.to_string(),
            BonkLookupTableCacheEntry {
                fetched_at: std::time::Instant::now(),
                table: table.clone(),
            },
        );
    }
    let _ = persist_bonk_lookup_table_account(BONK_USD1_SUPER_LOOKUP_TABLE, &table);
    if let Some(metrics) = metrics.as_deref_mut() {
        metrics.superAltRpcRefreshes = metrics.superAltRpcRefreshes.saturating_add(1);
    }
    vec![table]
}

async fn load_bonk_preferred_usd1_lookup_tables(
    rpc_url: &str,
    commitment: &str,
) -> Vec<AddressLookupTableAccount> {
    load_bonk_preferred_usd1_lookup_tables_with_metrics(rpc_url, commitment, None).await
}

fn bonk_compiled_transaction_size_bytes(compiled: &CompiledTransaction) -> Result<usize, String> {
    BASE64
        .decode(compiled.serializedBase64.as_bytes())
        .map(|raw| raw.len())
        .map_err(|error| format!("Failed to decode Bonk compiled transaction: {error}"))
}

fn bonk_lookup_table_candidate_key(tables: &[AddressLookupTableAccount]) -> String {
    tables
        .iter()
        .map(|table| table.key.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn bonk_lookup_table_candidates(
    base_lookup_tables: &[AddressLookupTableAccount],
    preferred_lookup_tables: &[AddressLookupTableAccount],
) -> Vec<Vec<AddressLookupTableAccount>> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    let merged_lookup_tables = merge_bonk_lookup_tables(&[
        preferred_lookup_tables.to_vec(),
        base_lookup_tables.to_vec(),
    ]);
    for candidate in [
        base_lookup_tables.to_vec(),
        preferred_lookup_tables.to_vec(),
        merged_lookup_tables,
    ] {
        let key = bonk_lookup_table_candidate_key(&candidate);
        if seen.insert(key) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn build_bonk_compiled_transaction_with_lookup_preference(
    label: &str,
    tx_format: NativeBonkTxFormat,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instructions: Vec<Instruction>,
    tx_config: &NativeBonkTxConfig,
    base_lookup_tables: &[AddressLookupTableAccount],
    preferred_lookup_tables: &[AddressLookupTableAccount],
) -> Result<CompiledTransaction, String> {
    if tx_format == NativeBonkTxFormat::Legacy {
        return build_bonk_compiled_transaction(
            label,
            tx_format,
            blockhash,
            last_valid_block_height,
            payer,
            extra_signers,
            instructions,
            tx_config,
        );
    }
    let mut best: Option<(CompiledTransaction, usize)> = None;
    let mut last_error = None;
    for lookup_tables in bonk_lookup_table_candidates(base_lookup_tables, preferred_lookup_tables) {
        match build_bonk_v0_compiled_transaction_with_lookup_tables(
            label,
            blockhash,
            last_valid_block_height,
            payer,
            extra_signers,
            instructions.clone(),
            tx_config,
            &lookup_tables,
        ) {
            Ok(compiled) => {
                let serialized_len = bonk_compiled_transaction_size_bytes(&compiled)?;
                let replace_best = match &best {
                    None => true,
                    Some((current, current_len)) => {
                        serialized_len < *current_len
                            || (serialized_len == *current_len
                                && compiled.lookupTablesUsed.len() < current.lookupTablesUsed.len())
                    }
                };
                if replace_best {
                    best = Some((compiled, serialized_len));
                }
            }
            Err(error) => last_error = Some(error),
        }
    }
    best.map(|(compiled, _)| compiled).ok_or_else(|| {
        last_error.unwrap_or_else(|| "Bonk v0 compile failed for all lookup table candidates.".to_string())
    })
}

fn bonk_compiled_transaction_fits_with_lookup_preference(
    label: &str,
    tx_format: NativeBonkTxFormat,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instructions: Vec<Instruction>,
    tx_config: &NativeBonkTxConfig,
    base_lookup_tables: &[AddressLookupTableAccount],
    preferred_lookup_tables: &[AddressLookupTableAccount],
) -> Result<bool, String> {
    match build_bonk_compiled_transaction_with_lookup_preference(
        label,
        tx_format,
        blockhash,
        last_valid_block_height,
        payer,
        extra_signers,
        instructions,
        tx_config,
        base_lookup_tables,
        preferred_lookup_tables,
    ) {
        Ok(compiled) => Ok(bonk_compiled_transaction_size_bytes(&compiled)? <= PACKET_LIMIT_BYTES),
        Err(_error) if tx_format == NativeBonkTxFormat::V0 => Ok(false),
        Err(error) => Err(error),
    }
}

fn filter_atomic_bonk_instructions(
    instructions: Vec<Instruction>,
    owner_pubkey: &Pubkey,
    tx_config: &NativeBonkTxConfig,
) -> Vec<Instruction> {
    instructions
        .into_iter()
        .filter(|instruction| {
            !is_compute_budget_instruction(instruction)
                && !is_inline_tip_instruction(
                    instruction,
                    owner_pubkey,
                    &tx_config.tip_account,
                    tx_config.tip_lamports,
                )
        })
        .collect()
}

fn build_bonk_atomic_tx_instructions(
    core_instructions: Vec<Instruction>,
    tx_config: &NativeBonkTxConfig,
    payer: &Pubkey,
    jitodontfront_enabled: bool,
) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    if tx_config.compute_unit_price_micro_lamports > 0 {
        instructions.push(build_compute_unit_price_instruction(
            tx_config.compute_unit_price_micro_lamports,
        )?);
    }
    if tx_config.compute_unit_limit > 0 {
        instructions.push(build_compute_unit_limit_instruction(tx_config.compute_unit_limit)?);
    }
    instructions.extend(apply_jitodontfront(
        core_instructions,
        jitodontfront_enabled,
    )?);
    if tx_config.tip_lamports > 0 && !tx_config.tip_account.trim().is_empty() {
        let tip_account = Pubkey::from_str(tx_config.tip_account.trim())
            .map_err(|error| format!("Invalid Jito tip account: {error}"))?;
        instructions.push(solana_system_interface::instruction::transfer(
            payer,
            &tip_account,
            tx_config.tip_lamports,
        ));
    }
    Ok(instructions)
}

fn bonk_launchpad_auth_pda() -> Result<Pubkey, String> {
    let program = bonk_launchpad_program_id()?;
    Ok(Pubkey::find_program_address(&[b"vault_auth_seed"], &program).0)
}

fn bonk_token_2022_program_id() -> Result<Pubkey, String> {
    Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
        .map_err(|error| format!("Invalid Token-2022 program id: {error}"))
}

pub fn derive_follow_owner_token_account(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address_with_program_id(
        owner,
        mint,
        &spl_token::id(),
    )
}

fn bonk_memo_program_id() -> Result<Pubkey, String> {
    Pubkey::from_str(MEMO_PROGRAM_ID).map_err(|error| format!("Invalid Memo program id: {error}"))
}

fn bonk_launchpad_cpi_event_pda() -> Result<Pubkey, String> {
    let program = bonk_launchpad_program_id()?;
    Ok(Pubkey::find_program_address(&[b"__event_authority"], &program).0)
}

fn bonk_launchpad_pool_vault_pda(pool_id: &Pubkey, mint: &Pubkey) -> Result<Pubkey, String> {
    let program = bonk_launchpad_program_id()?;
    Ok(Pubkey::find_program_address(&[b"pool_vault", pool_id.as_ref(), mint.as_ref()], &program).0)
}

fn bonk_platform_fee_vault_pda(platform_id: &Pubkey, mint: &Pubkey) -> Result<Pubkey, String> {
    let program = bonk_launchpad_program_id()?;
    Ok(Pubkey::find_program_address(&[platform_id.as_ref(), mint.as_ref()], &program).0)
}

fn bonk_creator_fee_vault_pda(creator: &Pubkey, mint: &Pubkey) -> Result<Pubkey, String> {
    let program = bonk_launchpad_program_id()?;
    Ok(Pubkey::find_program_address(&[creator.as_ref(), mint.as_ref()], &program).0)
}

fn bonk_clmm_pool_vault_pda(pool_id: &Pubkey, mint: &Pubkey) -> Result<Pubkey, String> {
    let program = bonk_clmm_program_id()?;
    Ok(Pubkey::find_program_address(&[b"pool_vault", pool_id.as_ref(), mint.as_ref()], &program).0)
}

fn bonk_clmm_ex_bitmap_pda(pool_id: &Pubkey) -> Result<Pubkey, String> {
    let program = bonk_clmm_program_id()?;
    Ok(Pubkey::find_program_address(
        &[b"pool_tick_array_bitmap_extension", pool_id.as_ref()],
        &program,
    )
    .0)
}

fn bonk_clmm_observation_pda(pool_id: &Pubkey) -> Result<Pubkey, String> {
    let program = bonk_clmm_program_id()?;
    Ok(Pubkey::find_program_address(&[b"observation", pool_id.as_ref()], &program).0)
}

fn bonk_metadata_program_id() -> Result<Pubkey, String> {
    Pubkey::from_str(MPL_TOKEN_METADATA_PROGRAM_ID)
        .map_err(|error| format!("Invalid token metadata program id: {error}"))
}

fn bonk_metadata_account_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    let metadata_program = bonk_metadata_program_id()?;
    Ok(Pubkey::find_program_address(
        &[b"metadata", metadata_program.as_ref(), mint.as_ref()],
        &metadata_program,
    )
    .0)
}

fn bonk_append_string_layout(data: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    let length = u32::try_from(bytes.len()).map_err(|_| "Bonk string field exceeded u32 length.".to_string())?;
    data.extend_from_slice(&length.to_le_bytes());
    data.extend_from_slice(bytes);
    Ok(())
}

fn build_bonk_initialize_v2_instruction(
    owner: &Pubkey,
    mint: &Pubkey,
    launch_mode: &str,
    token_name: &str,
    token_symbol: &str,
    token_uri: &str,
    defaults: &BonkLaunchDefaults,
) -> Result<Instruction, String> {
    let program_id = bonk_launchpad_program_id()?;
    let quote_mint = bonk_quote_mint(defaults.quote.asset)?;
    let config_id = Pubkey::from_str(&bonk_launch_config_id(defaults.quote.asset)?)
        .map_err(|error| format!("Invalid Bonk config id: {error}"))?;
    let platform_id = Pubkey::from_str(bonk_platform_id(launch_mode))
        .map_err(|error| format!("Invalid Bonk platform id: {error}"))?;
    let pool_id = Pubkey::find_program_address(&[b"pool", mint.as_ref(), quote_mint.as_ref()], &program_id).0;
    let vault_a = bonk_launchpad_pool_vault_pda(&pool_id, mint)?;
    let vault_b = bonk_launchpad_pool_vault_pda(&pool_id, &quote_mint)?;
    let metadata_id = bonk_metadata_account_pda(mint)?;
    let mut data = Vec::new();
    data.extend_from_slice(&BONK_INITIALIZE_V2_DISCRIMINATOR);
    data.push(u8::try_from(BONK_TOKEN_DECIMALS).map_err(|_| "Invalid Bonk token decimals.".to_string())?);
    bonk_append_string_layout(&mut data, token_name)?;
    bonk_append_string_layout(&mut data, token_symbol)?;
    bonk_append_string_layout(&mut data, token_uri)?;
    data.push(defaults.curve_type);
    data.extend_from_slice(&biguint_to_u64(&defaults.supply, "launch supply")?.to_le_bytes());
    if defaults.curve_type == 0 {
        data.extend_from_slice(
            &biguint_to_u64(&defaults.pool.total_sell_a, "launch total sell")?.to_le_bytes(),
        );
    }
    data.extend_from_slice(
        &biguint_to_u64(&defaults.total_fund_raising_b, "launch total fund raising")?.to_le_bytes(),
    );
    data.push(1u8);
    data.extend_from_slice(&0u64.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    data.push(0u8);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(config_id, false),
            AccountMeta::new_readonly(platform_id, false),
            AccountMeta::new_readonly(bonk_launchpad_auth_pda()?, false),
            AccountMeta::new(pool_id, false),
            AccountMeta::new(*mint, true),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(vault_b, false),
            AccountMeta::new(metadata_id, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(bonk_metadata_program_id()?, false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
            AccountMeta::new_readonly(bonk_launchpad_cpi_event_pda()?, false),
            AccountMeta::new_readonly(program_id, false),
        ],
        data,
    })
}

fn build_bonk_clmm_swap_exact_in_instruction(
    owner: &Pubkey,
    user_input_account: &Pubkey,
    user_output_account: &Pubkey,
    amount_in: u64,
    min_out: u64,
    traversed_tick_array_starts: &[i32],
) -> Result<Instruction, String> {
    let program_id = bonk_clmm_program_id()?;
    let pool_id = Pubkey::from_str(BONK_PINNED_USD1_ROUTE_POOL_ID)
        .map_err(|error| format!("Invalid pinned Bonk USD1 route pool id: {error}"))?;
    let amm_config = Pubkey::from_str(BONK_PREFERRED_USD1_ROUTE_CONFIG_ID)
        .map_err(|error| format!("Invalid pinned Bonk USD1 route config id: {error}"))?;
    let input_mint = bonk_quote_mint("sol")?;
    let output_mint = bonk_quote_mint("usd1")?;
    let input_vault = bonk_clmm_pool_vault_pda(&pool_id, &input_mint)?;
    let output_vault = bonk_clmm_pool_vault_pda(&pool_id, &output_mint)?;
    let observation_id = bonk_clmm_observation_pda(&pool_id)?;
    let ex_bitmap = bonk_clmm_ex_bitmap_pda(&pool_id)?;
    let tick_arrays = traversed_tick_array_starts
        .iter()
        .map(|start_index| bonk_derive_clmm_tick_array_address(&program_id, &pool_id, *start_index))
        .collect::<Vec<_>>();
    let mut data = Vec::with_capacity(8 + 8 + 8 + 16 + 1);
    data.extend_from_slice(&BONK_CLMM_SWAP_DISCRIMINATOR);
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&min_out.to_le_bytes());
    data.extend_from_slice(&BONK_CLMM_MIN_SQRT_PRICE_X64_PLUS_ONE.to_le_bytes());
    data.push(1u8);
    let mut accounts = vec![
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new_readonly(amm_config, false),
        AccountMeta::new(pool_id, false),
        AccountMeta::new(*user_input_account, false),
        AccountMeta::new(*user_output_account, false),
        AccountMeta::new(input_vault, false),
        AccountMeta::new(output_vault, false),
        AccountMeta::new(observation_id, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(bonk_token_2022_program_id()?, false),
        AccountMeta::new_readonly(bonk_memo_program_id()?, false),
        AccountMeta::new_readonly(input_mint, false),
        AccountMeta::new_readonly(output_mint, false),
        AccountMeta::new(ex_bitmap, false),
    ];
    accounts.extend(
        tick_arrays
            .into_iter()
            .map(|pubkey| AccountMeta::new(pubkey, false)),
    );
    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

fn build_bonk_buy_exact_in_instruction(
    owner: &Pubkey,
    pool_context: &NativeBonkPoolContext,
    user_token_account_a: &Pubkey,
    user_token_account_b: &Pubkey,
    amount_b: u64,
    min_amount_a: u64,
) -> Result<Instruction, String> {
    let launchpad_program = bonk_launchpad_program_id()?;
    let auth = bonk_launchpad_auth_pda()?;
    let vault_a = bonk_launchpad_pool_vault_pda(&pool_context.pool_id, &pool_context.pool.mint_a)?;
    let quote_mint = bonk_quote_mint(pool_context.quote.asset)?;
    let vault_b = bonk_launchpad_pool_vault_pda(&pool_context.pool_id, &quote_mint)?;
    let platform_claim_fee_vault =
        bonk_platform_fee_vault_pda(&pool_context.pool.platform_id, &quote_mint)?;
    let creator_claim_fee_vault =
        bonk_creator_fee_vault_pda(&pool_context.pool.creator, &quote_mint)?;
    let cpi_event = bonk_launchpad_cpi_event_pda()?;
    let token_program = spl_token::id();
    let mut data = Vec::with_capacity(32);
    data.extend_from_slice(&BONK_BUY_EXACT_IN_DISCRIMINATOR);
    data.extend_from_slice(&amount_b.to_le_bytes());
    data.extend_from_slice(&min_amount_a.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    Ok(Instruction {
        program_id: launchpad_program,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new_readonly(auth, false),
            AccountMeta::new_readonly(pool_context.pool.config_id, false),
            AccountMeta::new_readonly(pool_context.pool.platform_id, false),
            AccountMeta::new(pool_context.pool_id, false),
            AccountMeta::new(*user_token_account_a, false),
            AccountMeta::new(*user_token_account_b, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(vault_b, false),
            AccountMeta::new_readonly(pool_context.pool.mint_a, false),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(cpi_event, false),
            AccountMeta::new_readonly(launchpad_program, false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new(platform_claim_fee_vault, false),
            AccountMeta::new(creator_claim_fee_vault, false),
        ],
        data,
    })
}

fn build_bonk_sell_exact_in_instruction(
    owner: &Pubkey,
    pool_context: &NativeBonkPoolContext,
    user_token_account_a: &Pubkey,
    user_token_account_b: &Pubkey,
    amount_a: u64,
    min_amount_b: u64,
) -> Result<Instruction, String> {
    let launchpad_program = bonk_launchpad_program_id()?;
    let auth = bonk_launchpad_auth_pda()?;
    let vault_a = bonk_launchpad_pool_vault_pda(&pool_context.pool_id, &pool_context.pool.mint_a)?;
    let quote_mint = bonk_quote_mint(pool_context.quote.asset)?;
    let vault_b = bonk_launchpad_pool_vault_pda(&pool_context.pool_id, &quote_mint)?;
    let platform_claim_fee_vault =
        bonk_platform_fee_vault_pda(&pool_context.pool.platform_id, &quote_mint)?;
    let creator_claim_fee_vault =
        bonk_creator_fee_vault_pda(&pool_context.pool.creator, &quote_mint)?;
    let cpi_event = bonk_launchpad_cpi_event_pda()?;
    let token_program = spl_token::id();
    let mut data = Vec::with_capacity(32);
    data.extend_from_slice(&BONK_SELL_EXACT_IN_DISCRIMINATOR);
    data.extend_from_slice(&amount_a.to_le_bytes());
    data.extend_from_slice(&min_amount_b.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    Ok(Instruction {
        program_id: launchpad_program,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new_readonly(auth, false),
            AccountMeta::new_readonly(pool_context.pool.config_id, false),
            AccountMeta::new_readonly(pool_context.pool.platform_id, false),
            AccountMeta::new(pool_context.pool_id, false),
            AccountMeta::new(*user_token_account_a, false),
            AccountMeta::new(*user_token_account_b, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(vault_b, false),
            AccountMeta::new_readonly(pool_context.pool.mint_a, false),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(cpi_event, false),
            AccountMeta::new_readonly(launchpad_program, false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new(platform_claim_fee_vault, false),
            AccountMeta::new(creator_claim_fee_vault, false),
        ],
        data,
    })
}

fn build_bonk_wrapped_sol_open_instructions(
    owner: &Pubkey,
    wrapped_account: &Pubkey,
    lamports: u64,
) -> Result<Vec<Instruction>, String> {
    let token_program = spl_token::id();
    Ok(vec![
        solana_system_interface::instruction::create_account(
            owner,
            wrapped_account,
            lamports,
            BONK_SPL_TOKEN_ACCOUNT_LEN,
            &token_program,
        ),
        spl_token::instruction::initialize_account3(
            &token_program,
            wrapped_account,
            &bonk_quote_mint("sol")?,
            owner,
        )
        .map_err(|error| format!("Failed to build wrapped SOL initialize instruction: {error}"))?,
        spl_token::instruction::sync_native(&token_program, wrapped_account)
            .map_err(|error| format!("Failed to build sync-native instruction: {error}"))?,
    ])
}

fn build_bonk_wrapped_sol_close_instruction(owner: &Pubkey, wrapped_account: &Pubkey) -> Result<Instruction, String> {
    spl_token::instruction::close_account(&spl_token::id(), wrapped_account, owner, owner, &[])
        .map_err(|error| format!("Failed to build wrapped SOL close instruction: {error}"))
}

fn build_bonk_follow_pool_state(pool: &DecodedBonkLaunchpadPool) -> BonkCurvePoolState {
    BonkCurvePoolState {
        total_sell_a: bonk_biguint_from_u64(pool.total_sell_a),
        virtual_a: bonk_biguint_from_u64(pool.virtual_a),
        virtual_b: bonk_biguint_from_u64(pool.virtual_b),
        real_a: bonk_biguint_from_u64(pool.real_a),
        real_b: bonk_biguint_from_u64(pool.real_b),
    }
}

fn build_prelaunch_bonk_pool_context(
    defaults: &BonkLaunchDefaults,
    mint: &Pubkey,
    creator: &Pubkey,
    launch_mode: &str,
) -> Result<NativeBonkPoolContext, String> {
    let quote_mint = bonk_quote_mint(defaults.quote.asset)?;
    let launchpad_program = bonk_launchpad_program_id()?;
    let pool_id = Pubkey::find_program_address(
        &[b"pool", mint.as_ref(), quote_mint.as_ref()],
        &launchpad_program,
    )
    .0;
    let config_id = Pubkey::from_str(&bonk_launch_config_id(defaults.quote.asset)?)
        .map_err(|error| format!("Invalid Bonk config id: {error}"))?;
    let platform_id = Pubkey::from_str(bonk_platform_id(launch_mode))
        .map_err(|error| format!("Invalid Bonk platform id: {error}"))?;
    Ok(NativeBonkPoolContext {
        pool_id,
        pool: DecodedBonkLaunchpadPool {
            creator: *creator,
            status: 0,
            supply: biguint_to_u64(&defaults.supply, "prelaunch supply")?,
            config_id,
            total_sell_a: biguint_to_u64(&defaults.pool.total_sell_a, "prelaunch total sell")?,
            virtual_a: biguint_to_u64(&defaults.pool.virtual_a, "prelaunch virtual A")?,
            virtual_b: biguint_to_u64(&defaults.pool.virtual_b, "prelaunch virtual B")?,
            real_a: 0,
            real_b: 0,
            platform_id,
            mint_a: *mint,
        },
        config: DecodedBonkLaunchpadConfig {
            curve_type: defaults.curve_type,
            migrate_fee: 0,
            trade_fee_rate: biguint_to_u64(&defaults.trade_fee_rate, "prelaunch trade fee rate")?,
        },
        platform: DecodedBonkPlatformConfig {
            fee_rate: biguint_to_u64(&defaults.platform_fee_rate, "prelaunch platform fee rate")?,
            creator_fee_rate: biguint_to_u64(
                &defaults.creator_fee_rate,
                "prelaunch creator fee rate",
            )?,
        },
        quote: defaults.quote.clone(),
    })
}

async fn load_bonk_pool_context_by_pool_id(
    rpc_url: &str,
    pool_id_input: &str,
    quote_asset: &str,
    commitment: &str,
) -> Result<NativeBonkPoolContext, String> {
    let pool_id = Pubkey::from_str(pool_id_input).map_err(|error| format!("Invalid Bonk pool id: {error}"))?;
    let pool_data = fetch_account_data(rpc_url, pool_id_input, commitment).await?;
    let pool = decode_bonk_launchpad_pool(&pool_data)?;
    let config_id = pool.config_id.to_string();
    let platform_id = pool.platform_id.to_string();
    let (config_data, platform_data) = tokio::try_join!(
        fetch_account_data(rpc_url, &config_id, commitment),
        fetch_account_data(rpc_url, &platform_id, commitment),
    )?;
    Ok(NativeBonkPoolContext {
        pool_id,
        pool,
        config: decode_bonk_launchpad_config(&config_data)?,
        platform: decode_bonk_platform_config(&platform_data)?,
        quote: bonk_quote_asset_config(quote_asset),
    })
}

async fn load_live_bonk_pool_context(
    rpc_url: &str,
    mint: &Pubkey,
    quote_asset: &str,
    commitment: &str,
) -> Result<NativeBonkPoolContext, String> {
    let requested_quote = bonk_quote_asset_config(quote_asset);
    let candidate_assets = if requested_quote.asset == "usd1" {
        vec![requested_quote.asset, "sol"]
    } else {
        vec![requested_quote.asset, "usd1"]
    };
    let mut errors = Vec::new();
    for asset in candidate_assets {
        let quote = bonk_quote_asset_config(asset);
        let pool_id = derive_canonical_pool_id(quote.asset, &mint.to_string()).await?;
        for attempt in 0..6 {
            match load_bonk_pool_context_by_pool_id(rpc_url, &pool_id, quote.asset, commitment).await {
                Ok(context) => return Ok(context),
                Err(error) => {
                    errors.push(format!("{}:{}: {}", quote.asset, pool_id, error));
                    if attempt < 5 {
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
            }
        }
    }
    Err(format!(
        "Unable to resolve Bonk live pool context. Attempts: {}",
        errors.join(" | ")
    ))
}

fn read_spl_token_account_amount(data: &[u8]) -> Result<u64, String> {
    if data.len() < 72 {
        return Err("Token account data was shorter than expected.".to_string());
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&data[64..72]);
    Ok(u64::from_le_bytes(raw))
}

async fn fetch_bonk_owner_token_balance(
    rpc_url: &str,
    commitment: &str,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<Option<u64>, String> {
    let token_account =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            owner,
            mint,
            &spl_token::id(),
        );
    let data = match fetch_account_data(rpc_url, &token_account.to_string(), commitment).await {
        Ok(data) => data,
        Err(error) if error.contains("was not found.") => return Ok(None),
        Err(error) => return Err(error),
    };
    Ok(Some(read_spl_token_account_amount(&data)?))
}

async fn rpc_get_minimum_balance_for_rent_exemption(
    rpc_url: &str,
    commitment: &str,
    data_len: u64,
) -> Result<u64, String> {
    #[derive(Deserialize)]
    struct RentExemptionResponse {
        result: u64,
    }

    let payload = json!({
        "jsonrpc": "2.0",
        "id": "launchdeck-bonk-rent-exemption",
        "method": "getMinimumBalanceForRentExemption",
        "params": [
            data_len,
            {
                "commitment": commitment,
            }
        ]
    });
    let response = bonk_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bonk rent exemption: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bonk rent exemption: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RentExemptionResponse = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bonk rent exemption response: {error}"))?;
    Ok(parsed.result)
}

fn bonk_follow_buy_amounts(
    pool_context: &NativeBonkPoolContext,
    requested_amount_b: u64,
    slippage_bps: u64,
) -> Result<(u64, u64), String> {
    let details = bonk_follow_buy_quote_details(pool_context, requested_amount_b, slippage_bps)?;
    Ok((details.gross_input_b, details.min_amount_a))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BonkFollowBuyQuoteDetails {
    gross_input_b: u64,
    net_input_b: u64,
    amount_a: u64,
    min_amount_a: u64,
}

fn bonk_follow_buy_quote_details(
    pool_context: &NativeBonkPoolContext,
    requested_amount_b: u64,
    slippage_bps: u64,
) -> Result<BonkFollowBuyQuoteDetails, String> {
    let pool = build_bonk_follow_pool_state(&pool_context.pool);
    let fee_rate = bonk_total_fee_rate(
        &bonk_biguint_from_u64(pool_context.config.trade_fee_rate),
        &bonk_biguint_from_u64(pool_context.platform.fee_rate),
        &bonk_biguint_from_u64(pool_context.platform.creator_fee_rate),
    )?;
    let requested_amount_b_big = bonk_biguint_from_u64(requested_amount_b);
    let total_fee = bonk_calculate_fee(&requested_amount_b_big, &fee_rate);
    let amount_less_fee_b =
        bonk_big_sub(&requested_amount_b_big, &total_fee, "buy input after fee")?;
    let quoted_amount_a = bonk_curve_buy_exact_in(&pool, pool_context.config.curve_type, &amount_less_fee_b)?;
    let remaining_amount_a =
        bonk_big_sub(&pool.total_sell_a, &pool.real_a, "remaining sell amount")?;
    let (gross_input_b, net_input_b, amount_a) = if quoted_amount_a > remaining_amount_a {
        let capped_net_input_b = bonk_curve_buy_exact_out(
            &pool,
            pool_context.config.curve_type,
            &remaining_amount_a,
        )?;
        let gross_input_b = bonk_calculate_pre_fee(&capped_net_input_b, &fee_rate)?;
        (gross_input_b, capped_net_input_b, remaining_amount_a)
    } else {
        (requested_amount_b_big, amount_less_fee_b, quoted_amount_a)
    };
    let min_amount_a = bonk_build_min_amount_from_bps(&amount_a, slippage_bps);
    Ok(BonkFollowBuyQuoteDetails {
        gross_input_b: biguint_to_u64(&gross_input_b, "follow buy spend amount")?,
        net_input_b: biguint_to_u64(&net_input_b, "follow buy pool input amount")?,
        amount_a: biguint_to_u64(&amount_a, "follow buy quoted output")?,
        min_amount_a: biguint_to_u64(&min_amount_a, "follow buy min output")?,
    })
}

fn advance_prelaunch_bonk_pool_context_after_buy(
    pool_context: &NativeBonkPoolContext,
    requested_amount_b: u64,
    slippage_bps: u64,
) -> Result<NativeBonkPoolContext, String> {
    let details = bonk_follow_buy_quote_details(pool_context, requested_amount_b, slippage_bps)?;
    let mut next = pool_context.clone();
    next.pool.real_a = next.pool.real_a.saturating_add(details.amount_a);
    next.pool.real_b = next.pool.real_b.saturating_add(details.net_input_b);
    Ok(next)
}

fn bonk_follow_sell_amounts(
    pool_context: &NativeBonkPoolContext,
    sell_amount_a: u64,
    slippage_bps: u64,
) -> Result<u64, String> {
    let pool = build_bonk_follow_pool_state(&pool_context.pool);
    let quoted_amount_b = bonk_quote_sell_exact_in_amount_b(
        &pool,
        pool_context.config.curve_type,
        &bonk_biguint_from_u64(pool_context.config.trade_fee_rate),
        &bonk_biguint_from_u64(pool_context.platform.fee_rate),
        &bonk_biguint_from_u64(pool_context.platform.creator_fee_rate),
        &bonk_biguint_from_u64(sell_amount_a),
    )?;
    let min_amount_b = bonk_build_min_amount_from_bps(&quoted_amount_b, slippage_bps);
    biguint_to_u64(&min_amount_b, "follow sell min output")
}

async fn native_compile_bonk_buy_transaction_with_pool_context(
    rpc_url: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    owner: &Keypair,
    mint_pubkey: &Pubkey,
    pool_context: &NativeBonkPoolContext,
    requested_amount_b: u64,
    allow_ata_creation: bool,
    tx_format: NativeBonkTxFormat,
) -> Result<CompiledTransaction, String> {
    let owner_pubkey = owner.pubkey();
    let slippage_bps = slippage_bps_from_percent(&execution.buySlippagePercent)?;
    let (instruction_amount_b, min_amount_a) =
        bonk_follow_buy_amounts(pool_context, requested_amount_b, slippage_bps)?;
    let tip_lamports =
        resolve_follow_tip_lamports(&execution.buyProvider, &execution.buyTipSol, "buy tip")?;
    let tx_config = bonk_follow_tx_config(
        configured_default_sniper_buy_compute_unit_limit(),
        priority_fee_sol_to_micro_lamports(&execution.buyPriorityFeeSol)?,
        tip_lamports,
        jito_tip_account,
    )?;
    let token_program = spl_token::id();
    let user_token_account_a =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &owner_pubkey,
            mint_pubkey,
            &token_program,
        );
    let mut instructions = vec![
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &owner_pubkey,
            &owner_pubkey,
            mint_pubkey,
            &token_program,
        ),
    ];
    let mut extra_signers = Vec::new();
    let user_token_account_b = if pool_context.quote.asset == "sol" {
        let wrapped_signer = Keypair::new();
        let rent_exempt_lamports = rpc_get_minimum_balance_for_rent_exemption(
            rpc_url,
            &execution.commitment,
            BONK_SPL_TOKEN_ACCOUNT_LEN,
        )
        .await?;
        instructions.extend(build_bonk_wrapped_sol_open_instructions(
            &owner_pubkey,
            &wrapped_signer.pubkey(),
            rent_exempt_lamports.saturating_add(requested_amount_b),
        )?);
        extra_signers.push(wrapped_signer);
        extra_signers
            .last()
            .expect("wrapped SOL signer")
            .pubkey()
    } else {
        let quote_mint = bonk_quote_mint(pool_context.quote.asset)?;
        let quote_ata =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &owner_pubkey,
                &quote_mint,
                &token_program,
            );
        if allow_ata_creation {
            instructions.push(
                spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                    &owner_pubkey,
                    &owner_pubkey,
                    &quote_mint,
                    &token_program,
                ),
            );
        }
        quote_ata
    };
    instructions.push(build_bonk_buy_exact_in_instruction(
        &owner_pubkey,
        pool_context,
        &user_token_account_a,
        &user_token_account_b,
        instruction_amount_b,
        min_amount_a,
    )?);
    if pool_context.quote.asset == "sol" {
        instructions.push(build_bonk_wrapped_sol_close_instruction(
            &owner_pubkey,
            &user_token_account_b,
        )?);
    }
    let tx_instructions = with_bonk_tx_settings(
        instructions,
        &tx_config,
        &owner_pubkey,
        execution.buyJitodontfront,
    )?;
    let extra_signer_refs = extra_signers.iter().collect::<Vec<_>>();
    let (blockhash, last_valid_block_height) =
        fetch_latest_blockhash_cached(rpc_url, &execution.commitment).await?;
    let preferred_lookup_tables =
        if tx_format == NativeBonkTxFormat::V0 && pool_context.quote.asset == "usd1" {
            load_bonk_preferred_usd1_lookup_tables(rpc_url, &execution.commitment).await
        } else {
            vec![]
        };
    build_bonk_compiled_transaction_with_lookup_preference(
        "follow-buy",
        tx_format,
        &blockhash,
        last_valid_block_height,
        owner,
        &extra_signer_refs,
        tx_instructions,
        &tx_config,
        &[],
        &preferred_lookup_tables,
    )
}

async fn combine_atomic_bonk_usd1_follow_buy_transactions(
    rpc_url: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    owner: &Keypair,
    topup_transaction: &CompiledTransaction,
    action_transaction: &CompiledTransaction,
) -> Result<CompiledTransaction, String> {
    let tip_lamports =
        resolve_follow_tip_lamports(&execution.buyProvider, &execution.buyTipSol, "buy tip")?;
    let tx_config = bonk_follow_tx_config(
        configured_atomic_bonk_usd1_follow_buy_compute_unit_limit(
            topup_transaction.computeUnitLimit,
            action_transaction.computeUnitLimit,
        ),
        priority_fee_sol_to_micro_lamports(&execution.buyPriorityFeeSol)?,
        tip_lamports,
        jito_tip_account,
    )?;
    combine_atomic_bonk_transactions(
        rpc_url,
        &execution.commitment,
        owner,
        "follow-buy-atomic",
        &tx_config,
        execution.buyJitodontfront,
        &[],
        topup_transaction,
        action_transaction,
    )
    .await
}

fn configured_atomic_bonk_usd1_follow_buy_compute_unit_limit(
    topup_compute_unit_limit: Option<u64>,
    action_compute_unit_limit: Option<u64>,
) -> u64 {
    let child_default = configured_default_sniper_buy_compute_unit_limit();
    let merged_limit = topup_compute_unit_limit
        .unwrap_or(child_default)
        .saturating_add(action_compute_unit_limit.unwrap_or(child_default));
    merged_limit.max(configured_default_follow_up_compute_unit_limit())
}

async fn native_compile_follow_buy_transaction(
    rpc_url: &str,
    quote_asset: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    buy_amount: &str,
    allow_ata_creation: bool,
    pool_context_override: Option<&NativeBonkPoolContext>,
    usd1_route_setup_override: Option<&BonkUsd1RouteSetup>,
) -> Result<CompiledTransaction, String> {
    let owner = parse_owner_keypair(wallet_secret)?;
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bonk mint address: {error}"))?;
    let pool_context = if let Some(pool_context) = pool_context_override {
        pool_context.clone()
    } else {
        load_live_bonk_pool_context(rpc_url, &mint_pubkey, quote_asset, &execution.commitment).await?
    };
    let quote = bonk_quote_asset_config(quote_asset);
    let requested_amount_b = parse_decimal_u64(
        buy_amount,
        quote.decimals,
        &format!("follow buy amount {}", quote.label),
    )?;
    if quote.asset == "usd1" {
        let current_balance = fetch_bonk_owner_token_balance(
            rpc_url,
            &execution.commitment,
            &owner.pubkey(),
            &bonk_quote_mint("usd1")?,
        )
        .await?
        .unwrap_or_default();
        if current_balance < requested_amount_b {
            let slippage_bps = slippage_bps_from_percent(&execution.buySlippagePercent)?;
            let tip_lamports =
                resolve_follow_tip_lamports(&execution.buyProvider, &execution.buyTipSol, "buy tip")?;
            let buy_tx_config = bonk_follow_tx_config(
                configured_default_sniper_buy_compute_unit_limit(),
                priority_fee_sol_to_micro_lamports(&execution.buyPriorityFeeSol)?,
                tip_lamports,
                jito_tip_account,
            )?;
            let prepared_topup = native_prepare_bonk_usd1_topup(
                rpc_url,
                &execution.commitment,
                &owner.pubkey(),
                &bonk_biguint_from_u64(requested_amount_b),
                slippage_bps,
                None,
                usd1_route_setup_override,
            )
            .await?;
            let topup_transaction = native_compile_bonk_usd1_topup_from_prepared(
                rpc_url,
                &execution.commitment,
                &owner,
                allow_ata_creation,
                "usd1-topup",
                NativeBonkTxFormat::V0,
                &buy_tx_config,
                execution.buyJitodontfront,
                &prepared_topup,
            )
            .await?
            .ok_or_else(|| {
                "Native Bonk live USD1 follow buy could not prepare a required top-up transaction."
                    .to_string()
            })?;
            let action_transaction = native_compile_bonk_buy_transaction_with_pool_context(
                rpc_url,
                execution,
                jito_tip_account,
                &owner,
                &mint_pubkey,
                &pool_context,
                requested_amount_b,
                allow_ata_creation,
                NativeBonkTxFormat::V0,
            )
            .await?;
            return combine_atomic_bonk_usd1_follow_buy_transactions(
                rpc_url,
                execution,
                jito_tip_account,
                &owner,
                &topup_transaction,
                &action_transaction,
            )
            .await;
        }
    }
    native_compile_bonk_buy_transaction_with_pool_context(
        rpc_url,
        execution,
        jito_tip_account,
        &owner,
        &mint_pubkey,
        &pool_context,
        requested_amount_b,
        allow_ata_creation,
        if quote.asset == "usd1" {
            NativeBonkTxFormat::V0
        } else {
            select_bonk_native_tx_format(&execution.txFormat)
        },
    )
    .await
}

async fn native_compile_follow_sell_transaction_with_token_amount(
    rpc_url: &str,
    quote_asset: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    sell_percent: u8,
    token_amount_override: Option<u64>,
    pool_id_override: Option<&str>,
    launch_mode_override: Option<&str>,
    launch_creator_override: Option<&str>,
) -> Result<Option<CompiledTransaction>, String> {
    let owner = parse_owner_keypair(wallet_secret)?;
    let owner_pubkey = owner.pubkey();
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bonk mint address: {error}"))?;
    let raw_amount = if let Some(value) = token_amount_override {
        value
    } else {
        match fetch_bonk_owner_token_balance(rpc_url, &execution.commitment, &owner_pubkey, &mint_pubkey).await? {
            Some(value) => value,
            None => return Ok(None),
        }
    };
    if raw_amount == 0 {
        return Ok(None);
    }
    let sell_amount = (u128::from(raw_amount) * u128::from(sell_percent) / 100u128) as u64;
    if sell_amount == 0 {
        return Ok(None);
    }
    let pool_context =
        if token_amount_override.is_some()
            && pool_id_override.is_some()
            && launch_mode_override.is_some()
            && launch_creator_override.is_some()
        {
            let defaults = load_bonk_launch_defaults(
                rpc_url,
                launch_mode_override.unwrap_or_default(),
                quote_asset,
            )
            .await?;
            let creator = Pubkey::from_str(launch_creator_override.unwrap_or_default())
                .map_err(|error| format!("Invalid Bonk launch creator: {error}"))?;
            build_prelaunch_bonk_pool_context(&defaults, &mint_pubkey, &creator, launch_mode_override.unwrap_or_default())?
        } else if let Some(pool_id) = pool_id_override {
            load_bonk_pool_context_by_pool_id(rpc_url, pool_id, quote_asset, &execution.commitment).await?
        } else {
            load_live_bonk_pool_context(rpc_url, &mint_pubkey, quote_asset, &execution.commitment).await?
        };
    let slippage_bps = slippage_bps_from_percent(&execution.sellSlippagePercent)?;
    let min_amount_b = bonk_follow_sell_amounts(&pool_context, sell_amount, slippage_bps)?;
    let tip_lamports =
        resolve_follow_tip_lamports(&execution.sellProvider, &execution.sellTipSol, "sell tip")?;
    let tx_config = bonk_follow_tx_config(
        configured_default_dev_auto_sell_compute_unit_limit(),
        priority_fee_sol_to_micro_lamports(&execution.sellPriorityFeeSol)?,
        tip_lamports,
        jito_tip_account,
    )?;
    let tx_format = select_bonk_native_tx_format(&execution.txFormat);
    let token_program = spl_token::id();
    let user_token_account_a =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &owner_pubkey,
            &mint_pubkey,
            &token_program,
        );
    let mut instructions = Vec::new();
    let mut extra_signers = Vec::new();
    let user_token_account_b = if pool_context.quote.asset == "sol" {
        let wrapped_signer = Keypair::new();
        let rent_exempt_lamports = rpc_get_minimum_balance_for_rent_exemption(
            rpc_url,
            &execution.commitment,
            BONK_SPL_TOKEN_ACCOUNT_LEN,
        )
        .await?;
        instructions.extend(build_bonk_wrapped_sol_open_instructions(
            &owner_pubkey,
            &wrapped_signer.pubkey(),
            rent_exempt_lamports,
        )?);
        extra_signers.push(wrapped_signer);
        extra_signers
            .last()
            .expect("wrapped SOL signer")
            .pubkey()
    } else {
        let quote_mint = bonk_quote_mint(pool_context.quote.asset)?;
        let quote_ata =
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &owner_pubkey,
                &quote_mint,
                &token_program,
            );
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                &owner_pubkey,
                &owner_pubkey,
                &quote_mint,
                &token_program,
            ),
        );
        quote_ata
    };
    instructions.push(build_bonk_sell_exact_in_instruction(
        &owner_pubkey,
        &pool_context,
        &user_token_account_a,
        &user_token_account_b,
        sell_amount,
        min_amount_b,
    )?);
    if pool_context.quote.asset == "sol" {
        instructions.push(build_bonk_wrapped_sol_close_instruction(
            &owner_pubkey,
            &user_token_account_b,
        )?);
    }
    let tx_instructions = with_bonk_tx_settings(
        instructions,
        &tx_config,
        &owner_pubkey,
        execution.sellJitodontfront,
    )?;
    let extra_signer_refs = extra_signers.iter().collect::<Vec<_>>();
    let (blockhash, last_valid_block_height) =
        fetch_latest_blockhash_cached(rpc_url, &execution.commitment).await?;
    let preferred_lookup_tables =
        if tx_format == NativeBonkTxFormat::V0 && pool_context.quote.asset == "usd1" {
            load_bonk_preferred_usd1_lookup_tables(rpc_url, &execution.commitment).await
        } else {
            vec![]
        };
    Ok(Some(build_bonk_compiled_transaction_with_lookup_preference(
        "follow-sell",
        tx_format,
        &blockhash,
        last_valid_block_height,
        &owner,
        &extra_signer_refs,
        tx_instructions,
        &tx_config,
        &[],
        &preferred_lookup_tables,
    )?))
}

async fn native_compile_atomic_follow_buy_transaction(
    rpc_url: &str,
    launch_mode: &str,
    quote_asset: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    buy_amount: &str,
    allow_ata_creation: bool,
    predicted_prior_buy_quote_amount_b: Option<u64>,
) -> Result<CompiledTransaction, String> {
    let owner = parse_owner_keypair(wallet_secret)?;
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bonk mint address: {error}"))?;
    let launch_creator_pubkey = Pubkey::from_str(launch_creator)
        .map_err(|error| format!("Invalid Bonk launch creator: {error}"))?;
    let defaults = load_bonk_launch_defaults(rpc_url, launch_mode, quote_asset).await?;
    let buy_slippage_bps = slippage_bps_from_percent(&execution.buySlippagePercent)?;
    let mut pool_context = build_prelaunch_bonk_pool_context(
        &defaults,
        &mint_pubkey,
        &launch_creator_pubkey,
        launch_mode,
    )?;
    if let Some(requested_amount_b) = predicted_prior_buy_quote_amount_b {
        pool_context = advance_prelaunch_bonk_pool_context_after_buy(
            &pool_context,
            requested_amount_b,
            buy_slippage_bps,
        )?;
    }
    let requested_amount_b = parse_decimal_u64(
        buy_amount,
        defaults.quote.decimals,
        &format!("follow buy amount {}", defaults.quote.label),
    )?;
    if defaults.quote.asset == "usd1" {
        let current_balance = fetch_bonk_owner_token_balance(
            rpc_url,
            &execution.commitment,
            &owner.pubkey(),
            &bonk_quote_mint("usd1")?,
        )
        .await?
        .unwrap_or_default();
        if current_balance < requested_amount_b {
            let required_quote_amount =
                format_biguint_decimal(&bonk_biguint_from_u64(requested_amount_b), 6, 6);
            let topup_transaction = compile_sol_to_usd1_topup_transaction_with_format(
                rpc_url,
                execution,
                jito_tip_account,
                wallet_secret,
                &required_quote_amount,
                "usd1-topup",
                Some("v0"),
            )
            .await?
            .ok_or_else(|| {
                "Native Bonk atomic USD1 follow buy could not prepare a required top-up transaction."
                    .to_string()
            })?;
            let action_transaction = native_compile_bonk_buy_transaction_with_pool_context(
                rpc_url,
                execution,
                jito_tip_account,
                &owner,
                &mint_pubkey,
                &pool_context,
                requested_amount_b,
                allow_ata_creation,
                NativeBonkTxFormat::V0,
            )
            .await?;
            return combine_atomic_bonk_usd1_follow_buy_transactions(
                rpc_url,
                execution,
                jito_tip_account,
                &owner,
                &topup_transaction,
                &action_transaction,
            )
            .await;
        }
    }
    let tx_format = if defaults.quote.asset == "usd1" {
        NativeBonkTxFormat::V0
    } else {
        select_bonk_native_tx_format(&execution.txFormat)
    };
    native_compile_bonk_buy_transaction_with_pool_context(
        rpc_url,
        execution,
        jito_tip_account,
        &owner,
        &mint_pubkey,
        &pool_context,
        requested_amount_b,
        allow_ata_creation,
        tx_format,
    )
    .await
}

fn build_transaction_summaries(
    compiled_transactions: &[CompiledTransaction],
    dump_base64: bool,
) -> Vec<TransactionSummary> {
    compiled_transactions
        .iter()
        .map(|transaction| {
            let serialized_len = BASE64
                .decode(transaction.serializedBase64.as_bytes())
                .ok()
                .map(|bytes| bytes.len());
            let encoded_len = Some(transaction.serializedBase64.len());
            let mut summary = TransactionSummary {
                label: transaction.label.clone(),
                instructionSummary: Vec::<InstructionSummary>::new(),
                legacyLength: None,
                legacyBase64Length: None,
                v0Length: None,
                v0Base64Length: None,
                v0AltLength: None,
                v0AltBase64Length: None,
                legacyError: None,
                v0Error: None,
                v0AltError: None,
                lookupTablesUsed: transaction.lookupTablesUsed.clone(),
                fitsWithAlts: serialized_len
                    .map(|length| length <= PACKET_LIMIT_BYTES)
                    .unwrap_or(true),
                exceedsPacketLimit: serialized_len
                    .map(|length| length > PACKET_LIMIT_BYTES)
                    .unwrap_or(false),
                feeSettings: FeeSettings {
                    computeUnitLimit: transaction.computeUnitLimit.map(|value| value as i64),
                    computeUnitPriceMicroLamports: transaction
                        .computeUnitPriceMicroLamports
                        .map(|value| value as i64),
                    jitoTipLamports: transaction.inlineTipLamports.unwrap_or_default() as i64,
                    jitoTipAccount: transaction.inlineTipAccount.clone(),
                },
                base64: if dump_base64 {
                    Some(Value::String(transaction.serializedBase64.clone()))
                } else {
                    None
                },
                warnings: vec![],
            };
            match transaction.format.as_str() {
                "legacy" => {
                    summary.legacyLength = serialized_len;
                    summary.legacyBase64Length = encoded_len;
                }
                _ => {
                    summary.v0Length = serialized_len;
                    summary.v0Base64Length = encoded_len;
                }
            }
            summary
        })
        .collect()
}

fn validate_bonk_config(config: &NormalizedConfig) -> Result<(), String> {
    validate_launchpad_support(config).map_err(|error| error.to_string())
}

pub fn supports_native_bonk_compile(config: &NormalizedConfig) -> bool {
    config.launchpad == "bonk" && matches!(config.mode.as_str(), "regular" | "bonkers")
}

pub async fn quote_launch(
    rpc_url: &str,
    quote_asset: &str,
    launch_mode: &str,
    mode: &str,
    amount: &str,
) -> Result<Option<LaunchQuote>, String> {
    if amount.trim().is_empty() {
        return Ok(None);
    }
    let trimmed_mode = mode.trim().to_lowercase();
    let normalized_mode = if trimmed_mode.is_empty() {
        "sol"
    } else {
        trimmed_mode.as_str()
    };
    Ok(Some(
        native_quote_launch(rpc_url, quote_asset, launch_mode, normalized_mode, amount).await?,
    ))
}

async fn native_compile_sol_to_usd1_topup_transaction_with_format(
    rpc_url: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    required_quote_amount: &str,
    label_prefix: &str,
    tx_format_override: Option<&str>,
    route_setup_override: Option<&BonkUsd1RouteSetup>,
) -> Result<Option<CompiledTransaction>, String> {
    let owner = parse_owner_keypair(wallet_secret)?;
    let owner_pubkey = owner.pubkey();
    let required_quote_amount =
        parse_decimal_biguint(required_quote_amount, 6, "required USD1 amount")?;
    if required_quote_amount == BigUint::ZERO {
        return Ok(None);
    }
    let slippage_bps = slippage_bps_from_percent(&execution.buySlippagePercent)?;
    let usd1_mint = bonk_quote_mint("usd1")?;
    let current_quote_amount = bonk_biguint_from_u64(
        fetch_bonk_owner_token_balance(rpc_url, "processed", &owner_pubkey, &usd1_mint)
            .await?
            .unwrap_or(0),
    );
    if current_quote_amount >= required_quote_amount {
        return Ok(None);
    }
    let shortfall_quote_amount = bonk_big_sub(
        &required_quote_amount,
        &current_quote_amount,
        "Bonk USD1 shortfall amount",
    )?;
    let balance_lamports = bonk_rpc_get_balance_lamports(rpc_url, &owner_pubkey).await?;
    let min_remaining_lamports = bonk_usd1_min_remaining_lamports()?;
    let max_spendable_lamports = balance_lamports.saturating_sub(min_remaining_lamports);
    if max_spendable_lamports == 0 {
        return Err(format!(
            "Insufficient SOL headroom for USD1 top-up. Need at least {} SOL reserved after swap.",
            std::env::var("BONK_USD1_MIN_REMAINING_SOL").unwrap_or_else(|_| "0.02".to_string())
        ));
    }
    let input_lamports = native_quote_sol_input_for_usd1_output_with_max(
        rpc_url,
        &shortfall_quote_amount,
        slippage_bps,
        Some(BigUint::from(max_spendable_lamports)),
        route_setup_override,
    )
    .await?;
    let quote = native_quote_usd1_output_from_sol_input_with_metrics(
        rpc_url,
        &input_lamports,
        slippage_bps,
        None,
        route_setup_override,
    )
    .await?;
    if quote.min_out < shortfall_quote_amount {
        return Err("Native Bonk USD1 top-up quote could not satisfy required output.".to_string());
    }
    let amount_in = biguint_to_u64(&input_lamports, "Bonk USD1 top-up input lamports")?;
    let min_out = biguint_to_u64(&quote.min_out, "Bonk USD1 top-up minimum output")?;
    let tx_format = select_bonk_native_tx_format(tx_format_override.unwrap_or(&execution.txFormat));
    let tip_lamports = parse_decimal_u64(&execution.buyTipSol, 9, "buy tip")?;
    let tx_config = NativeBonkTxConfig {
        compute_unit_limit: u32::try_from(configured_default_launch_usd1_topup_compute_unit_limit())
            .map_err(|error| format!("Invalid USD1 top-up compute unit limit: {error}"))?,
        compute_unit_price_micro_lamports: priority_fee_sol_to_micro_lamports(
            &execution.buyPriorityFeeSol,
        )?,
        tip_lamports,
        tip_account: if tip_lamports > 0 {
            Pubkey::from_str(jito_tip_account)
                .map_err(|error| format!("Invalid Jito tip account: {error}"))?
                .to_string()
        } else {
            String::new()
        },
    };
    let token_program = spl_token::id();
    let user_output_account = spl_associated_token_account::get_associated_token_address_with_program_id(
        &owner_pubkey,
        &usd1_mint,
        &token_program,
    );
    let wrapped_signer = Keypair::new();
    let rent_exempt_lamports = rpc_get_minimum_balance_for_rent_exemption(
        rpc_url,
        &execution.commitment,
        BONK_SPL_TOKEN_ACCOUNT_LEN,
    )
    .await?;
    let mut instructions = vec![
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &owner_pubkey,
            &owner_pubkey,
            &usd1_mint,
            &token_program,
        ),
    ];
    instructions.extend(build_bonk_wrapped_sol_open_instructions(
        &owner_pubkey,
        &wrapped_signer.pubkey(),
        rent_exempt_lamports.saturating_add(amount_in),
    )?);
    instructions.push(build_bonk_clmm_swap_exact_in_instruction(
        &owner_pubkey,
        &wrapped_signer.pubkey(),
        &user_output_account,
        amount_in,
        min_out,
        &quote.traversed_tick_array_starts,
    )?);
    instructions.push(build_bonk_wrapped_sol_close_instruction(
        &owner_pubkey,
        &wrapped_signer.pubkey(),
    )?);
    let tx_instructions = with_bonk_tx_settings(
        instructions,
        &tx_config,
        &owner_pubkey,
        execution.buyJitodontfront,
    )?;
    let (blockhash, last_valid_block_height) =
        fetch_latest_blockhash_cached(rpc_url, &execution.commitment).await?;
    let preferred_lookup_tables = if tx_format == NativeBonkTxFormat::V0 {
        load_bonk_preferred_usd1_lookup_tables(rpc_url, &execution.commitment).await
    } else {
        vec![]
    };
    build_bonk_compiled_transaction_with_lookup_preference(
        label_prefix,
        tx_format,
        &blockhash,
        last_valid_block_height,
        &owner,
        &[&wrapped_signer],
        tx_instructions,
        &tx_config,
        &[],
        &preferred_lookup_tables,
    )
    .map(Some)
}

async fn compile_sol_to_usd1_topup_transaction_with_format(
    rpc_url: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    required_quote_amount: &str,
    label_prefix: &str,
    tx_format_override: Option<&str>,
) -> Result<Option<CompiledTransaction>, String> {
    native_compile_sol_to_usd1_topup_transaction_with_format(
        rpc_url,
        execution,
        jito_tip_account,
        wallet_secret,
        required_quote_amount,
        label_prefix,
        tx_format_override,
        None,
    )
    .await
}

pub async fn compile_sol_to_usd1_topup_transaction(
    rpc_url: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    required_quote_amount: &str,
    label_prefix: &str,
) -> Result<Option<CompiledTransaction>, String> {
    compile_sol_to_usd1_topup_transaction_with_format(
        rpc_url,
        execution,
        jito_tip_account,
        wallet_secret,
        required_quote_amount,
        label_prefix,
        None,
    )
    .await
}

fn helper_launch_response_to_native_result(response: HelperLaunchResponse) -> NativeBonkLaunchResult {
    NativeBonkLaunchResult {
        mint: response.mint,
        launch_creator: response.launchCreator,
        compiled_transactions: response
            .compiledTransactions
            .into_iter()
            .map(convert_compiled_transaction)
            .collect(),
        predicted_dev_buy_token_amount_raw: response.predictedDevBuyTokenAmountRaw,
        atomic_combined: response.atomicCombined,
        atomic_fallback_reason: response.atomicFallbackReason,
        usd1_launch_details: response.usd1LaunchDetails.map(|details| NativeBonkUsd1LaunchDetails {
            compile_path: details.compilePath,
            required_quote_amount: details.requiredQuoteAmount,
            current_quote_amount: details.currentQuoteAmount,
            shortfall_quote_amount: details.shortfallQuoteAmount,
            input_sol: details.inputSol,
            expected_quote_out: details.expectedQuoteOut,
            min_quote_out: details.minQuoteOut,
        }),
        usd1_quote_metrics: response.usd1QuoteMetrics,
        compiled_via_native: false,
    }
}

fn build_native_bonk_artifacts_from_launch_result(
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    built_at: String,
    rpc_url: &str,
    creator_public_key: String,
    config_path: Option<String>,
    result: NativeBonkLaunchResult,
) -> Result<NativeBonkArtifacts, String> {
    let compiled_transactions = result.compiled_transactions;
    let mut report = build_report(
        config,
        transport_plan,
        built_at,
        rpc_url.to_string(),
        creator_public_key,
        result.mint.clone(),
        None,
        config_path,
        vec![],
    );
    report.execution.notes.push(if result.compiled_via_native {
        "Bonk launch assembly uses the native Rust compile path.".to_string()
    } else {
        "Bonk launch assembly uses the Raydium LaunchLab SDK-backed compile bridge.".to_string()
    });
    if let Some(backend) = launchpad_action_backend("bonk", "build-launch") {
        let rollout_state = launchpad_action_rollout_state("bonk", "build-launch").unwrap_or("unknown");
        report.execution.notes.push(format!(
            "Launchpad backend owner: {backend} ({rollout_state})."
        ));
    }
    if result.atomic_combined {
        report
            .execution
            .notes
            .push("USD1 dev buy was assembled atomically with the launch transaction.".to_string());
    } else if let Some(reason) = result.atomic_fallback_reason.as_ref() {
        report
            .execution
            .notes
            .push(format!("USD1 dev buy uses split launch transactions: {reason}"));
    }
    if let Some(details) = result.usd1_launch_details.as_ref() {
        report.bonkUsd1Launch = Some(BonkUsd1LaunchSummary {
            compilePath: details.compile_path.clone(),
            currentQuoteAmount: details.current_quote_amount.clone(),
            requiredQuoteAmount: details.required_quote_amount.clone(),
            shortfallQuoteAmount: details.shortfall_quote_amount.clone(),
            inputSol: details.input_sol.clone(),
            expectedQuoteOut: details.expected_quote_out.clone(),
            minQuoteOut: details.min_quote_out.clone(),
            atomicFallbackReason: result.atomic_fallback_reason.clone(),
        });
    }
    if let Some(metrics_note) = result
        .usd1_quote_metrics
        .as_ref()
        .and_then(render_usd1_quote_metrics_note)
    {
        report.execution.notes.push(metrics_note);
    }
    report.transactions = build_transaction_summaries(&compiled_transactions, config.tx.dumpBase64);
    let text = render_report(&report);
    let report = serde_json::to_value(report).map_err(|error| error.to_string())?;
    Ok(NativeBonkArtifacts {
        creation_transactions: compiled_transactions.clone(),
        deferred_setup_transactions: vec![],
        compiled_transactions,
        report,
        text,
        compile_timings: NativeCompileTimings::default(),
        mint: result.mint,
        launch_creator: result.launch_creator,
    })
}

async fn resolve_bonk_launch_mint_keypair(rpc_url: &str, vanity_private_key: &str) -> Result<Keypair, String> {
    let trimmed = vanity_private_key.trim();
    if trimmed.is_empty() {
        return Ok(Keypair::new());
    }
    let bytes = read_keypair_bytes(trimmed).map_err(|error| format!("Invalid vanity private key: {error}"))?;
    let keypair = Keypair::try_from(bytes.as_slice())
        .map_err(|error| format!("Invalid vanity private key: {error}"))?;
    match fetch_account_data(rpc_url, &keypair.pubkey().to_string(), "confirmed").await {
        Ok(_) => Err(format!(
            "This vanity address has already been used on-chain. Generate a fresh one. ({})",
            keypair.pubkey()
        )),
        Err(error) if error.contains("was not found.") => Ok(keypair),
        Err(error) => Err(format!(
            "Failed to verify vanity private key availability: {error}"
        )),
    }
}

fn build_native_bonk_launch_dev_buy_instructions(
    owner: &Pubkey,
    mint: &Pubkey,
    pool_context: &NativeBonkPoolContext,
    requested_amount_b: &BigUint,
    slippage_bps: u64,
    min_amount_a_override: Option<&BigUint>,
    allow_ata_creation: bool,
) -> Result<Vec<Instruction>, String> {
    let token_program = spl_token::id();
    let user_token_account_a =
        spl_associated_token_account::get_associated_token_address_with_program_id(owner, mint, &token_program);
    let mut instructions = vec![
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            owner,
            owner,
            mint,
            &token_program,
        ),
    ];
    let (instruction_amount_b, min_amount_a) = if let Some(min_override) = min_amount_a_override {
        (
            biguint_to_u64(requested_amount_b, "launch dev buy amount")?,
            biguint_to_u64(min_override, "launch dev buy min token output")?,
        )
    } else {
        bonk_follow_buy_amounts(
            pool_context,
            biguint_to_u64(requested_amount_b, "launch dev buy amount")?,
            slippage_bps,
        )?
    };
    let user_token_account_b = if pool_context.quote.asset == "sol" {
        let wrapped_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
            owner,
            &bonk_quote_mint("sol")?,
            &token_program,
        );
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                owner,
                owner,
                &bonk_quote_mint("sol")?,
                &token_program,
            ),
        );
        instructions.push(solana_system_interface::instruction::transfer(
            owner,
            &wrapped_ata,
            instruction_amount_b,
        ));
        instructions.push(
            spl_token::instruction::sync_native(&token_program, &wrapped_ata)
                .map_err(|error| format!("Failed to build launch sync-native instruction: {error}"))?,
        );
        wrapped_ata
    } else {
        let quote_mint = bonk_quote_mint(pool_context.quote.asset)?;
        let quote_ata =
            spl_associated_token_account::get_associated_token_address_with_program_id(owner, &quote_mint, &token_program);
        if allow_ata_creation {
            instructions.push(
                spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                    owner,
                    owner,
                    &quote_mint,
                    &token_program,
                ),
            );
        }
        quote_ata
    };
    instructions.push(build_bonk_buy_exact_in_instruction(
        owner,
        pool_context,
        &user_token_account_a,
        &user_token_account_b,
        instruction_amount_b,
        min_amount_a,
    )?);
    Ok(instructions)
}

async fn native_compile_bonk_usd1_topup_from_prepared(
    rpc_url: &str,
    commitment: &str,
    owner: &Keypair,
    allow_ata_creation: bool,
    label_prefix: &str,
    tx_format: NativeBonkTxFormat,
    tx_config: &NativeBonkTxConfig,
    jitodontfront_enabled: bool,
    prepared: &NativeBonkPreparedUsd1Topup,
) -> Result<Option<CompiledTransaction>, String> {
    let Some(input_lamports) = prepared.input_lamports.as_ref() else {
        return Ok(None);
    };
    let owner_pubkey = owner.pubkey();
    let token_program = spl_token::id();
    let usd1_mint = bonk_quote_mint("usd1")?;
    let user_output_account = spl_associated_token_account::get_associated_token_address_with_program_id(
        &owner_pubkey,
        &usd1_mint,
        &token_program,
    );
    let wrapped_signer = Keypair::new();
    let rent_exempt_lamports =
        rpc_get_minimum_balance_for_rent_exemption(rpc_url, commitment, BONK_SPL_TOKEN_ACCOUNT_LEN).await?;
    let mut instructions = Vec::new();
    if allow_ata_creation {
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                &owner_pubkey,
                &owner_pubkey,
                &usd1_mint,
                &token_program,
            ),
        );
    }
    instructions.extend(build_bonk_wrapped_sol_open_instructions(
        &owner_pubkey,
        &wrapped_signer.pubkey(),
        rent_exempt_lamports.saturating_add(biguint_to_u64(input_lamports, "Bonk USD1 top-up input lamports")?),
    )?);
    instructions.push(build_bonk_clmm_swap_exact_in_instruction(
        &owner_pubkey,
        &wrapped_signer.pubkey(),
        &user_output_account,
        biguint_to_u64(input_lamports, "Bonk USD1 top-up input lamports")?,
        biguint_to_u64(
            prepared
                .min_quote_out
                .as_ref()
                .ok_or_else(|| "Bonk USD1 top-up minimum output was missing.".to_string())?,
            "Bonk USD1 top-up minimum output",
        )?,
        &prepared.traversed_tick_array_starts,
    )?);
    instructions.push(build_bonk_wrapped_sol_close_instruction(
        &owner_pubkey,
        &wrapped_signer.pubkey(),
    )?);
    let tx_instructions = with_bonk_tx_settings(instructions, tx_config, &owner_pubkey, jitodontfront_enabled)?;
    let (blockhash, last_valid_block_height) = fetch_latest_blockhash_cached(rpc_url, commitment).await?;
    let preferred_lookup_tables = if tx_format == NativeBonkTxFormat::V0 {
        load_bonk_preferred_usd1_lookup_tables(rpc_url, commitment).await
    } else {
        vec![]
    };
    Ok(Some(build_bonk_compiled_transaction_with_lookup_preference(
        label_prefix,
        tx_format,
        &blockhash,
        last_valid_block_height,
        owner,
        &[&wrapped_signer],
        tx_instructions,
        tx_config,
        &[],
        &preferred_lookup_tables,
    )?))
}

async fn combine_atomic_bonk_transactions(
    rpc_url: &str,
    commitment: &str,
    owner: &Keypair,
    label: &str,
    tx_config: &NativeBonkTxConfig,
    jitodontfront_enabled: bool,
    extra_signers: &[&Keypair],
    topup_transaction: &CompiledTransaction,
    action_transaction: &CompiledTransaction,
) -> Result<CompiledTransaction, String> {
    let topup = decompose_bonk_compiled_v0_transaction(rpc_url, topup_transaction, commitment).await?;
    let action = decompose_bonk_compiled_v0_transaction(rpc_url, action_transaction, commitment).await?;
    let owner_pubkey = owner.pubkey();
    let swap_instructions = filter_atomic_bonk_instructions(topup.instructions, &owner_pubkey, tx_config);
    let action_instructions = filter_atomic_bonk_instructions(action.instructions, &owner_pubkey, tx_config);
    let mut merged_instructions = build_bonk_atomic_tx_instructions(
        swap_instructions
            .into_iter()
            .chain(action_instructions.into_iter())
            .collect(),
        tx_config,
        &owner_pubkey,
        jitodontfront_enabled,
    )?;
    let generated_signers =
        rewrite_missing_bonk_instruction_signers(&owner_pubkey, &mut merged_instructions, extra_signers);
    let mut merged_signers = extra_signers.to_vec();
    let generated_signer_refs = generated_signers.iter().collect::<Vec<_>>();
    merged_signers.extend(generated_signer_refs.iter().copied());
    let merged_lookup_tables = merge_bonk_lookup_tables(&[topup.lookup_tables, action.lookup_tables]);
    let (blockhash, last_valid_block_height) = fetch_latest_blockhash_cached(rpc_url, commitment).await?;
    let preferred_lookup_tables = load_bonk_preferred_usd1_lookup_tables(rpc_url, commitment).await;
    build_bonk_compiled_transaction_with_lookup_preference(
        label,
        NativeBonkTxFormat::V0,
        &blockhash,
        last_valid_block_height,
        owner,
        &merged_signers,
        merged_instructions,
        tx_config,
        &merged_lookup_tables,
        &preferred_lookup_tables,
    )
}

async fn native_build_launch_result(
    rpc_url: &str,
    config: &NormalizedConfig,
    wallet_secret: &[u8],
    allow_ata_creation: bool,
) -> Result<NativeBonkLaunchResult, String> {
    let owner = parse_owner_keypair(wallet_secret)?;
    let owner_pubkey = owner.pubkey();
    let defaults = load_bonk_launch_defaults(rpc_url, &config.mode, &config.quoteAsset).await?;
    let slippage_bps = slippage_bps_from_percent(&config.execution.buySlippagePercent)?;
    let mint_keypair = resolve_bonk_launch_mint_keypair(rpc_url, &config.vanityPrivateKey).await?;
    let mint_pubkey = mint_keypair.pubkey();
    let predicted_dev_buy_token_amount_raw = native_predict_dev_buy_token_amount(rpc_url, config)
        .await?
        .map(|value| value.to_string());
    let create_only = config
        .devBuy
        .as_ref()
        .map(|dev_buy| dev_buy.mode.trim().is_empty() || dev_buy.amount.trim().is_empty())
        .unwrap_or(true);
    let tx_format = if defaults.quote.asset == "usd1" {
        NativeBonkTxFormat::V0
    } else {
        select_bonk_native_tx_format(&config.execution.txFormat)
    };
    let launch_tx_config = bonk_launch_tx_config(config)?;
    let mut usd1_quote_metrics = if defaults.quote.asset == "usd1" {
        Some(HelperUsd1QuoteMetrics::default())
    } else {
        None
    };
    let preferred_lookup_tables =
        if tx_format == NativeBonkTxFormat::V0 && defaults.quote.asset == "usd1" {
            load_bonk_preferred_usd1_lookup_tables_with_metrics(
                rpc_url,
                &config.execution.commitment,
                usd1_quote_metrics.as_mut(),
            )
            .await
        } else {
            vec![]
        };
    let single_bundle_tip_last_tx =
        uses_single_bundle_tip_last_tx(&config.execution.provider, &config.execution.mevMode);
    let mut launch_instructions = vec![build_bonk_initialize_v2_instruction(
        &owner_pubkey,
        &mint_pubkey,
        &config.mode,
        &config.token.name,
        &config.token.symbol,
        &config.token.uri,
        &defaults,
    )?];
    let mut prepared_usd1_topup = None;
    let mut usd1_launch_details = None;
    if !create_only {
        let dev_buy = config
            .devBuy
            .as_ref()
            .ok_or_else(|| "Bonk dev buy was missing after create-only detection.".to_string())?;
        let prelaunch_pool_context =
            build_prelaunch_bonk_pool_context(&defaults, &mint_pubkey, &owner_pubkey, &config.mode)?;
        let mut min_mint_a_amount = None;
        let requested_amount_b = if dev_buy.mode.trim().eq_ignore_ascii_case("tokens") {
            let requested_tokens =
                parse_decimal_biguint(&dev_buy.amount, BONK_TOKEN_DECIMALS, "dev buy tokens")?;
            let required_quote_amount =
                bonk_quote_buy_exact_out_amount_b(&defaults, &requested_tokens)?;
            min_mint_a_amount = Some(bonk_build_min_amount_from_bps(&requested_tokens, slippage_bps));
            required_quote_amount
        } else if defaults.quote.asset == "usd1" {
            let input_sol = parse_decimal_biguint(&dev_buy.amount, 9, "dev buy SOL")?;
            let usd1_route_quote = native_quote_usd1_output_from_sol_input_with_metrics(
                rpc_url,
                &input_sol,
                slippage_bps,
                usd1_quote_metrics.as_mut(),
                None,
            )
            .await?;
            usd1_route_quote.min_out
        } else {
            parse_decimal_biguint(
                &dev_buy.amount,
                defaults.quote.decimals,
                &format!("dev buy {}", defaults.quote.label),
            )?
        };
        if defaults.quote.asset == "usd1" {
            let prepared = native_prepare_bonk_usd1_topup(
                rpc_url,
                &config.execution.commitment,
                &owner_pubkey,
                &requested_amount_b,
                slippage_bps,
                usd1_quote_metrics.as_mut(),
                None,
            )
            .await?;
            prepared_usd1_topup = Some(prepared.clone());
            usd1_launch_details = Some(NativeBonkUsd1LaunchDetails {
                compile_path: if prepared.input_lamports.is_some() {
                    "split-topup+launch".to_string()
                } else {
                    "launch-only".to_string()
                },
                required_quote_amount: format_biguint_decimal(&prepared.required_quote_amount, 6, 6),
                current_quote_amount: format_biguint_decimal(&prepared.current_quote_amount, 6, 6),
                shortfall_quote_amount: format_biguint_decimal(&prepared.shortfall_quote_amount, 6, 6),
                input_sol: prepared
                    .input_lamports
                    .as_ref()
                    .map(|value| format_biguint_decimal(value, 9, 6)),
                expected_quote_out: prepared
                    .expected_quote_out
                    .as_ref()
                    .map(|value| format_biguint_decimal(value, 6, 6)),
                min_quote_out: prepared
                    .min_quote_out
                    .as_ref()
                    .map(|value| format_biguint_decimal(value, 6, 6)),
            });
        }
        launch_instructions.extend(build_native_bonk_launch_dev_buy_instructions(
            &owner_pubkey,
            &mint_pubkey,
            &prelaunch_pool_context,
            &requested_amount_b,
            slippage_bps,
            min_mint_a_amount.as_ref(),
            allow_ata_creation,
        )?);
    }
    let (blockhash, last_valid_block_height) =
        fetch_latest_blockhash_cached(rpc_url, &config.execution.commitment).await?;
    let mint_signer_refs = [&mint_keypair];
    let mut compiled_launch_transactions = split_bonk_instruction_bundle(
        "launch",
        tx_format,
        &blockhash,
        last_valid_block_height,
        &owner,
        &mint_signer_refs,
        launch_instructions,
        &launch_tx_config,
        config.execution.jitodontfront,
        single_bundle_tip_last_tx,
        &preferred_lookup_tables,
    )?;
    let mut atomic_combined = false;
    let mut atomic_fallback_reason = None;
    if let Some(prepared) = prepared_usd1_topup.as_ref() {
        if let Some(topup_transaction) = native_compile_bonk_usd1_topup_from_prepared(
            rpc_url,
            &config.execution.commitment,
            &owner,
            allow_ata_creation,
            "launch-usd1-topup",
            NativeBonkTxFormat::V0,
            &bonk_bundle_tx_config_for_index(
                &launch_tx_config,
                0,
                compiled_launch_transactions.len() + 1,
                single_bundle_tip_last_tx,
            ),
            config.execution.jitodontfront,
            prepared,
        )
        .await?
        {
            if compiled_launch_transactions.len() == 1 {
                match combine_atomic_bonk_transactions(
                    rpc_url,
                    &config.execution.commitment,
                    &owner,
                    "launch",
                    &launch_tx_config,
                    config.execution.jitodontfront,
                    &mint_signer_refs,
                    &topup_transaction,
                    compiled_launch_transactions.first().expect("launch tx"),
                )
                .await
                {
                    Ok(combined) => {
                        atomic_combined = true;
                        compiled_launch_transactions = vec![combined];
                        if let Some(details) = usd1_launch_details.as_mut() {
                            details.compile_path = "atomic-topup+launch".to_string();
                        }
                    }
                    Err(error) => {
                        atomic_fallback_reason = Some(format!("Atomic USD1 launch fallback: {error}"));
                        compiled_launch_transactions.insert(0, topup_transaction);
                    }
                }
            } else {
                atomic_fallback_reason = Some(
                    "Atomic USD1 launch requires exactly one top-up transaction and one launch transaction."
                        .to_string(),
                );
                compiled_launch_transactions.insert(0, topup_transaction);
            }
            if !atomic_combined && atomic_fallback_reason.is_none() {
                atomic_fallback_reason =
                    Some("USD1 launch path is using split top-up plus launch transactions.".to_string());
            }
        }
    }
    Ok(NativeBonkLaunchResult {
        mint: mint_pubkey.to_string(),
        launch_creator: owner_pubkey.to_string(),
        compiled_transactions: compiled_launch_transactions,
        predicted_dev_buy_token_amount_raw,
        atomic_combined,
        atomic_fallback_reason,
        usd1_launch_details,
        usd1_quote_metrics: usd1_quote_metrics.and_then(|metrics| {
            if render_usd1_quote_metrics_note(&metrics).is_some() {
                Some(metrics)
            } else {
                None
            }
        }),
        compiled_via_native: true,
    })
}

pub async fn try_compile_native_bonk(
    rpc_url: &str,
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    wallet_secret: &[u8],
    built_at: String,
    creator_public_key: String,
    config_path: Option<String>,
    allow_ata_creation: bool,
) -> Result<Option<NativeBonkArtifacts>, String> {
    if config.launchpad != "bonk" {
        return Ok(None);
    }
    validate_bonk_config(config)?;
    let launch_result =
        native_build_launch_result(rpc_url, config, wallet_secret, allow_ata_creation).await?;
    Ok(Some(build_native_bonk_artifacts_from_launch_result(
        config,
        transport_plan,
        built_at,
        rpc_url,
        creator_public_key,
        config_path,
        launch_result,
    )?))
}

pub async fn compile_follow_buy_transaction(
    rpc_url: &str,
    quote_asset: &str,
    execution: &NormalizedExecution,
    _token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    _launch_creator: &str,
    buy_amount_sol: &str,
    allow_ata_creation: bool,
    pool_context_override: Option<&NativeBonkPoolContext>,
    usd1_route_setup_override: Option<&BonkUsd1RouteSetup>,
) -> Result<CompiledTransaction, String> {
    native_compile_follow_buy_transaction(
        rpc_url,
        quote_asset,
        execution,
        jito_tip_account,
        wallet_secret,
        mint,
        buy_amount_sol,
        allow_ata_creation,
        pool_context_override,
        usd1_route_setup_override,
    )
    .await
}

pub async fn compile_atomic_follow_buy_transaction(
    rpc_url: &str,
    launch_mode: &str,
    quote_asset: &str,
    execution: &NormalizedExecution,
    _token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    buy_amount_sol: &str,
    allow_ata_creation: bool,
    predicted_prior_buy_quote_amount_b: Option<u64>,
) -> Result<CompiledTransaction, String> {
    native_compile_atomic_follow_buy_transaction(
        rpc_url,
        launch_mode,
        quote_asset,
        execution,
        jito_tip_account,
        wallet_secret,
        mint,
        launch_creator,
        buy_amount_sol,
        allow_ata_creation,
        predicted_prior_buy_quote_amount_b,
    )
    .await
}

pub async fn compile_follow_sell_transaction(
    rpc_url: &str,
    quote_asset: &str,
    execution: &NormalizedExecution,
    _token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    _launch_creator: &str,
    sell_percent: u8,
    _prefer_post_setup_creator_vault: bool,
) -> Result<Option<CompiledTransaction>, String> {
    compile_follow_sell_transaction_with_token_amount(
        rpc_url,
        quote_asset,
        execution,
        jito_tip_account,
        wallet_secret,
        mint,
        sell_percent,
        None,
        None,
        None,
        None,
    )
    .await
}

pub async fn compile_follow_sell_transaction_with_token_amount(
    rpc_url: &str,
    quote_asset: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    sell_percent: u8,
    token_amount_override: Option<u64>,
    pool_id_override: Option<&str>,
    launch_mode_override: Option<&str>,
    launch_creator_override: Option<&str>,
) -> Result<Option<CompiledTransaction>, String> {
    native_compile_follow_sell_transaction_with_token_amount(
        rpc_url,
        quote_asset,
        execution,
        jito_tip_account,
        wallet_secret,
        mint,
        sell_percent,
        token_amount_override,
        pool_id_override,
        launch_mode_override,
        launch_creator_override,
    )
    .await
}

pub async fn predict_dev_buy_token_amount(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<Option<u64>, String> {
    native_predict_dev_buy_token_amount(rpc_url, config).await
}

pub async fn predict_dev_buy_effect(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<Option<BonkPredictedDevBuyEffect>, String> {
    native_predict_dev_buy_effect(rpc_url, config).await
}

pub async fn load_live_follow_buy_pool_context(
    rpc_url: &str,
    mint: &str,
    quote_asset: &str,
    commitment: &str,
) -> Result<NativeBonkPoolContext, String> {
    let mint =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bonk mint address: {error}"))?;
    load_live_bonk_pool_context(rpc_url, &mint, quote_asset, commitment).await
}

pub async fn load_live_follow_buy_usd1_route_setup(
    rpc_url: &str,
) -> Result<BonkUsd1RouteSetup, String> {
    load_bonk_usd1_route_setup_fresh(rpc_url).await
}

pub async fn derive_canonical_pool_id(quote_asset: &str, mint: &str) -> Result<String, String> {
    let launchpad_program = bonk_launchpad_program_id()?;
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bonk mint address: {error}"))?;
    let quote_pubkey = bonk_quote_mint(quote_asset)?;
    let (pool_id, _) = Pubkey::find_program_address(
        &[b"pool", mint_pubkey.as_ref(), quote_pubkey.as_ref()],
        &launchpad_program,
    );
    Ok(pool_id.to_string())
}

pub async fn fetch_bonk_market_snapshot(
    rpc_url: &str,
    mint: &str,
    quote_asset: &str,
) -> Result<BonkMarketSnapshot, String> {
    native_fetch_bonk_market_snapshot(rpc_url, mint, quote_asset).await
}

pub async fn detect_bonk_import_context(
    rpc_url: &str,
    mint: &str,
) -> Result<Option<BonkImportContext>, String> {
    native_detect_bonk_import_context(rpc_url, mint).await
}

pub async fn detect_bonk_import_context_with_quote_asset(
    rpc_url: &str,
    mint: &str,
    quote_asset: &str,
) -> Result<Option<BonkImportContext>, String> {
    native_detect_bonk_import_context_with_quote_asset(rpc_url, mint, quote_asset).await
}

pub async fn poll_bonk_market_cap_lamports(
    rpc_url: &str,
    mint: &str,
    quote_asset: &str,
) -> Result<Option<u64>, String> {
    let snapshot = fetch_bonk_market_snapshot(rpc_url, mint, quote_asset).await?;
    let value = snapshot
        .marketCapLamports
        .parse::<u64>()
        .map_err(|error| format!("Invalid Bonk market cap response: {error}"))?;
    Ok(Some(value))
}

pub async fn warm_bonk_state(rpc_url: &str) -> Result<Value, String> {
    let (regular_sol, regular_usd1, bonkers_sol, bonkers_usd1) = tokio::try_join!(
        load_bonk_launch_defaults(rpc_url, "regular", "sol"),
        load_bonk_launch_defaults(rpc_url, "regular", "usd1"),
        load_bonk_launch_defaults(rpc_url, "bonkers", "sol"),
        load_bonk_launch_defaults(rpc_url, "bonkers", "usd1"),
    )?;
    let helper_launch_defaults = vec![
        json!({
            "mode": "regular",
            "quoteAsset": regular_sol.quote.asset,
            "platformId": bonk_platform_id("regular"),
            "configId": bonk_launch_config_id(regular_sol.quote.asset)?,
            "quoteMint": bonk_quote_mint(regular_sol.quote.asset)?.to_string(),
        }),
        json!({
            "mode": "regular",
            "quoteAsset": regular_usd1.quote.asset,
            "platformId": bonk_platform_id("regular"),
            "configId": bonk_launch_config_id(regular_usd1.quote.asset)?,
            "quoteMint": bonk_quote_mint(regular_usd1.quote.asset)?.to_string(),
        }),
        json!({
            "mode": "bonkers",
            "quoteAsset": bonkers_sol.quote.asset,
            "platformId": bonk_platform_id("bonkers"),
            "configId": bonk_launch_config_id(bonkers_sol.quote.asset)?,
            "quoteMint": bonk_quote_mint(bonkers_sol.quote.asset)?.to_string(),
        }),
        json!({
            "mode": "bonkers",
            "quoteAsset": bonkers_usd1.quote.asset,
            "platformId": bonk_platform_id("bonkers"),
            "configId": bonk_launch_config_id(bonkers_usd1.quote.asset)?,
            "quoteMint": bonk_quote_mint(bonkers_usd1.quote.asset)?.to_string(),
        }),
    ];
    let preview_launch_defaults = vec![
        json!({
            "mode": "regular",
            "quoteAsset": regular_sol.quote.asset,
            "quoteAssetLabel": regular_sol.quote.label,
            "quoteDecimals": regular_sol.quote.decimals,
            "supply": regular_sol.supply.to_string(),
            "totalFundRaisingB": regular_sol.total_fund_raising_b.to_string(),
            "tradeFeeRate": regular_sol.trade_fee_rate.to_string(),
            "platformFeeRate": regular_sol.platform_fee_rate.to_string(),
            "creatorFeeRate": regular_sol.creator_fee_rate.to_string(),
            "curveType": regular_sol.curve_type,
            "pool": {
                "totalSellA": regular_sol.pool.total_sell_a.to_string(),
                "virtualA": regular_sol.pool.virtual_a.to_string(),
                "virtualB": regular_sol.pool.virtual_b.to_string(),
                "realA": regular_sol.pool.real_a.to_string(),
                "realB": regular_sol.pool.real_b.to_string(),
            }
        }),
        json!({
            "mode": "regular",
            "quoteAsset": regular_usd1.quote.asset,
            "quoteAssetLabel": regular_usd1.quote.label,
            "quoteDecimals": regular_usd1.quote.decimals,
            "supply": regular_usd1.supply.to_string(),
            "totalFundRaisingB": regular_usd1.total_fund_raising_b.to_string(),
            "tradeFeeRate": regular_usd1.trade_fee_rate.to_string(),
            "platformFeeRate": regular_usd1.platform_fee_rate.to_string(),
            "creatorFeeRate": regular_usd1.creator_fee_rate.to_string(),
            "curveType": regular_usd1.curve_type,
            "pool": {
                "totalSellA": regular_usd1.pool.total_sell_a.to_string(),
                "virtualA": regular_usd1.pool.virtual_a.to_string(),
                "virtualB": regular_usd1.pool.virtual_b.to_string(),
                "realA": regular_usd1.pool.real_a.to_string(),
                "realB": regular_usd1.pool.real_b.to_string(),
            }
        }),
        json!({
            "mode": "bonkers",
            "quoteAsset": bonkers_sol.quote.asset,
            "quoteAssetLabel": bonkers_sol.quote.label,
            "quoteDecimals": bonkers_sol.quote.decimals,
            "supply": bonkers_sol.supply.to_string(),
            "totalFundRaisingB": bonkers_sol.total_fund_raising_b.to_string(),
            "tradeFeeRate": bonkers_sol.trade_fee_rate.to_string(),
            "platformFeeRate": bonkers_sol.platform_fee_rate.to_string(),
            "creatorFeeRate": bonkers_sol.creator_fee_rate.to_string(),
            "curveType": bonkers_sol.curve_type,
            "pool": {
                "totalSellA": bonkers_sol.pool.total_sell_a.to_string(),
                "virtualA": bonkers_sol.pool.virtual_a.to_string(),
                "virtualB": bonkers_sol.pool.virtual_b.to_string(),
                "realA": bonkers_sol.pool.real_a.to_string(),
                "realB": bonkers_sol.pool.real_b.to_string(),
            }
        }),
        json!({
            "mode": "bonkers",
            "quoteAsset": bonkers_usd1.quote.asset,
            "quoteAssetLabel": bonkers_usd1.quote.label,
            "quoteDecimals": bonkers_usd1.quote.decimals,
            "supply": bonkers_usd1.supply.to_string(),
            "totalFundRaisingB": bonkers_usd1.total_fund_raising_b.to_string(),
            "tradeFeeRate": bonkers_usd1.trade_fee_rate.to_string(),
            "platformFeeRate": bonkers_usd1.platform_fee_rate.to_string(),
            "creatorFeeRate": bonkers_usd1.creator_fee_rate.to_string(),
            "curveType": bonkers_usd1.curve_type,
            "pool": {
                "totalSellA": bonkers_usd1.pool.total_sell_a.to_string(),
                "virtualA": bonkers_usd1.pool.virtual_a.to_string(),
                "virtualB": bonkers_usd1.pool.virtual_b.to_string(),
                "realA": bonkers_usd1.pool.real_a.to_string(),
                "realB": bonkers_usd1.pool.real_b.to_string(),
            }
        }),
    ];
    let payload = json!({
        "ok": true,
        "backend": launchpad_action_backend("bonk", "startup-warm"),
        "rolloutState": launchpad_action_rollout_state("bonk", "startup-warm"),
        "launchDefaults": helper_launch_defaults.clone(),
        "warmedLaunchDefaults": helper_launch_defaults,
        "previewBasis": {
            "launchDefaults": preview_launch_defaults,
        },
        "usd1RoutePoolId": BONK_PINNED_USD1_ROUTE_POOL_ID,
        "usd1RouteConfigId": BONK_PREFERRED_USD1_ROUTE_CONFIG_ID,
        "usd1QuoteMetrics": HelperUsd1QuoteMetrics::default(),
    });
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_rpc_follow_tip_is_ignored_when_blank() {
        let tip_lamports =
            resolve_follow_tip_lamports("standard-rpc", "", "buy tip").expect("standard rpc tip");
        assert_eq!(tip_lamports, 0);
    }

    #[test]
    fn standard_rpc_follow_tip_is_ignored_even_when_present() {
        let tip_lamports = resolve_follow_tip_lamports("standard-rpc", "0.01", "sell tip")
            .expect("standard rpc tip");
        assert_eq!(tip_lamports, 0);
    }

    #[test]
    fn jito_follow_tip_preserves_non_blank_tip_value() {
        let tip_lamports =
            resolve_follow_tip_lamports("jito-bundle", "0.01", "buy tip").expect("jito tip");
        assert!(tip_lamports > 0);
    }

    #[test]
    fn hellomoon_follow_tip_requires_at_least_point_zero_zero_one_sol() {
        assert_eq!(
            resolve_follow_tip_lamports("hellomoon", "0.001", "buy tip").expect("tip"),
            1_000_000
        );
        resolve_follow_tip_lamports("hellomoon", "", "buy tip").expect_err("empty tip");
        let error = resolve_follow_tip_lamports("hellomoon", "0.0001", "buy tip")
            .expect_err("sub-minimum tip");
        assert!(error.contains("0.001 SOL"), "unexpected: {error}");
    }

    #[test]
    fn slippage_percent_maps_to_expected_bps() {
        assert_eq!(slippage_bps_from_percent("20").expect("20%"), 2_000);
        assert_eq!(slippage_bps_from_percent("0.5").expect("0.5%"), 50);
        assert_eq!(slippage_bps_from_percent("100").expect("100%"), 10_000);
    }

    #[test]
    fn migrated_launch_pool_is_preferred_over_non_canonical_pool() {
        let migrated = BonkMarketCandidate {
            mode: "regular".to_string(),
            quote_asset: "sol".to_string(),
            quote_asset_label: "SOL".to_string(),
            creator: String::new(),
            platform_id: String::new(),
            config_id: String::new(),
            pool_id: "migrated".to_string(),
            real_quote_reserves: 0,
            complete: true,
            detection_source: "raydium-standard".to_string(),
            launch_migrate_pool: true,
            tvl: 10.0,
            pool_type: "Standard".to_string(),
            launchpad_pool: None,
            raydium_pool: None,
        };
        let launchpad = BonkMarketCandidate {
            launch_migrate_pool: false,
            pool_type: "LaunchLab".to_string(),
            pool_id: "launchpad".to_string(),
            ..migrated.clone()
        };
        let candidates = vec![launchpad, migrated];
        let preferred =
            select_preferred_bonk_market_candidate(&candidates, "sol").expect("preferred");
        assert_eq!(preferred.pool_id, "migrated");
    }

    #[test]
    fn raydium_pools_response_accepts_nested_data_shape() {
        let payload: RaydiumPoolsResponse = serde_json::from_value(json!({
            "id": "resp",
            "success": true,
            "data": {
                "count": 1,
                "data": [
                    {
                        "id": "pool-a",
                        "price": 123.45,
                        "tvl": 999.0,
                        "type": "Standard",
                        "launchMigratePool": true,
                        "mintA": { "address": "So11111111111111111111111111111111111111112" },
                        "mintB": { "address": "mint-b" },
                        "config": { "id": "cfg" }
                    }
                ],
                "hasNextPage": false
            }
        }))
        .expect("nested response should decode");
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].id, "pool-a");
        assert!(payload.data[0].launch_migrate_pool);
        assert_eq!(payload.data[0].mint_a.address, "So11111111111111111111111111111111111111112");
    }

    #[test]
    fn migrated_raydium_market_cap_avoids_u128_overflow() {
        let pool = RaydiumPoolInfo {
            id: "pool-a".to_string(),
            price: 105_103.454_806_931_16,
            tvl: 999.0,
            pool_type: "Standard".to_string(),
            launch_migrate_pool: true,
            mint_a: RaydiumTokenAddress {
                address: "So11111111111111111111111111111111111111112".to_string(),
            },
            mint_b: RaydiumTokenAddress {
                address: "HtTYHz1Kf3rrQo6AqDLmss7gq5WrkWAaXn3tupUZbonk".to_string(),
            },
            config: None,
        };
        let market_cap = market_cap_from_raydium_pool_price(
            &pool,
            999_866_905_447_231,
            6,
            &bonk_quote_asset_config("sol"),
        )
        .expect("market cap");
        assert_eq!(market_cap, 9_513_168_784_831);
    }

    #[test]
    fn requested_quote_asset_breaks_ties_between_migrated_pools() {
        let sol = BonkMarketCandidate {
            mode: "regular".to_string(),
            quote_asset: "sol".to_string(),
            quote_asset_label: "SOL".to_string(),
            creator: String::new(),
            platform_id: String::new(),
            config_id: String::new(),
            pool_id: "sol".to_string(),
            real_quote_reserves: 0,
            complete: true,
            detection_source: "raydium-standard".to_string(),
            launch_migrate_pool: false,
            tvl: 25.0,
            pool_type: "Standard".to_string(),
            launchpad_pool: None,
            raydium_pool: None,
        };
        let usd1 = BonkMarketCandidate {
            quote_asset: "usd1".to_string(),
            quote_asset_label: "USD1".to_string(),
            pool_id: "usd1".to_string(),
            ..sol.clone()
        };
        let candidates = vec![sol, usd1];
        let preferred =
            select_preferred_bonk_market_candidate(&candidates, "usd1").expect("preferred");
        assert_eq!(preferred.pool_id, "usd1");
    }

    #[test]
    fn bonk_launch_config_id_matches_raydium_sdk_pda_layout() {
        assert_eq!(
            bonk_launch_config_id("sol").expect("sol config"),
            "6s1xP3hpbAfFoNtUNF8mfHsjr2Bd97JxFJRWLbL6aHuX"
        );
        assert_eq!(
            bonk_launch_config_id("usd1").expect("usd1 config"),
            "EPiZbnrThjyLnoQ6QQzkxeFqyL5uyg9RzNHHAudUPxBz"
        );
    }

    #[test]
    fn parse_raydium_launch_configs_payload_accepts_nested_api_shape() {
        let configs = parse_raydium_launch_configs_payload(json!({
            "success": true,
            "data": {
                "data": [{
                    "key": { "pubKey": "6s1xP3hpbAfFoNtUNF8mfHsjr2Bd97JxFJRWLbL6aHuX" },
                    "defaultParams": {
                        "supplyInit": "1000",
                        "totalFundRaisingB": "2000",
                        "totalSellA": "3000"
                    }
                }]
            }
        }))
        .expect("configs");
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].key.pubkey, "6s1xP3hpbAfFoNtUNF8mfHsjr2Bd97JxFJRWLbL6aHuX");
        assert_eq!(configs[0].default_params.supply_init, "1000");
    }

    fn test_launch_defaults() -> BonkLaunchDefaults {
        BonkLaunchDefaults {
            supply: BigUint::from(1_000_000_000u64),
            total_fund_raising_b: BigUint::from(1_000_000_000u64),
            quote: bonk_quote_asset_config("sol"),
            trade_fee_rate: BigUint::ZERO,
            platform_fee_rate: BigUint::ZERO,
            creator_fee_rate: BigUint::ZERO,
            curve_type: 1,
            pool: BonkCurvePoolState {
                total_sell_a: BigUint::from(500_000_000u64),
                virtual_a: BigUint::from(500_000_000u64),
                virtual_b: BigUint::from(1_000_000_000u64),
                real_a: BigUint::ZERO,
                real_b: BigUint::ZERO,
            },
        }
    }

    #[test]
    fn native_bonk_sol_quote_matches_fixed_price_defaults() {
        let quote = build_native_bonk_quote_from_defaults(&test_launch_defaults(), "sol", "0.25")
            .expect("quote");
        assert_eq!(quote.estimatedTokens, "125");
        assert_eq!(quote.estimatedSol, "0.25");
        assert_eq!(quote.estimatedQuoteAmount, "0.25");
        assert_eq!(quote.quoteAsset, "sol");
        assert_eq!(quote.quoteAssetLabel, "SOL");
        assert_eq!(quote.estimatedSupplyPercent, "12.5");
    }

    #[test]
    fn native_bonk_token_quote_matches_fixed_price_defaults() {
        let quote = build_native_bonk_quote_from_defaults(&test_launch_defaults(), "tokens", "125")
            .expect("quote");
        assert_eq!(quote.estimatedTokens, "125");
        assert_eq!(quote.estimatedSol, "0.25");
        assert_eq!(quote.estimatedQuoteAmount, "0.25");
        assert_eq!(quote.quoteAsset, "sol");
        assert_eq!(quote.estimatedSupplyPercent, "12.5");
    }

    #[test]
    fn native_bonk_fixed_price_follow_sell_matches_expected_quote() {
        let defaults = test_launch_defaults();
        let context = build_prelaunch_bonk_pool_context(
            &defaults,
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            "regular",
        )
        .expect("context");
        let min_amount_b =
            bonk_follow_sell_amounts(&context, 125_000_000, 0).expect("sell quote");
        assert_eq!(min_amount_b, 250_000_000);
    }

    #[test]
    fn advancing_prelaunch_bonk_pool_context_reduces_next_buy_quote() {
        let defaults = test_launch_defaults();
        let mint = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let context =
            build_prelaunch_bonk_pool_context(&defaults, &mint, &creator, "regular").expect("context");
        let base_quote =
            bonk_follow_buy_quote_details(&context, 800_000_000, 0).expect("base quote");
        let advanced = advance_prelaunch_bonk_pool_context_after_buy(&context, 800_000_000, 0)
            .expect("advanced context");
        let next_quote =
            bonk_follow_buy_quote_details(&advanced, 800_000_000, 0).expect("next quote");

        assert!(advanced.pool.real_a > context.pool.real_a);
        assert!(advanced.pool.real_b > context.pool.real_b);
        assert!(next_quote.amount_a < base_quote.amount_a);
    }

    #[test]
    fn bonk_follow_tx_format_uses_v0_for_non_legacy_inputs() {
        assert_eq!(
            select_bonk_native_tx_format("legacy"),
            NativeBonkTxFormat::Legacy
        );
        assert_eq!(select_bonk_native_tx_format("v0"), NativeBonkTxFormat::V0);
        assert_eq!(select_bonk_native_tx_format("auto"), NativeBonkTxFormat::V0);
        assert_eq!(
            select_bonk_native_tx_format("v0-alt"),
            NativeBonkTxFormat::V0
        );
    }

    #[test]
    fn rewrite_missing_bonk_instruction_signers_rebinds_ephemeral_signer_accounts() {
        let owner = Keypair::new();
        let missing = Pubkey::new_unique();
        let mut instructions = vec![Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![
                AccountMeta::new_readonly(owner.pubkey(), true),
                AccountMeta::new(missing, true),
                AccountMeta::new(Pubkey::new_unique(), false),
            ],
            data: vec![],
        }];
        let generated =
            rewrite_missing_bonk_instruction_signers(&owner.pubkey(), &mut instructions, &[]);
        assert_eq!(generated.len(), 1);
        assert_ne!(generated[0].pubkey(), missing);
        assert_eq!(instructions[0].accounts[1].pubkey, generated[0].pubkey());
        assert!(instructions[0].accounts[1].is_signer);
    }

    #[test]
    fn atomic_bonk_usd1_follow_buy_uses_merged_child_compute_budget() {
        assert_eq!(
            configured_atomic_bonk_usd1_follow_buy_compute_unit_limit(Some(120_000), Some(120_000)),
            240_000
        );
        assert!(
            configured_atomic_bonk_usd1_follow_buy_compute_unit_limit(None, None)
                >= configured_default_follow_up_compute_unit_limit()
        );
        assert_eq!(
            configured_atomic_bonk_usd1_follow_buy_compute_unit_limit(Some(50_000), Some(60_000)),
            configured_default_follow_up_compute_unit_limit()
        );
    }

    #[test]
    fn atomic_bonk_tx_envelope_orders_price_limit_core_then_tip() {
        let owner = Pubkey::new_unique();
        let tip_account = Pubkey::new_unique();
        let core_instruction = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![AccountMeta::new(owner, true)],
            data: vec![42],
        };
        let instructions = build_bonk_atomic_tx_instructions(
            vec![core_instruction.clone()],
            &NativeBonkTxConfig {
                compute_unit_limit: 500_000,
                compute_unit_price_micro_lamports: 1_234,
                tip_lamports: 5_000,
                tip_account: tip_account.to_string(),
            },
            &owner,
            false,
        )
        .expect("atomic instructions");
        assert_eq!(instructions.len(), 4);
        assert_eq!(instructions[0].program_id, compute_budget_program_id().expect("compute budget"));
        assert_eq!(instructions[0].data.first().copied(), Some(3));
        assert_eq!(instructions[1].program_id, compute_budget_program_id().expect("compute budget"));
        assert_eq!(instructions[1].data.first().copied(), Some(2));
        assert_eq!(instructions[2].program_id, core_instruction.program_id);
        assert_eq!(instructions[3].program_id, solana_system_interface::program::ID);
    }

    #[test]
    fn bonk_launch_bundle_tip_only_applies_to_last_transaction_when_requested() {
        let config = NativeBonkTxConfig {
            compute_unit_limit: 400_000,
            compute_unit_price_micro_lamports: 1_000,
            tip_lamports: 9_999,
            tip_account: Pubkey::new_unique().to_string(),
        };
        let first = bonk_bundle_tx_config_for_index(&config, 0, 3, true);
        let last = bonk_bundle_tx_config_for_index(&config, 2, 3, true);
        assert_eq!(first.tip_lamports, 0);
        assert!(first.tip_account.is_empty());
        assert_eq!(last.tip_lamports, 9_999);
        assert_eq!(last.tip_account, config.tip_account);
    }

    #[test]
    fn native_bonk_launch_initialize_v2_instruction_uses_expected_accounts() {
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let instruction = build_bonk_initialize_v2_instruction(
            &owner,
            &mint,
            "regular",
            "Launch Token",
            "LAUNCH",
            "https://example.invalid/meta.json",
            &test_launch_defaults(),
        )
        .expect("initialize");
        assert_eq!(instruction.program_id, bonk_launchpad_program_id().expect("launchpad"));
        assert_eq!(&instruction.data[..8], &BONK_INITIALIZE_V2_DISCRIMINATOR);
        assert_eq!(instruction.accounts.len(), 18);
        assert_eq!(instruction.accounts[0].pubkey, owner);
        assert_eq!(instruction.accounts[6].pubkey, mint);
        assert!(instruction.accounts[6].is_signer);
        assert_eq!(
            instruction.accounts[10].pubkey,
            bonk_metadata_account_pda(&mint).expect("metadata pda")
        );
    }

    #[test]
    fn native_bonk_launch_dev_buy_sol_uses_owner_wsol_ata_path() {
        let owner = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let context = build_prelaunch_bonk_pool_context(&test_launch_defaults(), &mint, &owner, "regular")
            .expect("context");
        let instructions = build_native_bonk_launch_dev_buy_instructions(
            &owner,
            &mint,
            &context,
            &BigUint::from(250_000_000u64),
            0,
            None,
            true,
        )
        .expect("instructions");
        assert_eq!(instructions.len(), 5);
        assert_eq!(instructions[0].program_id, spl_associated_token_account::id());
        assert_eq!(instructions[1].program_id, spl_associated_token_account::id());
        assert_eq!(instructions[2].program_id, solana_system_interface::program::ID);
        assert_eq!(instructions[3].program_id, spl_token::id());
        assert_eq!(instructions[4].program_id, bonk_launchpad_program_id().expect("launchpad"));
    }

    #[test]
    fn native_bonk_usd1_topup_swap_instruction_matches_pinned_clmm_layout() {
        let owner = Pubkey::new_unique();
        let input_account = Pubkey::new_unique();
        let output_account = Pubkey::new_unique();
        let instruction = build_bonk_clmm_swap_exact_in_instruction(
            &owner,
            &input_account,
            &output_account,
            123_456_789,
            45_000_000,
            &[0, -3_600],
        )
        .expect("swap instruction");
        assert_eq!(instruction.program_id, bonk_clmm_program_id().expect("clmm program"));
        assert_eq!(instruction.accounts[0].pubkey, owner);
        assert_eq!(instruction.accounts[3].pubkey, input_account);
        assert_eq!(instruction.accounts[4].pubkey, output_account);
        assert_eq!(instruction.accounts[13].pubkey, bonk_clmm_ex_bitmap_pda(
            &Pubkey::from_str(BONK_PINNED_USD1_ROUTE_POOL_ID).expect("pool"),
        )
        .expect("bitmap"));
        assert_eq!(instruction.accounts[14].pubkey, bonk_derive_clmm_tick_array_address(
            &bonk_clmm_program_id().expect("clmm"),
            &Pubkey::from_str(BONK_PINNED_USD1_ROUTE_POOL_ID).expect("pool"),
            0,
        ));
        assert_eq!(instruction.accounts[15].pubkey, bonk_derive_clmm_tick_array_address(
            &bonk_clmm_program_id().expect("clmm"),
            &Pubkey::from_str(BONK_PINNED_USD1_ROUTE_POOL_ID).expect("pool"),
            -3_600,
        ));
        assert_eq!(&instruction.data[..8], &BONK_CLMM_SWAP_DISCRIMINATOR);
        assert_eq!(u64::from_le_bytes(instruction.data[8..16].try_into().expect("amount in")), 123_456_789);
        assert_eq!(u64::from_le_bytes(instruction.data[16..24].try_into().expect("min out")), 45_000_000);
        assert_eq!(
            u128::from_le_bytes(instruction.data[24..40].try_into().expect("sqrt limit")),
            BONK_CLMM_MIN_SQRT_PRICE_X64_PLUS_ONE
        );
        assert_eq!(instruction.data[40], 1);
    }

    #[test]
    fn clmm_zero_tick_sqrt_price_matches_q64() {
        assert_eq!(bonk_sqrt_price_from_tick(0).expect("sqrt"), bonk_clmm_q64());
    }

    #[test]
    fn clmm_exact_input_quote_applies_slippage_to_min_out() {
        let tick_spacing = 60;
        let start_tick_index = 0;
        let mut ticks = Vec::new();
        for index in 0..BONK_CLMM_TICK_ARRAY_SIZE {
            let tick = start_tick_index + index * tick_spacing;
            ticks.push(BonkClmmTick {
                tick,
                liquidity_net: 0,
                liquidity_gross: if index == 0 {
                    BigUint::from(1u8)
                } else {
                    BigUint::ZERO
                },
            });
        }
        let sqrt_price_x64 = bonk_sqrt_price_from_tick(120).expect("current sqrt price");
        let setup = BonkUsd1RouteSetup {
            pool_id: Pubkey::new_unique(),
            program_id: Pubkey::new_unique(),
            tick_spacing,
            trade_fee_rate: 2_500,
            sqrt_price_x64: sqrt_price_x64.clone(),
            liquidity: BigUint::from(1_000_000_000_000u64),
            tick_current: 120,
            mint_a_decimals: 9,
            mint_b_decimals: 6,
            current_price: bonk_sqrt_price_x64_to_price(&sqrt_price_x64, 9, 6)
                .expect("current price"),
            tick_arrays_desc: vec![start_tick_index],
            tick_arrays: HashMap::from([(
                start_tick_index,
                BonkClmmTickArray {
                    start_tick_index,
                    ticks,
                },
            )]),
        };
        let quote = bonk_quote_usd1_from_exact_sol_input(&setup, &BigUint::from(1_000_000u64), 50)
            .expect("quote");
        assert!(quote.expected_out > BigUint::ZERO);
        assert!(quote.expected_out > quote.min_out);
        assert_eq!(
            quote.min_out,
            (&quote.expected_out * BigUint::from(9_950u64)) / BigUint::from(10_000u64)
        );
    }
}
