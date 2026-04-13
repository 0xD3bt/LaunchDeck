#![allow(non_snake_case, dead_code)]

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::{AddressLookupTableAccount, VersionedMessage, v0},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use solana_system_interface::instruction::transfer;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::Stdio,
    str::FromStr,
    sync::{Arc, Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::Semaphore,
    time::{Duration, Instant, sleep, timeout},
};

use crate::{
    config::{
        NormalizedConfig, NormalizedExecution, NormalizedRecipient,
        configured_bags_rust_blockhash_for_helper, configured_bags_setup_gate_commitment,
        configured_default_dev_auto_sell_compute_unit_limit,
        configured_default_launch_compute_unit_limit,
        configured_default_sniper_buy_compute_unit_limit, validate_launchpad_support,
    },
    follow::BagsLaunchMetadata,
    helper_worker::{
        HelperWorkerClient, HelperWorkerConfig, HelperWorkerError, helper_worker_enabled,
    },
    launchpad_dispatch::{launchpad_action_backend, launchpad_action_rollout_state},
    paths,
    provider_tip::provider_required_tip_lamports,
    pump_native::{LaunchQuote, NativeCompileTimings},
    report::{InstructionSummary, LaunchReport, TransactionSummary, build_report, render_report},
    rpc::{
        CompiledTransaction, fetch_latest_blockhash_cached,
        COMPILE_BLOCKHASH_MIN_REMAINING_BLOCKS, fetch_latest_blockhash_cached_with_prime,
    },
    transport::TransportPlan,
};

const PACKET_LIMIT_BYTES: usize = 1232;
const PRIORITY_FEE_PRICE_BASE_COMPUTE_UNIT_LIMIT: u64 = 1_000_000;
const DEFAULT_HELPER_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_HELPER_MAX_CONCURRENCY: usize = 4;
const DEFAULT_BAGS_SETUP_JITO_TIP_CAP_LAMPORTS: u64 = 1_000_000;
const DEFAULT_BAGS_SETUP_JITO_TIP_MIN_LAMPORTS: u64 = 1_000;
const DEFAULT_AUTO_FEE_JITO_TIP_PERCENTILE: &str = "p99";
const BAGS_ENGINE_FEE_ESTIMATE_MAX_AGE: Duration = Duration::from_secs(10);
const BAGS_DBC_PROGRAM_ID: &str = "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN";
const BAGS_DAMM_V2_PROGRAM_ID: &str = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG";
const BAGS_FEE_SHARE_V2_PROGRAM_ID: &str = "FEE2tBhCKAt7shrod19QttSVREUYPiyMzoku1mL1gqVK";
const BAGS_DBC_POOL_AUTHORITY: &str = "FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM";
const BAGS_DAMM_POOL_AUTHORITY: &str = "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC";
const COMPUTE_BUDGET_PROGRAM_ID: &str = "ComputeBudget111111111111111111111111111111";
const DEFAULT_BAGS_WALLET: &str = "3muhBpbVeoDy4fBrC1SWnfkUooy2Pn6woV1GxDUhESfC";
const DEFAULT_BAGS_CONFIG: &str = "AxpMibQQBqVbQF7EzBUeCbpxRkuk6yfTWRLGVLh5qrce";
const BAGS_CONFIG_TYPE_DEFAULT: &str = "fa29606e-5e48-4c37-827f-4b03d58ee23d";
const BAGS_CONFIG_TYPE_025_PRE_1_POST: &str = "d16d3585-6488-4a6c-9a6f-e6c39ca0fda3";
const BAGS_CONFIG_TYPE_1_PRE_025_POST: &str = "a7c8e1f2-3d4b-5a6c-9e0f-1b2c3d4e5f6a";
const BAGS_FEE_SHARE_V2_MAX_CLAIMERS_NON_LUT: usize = 15;
const BAGS_NATIVE_MINT: &str = "So11111111111111111111111111111111111111112";
const BAGS_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const BAGS_TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const JITODONTFRONT_ACCOUNT: &str = "jitodontfront111111111111111111111111111111";
const BAGS_TOTAL_SUPPLY_BASE_UNITS: &str = "1000000000000000000";
const BAGS_INITIAL_SQRT_PRICE_STR: &str = "3141367320245630";
const BAGS_MIGRATION_QUOTE_THRESHOLD_STR: &str = "85000000000";
const BAGS_CURVE_POINTS: [(&str, &str); 2] = [
    ("6401204812200420", "3929368168768468756200000000000000"),
    ("13043817825332782", "2425988008058820449100000000000000"),
];
const DBC_POOL_BY_BASE_MINT_OFFSET: usize = 136;
const DBC_POOL_CONFIG_DISCRIMINATOR: [u8; 8] = [26, 108, 14, 123, 116, 230, 129, 43];
const DBC_VIRTUAL_POOL_DISCRIMINATOR: [u8; 8] = [213, 224, 5, 209, 98, 69, 119, 92];
const CPAMM_POOL_DISCRIMINATOR: [u8; 8] = [241, 154, 109, 4, 17, 177, 109, 188];
const DBC_FEE_DENOMINATOR: u64 = 1_000_000_000;
const DBC_MAX_FEE_NUMERATOR: u64 = 990_000_000;
const DBC_BASIS_POINT_MAX: u64 = 10_000;
const DBC_RESOLUTION_BITS: usize = 64;
const DBC_SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
const CPAMM_MAX_FEE_NUMERATOR: u64 = 500_000_000;
const CPAMM_BASIS_POINT_MAX: u64 = 10_000;
const CPAMM_SCALE_OFFSET: usize = 64;
const DAMM_V2_MIGRATION_FEE_ADDRESS: [&str; 7] = [
    "7F6dnUcRuyM2TwR8myT1dYypFXpPSxqwKNSFNkxyNESd",
    "2nHK1kju6XjphBLbNxpM5XRGFj7p9U8vvNzyZiha1z6k",
    "Hv8Lmzmnju6m7kcokVKvwqz7QPmdX9XfKjJsXz8RXcjp",
    "2c4cYd4reUYVRAB9kUUkrq55VPyy2FNQ3FDL4o12JXmq",
    "AkmQWebAwFvWk55wBoCr5D62C6VVDTzi84NJuD9H7cFD",
    "DbCRBj8McvPYHJG1ukj8RE15h2dCNUdTAESG49XpQ44u",
    "A8gMrEPJkacWkcb3DGwtJwTe16HktSEfvwtuDh2MCtck",
];

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

fn bags_helper_fallback_status() -> &'static Mutex<HashMap<String, NativeHelperFallbackStatus>> {
    static STATUS: OnceLock<Mutex<HashMap<String, NativeHelperFallbackStatus>>> = OnceLock::new();
    STATUS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn record_bags_helper_fallback(action: &str, error: &str) {
    if let Ok(mut status) = bags_helper_fallback_status().lock() {
        let entry = status.entry(action.trim().to_string()).or_default();
        entry.count = entry.count.saturating_add(1);
        entry.lastError = error.trim().to_string();
        entry.lastFallbackAtMs = Some(unix_now_seconds().saturating_mul(1000));
    }
}

pub fn bags_runtime_status_payload() -> Value {
    let fallback_status = bags_helper_fallback_status()
        .lock()
        .map(|status| status.clone())
        .unwrap_or_default();
    json!({
        "fallbacks": fallback_status,
    })
}

#[derive(Debug, Clone)]
pub struct NativeBagsArtifacts {
    pub compiled_transactions: Vec<CompiledTransaction>,
    pub report: Value,
    pub text: String,
    pub compile_timings: NativeCompileTimings,
    pub mint: String,
    pub launch_creator: String,
    pub config_key: String,
    pub metadata_uri: String,
    pub migration_fee_option: Option<i64>,
    pub expected_migration_family: String,
    pub expected_damm_config_key: String,
    pub expected_damm_derivation_mode: String,
    pub pre_migration_dbc_pool_address: String,
    /// Populated for live send: Jito setup bundles before sequential setup transactions.
    pub setup_bundles: Vec<Vec<CompiledTransaction>>,
    /// Populated for live send: non-bundled setup transactions (e.g. direct config txs).
    pub setup_transactions: Vec<CompiledTransaction>,
    pub fee_estimate: BagsFeeEstimateSnapshot,
    pub prepare_launch_ms: Option<u128>,
    pub metadata_upload_ms: Option<u128>,
    pub fee_recipient_resolve_ms: Option<u128>,
}

#[derive(Debug, Clone)]
pub struct BagsLaunchTransactionArtifacts {
    pub compiled_transaction: CompiledTransaction,
    pub launch_build_ms: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagsFeeEstimateSnapshot {
    #[serde(default)]
    pub helius: Value,
    #[serde(default)]
    pub jito: Value,
    #[serde(default)]
    pub setupJitoTipLamports: u64,
    #[serde(default)]
    pub setupJitoTipSource: String,
    #[serde(default)]
    pub setupJitoTipPercentile: String,
    #[serde(default)]
    pub setupJitoTipCapLamports: u64,
    #[serde(default)]
    pub setupJitoTipMinLamports: u64,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagsMarketSnapshot {
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
pub struct BagsImportRecipient {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub githubUsername: String,
    #[serde(default)]
    pub shareBps: i64,
    #[serde(default)]
    pub sourceProvider: String,
    #[serde(default)]
    pub sourceUsername: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagsImportContext {
    pub launchpad: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub quoteAsset: String,
    #[serde(default)]
    pub creator: String,
    #[serde(default)]
    pub marketKey: String,
    #[serde(default)]
    pub configKey: String,
    #[serde(default)]
    pub venue: String,
    #[serde(default)]
    pub detectionSource: String,
    #[serde(default)]
    pub feeRecipients: Vec<BagsImportRecipient>,
    #[serde(default)]
    pub notes: Vec<String>,
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
struct HelperLaunchResponse {
    mint: String,
    launchCreator: String,
    #[serde(default)]
    configKey: String,
    #[serde(default)]
    metadataUri: String,
    #[serde(default)]
    identityLabel: String,
    #[serde(default)]
    migrationFeeOption: Option<i64>,
    #[serde(default)]
    expectedMigrationFamily: String,
    #[serde(default)]
    expectedDammConfigKey: String,
    #[serde(default)]
    expectedDammDerivationMode: String,
    #[serde(default)]
    preMigrationDbcPoolAddress: String,
    #[serde(default)]
    setupBundles: Vec<HelperBundleResponse>,
    #[serde(default)]
    setupTransactions: Vec<HelperCompiledTransaction>,
    compiledTransactions: Vec<HelperCompiledTransaction>,
    #[serde(default)]
    timings: Option<HelperPrepareLaunchTimings>,
}

#[derive(Debug, Clone, Deserialize)]
struct HelperBundleResponse {
    #[serde(default)]
    label: String,
    #[serde(default)]
    compiledTransactions: Vec<HelperCompiledTransaction>,
}

#[derive(Debug, Deserialize)]
struct HelperLaunchTransactionResponse {
    compiledTransaction: HelperCompiledTransaction,
    #[serde(default)]
    timings: Option<HelperLaunchBuildTimings>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
struct HelperPrepareLaunchTimings {
    #[serde(default)]
    prepareLaunchMs: Option<u128>,
    #[serde(default)]
    feeRecipientResolveMs: Option<u128>,
    #[serde(default)]
    metadataUploadMs: Option<u128>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
struct HelperLaunchBuildTimings {
    #[serde(default)]
    launchBuildMs: Option<u128>,
}

#[derive(Debug, Clone)]
struct NativePreparedBagsLaunch {
    mint: String,
    launch_creator: String,
    config_key: String,
    metadata_uri: String,
    identity_label: String,
    migration_fee_option: Option<i64>,
    expected_migration_family: String,
    expected_damm_config_key: String,
    expected_damm_derivation_mode: String,
    pre_migration_dbc_pool_address: String,
    compiled_transactions: Vec<CompiledTransaction>,
    setup_bundles: Vec<Vec<CompiledTransaction>>,
    setup_transactions: Vec<CompiledTransaction>,
    timings: HelperPrepareLaunchTimings,
}

#[derive(Debug, Clone, Deserialize)]
struct BagsTokenInfoResponse {
    #[serde(default)]
    tokenMint: String,
    #[serde(default)]
    tokenMetadata: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BagsApiSerializedTransaction {
    #[serde(default)]
    transaction: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BagsFeeShareConfigResponse {
    #[serde(default)]
    needsCreation: bool,
    #[serde(default)]
    transactions: Vec<BagsApiSerializedTransaction>,
    #[serde(default)]
    bundles: Vec<Vec<BagsApiSerializedTransaction>>,
    #[serde(default)]
    meteoraConfigKey: String,
}

#[derive(Debug, Deserialize)]
struct HelperFollowBuyResponse {
    compiledTransaction: HelperCompiledTransaction,
}

#[derive(Debug, Deserialize)]
struct HelperFollowSellResponse {
    compiledTransaction: Option<HelperCompiledTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagsFeeRecipientLookupResponse {
    #[serde(default)]
    pub found: bool,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub lookupTarget: String,
    #[serde(default)]
    pub wallet: String,
    #[serde(default)]
    pub resolvedUsername: String,
    #[serde(default)]
    pub githubUserId: String,
    #[serde(default)]
    pub notFound: bool,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BagsStoredCredentials {
    #[serde(default)]
    #[serde(rename = "apiKey")]
    api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BagsApiEnvelope<T> {
    #[serde(default)]
    success: bool,
    response: Option<T>,
    #[serde(default)]
    error: String,
}

fn bags_api_response_message(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Value::Object(map) => {
            for key in ["error", "message", "detail", "reason"] {
                if let Some(text) = map.get(key).and_then(Value::as_str) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
            let compact = value.to_string();
            let trimmed = compact.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        _ => {
            let compact = value.to_string();
            let trimmed = compact.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    }
}

fn summarize_bags_api_failure(
    action: &str,
    status: reqwest::StatusCode,
    error: &str,
    response: Option<&Value>,
    raw_body: &str,
) -> String {
    let normalized_error = error.trim();
    if !normalized_error.is_empty() {
        return normalized_error.to_string();
    }
    if let Some(message) = response.and_then(bags_api_response_message) {
        return message;
    }
    let compact_body = raw_body.trim();
    if compact_body.is_empty() {
        return format!("{action}: status {status}");
    }
    let preview = if compact_body.len() > 280 {
        format!("{}...", &compact_body[..277])
    } else {
        compact_body.to_string()
    };
    format!("{action}: status {status} | body {preview}")
}

#[derive(Debug, Clone, Deserialize)]
struct BagsLookupWalletResponse {
    #[serde(default)]
    provider: String,
    #[serde(default)]
    wallet: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct BagsTokenLaunchCreator {
    #[serde(default)]
    royaltyBps: i64,
    #[serde(default)]
    isCreator: bool,
    #[serde(default)]
    wallet: String,
    provider: Option<String>,
    providerUsername: Option<String>,
    twitterUsername: Option<String>,
    githubUsername: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcResponse<T> {
    result: T,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcAccountValue {
    data: (String, String),
}

#[derive(Debug, Clone, Deserialize)]
struct RpcAccountInfoResult {
    value: Option<RpcAccountValue>,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcProgramAccount {
    pubkey: String,
    account: RpcAccountValue,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcTokenAmountValue {
    amount: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcTokenAccountBalanceResult {
    value: RpcTokenAmountValue,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcTokenSupplyValue {
    amount: String,
    decimals: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcTokenSupplyResult {
    value: RpcTokenSupplyValue,
}

#[derive(Debug, Clone)]
struct BagsCurvePoint {
    sqrt_price: BigUint,
    liquidity: BigUint,
}

#[derive(Debug, Clone)]
struct DecodedDbcVirtualPool {
    config: Pubkey,
    creator: Pubkey,
    base_mint: Pubkey,
    base_vault: Pubkey,
    quote_vault: Pubkey,
    sqrt_price: u128,
    base_reserve: u64,
    quote_reserve: u64,
    volatility_accumulator: u128,
    activation_point: u64,
    pool_type: u8,
    is_migrated: bool,
}

#[derive(Debug, Clone)]
struct DecodedDbcBaseFeeConfig {
    cliff_fee_numerator: u64,
    first_factor: u16,
    second_factor: u64,
    third_factor: u64,
    base_fee_mode: u8,
}

#[derive(Debug, Clone)]
struct DecodedDbcDynamicFeeConfig {
    initialized: bool,
    variable_fee_control: u32,
    bin_step: u16,
    volatility_accumulator: u128,
}

#[derive(Debug, Clone)]
struct DecodedDbcPoolConfig {
    quote_mint: Pubkey,
    collect_fee_mode: u8,
    activation_type: u8,
    quote_token_flag: u8,
    migration_fee_option: u8,
    creator_trading_fee_percentage: u8,
    creator_migration_fee_percentage: u8,
    migration_quote_threshold: u64,
    sqrt_start_price: u128,
    curve: Vec<BagsCurvePoint>,
    base_fee: DecodedDbcBaseFeeConfig,
    dynamic_fee: DecodedDbcDynamicFeeConfig,
}

#[derive(Debug, Clone)]
struct DecodedDammBaseFee {
    cliff_fee_numerator: u64,
    fee_scheduler_mode: u8,
    number_of_period: u16,
    period_frequency: u64,
    reduction_factor: u64,
}

#[derive(Debug, Clone)]
struct DecodedDammDynamicFee {
    initialized: bool,
    variable_fee_control: u32,
    bin_step: u16,
    volatility_accumulator: u128,
}

#[derive(Debug, Clone)]
struct DecodedDammPoolFees {
    base_fee: DecodedDammBaseFee,
    collect_fee_mode: u8,
    dynamic_fee: DecodedDammDynamicFee,
}

#[derive(Debug, Clone)]
struct DecodedDammPool {
    token_a_mint: Pubkey,
    token_b_mint: Pubkey,
    token_a_vault: Pubkey,
    token_b_vault: Pubkey,
    liquidity: u128,
    sqrt_price: u128,
    activation_point: u64,
    activation_type: u8,
    token_a_flag: u8,
    token_b_flag: u8,
    collect_fee_mode: u8,
    creator: Pubkey,
    pool_fees: DecodedDammPoolFees,
}

#[derive(Debug, Clone)]
pub struct BagsDbcFollowBuyContext {
    pool_address: Pubkey,
    pool: DecodedDbcVirtualPool,
    config: DecodedDbcPoolConfig,
    current_point: u64,
}

#[derive(Debug, Clone)]
pub struct BagsDammFollowBuyContext {
    pool_address: Pubkey,
    pool: DecodedDammPool,
    current_point: u64,
}

#[derive(Debug, Clone)]
pub enum BagsFollowBuyContext {
    Dbc(BagsDbcFollowBuyContext),
    Damm(BagsDammFollowBuyContext),
}

#[derive(Debug, Clone)]
struct NativeFollowTxConfig {
    compute_unit_limit: u32,
    compute_unit_price_micro_lamports: u64,
    tip_lamports: u64,
    tip_account: String,
    jitodontfront: bool,
}

#[derive(Debug, Clone, Default)]
struct CachedBagsLaunchHints {
    config_key: Option<Pubkey>,
    migration_fee_option: Option<i64>,
    expected_migration_family: String,
    expected_damm_config_key: Option<Pubkey>,
    expected_damm_derivation_mode: String,
    pre_migration_dbc_pool_address: Option<Pubkey>,
}

#[derive(Debug, Clone)]
struct NativeBagsImportMarket {
    mode: String,
    market_key: String,
    config_key: String,
    venue: String,
    detection_source: String,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
struct NativeHelperFallbackStatus {
    count: u64,
    #[serde(skip_serializing_if = "String::is_empty")]
    lastError: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lastFallbackAtMs: Option<u64>,
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
    Err("Bags helper runtime has been retired; native Rust execution is required.".to_string())
}

fn bags_worker_enabled() -> bool {
    helper_worker_enabled("LAUNCHDECK_ENABLE_BAGS_HELPER_WORKER")
}

fn worker_client() -> Result<Arc<HelperWorkerClient>, String> {
    static CLIENT: OnceLock<Result<Arc<HelperWorkerClient>, String>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            let project_root = project_root()?;
            let script_path = helper_script_path()?;
            Ok(Arc::new(HelperWorkerClient::new(HelperWorkerConfig {
                helper_name: "Bags",
                project_root,
                script_path,
                timeout_ms: helper_timeout_ms(),
            })))
        })
        .clone()
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
        .map_err(|_| "Bags helper semaphore closed unexpectedly.".to_string())?;
    let script_path = helper_script_path()?;
    let request_bytes = serde_json::to_vec(request).map_err(|error| error.to_string())?;
    let mut child = Command::new("node")
        .arg(script_path)
        .current_dir(project_root()?)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start Bags helper: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&request_bytes)
            .await
            .map_err(|error| format!("Failed to send Bags helper request: {error}"))?;
    }
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Bags helper stdout was unavailable.".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Bags helper stderr was unavailable.".to_string())?;
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
        Ok(result) => result.map_err(|error| format!("Bags helper failed to complete: {error}"))?,
        Err(_) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err(format!(
                "Bags helper timed out after {}ms.",
                helper_timeout_ms()
            ));
        }
    };
    let output_stdout = stdout_task
        .await
        .map_err(|error| format!("Bags helper stdout task failed: {error}"))?
        .map_err(|error| format!("Failed to read Bags helper stdout: {error}"))?;
    let output_stderr = stderr_task
        .await
        .map_err(|error| format!("Bags helper stderr task failed: {error}"))?
        .map_err(|error| format!("Failed to read Bags helper stderr: {error}"))?;
    if !status.success() {
        let stderr = String::from_utf8_lossy(&output_stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Bags helper exited with a non-zero status.".to_string()
        } else {
            format!("Bags helper error: {stderr}")
        });
    }
    parse_helper_output(&output_stdout, "Bags")
}

async fn run_helper<T: Serialize, R: DeserializeOwned>(request: &T) -> Result<R, String> {
    if bags_worker_enabled() {
        match worker_client()?.request::<T, R>(request).await {
            Ok(response) => return Ok(response),
            Err(HelperWorkerError::Request(error)) => return Err(error),
            Err(HelperWorkerError::Transport(error)) => {
                eprintln!("Bags worker transport failed, falling back to one-shot helper: {error}");
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

fn effective_bags_setup_tip_lamports(
    config: &NormalizedConfig,
    fee_estimate: &BagsFeeEstimateSnapshot,
) -> u64 {
    let configured_tip_lamports =
        u64::try_from(config.tx.jitoTipLamports.max(0)).unwrap_or_default();
    let provider_required = provider_required_tip_lamports(&config.execution.provider).unwrap_or(0);
    fee_estimate
        .setupJitoTipLamports
        .max(configured_tip_lamports)
        .max(provider_required)
}

fn default_bags_fee_estimate_snapshot(
    requested_tip_lamports: u64,
    setup_jito_tip_cap_lamports: u64,
    setup_jito_tip_min_lamports: u64,
    percentile: &str,
    warning: String,
) -> BagsFeeEstimateSnapshot {
    let mut setup_jito_tip_lamports = requested_tip_lamports;
    let mut setup_jito_tip_source = "user-requested-fallback".to_string();
    if setup_jito_tip_lamports > 0 {
        setup_jito_tip_lamports = setup_jito_tip_lamports.max(setup_jito_tip_min_lamports);
    }
    if setup_jito_tip_cap_lamports > 0 {
        setup_jito_tip_lamports = setup_jito_tip_lamports.min(setup_jito_tip_cap_lamports);
    }
    if setup_jito_tip_lamports == 0 {
        setup_jito_tip_source = "none".to_string();
    }
    BagsFeeEstimateSnapshot {
        helius: json!({
            "source": "native-default",
            "launchPriorityLamports": Value::Null,
        }),
        jito: json!({
            "source": "native-default",
            "percentile": percentile,
            "tipLamports": requested_tip_lamports,
            "raw": Value::Null,
        }),
        setupJitoTipLamports: setup_jito_tip_lamports,
        setupJitoTipSource: setup_jito_tip_source,
        setupJitoTipPercentile: percentile.to_string(),
        setupJitoTipCapLamports: setup_jito_tip_cap_lamports,
        setupJitoTipMinLamports: setup_jito_tip_min_lamports,
        warnings: vec![warning],
    }
}

fn uses_single_bundle_tip_last_tx(provider: &str, mev_mode: &str) -> bool {
    provider.trim().eq_ignore_ascii_case("hellomoon")
        && mev_mode.trim().eq_ignore_ascii_case("secure")
}

fn bags_setup_jito_tip_cap_lamports() -> u64 {
    std::env::var("LAUNCHDECK_BAGS_SETUP_JITO_TIP_CAP_LAMPORTS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_BAGS_SETUP_JITO_TIP_CAP_LAMPORTS)
}

fn bags_setup_jito_tip_min_lamports() -> u64 {
    std::env::var("LAUNCHDECK_BAGS_SETUP_JITO_TIP_MIN_LAMPORTS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_BAGS_SETUP_JITO_TIP_MIN_LAMPORTS)
}

fn bags_setup_jito_tip_percentile() -> String {
    let value = std::env::var("LAUNCHDECK_AUTO_FEE_JITO_TIP_PERCENTILE")
        .unwrap_or_else(|_| DEFAULT_AUTO_FEE_JITO_TIP_PERCENTILE.to_string());
    let trimmed = value.trim().to_lowercase();
    match trimmed.as_str() {
        "p25" | "p50" | "p75" | "p95" | "p99" => trimmed,
        _ => DEFAULT_AUTO_FEE_JITO_TIP_PERCENTILE.to_string(),
    }
}

#[derive(Debug, Clone)]
struct CachedBagsFeeEstimate {
    snapshot: BagsFeeEstimateSnapshot,
    fetched_at: Instant,
}

fn bags_fee_estimate_cache() -> &'static Mutex<HashMap<String, CachedBagsFeeEstimate>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedBagsFeeEstimate>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn bags_fee_estimate_cache_key(
    rpc_url: &str,
    requested_tip_lamports: u64,
    setup_jito_tip_cap_lamports: u64,
    setup_jito_tip_min_lamports: u64,
    percentile: &str,
) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        rpc_url.trim(),
        requested_tip_lamports,
        setup_jito_tip_cap_lamports,
        setup_jito_tip_min_lamports,
        percentile.trim().to_lowercase()
    )
}

fn get_cached_bags_fee_estimate(
    rpc_url: &str,
    requested_tip_lamports: u64,
    setup_jito_tip_cap_lamports: u64,
    setup_jito_tip_min_lamports: u64,
    percentile: &str,
) -> Option<BagsFeeEstimateSnapshot> {
    let cache = bags_fee_estimate_cache().lock().ok()?;
    let entry = cache.get(&bags_fee_estimate_cache_key(
        rpc_url,
        requested_tip_lamports,
        setup_jito_tip_cap_lamports,
        setup_jito_tip_min_lamports,
        percentile,
    ))?;
    if entry.fetched_at.elapsed() > BAGS_ENGINE_FEE_ESTIMATE_MAX_AGE {
        return None;
    }
    Some(entry.snapshot.clone())
}

fn cache_bags_fee_estimate(
    rpc_url: &str,
    requested_tip_lamports: u64,
    setup_jito_tip_cap_lamports: u64,
    setup_jito_tip_min_lamports: u64,
    percentile: &str,
    snapshot: &BagsFeeEstimateSnapshot,
) {
    if let Ok(mut cache) = bags_fee_estimate_cache().lock() {
        cache.insert(
            bags_fee_estimate_cache_key(
                rpc_url,
                requested_tip_lamports,
                setup_jito_tip_cap_lamports,
                setup_jito_tip_min_lamports,
                percentile,
            ),
            CachedBagsFeeEstimate {
                snapshot: snapshot.clone(),
                fetched_at: Instant::now(),
            },
        );
    }
}

fn bags_fee_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("bags fee client")
    })
}

fn bags_api_base_url() -> String {
    std::env::var("BAGS_API_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://public-api-v2.bags.fm/api/v1".to_string())
}

fn bags_total_supply_base_units() -> BigUint {
    BigUint::parse_bytes(BAGS_TOTAL_SUPPLY_BASE_UNITS.as_bytes(), 10).expect("bags total supply")
}

fn bags_initial_sqrt_price() -> BigUint {
    BigUint::parse_bytes(BAGS_INITIAL_SQRT_PRICE_STR.as_bytes(), 10).expect("bags sqrt price")
}

fn bags_migration_quote_threshold() -> BigUint {
    BigUint::parse_bytes(BAGS_MIGRATION_QUOTE_THRESHOLD_STR.as_bytes(), 10)
        .expect("bags migration threshold")
}

fn bags_curve_points() -> Vec<BagsCurvePoint> {
    BAGS_CURVE_POINTS
        .iter()
        .map(|(sqrt_price, liquidity)| BagsCurvePoint {
            sqrt_price: BigUint::parse_bytes(sqrt_price.as_bytes(), 10)
                .expect("bags curve sqrt price"),
            liquidity: BigUint::parse_bytes(liquidity.as_bytes(), 10)
                .expect("bags curve liquidity"),
        })
        .collect()
}

fn biguint_from_u64(value: u64) -> BigUint {
    BigUint::from(value)
}

fn biguint_from_u128(value: u128) -> BigUint {
    BigUint::from(value)
}

fn parse_decimal_to_u128(raw: &str, decimals: u32, label: &str) -> Result<u128, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!("{label} is required."));
    }
    if value.starts_with('-') {
        return Err(format!("Invalid {label}: {value}"));
    }
    let mut parts = value.split('.');
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || whole.is_empty()
        || !whole.chars().all(|ch| ch.is_ascii_digit())
        || !fraction.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(format!("Invalid {label}: {value}"));
    }
    let whole_value = whole
        .parse::<u128>()
        .map_err(|_| format!("Invalid {label}: {value}"))?;
    let base = 10u128.pow(decimals);
    let mut padded_fraction = fraction.to_string();
    padded_fraction.extend(std::iter::repeat_n('0', decimals as usize));
    let padded_fraction = &padded_fraction[..decimals as usize];
    let fraction_value = if padded_fraction.is_empty() {
        0
    } else {
        padded_fraction
            .parse::<u128>()
            .map_err(|_| format!("Invalid {label}: {value}"))?
    };
    whole_value
        .checked_mul(base)
        .and_then(|scaled| scaled.checked_add(fraction_value))
        .ok_or_else(|| format!("{label} is too large."))
}

fn format_decimal_u128(value: u128, decimals: u32, precision: u32) -> String {
    let divisor = 10u128.pow(decimals);
    let whole = value / divisor;
    let fraction = value % divisor;
    if fraction == 0 {
        return whole.to_string();
    }
    let width = decimals as usize;
    let mut fraction_text = format!("{fraction:0width$}");
    fraction_text.truncate(precision.min(decimals) as usize);
    while fraction_text.ends_with('0') {
        fraction_text.pop();
    }
    if fraction_text.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{fraction_text}")
    }
}

fn format_biguint_decimal(value: &BigUint, decimals: u32, precision: u32) -> Result<String, String> {
    let raw = value
        .to_u128()
        .ok_or_else(|| "Bags quote amount overflowed u128 formatting range.".to_string())?;
    Ok(format_decimal_u128(raw, decimals, precision))
}

fn format_bags_supply_percent(value_base_units: &BigUint) -> String {
    if value_base_units.is_zero() {
        return "0".to_string();
    }
    let scaled = (value_base_units * BigUint::from(1_000_000u64)) / bags_total_supply_base_units();
    let whole = &scaled / BigUint::from(10_000u64);
    let fraction = &scaled % BigUint::from(10_000u64);
    if fraction.is_zero() {
        whole.to_string()
    } else {
        let mut fraction_text = fraction.to_string();
        while fraction_text.len() < 4 {
            fraction_text.insert(0, '0');
        }
        while fraction_text.ends_with('0') {
            fraction_text.pop();
        }
        format!("{whole}.{fraction_text}")
    }
}

fn bags_pre_migration_fee_bps_for_mode(mode: &str) -> u64 {
    match mode.trim().to_ascii_lowercase().as_str() {
        "bags-025-1" => 25,
        "bags-1-025" => 100,
        "bags-2-2" | "" => 200,
        _ => 200,
    }
}

fn bags_cliff_fee_numerator_for_mode(mode: &str) -> u64 {
    bags_pre_migration_fee_bps_for_mode(mode) * DBC_FEE_DENOMINATOR / 10_000
}

fn bags_mode_from_fee_values(pre_fee: u8, post_fee: u8) -> String {
    match (pre_fee, post_fee) {
        (2, 2) | (200, 200) => "bags-2-2".to_string(),
        (25, 100) => "bags-025-1".to_string(),
        (100, 25) => "bags-1-025".to_string(),
        _ => String::new(),
    }
}

fn big_div_rounding(numerator: BigUint, denominator: &BigUint, round_up: bool) -> BigUint {
    if round_up && !numerator.is_zero() {
        (numerator + denominator - BigUint::from(1u8)) / denominator
    } else {
        numerator / denominator
    }
}

fn big_sub(left: &BigUint, right: &BigUint, label: &str) -> Result<BigUint, String> {
    if left < right {
        return Err(format!("Bags math underflow while computing {label}."));
    }
    Ok(left - right)
}

fn bags_get_delta_amount_base_unsigned(
    lower_sqrt_price: &BigUint,
    upper_sqrt_price: &BigUint,
    liquidity: &BigUint,
    round_up: bool,
) -> Result<BigUint, String> {
    if liquidity.is_zero() {
        return Ok(BigUint::ZERO);
    }
    if lower_sqrt_price.is_zero() || upper_sqrt_price.is_zero() {
        return Err("Bags quote sqrt price cannot be zero.".to_string());
    }
    let numerator = big_sub(upper_sqrt_price, lower_sqrt_price, "base numerator")?;
    let denominator = lower_sqrt_price * upper_sqrt_price;
    Ok(big_div_rounding(liquidity * numerator, &denominator, round_up))
}

fn bags_get_delta_amount_quote_unsigned(
    lower_sqrt_price: &BigUint,
    upper_sqrt_price: &BigUint,
    liquidity: &BigUint,
    round_up: bool,
) -> Result<BigUint, String> {
    if liquidity.is_zero() {
        return Ok(BigUint::ZERO);
    }
    let delta = big_sub(upper_sqrt_price, lower_sqrt_price, "quote numerator")?;
    let product = liquidity * delta;
    let denominator = BigUint::from(1u8) << (DBC_RESOLUTION_BITS * 2);
    Ok(big_div_rounding(product, &denominator, round_up))
}

fn bags_get_next_sqrt_price_from_input(
    sqrt_price: &BigUint,
    liquidity: &BigUint,
    amount_in: &BigUint,
) -> Result<BigUint, String> {
    if sqrt_price.is_zero() || liquidity.is_zero() {
        return Err("Bags quote price or liquidity cannot be zero.".to_string());
    }
    if amount_in.is_zero() {
        return Ok(sqrt_price.clone());
    }
    let quotient = (amount_in << (DBC_RESOLUTION_BITS * 2)) / liquidity;
    Ok(sqrt_price + quotient)
}

fn bags_get_next_sqrt_price_from_base_output(
    sqrt_price: &BigUint,
    liquidity: &BigUint,
    amount_out: &BigUint,
) -> Result<BigUint, String> {
    if sqrt_price.is_zero() {
        return Err("Bags quote sqrt price cannot be zero.".to_string());
    }
    if amount_out.is_zero() {
        return Ok(sqrt_price.clone());
    }
    let product = amount_out * sqrt_price;
    let denominator = big_sub(liquidity, &product, "next sqrt denominator")?;
    Ok(big_div_rounding(
        liquidity * sqrt_price,
        &denominator,
        false,
    ))
}

fn bags_get_quote_to_base_output(amount_in: &BigUint) -> Result<BigUint, String> {
    let curve = bags_curve_points();
    let mut total_output = BigUint::ZERO;
    let mut sqrt_price = bags_initial_sqrt_price();
    let mut amount_left = amount_in.clone();
    for point in &curve {
        if point.sqrt_price.is_zero() || point.liquidity.is_zero() {
            break;
        }
        if point.sqrt_price > sqrt_price {
            let max_amount_in = bags_get_delta_amount_quote_unsigned(
                &sqrt_price,
                &point.sqrt_price,
                &point.liquidity,
                true,
            )?;
            if amount_left < max_amount_in {
                let next_sqrt_price =
                    bags_get_next_sqrt_price_from_input(&sqrt_price, &point.liquidity, &amount_left)?;
                let output_amount = bags_get_delta_amount_base_unsigned(
                    &sqrt_price,
                    &next_sqrt_price,
                    &point.liquidity,
                    false,
                )?;
                total_output += output_amount;
                amount_left = BigUint::ZERO;
                break;
            }
            total_output += bags_get_delta_amount_base_unsigned(
                &sqrt_price,
                &point.sqrt_price,
                &point.liquidity,
                false,
            )?;
            sqrt_price = point.sqrt_price.clone();
            amount_left = big_sub(&amount_left, &max_amount_in, "remaining quote input")?;
        }
    }
    if !amount_left.is_zero() {
        return Err("Not enough liquidity to process the entire amount".to_string());
    }
    Ok(total_output)
}

fn bags_get_quote_to_base_input_for_output(out_amount: &BigUint) -> Result<BigUint, String> {
    let curve = bags_curve_points();
    let mut total_input = BigUint::ZERO;
    let mut sqrt_price = bags_initial_sqrt_price();
    let mut amount_left = out_amount.clone();
    for point in &curve {
        if point.sqrt_price.is_zero() || point.liquidity.is_zero() {
            break;
        }
        if point.sqrt_price > sqrt_price {
            let max_amount_out = bags_get_delta_amount_base_unsigned(
                &sqrt_price,
                &point.sqrt_price,
                &point.liquidity,
                false,
            )?;
            if amount_left < max_amount_out {
                let next_sqrt_price = bags_get_next_sqrt_price_from_base_output(
                    &sqrt_price,
                    &point.liquidity,
                    &amount_left,
                )?;
                total_input += bags_get_delta_amount_quote_unsigned(
                    &sqrt_price,
                    &next_sqrt_price,
                    &point.liquidity,
                    true,
                )?;
                amount_left = BigUint::ZERO;
                break;
            }
            total_input += bags_get_delta_amount_quote_unsigned(
                &sqrt_price,
                &point.sqrt_price,
                &point.liquidity,
                true,
            )?;
            sqrt_price = point.sqrt_price.clone();
            amount_left = big_sub(&amount_left, &max_amount_out, "remaining base output")?;
        }
    }
    if !amount_left.is_zero() {
        return Err("Not enough liquidity".to_string());
    }
    Ok(total_input)
}

fn bags_get_fee_amount_included(amount: &BigUint, fee_numerator: u64) -> BigUint {
    if fee_numerator == 0 {
        return amount.clone();
    }
    let fee_numerator = BigUint::from(fee_numerator);
    let denominator = BigUint::from(DBC_FEE_DENOMINATOR) - &fee_numerator;
    big_div_rounding(amount * BigUint::from(DBC_FEE_DENOMINATOR), &denominator, true)
}

fn bags_get_fee_amount_excluded(amount: &BigUint, fee_numerator: u64) -> BigUint {
    if fee_numerator == 0 {
        return amount.clone();
    }
    let fee_numerator = BigUint::from(fee_numerator);
    let trading_fee = big_div_rounding(
        amount * fee_numerator,
        &BigUint::from(DBC_FEE_DENOMINATOR),
        true,
    );
    amount - trading_fee
}

fn native_quote_launch(launch_mode: &str, mode: &str, amount: &str) -> Result<Option<LaunchQuote>, String> {
    let input = amount.trim();
    if input.is_empty() {
        return Ok(None);
    }
    let buy_mode = mode.trim().to_ascii_lowercase();
    if buy_mode != "sol" && buy_mode != "tokens" {
        return Err(format!(
            "Unsupported Bags dev buy quote mode: {}. Expected sol or tokens.",
            if buy_mode.is_empty() { "(empty)" } else { buy_mode.as_str() }
        ));
    }
    let fee_numerator = bags_cliff_fee_numerator_for_mode(launch_mode);
    if buy_mode == "sol" {
        let buy_amount_lamports = parse_decimal_to_u128(input, 9, "buy amount")?;
        if buy_amount_lamports == 0 {
            return Ok(None);
        }
        let amount_in = biguint_from_u128(buy_amount_lamports);
        let after_fee = bags_get_fee_amount_excluded(&amount_in, fee_numerator);
        let output_tokens = bags_get_quote_to_base_output(&after_fee)?;
        return Ok(Some(LaunchQuote {
            mode: buy_mode,
            input: input.to_string(),
            estimatedTokens: format_biguint_decimal(&output_tokens, 9, 6)?,
            estimatedSol: format_decimal_u128(buy_amount_lamports, 9, 6),
            estimatedQuoteAmount: format_decimal_u128(buy_amount_lamports, 9, 6),
            quoteAsset: "sol".to_string(),
            quoteAssetLabel: "SOL".to_string(),
            estimatedSupplyPercent: format_bags_supply_percent(&output_tokens),
        }));
    }
    let desired_tokens = parse_decimal_to_u128(input, 9, "buy amount")?;
    if desired_tokens == 0 {
        return Ok(None);
    }
    let output_amount = biguint_from_u128(desired_tokens);
    let excluded_input = bags_get_quote_to_base_input_for_output(&output_amount)?;
    let required_input = bags_get_fee_amount_included(&excluded_input, fee_numerator);
    Ok(Some(LaunchQuote {
        mode: buy_mode,
        input: input.to_string(),
        estimatedTokens: format_decimal_u128(desired_tokens, 9, 6),
        estimatedSol: format_biguint_decimal(&required_input, 9, 6)?,
        estimatedQuoteAmount: format_biguint_decimal(&required_input, 9, 6)?,
        quoteAsset: "sol".to_string(),
        quoteAssetLabel: "SOL".to_string(),
        estimatedSupplyPercent: format_bags_supply_percent(&output_amount),
    }))
}

fn read_le_u8(data: &[u8], offset: &mut usize) -> Result<u8, String> {
    if *offset + 1 > data.len() {
        return Err("Bags account data ended unexpectedly while reading u8.".to_string());
    }
    let value = data[*offset];
    *offset += 1;
    Ok(value)
}

fn read_le_u16(data: &[u8], offset: &mut usize) -> Result<u16, String> {
    if *offset + 2 > data.len() {
        return Err("Bags account data ended unexpectedly while reading u16.".to_string());
    }
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&data[*offset..*offset + 2]);
    *offset += 2;
    Ok(u16::from_le_bytes(bytes))
}

fn read_le_u32(data: &[u8], offset: &mut usize) -> Result<u32, String> {
    if *offset + 4 > data.len() {
        return Err("Bags account data ended unexpectedly while reading u32.".to_string());
    }
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&data[*offset..*offset + 4]);
    *offset += 4;
    Ok(u32::from_le_bytes(bytes))
}

fn read_le_u64(data: &[u8], offset: &mut usize) -> Result<u64, String> {
    if *offset + 8 > data.len() {
        return Err("Bags account data ended unexpectedly while reading u64.".to_string());
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[*offset..*offset + 8]);
    *offset += 8;
    Ok(u64::from_le_bytes(bytes))
}

fn read_le_u128(data: &[u8], offset: &mut usize) -> Result<u128, String> {
    if *offset + 16 > data.len() {
        return Err("Bags account data ended unexpectedly while reading u128.".to_string());
    }
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&data[*offset..*offset + 16]);
    *offset += 16;
    Ok(u128::from_le_bytes(bytes))
}

fn read_pubkey(data: &[u8], offset: &mut usize) -> Result<Pubkey, String> {
    if *offset + 32 > data.len() {
        return Err("Bags account data ended unexpectedly while reading pubkey.".to_string());
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&data[*offset..*offset + 32]);
    *offset += 32;
    Ok(Pubkey::new_from_array(bytes))
}

fn skip_bytes(data: &[u8], offset: &mut usize, count: usize) -> Result<(), String> {
    if *offset + count > data.len() {
        return Err("Bags account data ended unexpectedly while skipping bytes.".to_string());
    }
    *offset += count;
    Ok(())
}

fn parse_optional_pubkey(value: &str) -> Option<Pubkey> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Pubkey::from_str(trimmed).ok()
    }
}

fn normalize_cached_bags_launch_hints(bags_launch: Option<&BagsLaunchMetadata>) -> CachedBagsLaunchHints {
    let Some(source) = bags_launch else {
        return CachedBagsLaunchHints::default();
    };
    CachedBagsLaunchHints {
        config_key: parse_optional_pubkey(&source.configKey),
        migration_fee_option: source.migrationFeeOption,
        expected_migration_family: source.expectedMigrationFamily.trim().to_string(),
        expected_damm_config_key: parse_optional_pubkey(&source.expectedDammConfigKey),
        expected_damm_derivation_mode: source.expectedDammDerivationMode.trim().to_string(),
        pre_migration_dbc_pool_address: parse_optional_pubkey(&source.preMigrationDbcPoolAddress),
    }
}

fn bags_token_program_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(BAGS_TOKEN_PROGRAM_ID).map_err(|error| format!("Invalid SPL token program id: {error}"))
}

pub fn derive_follow_owner_token_account(owner: &Pubkey, mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(get_associated_token_address_with_program_id(
        owner,
        mint,
        &bags_token_program_pubkey()?,
    ))
}

fn bags_token_2022_program_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(BAGS_TOKEN_2022_PROGRAM_ID)
        .map_err(|error| format!("Invalid SPL Token-2022 program id: {error}"))
}

fn bags_jitodontfront_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(JITODONTFRONT_ACCOUNT)
        .map_err(|error| format!("Invalid jitodontfront account: {error}"))
}

fn compute_budget_program_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(COMPUTE_BUDGET_PROGRAM_ID)
        .map_err(|error| format!("Invalid Compute Budget program id: {error}"))
}

fn build_compute_unit_limit_instruction(compute_unit_limit: u32) -> Result<Instruction, String> {
    let mut data = vec![2];
    data.extend_from_slice(&compute_unit_limit.to_le_bytes());
    Ok(Instruction {
        program_id: compute_budget_program_pubkey()?,
        accounts: vec![],
        data,
    })
}

fn build_compute_unit_price_instruction(micro_lamports: u64) -> Result<Instruction, String> {
    let mut data = vec![3];
    data.extend_from_slice(&micro_lamports.to_le_bytes());
    Ok(Instruction {
        program_id: compute_budget_program_pubkey()?,
        accounts: vec![],
        data,
    })
}

#[derive(Debug, Clone)]
struct NativeBagsVersionedTxConfig {
    compute_unit_limit: u64,
    compute_unit_price_micro_lamports: u64,
    tip_lamports: u64,
    tip_account: String,
    jitodontfront: bool,
}

impl NativeBagsVersionedTxConfig {
    fn without_inline_tip(&self) -> Self {
        Self {
            compute_unit_limit: self.compute_unit_limit,
            compute_unit_price_micro_lamports: self.compute_unit_price_micro_lamports,
            tip_lamports: 0,
            tip_account: String::new(),
            jitodontfront: self.jitodontfront,
        }
    }
}

fn decode_compute_budget_units(instruction: &Instruction) -> Option<u64> {
    if instruction.program_id != compute_budget_program_pubkey().ok()? {
        return None;
    }
    if instruction.data.len() == 5 && instruction.data.first().copied() == Some(2) {
        let mut raw = [0u8; 4];
        raw.copy_from_slice(&instruction.data[1..5]);
        return Some(u32::from_le_bytes(raw) as u64);
    }
    None
}

fn decode_compute_budget_price(instruction: &Instruction) -> Option<u64> {
    if instruction.program_id != compute_budget_program_pubkey().ok()? {
        return None;
    }
    if instruction.data.len() == 9 && instruction.data.first().copied() == Some(3) {
        let mut raw = [0u8; 8];
        raw.copy_from_slice(&instruction.data[1..9]);
        return Some(u64::from_le_bytes(raw));
    }
    None
}

fn split_compute_budget_instructions(
    instructions: Vec<Instruction>,
) -> (Vec<Instruction>, Vec<Instruction>, Option<u64>, Option<u64>) {
    let mut non_compute_budget_instructions = Vec::new();
    let mut preserved_compute_budget_instructions = Vec::new();
    let mut compute_unit_limit = None;
    let mut compute_unit_price_micro_lamports = None;
    for instruction in instructions {
        if instruction.program_id != compute_budget_program_pubkey().unwrap_or_default() {
            non_compute_budget_instructions.push(instruction);
            continue;
        }
        if let Some(units) = decode_compute_budget_units(&instruction).filter(|value| *value > 0) {
            compute_unit_limit = Some(units);
            continue;
        }
        if let Some(price) = decode_compute_budget_price(&instruction).filter(|value| *value > 0) {
            compute_unit_price_micro_lamports = Some(price);
            continue;
        }
        preserved_compute_budget_instructions.push(instruction);
    }
    (
        non_compute_budget_instructions,
        preserved_compute_budget_instructions,
        compute_unit_limit,
        compute_unit_price_micro_lamports,
    )
}

fn build_merged_compute_budget_instructions(
    existing_compute_unit_limit: Option<u64>,
    existing_compute_unit_price_micro_lamports: Option<u64>,
    tx_config: &NativeBagsVersionedTxConfig,
) -> Result<Vec<Instruction>, String> {
    let effective_compute_unit_limit =
        existing_compute_unit_limit.unwrap_or_default().max(tx_config.compute_unit_limit);
    let effective_compute_unit_price_micro_lamports = existing_compute_unit_price_micro_lamports
        .unwrap_or_default()
        .max(tx_config.compute_unit_price_micro_lamports);
    let mut instructions = Vec::new();
    if effective_compute_unit_limit > 0 {
        instructions.push(build_compute_unit_limit_instruction(
            u32::try_from(effective_compute_unit_limit)
                .map_err(|_| "Bags compute unit limit exceeded u32.".to_string())?,
        )?);
    }
    if effective_compute_unit_price_micro_lamports > 0 {
        instructions.push(build_compute_unit_price_instruction(
            effective_compute_unit_price_micro_lamports,
        )?);
    }
    Ok(instructions)
}

fn build_inline_tip_instruction(
    owner_pubkey: &Pubkey,
    tip_account: &str,
    tip_lamports: u64,
) -> Result<Option<Instruction>, String> {
    if tip_account.trim().is_empty() || tip_lamports == 0 {
        return Ok(None);
    }
    let tip_pubkey = Pubkey::from_str(tip_account.trim())
        .map_err(|error| format!("Invalid Jito tip account: {error}"))?;
    Ok(Some(transfer(owner_pubkey, &tip_pubkey, tip_lamports)))
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

fn versioned_transaction_has_additional_required_signers(
    transaction: &VersionedTransaction,
    owner_pubkey: &Pubkey,
) -> bool {
    let required_signatures = usize::from(transaction.message.header().num_required_signatures);
    transaction
        .message
        .static_account_keys()
        .iter()
        .take(required_signatures)
        .any(|pubkey| pubkey != owner_pubkey)
}

fn sign_existing_versioned_transaction_with_owner(
    transaction: &mut VersionedTransaction,
    owner: &Keypair,
) -> Result<(), String> {
    let required_signatures = usize::from(transaction.message.header().num_required_signatures);
    let owner_index = transaction
        .message
        .static_account_keys()
        .iter()
        .take(required_signatures)
        .position(|pubkey| pubkey == &owner.pubkey())
        .ok_or_else(|| "Owner pubkey was not a required signer on the Bags transaction.".to_string())?;
    if transaction.signatures.len() != required_signatures {
        return Err("Bags transaction signatures did not match required signer count.".to_string());
    }
    transaction.signatures[owner_index] = owner.sign_message(&transaction.message.serialize());
    Ok(())
}

async fn load_lookup_table_account_for_bags_transaction(
    rpc_url: &str,
    address: &Pubkey,
    commitment: &str,
) -> Result<AddressLookupTableAccount, String> {
    let Some(data) =
        rpc_fetch_account_data(rpc_url, address, commitment, "address-lookup-table").await?
    else {
        return Err(format!("Address lookup table not found: {address}"));
    };
    let table = AddressLookupTable::deserialize(&data)
        .map_err(|error| format!("Failed to decode address lookup table {address}: {error}"))?;
    Ok(AddressLookupTableAccount {
        key: *address,
        addresses: table.addresses.to_vec(),
    })
}

async fn resolve_lookup_table_accounts_for_bags_transaction(
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
            load_lookup_table_account_for_bags_transaction(
                rpc_url,
                &lookup.account_key,
                commitment,
            )
            .await?,
        );
    }
    Ok(resolved)
}

fn resolve_bags_transaction_account_keys(
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

fn decompile_bags_versioned_transaction_instructions(
    transaction: &VersionedTransaction,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<Vec<Instruction>, String> {
    let account_keys = resolve_bags_transaction_account_keys(transaction, lookup_tables)?;
    let mut instructions = Vec::new();
    for compiled in transaction.message.instructions() {
        let program_id = account_keys
            .get(usize::from(compiled.program_id_index))
            .copied()
            .ok_or_else(|| "Bags transaction referenced a missing program account.".to_string())?;
        let mut accounts = Vec::with_capacity(compiled.accounts.len());
        for account_index in &compiled.accounts {
            let index = usize::from(*account_index);
            let pubkey = account_keys
                .get(index)
                .copied()
                .ok_or_else(|| "Bags transaction referenced a missing account meta.".to_string())?;
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

async fn ensure_tx_config_on_bags_versioned_transaction(
    rpc_url: &str,
    owner: &Keypair,
    mut transaction: VersionedTransaction,
    tx_config: &NativeBagsVersionedTxConfig,
    commitment: &str,
    blockhash_override: Option<(String, u64)>,
) -> Result<VersionedTransaction, String> {
    if versioned_transaction_has_additional_required_signers(&transaction, &owner.pubkey()) {
        sign_existing_versioned_transaction_with_owner(&mut transaction, owner)?;
        return Ok(transaction);
    }
    let fresh_blockhash = if let Some((blockhash, _)) = blockhash_override {
        Hash::from_str(blockhash.trim())
            .map_err(|error| format!("Invalid Bags blockhash override: {error}"))?
    } else {
        let (blockhash, _) = fetch_latest_blockhash_cached(rpc_url, commitment).await?;
        Hash::from_str(blockhash.trim())
            .map_err(|error| format!("Invalid Bags latest blockhash: {error}"))?
    };
    let address_lookup_table_accounts =
        resolve_lookup_table_accounts_for_bags_transaction(rpc_url, &transaction, commitment).await?;
    let instructions =
        decompile_bags_versioned_transaction_instructions(&transaction, &address_lookup_table_accounts)?;
    let (
        mut filtered_instructions,
        preserved_compute_budget_instructions,
        existing_compute_unit_limit,
        existing_compute_unit_price_micro_lamports,
    ) = split_compute_budget_instructions(instructions);
    if tx_config.jitodontfront {
        let dontfront = bags_jitodontfront_pubkey()?;
        for instruction in &mut filtered_instructions {
            apply_jitodontfront_to_instruction(instruction, &dontfront);
        }
    }
    let tip_instruction = build_inline_tip_instruction(
        &owner.pubkey(),
        &tx_config.tip_account,
        tx_config.tip_lamports,
    )?;
    let has_tip = filtered_instructions.iter().any(|instruction| {
        is_inline_tip_instruction(
            instruction,
            &owner.pubkey(),
            &tx_config.tip_account,
            tx_config.tip_lamports,
        )
    });
    if let Some(tip_instruction) = tip_instruction {
        if !has_tip {
            filtered_instructions.push(tip_instruction);
        }
    }
    let compute_budget_instructions = build_merged_compute_budget_instructions(
        existing_compute_unit_limit,
        existing_compute_unit_price_micro_lamports,
        tx_config,
    )?;
    let rebuilt_instructions = compute_budget_instructions
        .into_iter()
        .chain(preserved_compute_budget_instructions.into_iter())
        .chain(filtered_instructions.into_iter())
        .collect::<Vec<_>>();
    let message = v0::Message::try_compile(
        &owner.pubkey(),
        &rebuilt_instructions,
        &address_lookup_table_accounts,
        fresh_blockhash,
    )
    .map_err(|error| format!("Failed to rebuild Bags versioned transaction: {error}"))?;
    VersionedTransaction::try_new(VersionedMessage::V0(message), &[owner])
        .map_err(|error| format!("Failed to sign rebuilt Bags versioned transaction: {error}"))
}

fn bags_transaction_label(label_prefix: &str, index: usize, total: usize) -> String {
    if total == 1 {
        label_prefix.to_string()
    } else {
        format!("{label_prefix}-{}", index + 1)
    }
}

fn compiled_transaction_from_bags_versioned_transaction(
    transaction: &VersionedTransaction,
    label: String,
    last_valid_block_height: u64,
    compute_unit_limit: Option<u64>,
    compute_unit_price_micro_lamports: Option<u64>,
    inline_tip_lamports: Option<u64>,
    inline_tip_account: Option<String>,
) -> Result<CompiledTransaction, String> {
    let serialized = bincode::serialize(transaction)
        .map_err(|error| format!("Failed to serialize Bags versioned transaction: {error}"))?;
    let serialized_base64 = BASE64.encode(&serialized);
    Ok(CompiledTransaction {
        label,
        format: "v0".to_string(),
        blockhash: transaction.message.recent_blockhash().to_string(),
        lastValidBlockHeight: last_valid_block_height,
        serializedBase64: serialized_base64.clone(),
        signature: crate::rpc::precompute_transaction_signature(&serialized_base64),
        lookupTablesUsed: transaction
            .message
            .address_table_lookups()
            .unwrap_or(&[])
            .iter()
            .map(|lookup| lookup.account_key.to_string())
            .collect(),
        computeUnitLimit: compute_unit_limit,
        computeUnitPriceMicroLamports: compute_unit_price_micro_lamports,
        inlineTipLamports: inline_tip_lamports,
        inlineTipAccount: inline_tip_account,
    })
}

fn normalize_bags_versioned_transactions(
    transactions: &[VersionedTransaction],
    label_prefix: &str,
    last_valid_block_height: u64,
    compute_unit_limit: Option<u64>,
    compute_unit_price_micro_lamports: Option<u64>,
    inline_tip_lamports: Option<u64>,
    inline_tip_account: Option<String>,
) -> Result<Vec<CompiledTransaction>, String> {
    transactions
        .iter()
        .enumerate()
        .map(|(index, transaction)| {
            compiled_transaction_from_bags_versioned_transaction(
                transaction,
                bags_transaction_label(label_prefix, index, transactions.len()),
                last_valid_block_height,
                compute_unit_limit,
                compute_unit_price_micro_lamports,
                inline_tip_lamports,
                inline_tip_account.clone(),
            )
        })
        .collect()
}

fn token_program_for_flag(flag: u8) -> Result<Pubkey, String> {
    if flag == 0 {
        bags_token_program_pubkey()
    } else {
        bags_token_2022_program_pubkey()
    }
}

fn bags_dbc_pool_authority_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(BAGS_DBC_POOL_AUTHORITY)
        .map_err(|error| format!("Invalid Bags DBC pool authority: {error}"))
}

fn bags_damm_pool_authority_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(BAGS_DAMM_POOL_AUTHORITY)
        .map_err(|error| format!("Invalid Bags DAMM pool authority: {error}"))
}

fn derive_anchor_event_authority(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], program_id).0
}

fn parse_owner_keypair(secret: &[u8]) -> Result<Keypair, String> {
    Keypair::try_from(secret).map_err(|error| format!("Invalid owner secret key: {error}"))
}

fn helper_slippage_minimum_amount(amount_out: u64, slippage_bps: u64) -> u64 {
    if slippage_bps == 0 {
        amount_out
    } else {
        ((u128::from(amount_out) * u128::from(10_000u64.saturating_sub(slippage_bps.min(10_000)))) / 10_000u128)
            as u64
    }
}

fn build_local_trade_fail_closed_error(code: &str, message: &str, extras: &[(&str, String)]) -> String {
    let detail = extras
        .iter()
        .filter_map(|(key, value)| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(format!("{key}={trimmed}"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if detail.is_empty() {
        format!("[bags-local:{code}] {message}")
    } else {
        format!("[bags-local:{code}] {message} ({detail})")
    }
}

fn build_follow_buy_tx_config(
    execution: &NormalizedExecution,
    jito_tip_account: &str,
) -> Result<NativeFollowTxConfig, String> {
    Ok(NativeFollowTxConfig {
        compute_unit_limit: u32::try_from(configured_default_sniper_buy_compute_unit_limit())
            .map_err(|_| "Configured Bags buy compute unit limit is too large.".to_string())?,
        compute_unit_price_micro_lamports: priority_fee_sol_to_micro_lamports(&execution.buyPriorityFeeSol)?,
        tip_lamports: follow_tip_lamports_for_provider(&execution.buyProvider, &execution.buyTipSol, "buy tip")?,
        tip_account: jito_tip_account.trim().to_string(),
        jitodontfront: execution.buyJitodontfront,
    })
}

fn build_follow_sell_tx_config(
    execution: &NormalizedExecution,
    jito_tip_account: &str,
) -> Result<NativeFollowTxConfig, String> {
    Ok(NativeFollowTxConfig {
        compute_unit_limit: u32::try_from(configured_default_dev_auto_sell_compute_unit_limit())
            .map_err(|_| "Configured Bags sell compute unit limit is too large.".to_string())?,
        compute_unit_price_micro_lamports: priority_fee_sol_to_micro_lamports(&execution.sellPriorityFeeSol)?,
        tip_lamports: follow_tip_lamports_for_provider(
            &execution.sellProvider,
            &execution.sellTipSol,
            "sell tip",
        )?,
        tip_account: jito_tip_account.trim().to_string(),
        jitodontfront: execution.sellJitodontfront,
    })
}

fn apply_jitodontfront_to_instruction(instruction: &mut Instruction, dontfront: &Pubkey) {
    if instruction.accounts.iter().any(|meta| meta.pubkey == *dontfront) {
        return;
    }
    instruction
        .accounts
        .push(AccountMeta::new_readonly(*dontfront, false));
}

fn build_native_follow_instructions(
    mut core_instructions: Vec<Instruction>,
    tx_config: &NativeFollowTxConfig,
    payer: &Pubkey,
) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    if tx_config.compute_unit_limit > 0 {
        instructions.push(build_compute_unit_limit_instruction(tx_config.compute_unit_limit)?);
    }
    if tx_config.compute_unit_price_micro_lamports > 0 {
        instructions.push(build_compute_unit_price_instruction(
            tx_config.compute_unit_price_micro_lamports,
        )?);
    }
    if tx_config.jitodontfront {
        let dontfront = bags_jitodontfront_pubkey()?;
        for instruction in &mut core_instructions {
            apply_jitodontfront_to_instruction(instruction, &dontfront);
        }
    }
    instructions.extend(core_instructions);
    if tx_config.tip_lamports > 0 && !tx_config.tip_account.trim().is_empty() {
        let tip_account = Pubkey::from_str(tx_config.tip_account.trim())
            .map_err(|error| format!("Invalid Jito tip account: {error}"))?;
        instructions.push(transfer(payer, &tip_account, tx_config.tip_lamports));
    }
    Ok(instructions)
}

async fn compile_legacy_follow_transaction(
    label: &str,
    rpc_url: &str,
    commitment: &str,
    owner: &Keypair,
    tx_config: &NativeFollowTxConfig,
    core_instructions: Vec<Instruction>,
) -> Result<CompiledTransaction, String> {
    let instructions = build_native_follow_instructions(core_instructions, tx_config, &owner.pubkey())?;
    let (blockhash, last_valid_block_height) =
        fetch_latest_blockhash_cached(rpc_url, commitment).await?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&owner.pubkey()),
        &[owner],
        Hash::from_str(&blockhash).map_err(|error| format!("Invalid blockhash for follow transaction: {error}"))?,
    );
    let serialized = bincode::serialize(&transaction)
        .map_err(|error| format!("Failed to serialize Bags follow transaction: {error}"))?;
    let serialized_base64 = BASE64.encode(serialized);
    let signature = crate::rpc::precompute_transaction_signature(&serialized_base64);
    Ok(CompiledTransaction {
        label: label.to_string(),
        format: "legacy".to_string(),
        blockhash,
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

fn decode_dbc_virtual_pool(data: &[u8]) -> Result<DecodedDbcVirtualPool, String> {
    if data.len() < 8 || data[..8] != DBC_VIRTUAL_POOL_DISCRIMINATOR {
        return Err("Unexpected DBC virtual pool discriminator.".to_string());
    }
    let mut offset = 8;
    skip_bytes(data, &mut offset, 32)?;
    let volatility_accumulator = read_le_u128(data, &mut offset)?;
    skip_bytes(data, &mut offset, 16)?;
    let config = read_pubkey(data, &mut offset)?;
    let creator = read_pubkey(data, &mut offset)?;
    let base_mint = read_pubkey(data, &mut offset)?;
    let base_vault = read_pubkey(data, &mut offset)?;
    let quote_vault = read_pubkey(data, &mut offset)?;
    let base_reserve = read_le_u64(data, &mut offset)?;
    let quote_reserve = read_le_u64(data, &mut offset)?;
    skip_bytes(data, &mut offset, 32)?;
    let sqrt_price = read_le_u128(data, &mut offset)?;
    let activation_point = read_le_u64(data, &mut offset)?;
    let pool_type = read_le_u8(data, &mut offset)?;
    let is_migrated = read_le_u8(data, &mut offset)? != 0;
    Ok(DecodedDbcVirtualPool {
        config,
        creator,
        base_mint,
        base_vault,
        quote_vault,
        sqrt_price,
        base_reserve,
        quote_reserve,
        volatility_accumulator,
        activation_point,
        pool_type,
        is_migrated,
    })
}

fn decode_dbc_pool_config(data: &[u8]) -> Result<DecodedDbcPoolConfig, String> {
    if data.len() < 8 || data[..8] != DBC_POOL_CONFIG_DISCRIMINATOR {
        return Err("Unexpected DBC pool config discriminator.".to_string());
    }
    let mut offset = 8;
    let quote_mint = read_pubkey(data, &mut offset)?;
    skip_bytes(data, &mut offset, 64)?;
    let cliff_fee_numerator = read_le_u64(data, &mut offset)?;
    let second_factor = read_le_u64(data, &mut offset)?;
    let third_factor = read_le_u64(data, &mut offset)?;
    let first_factor = read_le_u16(data, &mut offset)?;
    let base_fee_mode = read_le_u8(data, &mut offset)?;
    skip_bytes(data, &mut offset, 5)?;
    let dynamic_fee_initialized = read_le_u8(data, &mut offset)? != 0;
    skip_bytes(data, &mut offset, 7)?;
    let _max_volatility_accumulator = read_le_u32(data, &mut offset)?;
    let variable_fee_control = read_le_u32(data, &mut offset)?;
    let bin_step = read_le_u16(data, &mut offset)?;
    skip_bytes(data, &mut offset, 6)?;
    skip_bytes(data, &mut offset, 8)?;
    skip_bytes(data, &mut offset, 16)?;
    skip_bytes(data, &mut offset, 48)?;
    let collect_fee_mode = read_le_u8(data, &mut offset)?;
    let _migration_option = read_le_u8(data, &mut offset)?;
    let activation_type = read_le_u8(data, &mut offset)?;
    let _token_decimal = read_le_u8(data, &mut offset)?;
    let _version = read_le_u8(data, &mut offset)?;
    let _token_type = read_le_u8(data, &mut offset)?;
    let quote_token_flag = read_le_u8(data, &mut offset)?;
    skip_bytes(data, &mut offset, 4)?;
    let migration_fee_option = read_le_u8(data, &mut offset)?;
    let _fixed_token_supply_flag = read_le_u8(data, &mut offset)?;
    let creator_trading_fee_percentage = read_le_u8(data, &mut offset)?;
    let _token_update_authority = read_le_u8(data, &mut offset)?;
    let _migration_fee_percentage = read_le_u8(data, &mut offset)?;
    let creator_migration_fee_percentage = read_le_u8(data, &mut offset)?;
    skip_bytes(data, &mut offset, 7)?;
    let _swap_base_amount = read_le_u64(data, &mut offset)?;
    let migration_quote_threshold = read_le_u64(data, &mut offset)?;
    skip_bytes(data, &mut offset, 8)?;
    skip_bytes(data, &mut offset, 16)?;
    skip_bytes(data, &mut offset, 48)?;
    skip_bytes(data, &mut offset, 16)?;
    skip_bytes(data, &mut offset, 4)?;
    skip_bytes(data, &mut offset, 12)?;
    skip_bytes(data, &mut offset, 16)?;
    let sqrt_start_price = read_le_u128(data, &mut offset)?;
    let mut curve = Vec::with_capacity(20);
    for _ in 0..20 {
        curve.push(BagsCurvePoint {
            sqrt_price: biguint_from_u128(read_le_u128(data, &mut offset)?),
            liquidity: biguint_from_u128(read_le_u128(data, &mut offset)?),
        });
    }
    Ok(DecodedDbcPoolConfig {
        quote_mint,
        collect_fee_mode,
        activation_type,
        quote_token_flag,
        migration_fee_option,
        creator_trading_fee_percentage,
        creator_migration_fee_percentage,
        migration_quote_threshold,
        sqrt_start_price,
        curve,
        base_fee: DecodedDbcBaseFeeConfig {
            cliff_fee_numerator,
            first_factor,
            second_factor,
            third_factor,
            base_fee_mode,
        },
        dynamic_fee: DecodedDbcDynamicFeeConfig {
            initialized: dynamic_fee_initialized,
            variable_fee_control,
            bin_step,
            volatility_accumulator: 0,
        },
    })
}

fn decode_damm_pool(data: &[u8]) -> Result<DecodedDammPool, String> {
    if data.len() < 8 || data[..8] != CPAMM_POOL_DISCRIMINATOR {
        return Err("Unexpected DAMM v2 pool discriminator.".to_string());
    }
    let mut offset = 8;
    let cliff_fee_numerator = read_le_u64(data, &mut offset)?;
    let fee_scheduler_mode = read_le_u8(data, &mut offset)?;
    skip_bytes(data, &mut offset, 5)?;
    let number_of_period = read_le_u16(data, &mut offset)?;
    let period_frequency = read_le_u64(data, &mut offset)?;
    let reduction_factor = read_le_u64(data, &mut offset)?;
    skip_bytes(data, &mut offset, 8)?;
    let protocol_fee_percent = read_le_u8(data, &mut offset)?;
    let _partner_fee_percent = read_le_u8(data, &mut offset)?;
    let _referral_fee_percent = read_le_u8(data, &mut offset)?;
    skip_bytes(data, &mut offset, 5)?;
    let dynamic_fee_initialized = read_le_u8(data, &mut offset)? != 0;
    skip_bytes(data, &mut offset, 7)?;
    let _max_volatility_accumulator = read_le_u32(data, &mut offset)?;
    let variable_fee_control = read_le_u32(data, &mut offset)?;
    let bin_step = read_le_u16(data, &mut offset)?;
    skip_bytes(data, &mut offset, 6)?;
    let _last_update_timestamp = read_le_u64(data, &mut offset)?;
    skip_bytes(data, &mut offset, 16)?;
    skip_bytes(data, &mut offset, 16)?;
    let volatility_accumulator = read_le_u128(data, &mut offset)?;
    skip_bytes(data, &mut offset, 16)?;
    skip_bytes(data, &mut offset, 16)?;
    let token_a_mint = read_pubkey(data, &mut offset)?;
    let token_b_mint = read_pubkey(data, &mut offset)?;
    let token_a_vault = read_pubkey(data, &mut offset)?;
    let token_b_vault = read_pubkey(data, &mut offset)?;
    skip_bytes(data, &mut offset, 64)?;
    let liquidity = read_le_u128(data, &mut offset)?;
    skip_bytes(data, &mut offset, 16)?;
    skip_bytes(data, &mut offset, 32)?;
    skip_bytes(data, &mut offset, 32)?;
    let sqrt_price = read_le_u128(data, &mut offset)?;
    let activation_point = read_le_u64(data, &mut offset)?;
    let activation_type = read_le_u8(data, &mut offset)?;
    let _pool_status = read_le_u8(data, &mut offset)?;
    let token_a_flag = read_le_u8(data, &mut offset)?;
    let token_b_flag = read_le_u8(data, &mut offset)?;
    let collect_fee_mode = read_le_u8(data, &mut offset)?;
    skip_bytes(data, &mut offset, 3)?;
    skip_bytes(data, &mut offset, 64)?;
    skip_bytes(data, &mut offset, 16)?;
    skip_bytes(data, &mut offset, 64)?;
    let creator = read_pubkey(data, &mut offset)?;
    let _ = protocol_fee_percent;
    Ok(DecodedDammPool {
        token_a_mint,
        token_b_mint,
        token_a_vault,
        token_b_vault,
        liquidity,
        sqrt_price,
        activation_point,
        activation_type,
        token_a_flag,
        token_b_flag,
        collect_fee_mode,
        creator,
        pool_fees: DecodedDammPoolFees {
            base_fee: DecodedDammBaseFee {
                cliff_fee_numerator,
                fee_scheduler_mode,
                number_of_period,
                period_frequency,
                reduction_factor,
            },
            collect_fee_mode,
            dynamic_fee: DecodedDammDynamicFee {
                initialized: dynamic_fee_initialized,
                variable_fee_control,
                bin_step,
                volatility_accumulator,
            },
        },
    })
}

fn bags_native_mint_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(BAGS_NATIVE_MINT).map_err(|error| format!("Invalid Bags native mint: {error}"))
}

fn bags_dbc_program_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(BAGS_DBC_PROGRAM_ID).map_err(|error| format!("Invalid Bags DBC program id: {error}"))
}

fn bags_damm_v2_program_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(BAGS_DAMM_V2_PROGRAM_ID)
        .map_err(|error| format!("Invalid Bags DAMM v2 program id: {error}"))
}

fn pubkey_order_pair<'a>(left: &'a Pubkey, right: &'a Pubkey) -> (&'a Pubkey, &'a Pubkey) {
    if left.to_bytes() > right.to_bytes() {
        (left, right)
    } else {
        (right, left)
    }
}

fn derive_dbc_pool_address(quote_mint: &Pubkey, base_mint: &Pubkey, config: &Pubkey) -> Result<Pubkey, String> {
    let program_id = bags_dbc_program_pubkey()?;
    let (first, second) = pubkey_order_pair(quote_mint, base_mint);
    let (pool, _) = Pubkey::find_program_address(
        &[b"pool", &config.to_bytes(), &first.to_bytes(), &second.to_bytes()],
        &program_id,
    );
    Ok(pool)
}

fn derive_damm_pool_address(config: &Pubkey, mint: &Pubkey, quote_mint: &Pubkey) -> Result<Pubkey, String> {
    let program_id = bags_damm_v2_program_pubkey()?;
    let (first, second) = pubkey_order_pair(mint, quote_mint);
    let (pool, _) = Pubkey::find_program_address(
        &[b"pool", &config.to_bytes(), &first.to_bytes(), &second.to_bytes()],
        &program_id,
    );
    Ok(pool)
}

fn derive_damm_customizable_pool_address(mint: &Pubkey, quote_mint: &Pubkey) -> Result<Pubkey, String> {
    let program_id = bags_damm_v2_program_pubkey()?;
    let (first, second) = pubkey_order_pair(mint, quote_mint);
    let (pool, _) =
        Pubkey::find_program_address(&[b"cpool", &first.to_bytes(), &second.to_bytes()], &program_id);
    Ok(pool)
}

fn expected_migration_family_from_config(config: &DecodedDbcPoolConfig) -> String {
    match config.migration_fee_option {
        0..=5 => "fixed".to_string(),
        6 => "customizable".to_string(),
        _ => String::new(),
    }
}

fn derive_canonical_damm_pool_address(
    mint: &Pubkey,
    config: &DecodedDbcPoolConfig,
) -> Result<Option<Pubkey>, String> {
    let native_mint = bags_native_mint_pubkey()?;
    match config.migration_fee_option {
        0..=5 => {
            let config_address =
                Pubkey::from_str(DAMM_V2_MIGRATION_FEE_ADDRESS[config.migration_fee_option as usize])
                    .map_err(|error| format!("Invalid DAMM migration fee address: {error}"))?;
            Ok(Some(derive_damm_pool_address(&config_address, mint, &native_mint)?))
        }
        6 => Ok(Some(derive_damm_customizable_pool_address(mint, &native_mint)?)),
        _ => Ok(None),
    }
}

fn is_completed_dbc_pool(pool: &DecodedDbcVirtualPool, config: &DecodedDbcPoolConfig) -> bool {
    pool.is_migrated || pool.quote_reserve >= config.migration_quote_threshold
}

async fn rpc_fetch_first_dbc_pool_by_mint(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
) -> Result<Option<(Pubkey, Vec<u8>)>, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "launchdeck-bags-dbc-pool-by-mint",
        "method": "getProgramAccounts",
        "params": [
            BAGS_DBC_PROGRAM_ID,
            {
                "commitment": commitment,
                "encoding": "base64",
                "filters": [
                    {
                        "memcmp": {
                            "offset": DBC_POOL_BY_BASE_MINT_OFFSET,
                            "bytes": mint.to_string(),
                        }
                    }
                ]
            }
        ]
    });
    let response = bags_fee_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bags DBC pool by mint: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bags DBC pool by mint: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RpcResponse<Vec<RpcProgramAccount>> = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bags DBC pool response: {error}"))?;
    let Some(account) = parsed.result.into_iter().next() else {
        return Ok(None);
    };
    let pubkey =
        Pubkey::from_str(&account.pubkey).map_err(|error| format!("Invalid DBC pool pubkey: {error}"))?;
    let bytes = BASE64
        .decode(account.account.data.0.trim())
        .map_err(|error| format!("Failed to decode Bags DBC pool account: {error}"))?;
    Ok(Some((pubkey, bytes)))
}

async fn rpc_fetch_account_data(
    rpc_url: &str,
    address: &Pubkey,
    commitment: &str,
    label: &str,
) -> Result<Option<Vec<u8>>, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": format!("launchdeck-bags-{label}-account"),
        "method": "getAccountInfo",
        "params": [
            address.to_string(),
            {
                "commitment": commitment,
                "encoding": "base64",
            }
        ]
    });
    let response = bags_fee_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bags {label} account: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bags {label} account: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RpcResponse<RpcAccountInfoResult> = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bags {label} account response: {error}"))?;
    let Some(value) = parsed.result.value else {
        return Ok(None);
    };
    let bytes = BASE64
        .decode(value.data.0.trim())
        .map_err(|error| format!("Failed to decode Bags {label} account: {error}"))?;
    Ok(Some(bytes))
}

async fn rpc_get_slot(rpc_url: &str, commitment: &str) -> Result<u64, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "launchdeck-bags-slot",
        "method": "getSlot",
        "params": [{ "commitment": commitment }]
    });
    let response = bags_fee_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bags current slot: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bags current slot: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RpcResponse<u64> = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bags current slot response: {error}"))?;
    Ok(parsed.result)
}

async fn rpc_get_block_time(rpc_url: &str, slot: u64) -> Result<Option<i64>, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "launchdeck-bags-block-time",
        "method": "getBlockTime",
        "params": [slot]
    });
    let response = bags_fee_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bags block time: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bags block time: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RpcResponse<Option<i64>> = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bags block time response: {error}"))?;
    Ok(parsed.result)
}

async fn fetch_bags_token_supply_value(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
) -> Result<RpcTokenSupplyValue, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "launchdeck-bags-token-supply",
        "method": "getTokenSupply",
        "params": [
            mint.to_string(),
            { "commitment": commitment }
        ]
    });
    let response = bags_fee_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bags token supply: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bags token supply: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RpcResponse<RpcTokenSupplyResult> = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bags token supply response: {error}"))?;
    Ok(parsed.result.value)
}

async fn fetch_bags_token_account_amount(
    rpc_url: &str,
    token_account: &Pubkey,
    commitment: &str,
) -> Result<u64, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "launchdeck-bags-token-balance",
        "method": "getTokenAccountBalance",
        "params": [
            token_account.to_string(),
            { "commitment": commitment }
        ]
    });
    let response = bags_fee_http_client()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to fetch Bags token account balance: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch Bags token account balance: RPC returned status {}.",
            response.status()
        ));
    }
    let parsed: RpcResponse<RpcTokenAccountBalanceResult> = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bags token account balance response: {error}"))?;
    parsed
        .result
        .value
        .amount
        .trim()
        .parse::<u64>()
        .map_err(|error| format!("Invalid Bags token account balance amount: {error}"))
}

fn unix_now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

async fn current_point_for_dbc_config(
    rpc_url: &str,
    config: &DecodedDbcPoolConfig,
    commitment: &str,
) -> Result<u64, String> {
    let current_slot = rpc_get_slot(rpc_url, commitment).await?;
    if config.activation_type == 0 {
        return Ok(current_slot);
    }
    Ok(rpc_get_block_time(rpc_url, current_slot)
        .await?
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or_else(unix_now_seconds))
}

async fn current_time_for_damm(rpc_url: &str, commitment: &str) -> Result<(u64, u64), String> {
    let current_slot = rpc_get_slot(rpc_url, commitment).await?;
    let current_time = rpc_get_block_time(rpc_url, current_slot)
        .await?
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or_else(unix_now_seconds);
    Ok((current_slot, current_time))
}

fn resolve_cached_damm_pool_address(
    mint: &Pubkey,
    cached: &CachedBagsLaunchHints,
) -> Result<Option<(Pubkey, Option<Pubkey>)>, String> {
    let native_mint = bags_native_mint_pubkey()?;
    if cached.migration_fee_option == Some(6)
        || cached.expected_migration_family.eq_ignore_ascii_case("customizable")
    {
        return Ok(Some((
            derive_damm_customizable_pool_address(mint, &native_mint)?,
            cached.expected_damm_config_key,
        )));
    }
    if let Some(config_address) = cached.expected_damm_config_key {
        return Ok(Some((
            derive_damm_pool_address(&config_address, mint, &native_mint)?,
            Some(config_address),
        )));
    }
    if let Some(migration_fee_option) = cached.migration_fee_option {
        if (0..=5).contains(&migration_fee_option) {
            let config_address =
                Pubkey::from_str(DAMM_V2_MIGRATION_FEE_ADDRESS[migration_fee_option as usize])
                    .map_err(|error| format!("Invalid DAMM migration fee address: {error}"))?;
            return Ok(Some((
                derive_damm_pool_address(&config_address, mint, &native_mint)?,
                Some(config_address),
            )));
        }
    }
    Ok(None)
}

async fn rpc_account_exists(
    rpc_url: &str,
    address: &Pubkey,
    commitment: &str,
    label: &str,
) -> Result<bool, String> {
    Ok(rpc_fetch_account_data(rpc_url, address, commitment, label)
        .await?
        .is_some())
}

async fn maybe_create_ata_instruction(
    rpc_url: &str,
    commitment: &str,
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
    label: &str,
) -> Result<(Pubkey, Option<Instruction>), String> {
    let ata = get_associated_token_address_with_program_id(owner, mint, token_program);
    if rpc_account_exists(rpc_url, &ata, commitment, label).await? {
        Ok((ata, None))
    } else {
        Ok((
            ata,
            Some(create_associated_token_account_idempotent(
                payer,
                owner,
                mint,
                token_program,
            )),
        ))
    }
}

fn build_wrap_sol_instructions(owner: &Pubkey, wrapped_ata: &Pubkey, amount_lamports: u64) -> Result<Vec<Instruction>, String> {
    Ok(vec![
        transfer(owner, wrapped_ata, amount_lamports),
        spl_token::instruction::sync_native(&bags_token_program_pubkey()?, wrapped_ata)
            .map_err(|error| format!("Failed to build sync-native instruction: {error}"))?,
    ])
}

fn build_unwrap_sol_instruction(owner: &Pubkey, receiver: &Pubkey) -> Result<Instruction, String> {
    let wrapped_ata = get_associated_token_address_with_program_id(
        owner,
        &bags_native_mint_pubkey()?,
        &bags_token_program_pubkey()?,
    );
    spl_token::instruction::close_account(
        &bags_token_program_pubkey()?,
        &wrapped_ata,
        receiver,
        owner,
        &[],
    )
    .map_err(|error| format!("Failed to build close wrapped SOL instruction: {error}"))
}

fn build_swap_instruction_data(amount_in: u64, minimum_amount_out: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(24);
    data.extend_from_slice(&DBC_SWAP_DISCRIMINATOR);
    data.extend_from_slice(&amount_in.to_le_bytes());
    data.extend_from_slice(&minimum_amount_out.to_le_bytes());
    data
}

fn build_dbc_swap_instruction(
    owner: &Pubkey,
    pool_address: &Pubkey,
    pool: &DecodedDbcVirtualPool,
    config: &DecodedDbcPoolConfig,
    input_token_account: &Pubkey,
    output_token_account: &Pubkey,
    swap_base_for_quote: bool,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<Instruction, String> {
    let program_id = bags_dbc_program_pubkey()?;
    let event_authority = derive_anchor_event_authority(&program_id);
    let base_token_program = token_program_for_flag(pool.pool_type)?;
    let quote_token_program = token_program_for_flag(config.quote_token_flag)?;
    let (token_base_program, token_quote_program) = if swap_base_for_quote {
        (base_token_program, quote_token_program)
    } else {
        (quote_token_program, base_token_program)
    };
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(bags_dbc_pool_authority_pubkey()?, false),
            AccountMeta::new_readonly(pool.config, false),
            AccountMeta::new(*pool_address, false),
            AccountMeta::new(*input_token_account, false),
            AccountMeta::new(*output_token_account, false),
            AccountMeta::new(pool.base_vault, false),
            AccountMeta::new(pool.quote_vault, false),
            AccountMeta::new_readonly(pool.base_mint, false),
            AccountMeta::new_readonly(config.quote_mint, false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new_readonly(token_base_program, false),
            AccountMeta::new_readonly(token_quote_program, false),
            AccountMeta::new(program_id, false),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(program_id, false),
        ],
        data: build_swap_instruction_data(amount_in, minimum_amount_out),
    })
}

fn build_damm_swap_instruction(
    owner: &Pubkey,
    pool_address: &Pubkey,
    pool: &DecodedDammPool,
    input_token_account: &Pubkey,
    output_token_account: &Pubkey,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<Instruction, String> {
    let program_id = bags_damm_v2_program_pubkey()?;
    let event_authority = derive_anchor_event_authority(&program_id);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(bags_damm_pool_authority_pubkey()?, false),
            AccountMeta::new(*pool_address, false),
            AccountMeta::new(*input_token_account, false),
            AccountMeta::new(*output_token_account, false),
            AccountMeta::new(pool.token_a_vault, false),
            AccountMeta::new(pool.token_b_vault, false),
            AccountMeta::new_readonly(pool.token_a_mint, false),
            AccountMeta::new_readonly(pool.token_b_mint, false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new_readonly(token_program_for_flag(pool.token_a_flag)?, false),
            AccountMeta::new_readonly(token_program_for_flag(pool.token_b_flag)?, false),
            AccountMeta::new(program_id, false),
            AccountMeta::new_readonly(event_authority, false),
            AccountMeta::new_readonly(program_id, false),
        ],
        data: build_swap_instruction_data(amount_in, minimum_amount_out),
    })
}

fn big_pow_q64(base_q64: &BigUint, mut exp: u64) -> BigUint {
    let one = BigUint::from(1u8) << DBC_RESOLUTION_BITS;
    let mut result = one.clone();
    let mut squared = base_q64.clone();
    while exp > 0 {
        if exp & 1 == 1 {
            result = (result * &squared) >> DBC_RESOLUTION_BITS;
        }
        squared = (&squared * &squared) >> DBC_RESOLUTION_BITS;
        exp >>= 1;
    }
    result
}

fn bags_dbc_rate_limiter_applies(
    base_fee_mode: u8,
    trade_direction: u8,
    current_point: u64,
    activation_point: u64,
    max_limiter_duration: u64,
) -> bool {
    base_fee_mode == 2
        && trade_direction == 1
        && current_point >= activation_point
        && current_point <= activation_point.saturating_add(max_limiter_duration)
}

fn bags_dbc_rate_limiter_fee_numerator(
    cliff_fee_numerator: u64,
    reference_amount: u64,
    fee_increment_bps: u16,
    input_amount: u64,
) -> u64 {
    if reference_amount == 0 || input_amount <= reference_amount {
        return cliff_fee_numerator.min(DBC_MAX_FEE_NUMERATOR);
    }
    let c = BigUint::from(cliff_fee_numerator);
    let diff = BigUint::from(input_amount - reference_amount);
    let reference = BigUint::from(reference_amount);
    let a = &diff / &reference;
    let b = &diff % &reference;
    let max_index = if fee_increment_bps == 0 {
        BigUint::ZERO
    } else {
        BigUint::from((DBC_MAX_FEE_NUMERATOR.saturating_sub(cliff_fee_numerator))
            / ((u64::from(fee_increment_bps) * DBC_FEE_DENOMINATOR) / DBC_BASIS_POINT_MAX))
    };
    let i = (BigUint::from(fee_increment_bps) * BigUint::from(DBC_FEE_DENOMINATOR))
        / BigUint::from(DBC_BASIS_POINT_MAX);
    let trading_fee_numerator = if a < max_index {
        let numerator1 = &c
            + (&c * &a)
            + ((&i * &a * (&a + BigUint::from(1u8))) / BigUint::from(2u8));
        let numerator2 = &c + (&i * (&a + BigUint::from(1u8)));
        (&reference * numerator1) + (&b * numerator2)
    } else {
        let numerator1 = &c
            + (&c * &max_index)
            + ((&i * &max_index * (&max_index + BigUint::from(1u8))) / BigUint::from(2u8));
        let first_fee = &reference * numerator1;
        let d = &a - &max_index;
        let left_amount = (&d * &reference) + &b;
        first_fee + (left_amount * BigUint::from(DBC_MAX_FEE_NUMERATOR))
    };
    let trading_fee =
        big_div_rounding(trading_fee_numerator, &BigUint::from(DBC_FEE_DENOMINATOR), true);
    big_div_rounding(
        trading_fee * BigUint::from(DBC_FEE_DENOMINATOR),
        &BigUint::from(input_amount),
        true,
    )
    .to_u64()
    .unwrap_or(DBC_MAX_FEE_NUMERATOR)
    .min(DBC_MAX_FEE_NUMERATOR)
}

fn bags_dbc_trade_fee_numerator_for_amount(
    pool: &DecodedDbcVirtualPool,
    config: &DecodedDbcPoolConfig,
    current_point: u64,
    trade_direction: u8,
    amount_for_fee: Option<u64>,
) -> u64 {
    let base = &config.base_fee;
    let mut fee_numerator = base.cliff_fee_numerator;
    if base.base_fee_mode == 2 {
        if current_point < pool.activation_point {
            fee_numerator = base.cliff_fee_numerator;
        } else if current_point > pool.activation_point.saturating_add(base.second_factor) {
            fee_numerator = base.cliff_fee_numerator;
        } else if let Some(amount_for_fee) = amount_for_fee {
            if bags_dbc_rate_limiter_applies(
                base.base_fee_mode,
                trade_direction,
                current_point,
                pool.activation_point,
                base.second_factor,
            ) {
                fee_numerator = bags_dbc_rate_limiter_fee_numerator(
                    base.cliff_fee_numerator,
                    base.third_factor,
                    base.first_factor,
                    amount_for_fee,
                );
            }
        }
    } else if base.second_factor > 0 {
        let period = if current_point < pool.activation_point {
            u64::from(base.first_factor)
        } else {
            ((current_point - pool.activation_point) / base.second_factor).min(u64::from(base.first_factor))
        };
        if base.base_fee_mode == 0 {
            fee_numerator = base
                .cliff_fee_numerator
                .saturating_sub(period.saturating_mul(base.third_factor));
        } else if base.base_fee_mode == 1 {
            let one = BigUint::from(1u8) << DBC_RESOLUTION_BITS;
            let reduction =
                (BigUint::from(base.third_factor) << DBC_RESOLUTION_BITS) / BigUint::from(DBC_BASIS_POINT_MAX);
            let decay_base = if reduction >= one { BigUint::ZERO } else { &one - reduction };
            let decay = big_pow_q64(&decay_base, period);
            fee_numerator = ((BigUint::from(base.cliff_fee_numerator) * decay) >> DBC_RESOLUTION_BITS)
                .to_u64()
                .unwrap_or(base.cliff_fee_numerator);
        }
    }
    if config.dynamic_fee.initialized && config.dynamic_fee.variable_fee_control > 0 {
        let dynamic = BigUint::from(pool.volatility_accumulator)
            * BigUint::from(config.dynamic_fee.bin_step)
            * BigUint::from(pool.volatility_accumulator)
            * BigUint::from(config.dynamic_fee.bin_step)
            * BigUint::from(config.dynamic_fee.variable_fee_control);
        let dynamic = (dynamic + BigUint::from(99_999_999_999u64)) / BigUint::from(100_000_000_000u64);
        fee_numerator = (BigUint::from(fee_numerator) + dynamic)
            .to_u64()
            .unwrap_or(DBC_MAX_FEE_NUMERATOR)
            .min(DBC_MAX_FEE_NUMERATOR);
    }
    fee_numerator.min(DBC_MAX_FEE_NUMERATOR)
}

fn bags_dbc_fee_on_amount(
    amount: u64,
    pool: &DecodedDbcVirtualPool,
    config: &DecodedDbcPoolConfig,
    current_point: u64,
    trade_direction: u8,
) -> Result<u64, String> {
    let fee_numerator = bags_dbc_trade_fee_numerator_for_amount(
        pool,
        config,
        current_point,
        trade_direction,
        Some(amount),
    );
    let amount_after_fee = bags_get_fee_amount_excluded(&BigUint::from(amount), fee_numerator);
    amount_after_fee
        .to_u64()
        .ok_or_else(|| "Bags fee-adjusted amount overflowed u64.".to_string())
}

fn bags_dbc_swap_amount_from_base_to_quote(
    current_sqrt_price: &BigUint,
    curve: &[BagsCurvePoint],
    amount_in: &BigUint,
) -> Result<BigUint, String> {
    if amount_in.is_zero() {
        return Ok(BigUint::ZERO);
    }
    let mut total_output = BigUint::ZERO;
    let mut sqrt_price = current_sqrt_price.clone();
    let mut amount_left = amount_in.clone();
    for index in (0..curve.len()).rev() {
        let point = &curve[index];
        if point.sqrt_price.is_zero() || point.liquidity.is_zero() {
            continue;
        }
        if point.sqrt_price < sqrt_price {
            let current_liquidity = if index + 1 < curve.len() {
                curve[index + 1].liquidity.clone()
            } else {
                point.liquidity.clone()
            };
            if current_liquidity.is_zero() {
                continue;
            }
            let max_amount_in = bags_get_delta_amount_base_unsigned(
                &point.sqrt_price,
                &sqrt_price,
                &current_liquidity,
                true,
            )?;
            if amount_left < max_amount_in {
                let next_sqrt_price =
                    bags_get_next_sqrt_price_from_amount_base_rounding_up(&sqrt_price, &current_liquidity, &amount_left)?;
                total_output += bags_get_delta_amount_quote_unsigned(
                    &next_sqrt_price,
                    &sqrt_price,
                    &current_liquidity,
                    false,
                )?;
                amount_left = BigUint::ZERO;
                break;
            }
            total_output += bags_get_delta_amount_quote_unsigned(
                &point.sqrt_price,
                &sqrt_price,
                &current_liquidity,
                false,
            )?;
            sqrt_price = point.sqrt_price.clone();
            amount_left = big_sub(&amount_left, &max_amount_in, "remaining base input")?;
        }
    }
    if !amount_left.is_zero() {
        if curve.is_empty() || curve[0].liquidity.is_zero() {
            return Err("Not enough liquidity to process the entire amount".to_string());
        }
        let next_sqrt_price =
            bags_get_next_sqrt_price_from_amount_base_rounding_up(&sqrt_price, &curve[0].liquidity, &amount_left)?;
        total_output += bags_get_delta_amount_quote_unsigned(
            &next_sqrt_price,
            &sqrt_price,
            &curve[0].liquidity,
            false,
        )?;
    }
    Ok(total_output)
}

fn bags_dbc_swap_amount_from_quote_to_base(
    current_sqrt_price: &BigUint,
    curve: &[BagsCurvePoint],
    amount_in: &BigUint,
) -> Result<BigUint, String> {
    if amount_in.is_zero() {
        return Ok(BigUint::ZERO);
    }
    let mut total_output = BigUint::ZERO;
    let mut sqrt_price = current_sqrt_price.clone();
    let mut amount_left = amount_in.clone();
    for point in curve {
        if point.sqrt_price.is_zero() || point.liquidity.is_zero() {
            break;
        }
        if point.sqrt_price > sqrt_price {
            let max_amount_in = bags_get_delta_amount_quote_unsigned(
                &sqrt_price,
                &point.sqrt_price,
                &point.liquidity,
                true,
            )?;
            if amount_left < max_amount_in {
                let next_sqrt_price =
                    bags_get_next_sqrt_price_from_input(&sqrt_price, &point.liquidity, &amount_left)?;
                total_output += bags_get_delta_amount_base_unsigned(
                    &sqrt_price,
                    &next_sqrt_price,
                    &point.liquidity,
                    false,
                )?;
                amount_left = BigUint::ZERO;
                break;
            }
            total_output += bags_get_delta_amount_base_unsigned(
                &sqrt_price,
                &point.sqrt_price,
                &point.liquidity,
                false,
            )?;
            sqrt_price = point.sqrt_price.clone();
            amount_left = big_sub(&amount_left, &max_amount_in, "remaining quote input")?;
        }
    }
    if !amount_left.is_zero() {
        return Err("Not enough liquidity to process the entire amount".to_string());
    }
    Ok(total_output)
}

fn bags_get_next_sqrt_price_from_amount_base_rounding_up(
    sqrt_price: &BigUint,
    liquidity: &BigUint,
    amount_in: &BigUint,
) -> Result<BigUint, String> {
    if amount_in.is_zero() {
        return Ok(sqrt_price.clone());
    }
    let denominator = liquidity + (amount_in * sqrt_price);
    Ok(big_div_rounding(liquidity * sqrt_price, &denominator, true))
}

fn bags_dbc_swap_quote_exact_in(
    pool: &DecodedDbcVirtualPool,
    config: &DecodedDbcPoolConfig,
    swap_base_for_quote: bool,
    amount_in: u64,
    slippage_bps: u64,
    current_point: u64,
) -> Result<(u64, u64), String> {
    let trade_direction = if swap_base_for_quote { 0u8 } else { 1u8 };
    let fees_on_input = trade_direction == 1 && config.collect_fee_mode == 0;
    let amount_for_swap = if fees_on_input {
        bags_dbc_fee_on_amount(amount_in, pool, config, current_point, trade_direction)?
    } else {
        amount_in
    };
    let raw_output = if swap_base_for_quote {
        bags_dbc_swap_amount_from_base_to_quote(
            &biguint_from_u128(pool.sqrt_price),
            &config.curve,
            &BigUint::from(amount_for_swap),
        )?
    } else {
        bags_dbc_swap_amount_from_quote_to_base(
            &biguint_from_u128(pool.sqrt_price),
            &config.curve,
            &BigUint::from(amount_for_swap),
        )?
    };
    let output = if fees_on_input {
        raw_output
    } else {
        let fee_numerator = bags_dbc_trade_fee_numerator_for_amount(
            pool,
            config,
            current_point,
            trade_direction,
            raw_output.to_u64(),
        );
        bags_get_fee_amount_excluded(&raw_output, fee_numerator)
    };
    let output_u64 = output
        .to_u64()
        .ok_or_else(|| "Bags follow quote output overflowed u64.".to_string())?;
    Ok((output_u64, helper_slippage_minimum_amount(output_u64, slippage_bps)))
}

fn cpamm_get_fee_mode(collect_fee_mode: u8, b_to_a: bool) -> bool {
    b_to_a && collect_fee_mode == 1
}

fn cpamm_get_total_fee_on_amount(amount: &BigUint, fee_numerator: &BigUint) -> BigUint {
    big_div_rounding(
        amount * fee_numerator,
        &BigUint::from(DBC_FEE_DENOMINATOR),
        true,
    )
}

fn cpamm_get_next_sqrt_price(
    amount: &BigUint,
    sqrt_price: &BigUint,
    liquidity: &BigUint,
    a_to_b: bool,
) -> Result<BigUint, String> {
    if a_to_b {
        let denominator = liquidity + (amount * sqrt_price);
        Ok(big_div_rounding(liquidity * sqrt_price, &denominator, true))
    } else {
        Ok(sqrt_price + ((amount << (CPAMM_SCALE_OFFSET * 2)) / liquidity))
    }
}

fn cpamm_get_amount_a_from_liquidity_delta(
    liquidity: &BigUint,
    current_sqrt_price: &BigUint,
    max_sqrt_price: &BigUint,
    round_up: bool,
) -> Result<BigUint, String> {
    let denominator = current_sqrt_price * max_sqrt_price;
    Ok(big_div_rounding(
        liquidity * big_sub(max_sqrt_price, current_sqrt_price, "cpamm amount a delta")?,
        &denominator,
        round_up,
    ))
}

fn cpamm_get_amount_b_from_liquidity_delta(
    liquidity: &BigUint,
    current_sqrt_price: &BigUint,
    min_sqrt_price: &BigUint,
    round_up: bool,
) -> Result<BigUint, String> {
    let result = liquidity * big_sub(current_sqrt_price, min_sqrt_price, "cpamm amount b delta")?;
    let denominator = BigUint::from(1u8) << (CPAMM_SCALE_OFFSET * 2);
    Ok(big_div_rounding(result, &denominator, round_up))
}

fn cpamm_trade_fee_numerator(pool: &DecodedDammPool, current_point: u64) -> BigUint {
    let base = &pool.pool_fees.base_fee;
    let mut fee_numerator = BigUint::from(base.cliff_fee_numerator);
    if base.period_frequency > 0 && current_point >= pool.activation_point {
        let period = ((current_point - pool.activation_point) / base.period_frequency)
            .min(u64::from(base.number_of_period));
        if base.fee_scheduler_mode == 0 {
            fee_numerator = BigUint::from(base.cliff_fee_numerator.saturating_sub(
                period.saturating_mul(base.reduction_factor),
            ));
        } else {
            let one = BigUint::from(1u8) << CPAMM_SCALE_OFFSET;
            let reduction =
                (BigUint::from(base.reduction_factor) << CPAMM_SCALE_OFFSET) / BigUint::from(CPAMM_BASIS_POINT_MAX);
            let decay_base = if reduction >= one { BigUint::ZERO } else { &one - reduction };
            let decay = big_pow_q64(&decay_base, period);
            fee_numerator = (BigUint::from(base.cliff_fee_numerator) * decay) >> CPAMM_SCALE_OFFSET;
        }
    }
    let dynamic = &pool.pool_fees.dynamic_fee;
    if dynamic.initialized && dynamic.variable_fee_control > 0 {
        let dynamic_fee = BigUint::from(dynamic.variable_fee_control)
            * BigUint::from(dynamic.volatility_accumulator)
            * BigUint::from(dynamic.bin_step)
            * BigUint::from(dynamic.volatility_accumulator)
            * BigUint::from(dynamic.bin_step);
        fee_numerator +=
            (dynamic_fee + BigUint::from(99_999_999_999u64)) / BigUint::from(100_000_000_000u64);
    }
    fee_numerator.min(BigUint::from(CPAMM_MAX_FEE_NUMERATOR))
}

fn cpamm_swap_amount_out(
    in_amount: &BigUint,
    input_token_mint: &Pubkey,
    pool: &DecodedDammPool,
    current_point: u64,
) -> Result<BigUint, String> {
    let a_to_b = pool.token_a_mint == *input_token_mint;
    let b_to_a = !a_to_b;
    let trade_fee_numerator = cpamm_trade_fee_numerator(pool, current_point);
    let fee_on_input = cpamm_get_fee_mode(pool.collect_fee_mode, b_to_a);
    let actual_in_amount = if fee_on_input {
        big_sub(
            in_amount,
            &cpamm_get_total_fee_on_amount(in_amount, &trade_fee_numerator),
            "cpamm input fee",
        )?
    } else {
        in_amount.clone()
    };
    let sqrt_price = biguint_from_u128(pool.sqrt_price);
    let liquidity = biguint_from_u128(pool.liquidity);
    let next_sqrt_price = cpamm_get_next_sqrt_price(&actual_in_amount, &sqrt_price, &liquidity, a_to_b)?;
    let raw_out = if a_to_b {
        cpamm_get_amount_b_from_liquidity_delta(&liquidity, &sqrt_price, &next_sqrt_price, false)?
    } else {
        cpamm_get_amount_a_from_liquidity_delta(&liquidity, &sqrt_price, &next_sqrt_price, false)?
    };
    if fee_on_input {
        Ok(raw_out)
    } else {
        big_sub(
            &raw_out,
            &cpamm_get_total_fee_on_amount(&raw_out, &trade_fee_numerator),
            "cpamm output fee",
        )
    }
}

async fn load_canonical_dbc_market(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<Option<(Pubkey, DecodedDbcVirtualPool, DecodedDbcPoolConfig)>, String> {
    let cached = normalize_cached_bags_launch_hints(bags_launch);
    let Some((pool_address, pool_bytes)) = rpc_fetch_first_dbc_pool_by_mint(rpc_url, mint, commitment).await?
    else {
        return Ok(None);
    };
    if let Some(expected_pool) = cached.pre_migration_dbc_pool_address {
        if pool_address != expected_pool {
            return Ok(None);
        }
    }
    let pool = decode_dbc_virtual_pool(&pool_bytes)?;
    if pool.base_mint != *mint {
        return Ok(None);
    }
    if let Some(expected_config) = cached.config_key {
        if pool.config != expected_config {
            return Ok(None);
        }
    }
    let config_key = cached.config_key.unwrap_or(pool.config);
    let Some(config_bytes) =
        rpc_fetch_account_data(rpc_url, &config_key, commitment, "dbc-config").await?
    else {
        return Ok(None);
    };
    let config = decode_dbc_pool_config(&config_bytes)?;
    if config.quote_mint != bags_native_mint_pubkey()? {
        return Ok(None);
    }
    let derived_pool = derive_dbc_pool_address(&config.quote_mint, mint, &config_key)?;
    if let Some(expected_pool) = cached.pre_migration_dbc_pool_address {
        if derived_pool != expected_pool {
            return Ok(None);
        }
    }
    if derived_pool != pool_address {
        return Ok(None);
    }
    Ok(Some((pool_address, pool, config)))
}

async fn native_fetch_local_dbc_market_snapshot(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<Option<BagsMarketSnapshot>, String> {
    let Some((_pool_address, pool, config)) =
        load_canonical_dbc_market(rpc_url, mint, commitment, bags_launch).await?
    else {
        return Ok(None);
    };
    if is_completed_dbc_pool(&pool, &config) {
        return Ok(None);
    }
    let supply = fetch_bags_token_supply_value(rpc_url, mint, commitment).await?;
    let supply_amount = BigUint::parse_bytes(supply.amount.trim().as_bytes(), 10)
        .ok_or_else(|| format!("Invalid Bags token supply amount for {mint}: {}", supply.amount))?;
    let price_quote_amount = BigUint::from(10u64).pow(supply.decimals);
    let current_point = current_point_for_dbc_config(rpc_url, &config, commitment).await?;
    let raw_out = bags_dbc_swap_amount_from_base_to_quote(
        &biguint_from_u128(pool.sqrt_price),
        &config.curve,
        &price_quote_amount,
    )?;
    let out_after_fee = bags_get_fee_amount_excluded(
        &raw_out,
        bags_dbc_trade_fee_numerator_for_amount(
            &pool,
            &config,
            current_point,
            0,
            raw_out.to_u64(),
        ),
    );
    let market_cap = if price_quote_amount.is_zero() {
        BigUint::ZERO
    } else {
        (supply_amount.clone() * &out_after_fee) / &price_quote_amount
    };
    Ok(Some(BagsMarketSnapshot {
        mint: mint.to_string(),
        creator: pool.creator.to_string(),
        virtualTokenReserves: pool.base_reserve.to_string(),
        virtualSolReserves: pool.quote_reserve.to_string(),
        realTokenReserves: pool.base_reserve.to_string(),
        realSolReserves: pool.quote_reserve.to_string(),
        tokenTotalSupply: supply_amount.to_string(),
        complete: false,
        marketCapLamports: market_cap.to_string(),
        marketCapSol: format_biguint_decimal(&market_cap, 9, 6)?,
        quoteAsset: "sol".to_string(),
        quoteAssetLabel: "SOL".to_string(),
    }))
}

async fn native_fetch_local_damm_market_snapshot(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<Option<BagsMarketSnapshot>, String> {
    let cached = normalize_cached_bags_launch_hints(bags_launch);
    let Some((pool_address, pool, config)) =
        load_canonical_dbc_market(rpc_url, mint, commitment, bags_launch).await?
    else {
        return Ok(None);
    };
    if !pool.is_migrated {
        return Ok(None);
    }
    let resolved = if let Some(resolved) = resolve_cached_damm_pool_address(mint, &cached)? {
        Some(resolved)
    } else if let Some(pool_address) = derive_canonical_damm_pool_address(mint, &config)? {
        let config_address = if config.migration_fee_option <= 6 {
            Some(
                Pubkey::from_str(DAMM_V2_MIGRATION_FEE_ADDRESS[config.migration_fee_option as usize])
                    .map_err(|error| format!("Invalid DAMM migration fee address: {error}"))?,
            )
        } else {
            None
        };
        Some((pool_address, config_address))
    } else {
        None
    };
    let Some((resolved_pool_address, _resolved_config_address)) = resolved else {
        return Ok(None);
    };
    let Some(pool_bytes) =
        rpc_fetch_account_data(rpc_url, &resolved_pool_address, commitment, "damm-v2-pool").await?
    else {
        return Ok(None);
    };
    let damm_pool = decode_damm_pool(&pool_bytes)?;
    let supply = fetch_bags_token_supply_value(rpc_url, mint, commitment).await?;
    let supply_amount = BigUint::parse_bytes(supply.amount.trim().as_bytes(), 10)
        .ok_or_else(|| format!("Invalid Bags token supply amount for {mint}: {}", supply.amount))?;
    let price_quote_amount = BigUint::from(10u64).pow(supply.decimals);
    let (current_slot, current_time) = current_time_for_damm(rpc_url, commitment).await?;
    let current_point = if damm_pool.activation_type == 0 {
        current_slot
    } else {
        current_time
    };
    let out_amount = cpamm_swap_amount_out(&price_quote_amount, mint, &damm_pool, current_point)?;
    let market_cap = if price_quote_amount.is_zero() {
        BigUint::ZERO
    } else {
        (supply_amount.clone() * &out_amount) / &price_quote_amount
    };
    let token_a_reserve =
        BigUint::from(fetch_bags_token_account_amount(rpc_url, &damm_pool.token_a_vault, commitment).await?);
    let token_b_reserve =
        BigUint::from(fetch_bags_token_account_amount(rpc_url, &damm_pool.token_b_vault, commitment).await?);
    let is_token_a_base = damm_pool.token_a_mint == *mint;
    let (real_token_reserves, real_sol_reserves) = if is_token_a_base {
        (token_a_reserve, token_b_reserve)
    } else {
        (token_b_reserve, token_a_reserve)
    };
    let _ = pool_address;
    Ok(Some(BagsMarketSnapshot {
        mint: mint.to_string(),
        creator: damm_pool.creator.to_string(),
        virtualTokenReserves: "0".to_string(),
        virtualSolReserves: "0".to_string(),
        realTokenReserves: real_token_reserves.to_string(),
        realSolReserves: real_sol_reserves.to_string(),
        tokenTotalSupply: supply_amount.to_string(),
        complete: true,
        marketCapLamports: market_cap.to_string(),
        marketCapSol: format_biguint_decimal(&market_cap, 9, 6)?,
        quoteAsset: "sol".to_string(),
        quoteAssetLabel: "SOL".to_string(),
    }))
}

async fn native_fetch_bags_market_snapshot(
    rpc_url: &str,
    mint: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<BagsMarketSnapshot, String> {
    if rpc_url.trim().is_empty() {
        return Err("SOLANA_RPC_URL is required for Bagsapp integration.".to_string());
    }
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bags mint address: {error}"))?;
    if let Some(snapshot) =
        native_fetch_local_dbc_market_snapshot(rpc_url, &mint_pubkey, "processed", bags_launch).await?
    {
        return Ok(snapshot);
    }
    if let Some(snapshot) =
        native_fetch_local_damm_market_snapshot(rpc_url, &mint_pubkey, "processed", bags_launch).await?
    {
        return Ok(snapshot);
    }
    Err(format!("No canonical Bags market snapshot found for {mint}."))
}

async fn detect_local_canonical_import_market(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
) -> Result<Option<NativeBagsImportMarket>, String> {
    let Some((pool_address, pool, config)) =
        load_canonical_dbc_market(rpc_url, mint, commitment, None).await?
    else {
        return Ok(None);
    };
    if !pool.is_migrated && !is_completed_dbc_pool(&pool, &config) {
        return Ok(Some(NativeBagsImportMarket {
            mode: bags_mode_from_fee_values(
                config.creator_trading_fee_percentage,
                config.creator_migration_fee_percentage,
            ),
            market_key: pool_address.to_string(),
            config_key: pool.config.to_string(),
            venue: "Meteora Dynamic Bonding Curve".to_string(),
            detection_source: "bags-state+rpc-dbc".to_string(),
            notes: vec![
                "Recovered canonical pre-migration DBC market from RPC without Bags trade quotes."
                    .to_string(),
            ],
        }));
    }
    if pool.is_migrated {
        let Some(damm_pool) = derive_canonical_damm_pool_address(mint, &config)? else {
            return Ok(None);
        };
        if rpc_fetch_account_data(rpc_url, &damm_pool, commitment, "damm-v2-pool")
            .await?
            .is_none()
        {
            return Ok(None);
        }
        let mut notes =
            vec!["Recovered canonical post-migration DAMM v2 market from RPC without Bags trade quotes."
                .to_string()];
        let family = expected_migration_family_from_config(&config);
        if !family.is_empty() {
            notes.push(format!("Resolved migration family from DBC config: {family}."));
        }
        return Ok(Some(NativeBagsImportMarket {
            mode: bags_mode_from_fee_values(
                config.creator_trading_fee_percentage,
                config.creator_migration_fee_percentage,
            ),
            market_key: damm_pool.to_string(),
            config_key: pool.config.to_string(),
            venue: "Meteora DAMM v2".to_string(),
            detection_source: "bags-state+rpc-damm-v2".to_string(),
            notes,
        }));
    }
    Ok(None)
}

async fn fetch_bags_token_creators(
    mint: &str,
    api_key: &str,
) -> Result<Vec<BagsTokenLaunchCreator>, String> {
    let response = bags_fee_http_client()
        .get(format!("{}/token-launch/creator/v3", bags_api_base_url()))
        .header("x-api-key", api_key)
        .query(&[("tokenMint", mint)])
        .send()
        .await
        .map_err(|error| format!("Failed to query Bags creator routes: {error}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let message = match serde_json::from_str::<Value>(&body) {
            Ok(value) => value
                .get("error")
                .and_then(Value::as_str)
                .or_else(|| value.get("message").and_then(Value::as_str))
                .map(|value| value.to_string())
                .unwrap_or_else(|| format!("Request failed with status {status}")),
            Err(_) => {
                let trimmed = body.trim();
                if trimmed.is_empty() {
                    format!("Request failed with status {status}")
                } else {
                    trimmed.to_string()
                }
            }
        };
        return Err(message);
    }
    let payload: BagsApiEnvelope<Vec<BagsTokenLaunchCreator>> = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Bags creator routes response: {error}"))?;
    if payload.success {
        Ok(payload.response.unwrap_or_default())
    } else {
        Err(if payload.error.trim().is_empty() {
            "Unknown Bags creator route error.".to_string()
        } else {
            payload.error
        })
    }
}

async fn native_detect_bags_import_context(
    rpc_url: &str,
    mint: &str,
) -> Result<Option<BagsImportContext>, String> {
    if rpc_url.trim().is_empty() {
        return Err("SOLANA_RPC_URL is required for Bagsapp integration.".to_string());
    }
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bags mint address: {error}"))?;
    let credentials = read_active_bags_credentials();
    let mut notes = Vec::new();
    let creators = if credentials.api_key.trim().is_empty() {
        notes.push("Bags creator routes were skipped because no Bags API key is configured.".to_string());
        Vec::new()
    } else {
        match fetch_bags_token_creators(mint, credentials.api_key.trim()).await {
            Ok(creators) => creators,
            Err(error) => {
                notes.push(format!(
                    "Bags creator routes could not be recovered from Bags state: {error}."
                ));
                Vec::new()
            }
        }
    };
    let creator_wallet = creators
        .iter()
        .find(|entry| entry.isCreator)
        .or_else(|| creators.first())
        .map(|entry| entry.wallet.trim().to_string())
        .unwrap_or_default();
    let mut fee_recipients = Vec::new();
    for entry in creators.iter().filter(|entry| entry.royaltyBps > 0) {
        let wallet = entry.wallet.trim().to_string();
        if !creator_wallet.is_empty() && wallet == creator_wallet {
            continue;
        }
        let provider = entry
            .provider
            .clone()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let provider_username = entry
            .providerUsername
            .clone()
            .or_else(|| entry.githubUsername.clone())
            .or_else(|| entry.twitterUsername.clone())
            .unwrap_or_default()
            .trim()
            .trim_start_matches('@')
            .to_string();
        if !provider.is_empty()
            && provider != "github"
            && provider != "solana"
            && provider != "wallet"
            && !provider_username.is_empty()
        {
            notes.push(format!(
                "Recovered {provider} fee route @{provider_username} as wallet {}.",
                wallet
            ));
        }
        let is_supported_social = matches!(provider.as_str(), "github" | "twitter" | "x" | "kick" | "tiktok");
        fee_recipients.push(if is_supported_social && !provider_username.is_empty() {
            BagsImportRecipient {
                r#type: provider.clone(),
                githubUsername: provider_username.clone(),
                address: String::new(),
                shareBps: entry.royaltyBps,
                sourceProvider: provider,
                sourceUsername: provider_username,
            }
        } else {
            BagsImportRecipient {
                r#type: "wallet".to_string(),
                githubUsername: String::new(),
                address: wallet,
                shareBps: entry.royaltyBps,
                sourceProvider: provider,
                sourceUsername: provider_username,
            }
        });
    }
    let local_market =
        match detect_local_canonical_import_market(rpc_url, &mint_pubkey, "processed").await {
            Ok(value) => value,
            Err(_) => None,
        };
    let (mode, market_key, config_key, venue, detection_source) = if let Some(local_market) = local_market {
        notes.extend(local_market.notes.clone());
        (
            local_market.mode,
            local_market.market_key,
            local_market.config_key,
            local_market.venue,
            local_market.detection_source,
        )
    } else {
        notes.push("Canonical Bags market could not be recovered from RPC-only state.".to_string());
        (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            "bags-state".to_string(),
        )
    };
    if mode.is_empty() {
        notes.push("Bags mode could not be recovered confidently from current market state.".to_string());
    }
    if creators.is_empty() && market_key.is_empty() {
        return Ok(None);
    }
    Ok(Some(BagsImportContext {
        launchpad: "bagsapp".to_string(),
        mode,
        quoteAsset: "sol".to_string(),
        creator: creator_wallet,
        marketKey: market_key,
        configKey: config_key,
        venue,
        detectionSource: detection_source,
        feeRecipients: fee_recipients,
        notes,
    }))
}

fn read_bags_credentials_file(path: PathBuf) -> BagsStoredCredentials {
    let raw = fs::read_to_string(path).unwrap_or_default();
    if raw.trim().is_empty() {
        BagsStoredCredentials::default()
    } else {
        serde_json::from_str::<BagsStoredCredentials>(&raw).unwrap_or_default()
    }
}

fn read_active_bags_credentials() -> BagsStoredCredentials {
    let persisted = read_bags_credentials_file(paths::bags_credentials_path());
    let session = read_bags_credentials_file(paths::bags_session_path());
    BagsStoredCredentials {
        api_key: if session.api_key.trim().is_empty() {
            if persisted.api_key.trim().is_empty() {
                std::env::var("BAGS_API_KEY")
                    .ok()
                    .map(|value| value.trim().to_string())
                    .unwrap_or_default()
            } else {
                persisted.api_key
            }
        } else {
            session.api_key
        },
    }
}

fn bags_launch_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("bags launch client")
    })
}

fn require_bags_api_key() -> Result<String, String> {
    let api_key = read_active_bags_credentials().api_key.trim().to_string();
    if api_key.is_empty() {
        Err("BAGS_API_KEY is required for Bagsapp integration.".to_string())
    } else {
        Ok(api_key)
    }
}

fn default_bags_wallet_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(DEFAULT_BAGS_WALLET)
        .map_err(|error| format!("Invalid default Bags partner wallet: {error}"))
}

fn default_bags_config_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(DEFAULT_BAGS_CONFIG)
        .map_err(|error| format!("Invalid default Bags partner config: {error}"))
}

fn bags_fee_share_v2_program_pubkey() -> Result<Pubkey, String> {
    Pubkey::from_str(BAGS_FEE_SHARE_V2_PROGRAM_ID)
        .map_err(|error| format!("Invalid Bags fee-share v2 program id: {error}"))
}

fn bags_config_type_for_mode(mode: &str) -> &'static str {
    match mode.trim().to_ascii_lowercase().as_str() {
        "bags-025-1" => BAGS_CONFIG_TYPE_025_PRE_1_POST,
        "bags-1-025" => BAGS_CONFIG_TYPE_1_PRE_025_POST,
        _ => BAGS_CONFIG_TYPE_DEFAULT,
    }
}

fn derive_bags_fee_share_v2_partner_config_pda(partner: &Pubkey) -> Result<Pubkey, String> {
    let program_id = bags_fee_share_v2_program_pubkey()?;
    let (partner_config, _) =
        Pubkey::find_program_address(&[b"partner_config", &partner.to_bytes()], &program_id);
    Ok(partner_config)
}

fn decode_bags_partner_config_partner(raw: &[u8]) -> Result<Pubkey, String> {
    const PARTNER_OFFSET: usize = 8 + 40;
    const PARTNER_END: usize = PARTNER_OFFSET + 32;
    if raw.len() < PARTNER_END {
        return Err("Bags partner config account was shorter than expected.".to_string());
    }
    let bytes: [u8; 32] = raw[PARTNER_OFFSET..PARTNER_END]
        .try_into()
        .map_err(|_| "Bags partner config partner key was malformed.".to_string())?;
    Ok(Pubkey::new_from_array(bytes))
}

async fn get_bags_partner_launch_params(
    rpc_url: &str,
    commitment: &str,
) -> Result<Option<(Pubkey, Pubkey)>, String> {
    let default_wallet = default_bags_wallet_pubkey()?;
    let default_config = default_bags_config_pubkey()?;
    let partner_config_address = derive_bags_fee_share_v2_partner_config_pda(&default_wallet)?;
    let Some(raw) = rpc_fetch_account_data(
        rpc_url,
        &partner_config_address,
        commitment,
        "bags-partner-config",
    )
    .await?
    else {
        return Ok(None);
    };
    let partner = decode_bags_partner_config_partner(&raw)?;
    if partner != default_wallet {
        return Err("Bags partner config resolved to an unexpected partner wallet.".to_string());
    }
    Ok(Some((default_wallet, default_config)))
}

fn load_bags_launch_image(path_value: &str) -> Result<(Vec<u8>, String, &'static str), String> {
    let trimmed = path_value.trim();
    if trimmed.is_empty() {
        return Err("Bags launch requires a readable local image file.".to_string());
    }
    let absolute_path = PathBuf::from(trimmed);
    let metadata = fs::metadata(&absolute_path)
        .map_err(|_| "Bags launch requires a readable local image file.".to_string())?;
    if !metadata.is_file() {
        return Err("Bags launch requires a readable local image file.".to_string());
    }
    let bytes = fs::read(&absolute_path)
        .map_err(|_| "Bags launch requires a readable local image file.".to_string())?;
    let filename = absolute_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("token-image.png")
        .to_string();
    let content_type = match absolute_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "avif" => "image/avif",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "image/png",
    };
    Ok((bytes, filename, content_type))
}

async fn upload_bags_token_info_and_metadata(
    api_key: &str,
    config: &NormalizedConfig,
) -> Result<BagsTokenInfoResponse, String> {
    let (image_bytes, filename, content_type) = load_bags_launch_image(&config.imageLocalPath)?;
    let image_part = Part::bytes(image_bytes)
        .file_name(filename)
        .mime_str(content_type)
        .map_err(|error| format!("Failed to prepare Bags launch image upload: {error}"))?;
    let mut form = Form::new()
        .part("image", image_part)
        .text("name", config.token.name.clone())
        .text("symbol", config.token.symbol.trim().to_string())
        .text("description", config.token.description.clone());
    if !config.token.telegram.trim().is_empty() {
        form = form.text("telegram", config.token.telegram.clone());
    }
    if !config.token.website.trim().is_empty() {
        form = form.text("website", config.token.website.clone());
    }
    if !config.token.twitter.trim().is_empty() {
        form = form.text("twitter", config.token.twitter.clone());
    }
    let response = bags_launch_http_client()
        .post(format!("{}/token-launch/create-token-info", bags_api_base_url()))
        .header("x-api-key", api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|error| format!("Failed to upload Bags token metadata: {error}"))?;
    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|error| format!("Failed to read Bags token metadata upload response: {error}"))?;
    let payload: BagsApiEnvelope<Value> = serde_json::from_str(&response_text)
        .map_err(|error| format!("Failed to parse Bags token metadata upload response: {error}"))?;
    if !status.is_success() || !payload.success {
        return Err(summarize_bags_api_failure(
            "Failed to upload Bags token metadata",
            status,
            &payload.error,
            payload.response.as_ref(),
            &response_text,
        ));
    }
    payload
        .response
        .map(serde_json::from_value::<BagsTokenInfoResponse>)
        .transpose()
        .map_err(|error| format!("Failed to parse Bags token metadata payload: {error}"))?
        .ok_or_else(|| "Bags token metadata upload returned an empty response.".to_string())
}

async fn resolve_bags_fee_claimers(
    owner_public_key: &Pubkey,
    fee_sharing: &[NormalizedRecipient],
    rpc_url: &str,
) -> Result<Vec<(Pubkey, u16)>, String> {
    let owner_base58 = owner_public_key.to_string();
    let mut merged_claimers: HashMap<String, u64> = HashMap::new();
    let mut claimer_order = Vec::new();
    let mut allocated_non_owner_bps = 0u64;
    for row in fee_sharing {
        if row.shareBps <= 0 {
            continue;
        }
        let share_bps = u64::try_from(row.shareBps)
            .map_err(|_| "Bags fee-share rows exceed supported basis points.".to_string())?;
        let recipient_type = row
            .r#type
            .as_deref()
            .unwrap_or("wallet")
            .trim()
            .to_ascii_lowercase();
        let wallet = if recipient_type == "wallet" {
            Pubkey::from_str(row.address.trim()).map_err(|error| {
                format!(
                    "Unsupported Bags fee-share recipient address {}: {error}",
                    row.address.trim()
                )
            })?
        } else if ["github", "twitter", "x", "kick", "tiktok"].contains(&recipient_type.as_str()) {
            if let Some(cached_wallet) = parse_optional_pubkey(&row.address) {
                cached_wallet
            } else {
                let lookup = lookup_bags_fee_recipient(
                    rpc_url,
                    &recipient_type,
                    &row.githubUsername,
                    &row.githubUserId,
                )
                .await?;
                if lookup.wallet.trim().is_empty() {
                    return Err(if !lookup.error.trim().is_empty() {
                        lookup.error
                    } else if lookup.notFound {
                        format!(
                            "Failed to get launch wallet for {} user {}: not found",
                            lookup.provider,
                            lookup.lookupTarget
                        )
                    } else {
                        format!(
                            "Failed to get launch wallet for {} user {}",
                            lookup.provider,
                            lookup.lookupTarget
                        )
                    });
                }
                Pubkey::from_str(lookup.wallet.trim())
                    .map_err(|error| format!("Invalid Bags fee-share wallet returned by API: {error}"))?
            }
        } else {
            return Err(format!(
                "Unsupported Bags fee-share recipient type: {}",
                recipient_type
            ));
        };
        let wallet_base58 = wallet.to_string();
        if wallet_base58 == owner_base58 {
            continue;
        }
        allocated_non_owner_bps = allocated_non_owner_bps.saturating_add(share_bps);
        if !merged_claimers.contains_key(&wallet_base58) {
            claimer_order.push(wallet_base58.clone());
        }
        *merged_claimers.entry(wallet_base58).or_insert(0) += share_bps;
    }
    if allocated_non_owner_bps > 10_000 {
        return Err("Bags fee-share rows exceed 10000 total bps.".to_string());
    }
    let mut resolved = claimer_order
        .into_iter()
        .map(|address| {
            let user_bps = *merged_claimers
                .get(&address)
                .ok_or_else(|| format!("Missing merged Bags fee-share wallet {address}."))?;
            let pubkey = Pubkey::from_str(&address)
                .map_err(|error| format!("Invalid merged Bags fee-share wallet {address}: {error}"))?;
            let user_bps = u16::try_from(user_bps)
                .map_err(|_| format!("Bags fee-share bps overflowed for wallet {address}."))?;
            Ok((pubkey, user_bps))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let creator_bps = 10_000u64.saturating_sub(allocated_non_owner_bps);
    if creator_bps > 0 || resolved.is_empty() {
        resolved.insert(
            0,
            (
                *owner_public_key,
                if creator_bps > 0 {
                    u16::try_from(creator_bps)
                        .map_err(|_| "Creator Bags fee-share bps overflowed.".to_string())?
                } else {
                    10_000u16
                },
            ),
        );
    }
    Ok(resolved)
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

fn jito_tip_percentile_value<'a>(sample: &'a Value, percentile: &str) -> Option<&'a Value> {
    match percentile {
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
    }
}

fn extract_jito_tip_floor_lamports(payload: &Value, percentile: &str) -> Option<u64> {
    if let Some(value) = normalize_estimate_to_lamports(jito_tip_percentile_value(payload, percentile))
    {
        return Some(value);
    }
    if let Some(value) = payload
        .get("params")
        .and_then(|params| params.get("result"))
        .and_then(|result| extract_jito_tip_floor_lamports(result, percentile))
    {
        return Some(value);
    }
    for key in ["data", "result", "value"] {
        if let Some(value) = payload
            .get(key)
            .and_then(|child| extract_jito_tip_floor_lamports(child, percentile))
        {
            return Some(value);
        }
    }
    payload.as_array().and_then(|entries| {
        entries
            .iter()
            .find_map(|entry| extract_jito_tip_floor_lamports(entry, percentile))
    })
}

async fn fetch_bags_engine_fee_estimate(
    rpc_url: &str,
    requested_tip_lamports: u64,
    setup_jito_tip_cap_lamports: u64,
    setup_jito_tip_min_lamports: u64,
    percentile: &str,
) -> Result<BagsFeeEstimateSnapshot, String> {
    if let Some(snapshot) = get_cached_bags_fee_estimate(
        rpc_url,
        requested_tip_lamports,
        setup_jito_tip_cap_lamports,
        setup_jito_tip_min_lamports,
        percentile,
    ) {
        return Ok(snapshot);
    }
    let response = bags_fee_http_client()
        .get("https://bundles.jito.wtf/api/v1/bundles/tip_floor")
        .send()
        .await
        .map_err(|error| format!("Jito tip floor request failed: {error}"))?;
    let payload = response
        .json::<Value>()
        .await
        .map_err(|error| format!("Failed to decode Jito tip floor: {error}"))?;
    let estimated_jito_tip_lamports =
        extract_jito_tip_floor_lamports(&payload, percentile).unwrap_or_default();
    let mut warnings = Vec::new();
    let mut setup_jito_tip_lamports = estimated_jito_tip_lamports;
    let mut setup_jito_tip_source = "engine-jito-tip-floor".to_string();
    if setup_jito_tip_lamports == 0 && requested_tip_lamports > 0 {
        setup_jito_tip_lamports = requested_tip_lamports;
        setup_jito_tip_source = "user-requested-fallback".to_string();
    }
    if setup_jito_tip_lamports > 0 {
        setup_jito_tip_lamports = setup_jito_tip_lamports.max(setup_jito_tip_min_lamports);
    }
    if setup_jito_tip_cap_lamports > 0 {
        setup_jito_tip_lamports = setup_jito_tip_lamports.min(setup_jito_tip_cap_lamports);
    }
    if setup_jito_tip_lamports == 0 {
        setup_jito_tip_source = "none".to_string();
        warnings.push(format!(
            "Engine Jito tip floor did not include a usable {} estimate.",
            percentile
        ));
    }
    let snapshot = BagsFeeEstimateSnapshot {
        helius: json!({
            "source": "engine-fee-market",
            "launchPriorityLamports": Value::Null,
        }),
        jito: json!({
            "source": "engine-jito-tip-floor",
            "percentile": percentile,
            "tipLamports": estimated_jito_tip_lamports,
            "raw": payload,
        }),
        setupJitoTipLamports: setup_jito_tip_lamports,
        setupJitoTipSource: setup_jito_tip_source,
        setupJitoTipPercentile: percentile.to_string(),
        setupJitoTipCapLamports: setup_jito_tip_cap_lamports,
        setupJitoTipMinLamports: setup_jito_tip_min_lamports,
        warnings,
    };
    cache_bags_fee_estimate(
        rpc_url,
        requested_tip_lamports,
        setup_jito_tip_cap_lamports,
        setup_jito_tip_min_lamports,
        percentile,
        &snapshot,
    );
    Ok(snapshot)
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

fn follow_tip_lamports_for_provider(
    provider: &str,
    tip_sol: &str,
    label: &str,
) -> Result<u64, String> {
    let tip_lamports = parse_decimal_u64(tip_sol, 9, label)?;
    if provider.trim().eq_ignore_ascii_case("hellomoon") {
        if tip_sol.trim().is_empty() {
            return Err(format!(
                "{label} cannot be empty when using Hello Moon for follow / snipe / auto-sell."
            ));
        }
        if tip_lamports < 1_000_000 {
            return Err(format!(
                "{label} must be at least 0.001 SOL when using Hello Moon for follow / snipe / auto-sell."
            ));
        }
    }
    Ok(tip_lamports)
}

fn decode_secret_base64(secret: &[u8]) -> String {
    format!("base64:{}", BASE64.encode(secret))
}

fn helper_bags_launch_metadata(metadata: Option<&BagsLaunchMetadata>) -> Value {
    match metadata {
        Some(metadata) => json!({
            "configKey": metadata.configKey,
            "migrationFeeOption": metadata.migrationFeeOption,
            "expectedMigrationFamily": metadata.expectedMigrationFamily,
            "expectedDammConfigKey": metadata.expectedDammConfigKey,
            "expectedDammDerivationMode": metadata.expectedDammDerivationMode,
            "preMigrationDbcPoolAddress": metadata.preMigrationDbcPoolAddress,
        }),
        None => Value::Null,
    }
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

fn append_bags_fee_estimate_notes(report: &mut LaunchReport, estimate: &BagsFeeEstimateSnapshot) {
    if estimate.setupJitoTipLamports > 0 {
        report.execution.notes.push(format!(
            "Bags setup bundle tip policy: source={} | percentile={} | selected={} lamports | cap={} lamports.",
            if estimate.setupJitoTipSource.trim().is_empty() {
                "unknown"
            } else {
                estimate.setupJitoTipSource.trim()
            },
            if estimate.setupJitoTipPercentile.trim().is_empty() {
                DEFAULT_AUTO_FEE_JITO_TIP_PERCENTILE
            } else {
                estimate.setupJitoTipPercentile.trim()
            },
            estimate.setupJitoTipLamports,
            estimate.setupJitoTipCapLamports
        ));
    }
    for warning in &estimate.warnings {
        if !warning.trim().is_empty() {
            report
                .execution
                .notes
                .push(format!("Bags fee estimate note: {}", warning.trim()));
        }
    }
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
                feeSettings: crate::report::FeeSettings {
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

async fn create_bags_fee_share_config(
    api_key: &str,
    owner: &Pubkey,
    token_mint: &Pubkey,
    fee_claimers: &[(Pubkey, u16)],
    partner_params: Option<(Pubkey, Pubkey)>,
    bags_config_type: &str,
    tip_account: Option<&Pubkey>,
    tip_lamports: u64,
) -> Result<BagsFeeShareConfigResponse, String> {
    let total_bps: u64 = fee_claimers.iter().map(|(_, bps)| u64::from(*bps)).sum();
    if total_bps != 10_000 {
        return Err("Total BPS must be 10000".to_string());
    }
    if fee_claimers.len() > 100 {
        return Err("Total fee claimers must be less than 100".to_string());
    }
    if fee_claimers.len() > BAGS_FEE_SHARE_V2_MAX_CLAIMERS_NON_LUT {
        return Err(
            "Total fee claimers exceeds BAGS_FEE_SHARE_V2_MAX_CLAIMERS_NON_LUT; please provide an additional lookup tables."
                .to_string(),
        );
    }
    let mut payload = json!({
        "basisPointsArray": fee_claimers.iter().map(|(_, user_bps)| *user_bps).collect::<Vec<_>>(),
        "payer": owner.to_string(),
        "baseMint": token_mint.to_string(),
        "claimersArray": fee_claimers.iter().map(|(user, _)| user.to_string()).collect::<Vec<_>>(),
        "bagsConfigType": bags_config_type,
    });
    if let Some((partner, partner_config)) = partner_params {
        payload["partner"] = Value::String(partner.to_string());
        payload["partnerConfig"] = Value::String(partner_config.to_string());
    }
    if let Some(tip_wallet) = tip_account {
        if tip_lamports > 0 {
            payload["tipWallet"] = Value::String(tip_wallet.to_string());
            payload["tipLamports"] = Value::Number(tip_lamports.into());
        }
    }
    let response = bags_launch_http_client()
        .post(format!("{}/fee-share/config", bags_api_base_url()))
        .header("x-api-key", api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to create Bags fee-share config: {error}"))?;
    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|error| format!("Failed to read Bags fee-share config response: {error}"))?;
    let envelope: BagsApiEnvelope<Value> = serde_json::from_str(&response_text)
        .map_err(|error| format!("Failed to parse Bags fee-share config response: {error}"))?;
    if !status.is_success() || !envelope.success {
        return Err(summarize_bags_api_failure(
            "Failed to create Bags fee-share config",
            status,
            &envelope.error,
            envelope.response.as_ref(),
            &response_text,
        ));
    }
    let payload = envelope
        .response
        .map(serde_json::from_value::<BagsFeeShareConfigResponse>)
        .transpose()
        .map_err(|error| format!("Failed to parse Bags fee-share config payload: {error}"))?
        .ok_or_else(|| "Bags fee-share config response was empty.".to_string())?;
    if !payload.needsCreation {
        return Err("Config already exists".to_string());
    }
    Ok(payload)
}

async fn create_bags_launch_transaction_bytes(
    api_key: &str,
    metadata_uri: &str,
    token_mint: &Pubkey,
    owner: &Pubkey,
    initial_buy_lamports: u64,
    config_key: &Pubkey,
    tip_account: Option<&Pubkey>,
    tip_lamports: u64,
) -> Result<String, String> {
    let payload = build_bags_launch_transaction_payload(
        metadata_uri,
        token_mint,
        owner,
        initial_buy_lamports,
        config_key,
        tip_account,
        tip_lamports,
    );
    let response = bags_launch_http_client()
        .post(format!(
            "{}/token-launch/create-launch-transaction",
            bags_api_base_url()
        ))
        .header("x-api-key", api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Failed to create Bags launch transaction: {error}"))?;
    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|error| format!("Failed to read Bags launch transaction response: {error}"))?;
    let envelope: BagsApiEnvelope<Value> = serde_json::from_str(&response_text)
        .map_err(|error| format!("Failed to parse Bags launch transaction response: {error}"))?;
    if !status.is_success() || !envelope.success {
        return Err(summarize_bags_api_failure(
            "Failed to create Bags launch transaction",
            status,
            &envelope.error,
            envelope.response.as_ref(),
            &response_text,
        ));
    }
    envelope
        .response
        .and_then(|value| value.as_str().map(|text| text.to_string()))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Bags launch transaction response was empty.".to_string())
}

fn build_bags_launch_transaction_payload(
    metadata_uri: &str,
    token_mint: &Pubkey,
    owner: &Pubkey,
    initial_buy_lamports: u64,
    config_key: &Pubkey,
    tip_account: Option<&Pubkey>,
    tip_lamports: u64,
) -> Value {
    let mut payload = json!({
        "ipfs": metadata_uri,
        "tokenMint": token_mint.to_string(),
        "wallet": owner.to_string(),
        "initialBuyLamports": initial_buy_lamports,
        "configKey": config_key.to_string(),
    });
    if let Some(tip_wallet) = tip_account {
        if tip_lamports > 0 {
            payload["tipWallet"] = Value::String(tip_wallet.to_string());
            payload["tipLamports"] = Value::Number(tip_lamports.into());
        }
    }
    payload
}

fn decode_bags_versioned_transaction(encoded: &str) -> Result<VersionedTransaction, String> {
    let bytes = bs58::decode(encoded.trim())
        .into_vec()
        .map_err(|error| format!("Failed to decode Bags transaction payload: {error}"))?;
    bincode::deserialize::<VersionedTransaction>(&bytes)
        .map_err(|error| format!("Failed to deserialize Bags versioned transaction: {error}"))
}

fn helper_prepare_response_to_native_prepared(
    response: HelperLaunchResponse,
    include_send_phases: bool,
) -> NativePreparedBagsLaunch {
    let setup_bundles = if include_send_phases {
        response
            .setupBundles
            .iter()
            .map(|bundle| {
                bundle
                    .compiledTransactions
                    .iter()
                    .cloned()
                    .map(convert_compiled_transaction)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let setup_transactions = if include_send_phases {
        response
            .setupTransactions
            .iter()
            .cloned()
            .map(convert_compiled_transaction)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    NativePreparedBagsLaunch {
        mint: response.mint,
        launch_creator: response.launchCreator,
        config_key: response.configKey,
        metadata_uri: response.metadataUri,
        identity_label: response.identityLabel,
        migration_fee_option: response.migrationFeeOption,
        expected_migration_family: response.expectedMigrationFamily,
        expected_damm_config_key: response.expectedDammConfigKey,
        expected_damm_derivation_mode: response.expectedDammDerivationMode,
        pre_migration_dbc_pool_address: response.preMigrationDbcPoolAddress,
        compiled_transactions: response
            .compiledTransactions
            .into_iter()
            .map(convert_compiled_transaction)
            .collect(),
        setup_bundles,
        setup_transactions,
        timings: response.timings.unwrap_or(HelperPrepareLaunchTimings {
            prepareLaunchMs: None,
            feeRecipientResolveMs: None,
            metadataUploadMs: None,
        }),
    }
}

fn build_native_bags_artifacts_from_prepared(
    prepared: NativePreparedBagsLaunch,
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    built_at: String,
    rpc_url: &str,
    creator_public_key: String,
    config_path: Option<String>,
    include_send_phases: bool,
    fee_estimate: BagsFeeEstimateSnapshot,
    backend_summary: &str,
) -> Result<NativeBagsArtifacts, String> {
    let mut report = build_report(
        config,
        transport_plan,
        built_at,
        rpc_url.to_string(),
        creator_public_key,
        prepared.mint.clone(),
        None,
        config_path,
        vec![],
    );
    report.execution.notes.push(backend_summary.to_string());
    if let Some(backend) = launchpad_action_backend("bagsapp", "prepare-launch") {
        let rollout_state =
            launchpad_action_rollout_state("bagsapp", "prepare-launch").unwrap_or("unknown");
        report.execution.notes.push(format!(
            "Launchpad backend owner: {backend} ({rollout_state})."
        ));
    }
    append_bags_fee_estimate_notes(&mut report, &fee_estimate);
    if !prepared.config_key.trim().is_empty() {
        report.execution.notes.push(format!(
            "Bags fee-share config {} for this launch: {}",
            if include_send_phases {
                "prepared"
            } else {
                "created"
            },
            prepared.config_key
        ));
    }
    if include_send_phases {
        report.execution.notes.push(
            "Bags launch is sent in two tracked phases: fee-share config setup first, then the token creation transaction."
                .to_string(),
        );
    }
    if !prepared.identity_label.trim().is_empty() {
        report
            .execution
            .notes
            .push(format!("Identity: {}", prepared.identity_label.trim()));
    }
    if let Some(migration_fee_option) = prepared.migration_fee_option {
        report.execution.notes.push(format!(
            "Bags migration fee option recorded at launch setup: {}.",
            migration_fee_option
        ));
    }
    if !prepared.expected_migration_family.trim().is_empty() {
        report.execution.notes.push(format!(
            "Expected post-migration family: {}.",
            prepared.expected_migration_family.trim()
        ));
    }
    if !prepared.expected_damm_config_key.trim().is_empty() {
        report.execution.notes.push(format!(
            "Expected DAMM config key for migration: {}.",
            prepared.expected_damm_config_key.trim()
        ));
    }
    if !prepared.pre_migration_dbc_pool_address.trim().is_empty() {
        report.execution.notes.push(format!(
            "Deterministic pre-migration DBC pool address: {}.",
            prepared.pre_migration_dbc_pool_address.trim()
        ));
    }
    report.transactions = build_transaction_summaries(&prepared.compiled_transactions, config.tx.dumpBase64);
    let text = render_report(&report);
    let mut report = serde_json::to_value(report).map_err(|error| error.to_string())?;
    if !prepared.metadata_uri.trim().is_empty() {
        report["metadataUri"] = Value::String(prepared.metadata_uri.clone());
    }
    if let Some(migration_fee_option) = prepared.migration_fee_option {
        report["migrationFeeOption"] = Value::Number(migration_fee_option.into());
    }
    if !prepared.expected_migration_family.trim().is_empty() {
        report["expectedMigrationFamily"] =
            Value::String(prepared.expected_migration_family.clone());
    }
    if !prepared.expected_damm_config_key.trim().is_empty() {
        report["expectedDammConfigKey"] =
            Value::String(prepared.expected_damm_config_key.clone());
    }
    if !prepared.expected_damm_derivation_mode.trim().is_empty() {
        report["expectedDammDerivationMode"] =
            Value::String(prepared.expected_damm_derivation_mode.clone());
    }
    if !prepared.pre_migration_dbc_pool_address.trim().is_empty() {
        report["preMigrationDbcPoolAddress"] =
            Value::String(prepared.pre_migration_dbc_pool_address.clone());
    }
    report["bagsSetupFeeEstimate"] =
        serde_json::to_value(&fee_estimate).map_err(|error| error.to_string())?;
    Ok(NativeBagsArtifacts {
        compiled_transactions: prepared.compiled_transactions,
        report,
        text,
        compile_timings: NativeCompileTimings::default(),
        mint: prepared.mint,
        launch_creator: prepared.launch_creator,
        config_key: prepared.config_key,
        metadata_uri: prepared.metadata_uri,
        migration_fee_option: prepared.migration_fee_option,
        expected_migration_family: prepared.expected_migration_family,
        expected_damm_config_key: prepared.expected_damm_config_key,
        expected_damm_derivation_mode: prepared.expected_damm_derivation_mode,
        pre_migration_dbc_pool_address: prepared.pre_migration_dbc_pool_address,
        setup_bundles: prepared.setup_bundles,
        setup_transactions: prepared.setup_transactions,
        fee_estimate,
        prepare_launch_ms: prepared.timings.prepareLaunchMs,
        metadata_upload_ms: prepared.timings.metadataUploadMs,
        fee_recipient_resolve_ms: prepared.timings.feeRecipientResolveMs,
    })
}

struct LaunchMigrationSummary {
    migration_fee_option: Option<i64>,
    expected_migration_family: String,
    expected_damm_config_key: String,
    expected_damm_derivation_mode: String,
    pre_migration_dbc_pool_address: String,
}

async fn summarize_launch_migration_config(
    rpc_url: &str,
    mint: &Pubkey,
    config_key: &Pubkey,
    commitment: &str,
) -> Result<LaunchMigrationSummary, String> {
    let Some(config_bytes) =
        rpc_fetch_account_data(rpc_url, config_key, commitment, "launch-dbc-config").await?
    else {
        return Ok(LaunchMigrationSummary {
            migration_fee_option: None,
            expected_migration_family: String::new(),
            expected_damm_config_key: String::new(),
            expected_damm_derivation_mode: String::new(),
            pre_migration_dbc_pool_address: String::new(),
        });
    };
    let config = decode_dbc_pool_config(&config_bytes)?;
    let expected_damm_config_key = if config.migration_fee_option <= 6 {
        DAMM_V2_MIGRATION_FEE_ADDRESS
            .get(config.migration_fee_option as usize)
            .copied()
            .unwrap_or_default()
            .to_string()
    } else {
        String::new()
    };
    Ok(LaunchMigrationSummary {
        migration_fee_option: Some(i64::from(config.migration_fee_option)),
        expected_migration_family: expected_migration_family_from_config(&config),
        expected_damm_config_key,
        expected_damm_derivation_mode: if config.migration_fee_option == 6 {
            "customizable".to_string()
        } else {
            "config-derived".to_string()
        },
        pre_migration_dbc_pool_address: derive_dbc_pool_address(&bags_native_mint_pubkey()?, mint, config_key)?
            .to_string(),
    })
}

async fn native_prepare_bags_launch(
    rpc_url: &str,
    config: &NormalizedConfig,
    owner: &Keypair,
    setup_tip_lamports: u64,
    blockhash_override: Option<(String, u64)>,
) -> Result<NativePreparedBagsLaunch, String> {
    let prepare_started_at = Instant::now();
    let api_key = require_bags_api_key()?;
    let fee_recipient_resolve_started_at = Instant::now();
    let fee_claimers =
        resolve_bags_fee_claimers(&owner.pubkey(), &config.feeSharing.recipients, rpc_url).await?;
    let fee_recipient_resolve_ms = fee_recipient_resolve_started_at.elapsed().as_millis();
    if fee_claimers.len() > BAGS_FEE_SHARE_V2_MAX_CLAIMERS_NON_LUT {
        return Err(
            "LaunchDeck Bags fee sharing currently supports up to 15 total claimers including the creator."
                .to_string(),
        );
    }
    let metadata_upload_started_at = Instant::now();
    let token_info = upload_bags_token_info_and_metadata(&api_key, config).await?;
    let metadata_upload_ms = metadata_upload_started_at.elapsed().as_millis();
    let token_mint = Pubkey::from_str(token_info.tokenMint.trim())
        .map_err(|error| format!("Bags metadata upload returned an invalid mint: {error}"))?;
    let partner_launch_params =
        get_bags_partner_launch_params(rpc_url, &config.execution.commitment).await?;
    let setup_tip_account = parse_optional_pubkey(&config.tx.jitoTipAccount);
    let config_result = create_bags_fee_share_config(
        &api_key,
        &owner.pubkey(),
        &token_mint,
        &fee_claimers,
        partner_launch_params,
        bags_config_type_for_mode(&config.mode),
        setup_tip_account.as_ref(),
        setup_tip_lamports,
    )
    .await?;
    let config_key = Pubkey::from_str(config_result.meteoraConfigKey.trim())
        .map_err(|error| format!("Bags fee-share config returned an invalid key: {error}"))?;
    let launch_migration = summarize_launch_migration_config(
        rpc_url,
        &token_mint,
        &config_key,
        &config.execution.commitment,
    )
    .await?;
    let shared_last_valid_block_height = if let Some((_, last_valid_block_height)) =
        blockhash_override.clone()
    {
        last_valid_block_height
    } else {
        fetch_latest_blockhash_cached(rpc_url, &config.execution.commitment)
            .await?
            .1
    };
    let direct_tx_config = NativeBagsVersionedTxConfig {
        compute_unit_limit: config
            .tx
            .computeUnitLimit
            .and_then(|value| u64::try_from(value).ok())
            .unwrap_or_else(configured_default_launch_compute_unit_limit),
        compute_unit_price_micro_lamports: u64::try_from(
            config.tx.computeUnitPriceMicroLamports.unwrap_or_default().max(0),
        )
        .unwrap_or_default(),
        tip_lamports: setup_tip_lamports,
        tip_account: config.tx.jitoTipAccount.clone(),
        jitodontfront: config.execution.jitodontfront,
    };
    let bundled_tx_config = if uses_single_bundle_tip_last_tx(
        &config.execution.provider,
        &config.execution.mevMode,
    ) {
        direct_tx_config.without_inline_tip()
    } else {
        direct_tx_config.clone()
    };
    let mut signed_setup_transactions = Vec::new();
    for transaction in &config_result.transactions {
        let decoded = decode_bags_versioned_transaction(&transaction.transaction)?;
        let ensured = ensure_tx_config_on_bags_versioned_transaction(
            rpc_url,
            owner,
            decoded,
            &direct_tx_config,
            &config.execution.commitment,
            blockhash_override.clone(),
        )
        .await?;
        signed_setup_transactions.push(ensured);
    }
    let setup_transactions = normalize_bags_versioned_transactions(
        &signed_setup_transactions,
        "bags-config-direct",
        shared_last_valid_block_height,
        Some(direct_tx_config.compute_unit_limit),
        Some(direct_tx_config.compute_unit_price_micro_lamports),
        if direct_tx_config.tip_lamports > 0 {
            Some(direct_tx_config.tip_lamports)
        } else {
            None
        },
        if direct_tx_config.tip_account.trim().is_empty() {
            None
        } else {
            Some(direct_tx_config.tip_account.clone())
        },
    )?;
    let mut setup_bundles = Vec::new();
    for (index, bundle) in config_result.bundles.iter().enumerate() {
        let mut signed_bundle_transactions = Vec::new();
        for transaction in bundle {
            let decoded = decode_bags_versioned_transaction(&transaction.transaction)?;
            let ensured = ensure_tx_config_on_bags_versioned_transaction(
                rpc_url,
                owner,
                decoded,
                &bundled_tx_config,
                &config.execution.commitment,
                blockhash_override.clone(),
            )
            .await?;
            signed_bundle_transactions.push(ensured);
        }
        let compiled_bundle_transactions = normalize_bags_versioned_transactions(
            &signed_bundle_transactions,
            &format!("bags-config-bundle-{}", index + 1),
            shared_last_valid_block_height,
            Some(direct_tx_config.compute_unit_limit),
            Some(direct_tx_config.compute_unit_price_micro_lamports),
            if bundled_tx_config.tip_lamports > 0 {
                Some(bundled_tx_config.tip_lamports)
            } else {
                None
            },
            if bundled_tx_config.tip_account.trim().is_empty() {
                None
            } else {
                Some(bundled_tx_config.tip_account.clone())
            },
        )?;
        setup_bundles.push(compiled_bundle_transactions);
    }
    let mut compiled_transactions = setup_bundles.iter().flatten().cloned().collect::<Vec<_>>();
    compiled_transactions.extend(setup_transactions.iter().cloned());
    Ok(NativePreparedBagsLaunch {
        mint: token_mint.to_string(),
        launch_creator: owner.pubkey().to_string(),
        config_key: config_key.to_string(),
        metadata_uri: token_info.tokenMetadata,
        identity_label: "Wallet Only".to_string(),
        migration_fee_option: launch_migration.migration_fee_option,
        expected_migration_family: launch_migration.expected_migration_family,
        expected_damm_config_key: launch_migration.expected_damm_config_key,
        expected_damm_derivation_mode: launch_migration.expected_damm_derivation_mode,
        pre_migration_dbc_pool_address: launch_migration.pre_migration_dbc_pool_address,
        compiled_transactions,
        setup_bundles,
        setup_transactions,
        timings: HelperPrepareLaunchTimings {
            prepareLaunchMs: Some(prepare_started_at.elapsed().as_millis()),
            feeRecipientResolveMs: Some(fee_recipient_resolve_ms),
            metadataUploadMs: Some(metadata_upload_ms),
        },
    })
}

pub async fn estimate_bags_fee_market(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<BagsFeeEstimateSnapshot, String> {
    let requested_tip_lamports =
        u64::try_from(config.tx.jitoTipLamports.max(0)).unwrap_or_default();
    let setup_jito_tip_cap_lamports = bags_setup_jito_tip_cap_lamports();
    let setup_jito_tip_min_lamports = bags_setup_jito_tip_min_lamports();
    let percentile = bags_setup_jito_tip_percentile();
    match fetch_bags_engine_fee_estimate(
        rpc_url,
        requested_tip_lamports,
        setup_jito_tip_cap_lamports,
        setup_jito_tip_min_lamports,
        &percentile,
    )
    .await
    {
        Ok(snapshot) => Ok(snapshot),
        Err(error) => Ok(default_bags_fee_estimate_snapshot(
            requested_tip_lamports,
            setup_jito_tip_cap_lamports,
            setup_jito_tip_min_lamports,
            &percentile,
            format!(
                "Bags engine fee-market snapshot unavailable; using native fallback defaults. {error}"
            ),
        )),
    }
}

pub async fn quote_launch(
    _rpc_url: &str,
    launch_mode: &str,
    mode: &str,
    amount: &str,
) -> Result<Option<LaunchQuote>, String> {
    if amount.trim().is_empty() {
        return Ok(None);
    }
    native_quote_launch(launch_mode, mode, amount)
}

/// Startup warm: Rust-owned Bags runtime readiness payload.
pub async fn warm_bags_helper_ping() -> Result<Value, String> {
    let credentials_configured = !read_active_bags_credentials().api_key.trim().is_empty();
    Ok(json!({
        "ok": true,
        "backend": launchpad_action_backend("bagsapp", "startup-warm"),
        "rolloutState": launchpad_action_rollout_state("bagsapp", "startup-warm"),
        "status": if credentials_configured { "ready" } else { "missing" },
        "credentialsConfigured": credentials_configured,
        "apiBaseUrl": bags_api_base_url(),
        "note": if credentials_configured {
            "Bags startup warm is owned by the Rust runtime."
        } else {
            "BAGS startup warm is optional. The app can still run normally, but native Bags actions need BAGS_API_KEY."
        },
    }))
}

fn build_prepare_launch_helper_request(
    config: &NormalizedConfig,
    wallet_secret: &[u8],
    rpc_url: &str,
    setup_tip_lamports: u64,
    blockhash_override: Option<(String, u64)>,
) -> Result<Value, String> {
    let mut request = json!({
        "action": "prepare-launch",
        "rpcUrl": rpc_url,
        "commitment": config.execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
        "slippageBps": slippage_bps_from_percent(&config.execution.buySlippagePercent)?,
        "txConfig": helper_tx_config(
            config
                .tx
                .computeUnitLimit
                .and_then(|value| u64::try_from(value).ok())
                .or_else(|| Some(configured_default_launch_compute_unit_limit())),
            u64::try_from(config.tx.computeUnitPriceMicroLamports.unwrap_or_default().max(0))
                .unwrap_or_default(),
            setup_tip_lamports,
            &config.tx.jitoTipAccount,
            config.execution.jitodontfront,
            uses_single_bundle_tip_last_tx(&config.execution.provider, &config.execution.mevMode),
        ),
        "mode": config.mode,
        "imageLocalPath": config.imageLocalPath,
        "token": {
            "name": config.token.name,
            "symbol": config.token.symbol,
            "description": config.token.description,
            "website": config.token.website,
            "twitter": config.token.twitter,
            "telegram": config.token.telegram,
        },
        "feeSharing": config.feeSharing.recipients.iter().map(|entry| json!({
            "type": entry.r#type.clone().unwrap_or_else(|| "wallet".to_string()),
            "address": entry.address,
            "githubUsername": entry.githubUsername,
            "githubUserId": entry.githubUserId,
            "shareBps": entry.shareBps,
        })).collect::<Vec<_>>(),
        "devBuy": config.devBuy.as_ref().map(|dev_buy| json!({
            "mode": dev_buy.mode,
            "amount": dev_buy.amount,
        })),
        "identityLabel": "Wallet Only",
    });
    if let Some((blockhash, last_valid_block_height)) = blockhash_override {
        if let Some(obj) = request.as_object_mut() {
            obj.insert("recentBlockhash".to_string(), Value::String(blockhash));
            obj.insert(
                "lastValidBlockHeight".to_string(),
                Value::Number(last_valid_block_height.into()),
            );
        }
    }
    Ok(request)
}

pub async fn try_compile_native_bags(
    rpc_url: &str,
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    wallet_secret: &[u8],
    built_at: String,
    creator_public_key: String,
    config_path: Option<String>,
    include_send_phases: bool,
    launch_blockhash_prime: Option<(String, u64)>,
) -> Result<Option<NativeBagsArtifacts>, String> {
    if config.launchpad != "bagsapp" {
        return Ok(None);
    }
    validate_launchpad_support(config).map_err(|error| error.to_string())?;
    let (fee_estimate, blockhash_override) = if configured_bags_rust_blockhash_for_helper() {
        let fee_fut = estimate_bags_fee_market(rpc_url, config);
        let bh_fut = fetch_latest_blockhash_cached_with_prime(
            rpc_url,
            &config.execution.commitment,
            launch_blockhash_prime,
            COMPILE_BLOCKHASH_MIN_REMAINING_BLOCKS,
        );
        let (fee_res, bh_res) = tokio::join!(fee_fut, bh_fut);
        (fee_res?, Some(bh_res?))
    } else {
        (estimate_bags_fee_market(rpc_url, config).await?, None)
    };
    let setup_tip_lamports = effective_bags_setup_tip_lamports(config, &fee_estimate);
    let owner = parse_owner_keypair(wallet_secret)?;
    let prepared = native_prepare_bags_launch(
        rpc_url,
        config,
        &owner,
        setup_tip_lamports,
        blockhash_override,
    )
    .await?;
    build_native_bags_artifacts_from_prepared(
        prepared,
        config,
        transport_plan,
        built_at,
        rpc_url,
        creator_public_key,
        config_path,
        include_send_phases,
        fee_estimate,
        "Bags setup is now compiled directly inside the Rust runtime via Bags public API responses, while LaunchDeck still owns transport selection and preserves inline fee metadata on the compiled setup transactions.",
    )
    .map(Some)
}

pub async fn compile_launch_transaction(
    rpc_url: &str,
    config: &NormalizedConfig,
    wallet_secret: &[u8],
    mint: &str,
    config_key: &str,
    metadata_uri: &str,
) -> Result<BagsLaunchTransactionArtifacts, String> {
    let tip_lamports = u64::try_from(config.tx.jitoTipLamports.max(0)).unwrap_or_default();
    let setup_gate_commitment = configured_bags_setup_gate_commitment();
    let blockhash_override = if configured_bags_rust_blockhash_for_helper() {
        Some(fetch_latest_blockhash_cached(rpc_url, &config.execution.commitment).await?)
    } else {
        None
    };
    let owner = parse_owner_keypair(wallet_secret)?;
    let api_key = require_bags_api_key()?;
    let token_mint =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bags mint address: {error}"))?;
    let config_key = Pubkey::from_str(config_key)
        .map_err(|error| format!("Invalid Bags config key: {error}"))?;
    let initial_buy_lamports = if let Some(dev_buy) = config.devBuy.as_ref() {
        if dev_buy.amount.trim().is_empty() {
            0
        } else {
            u64::try_from(parse_decimal_to_u128(
                &dev_buy.amount,
                9,
                "dev buy amount",
            )?)
            .map_err(|_| "dev buy amount is too large.".to_string())?
        }
    } else {
        0
    };
    let tip_account = parse_optional_pubkey(&config.tx.jitoTipAccount);
    let tx_config = NativeBagsVersionedTxConfig {
        compute_unit_limit: config
            .tx
            .computeUnitLimit
            .and_then(|value| u64::try_from(value).ok())
            .unwrap_or_else(configured_default_launch_compute_unit_limit),
        compute_unit_price_micro_lamports: u64::try_from(
            config.tx.computeUnitPriceMicroLamports.unwrap_or_default().max(0),
        )
        .unwrap_or_default(),
        tip_lamports,
        tip_account: config.tx.jitoTipAccount.clone(),
        jitodontfront: config.execution.jitodontfront,
    };
    let launch_build_started_at = Instant::now();
    let mut launch_error = None;
    let mut launch_transaction = None;
    for attempt in 0..5 {
        match create_bags_launch_transaction_bytes(
            &api_key,
            metadata_uri,
            &token_mint,
            &owner.pubkey(),
            initial_buy_lamports,
            &config_key,
            tip_account.as_ref(),
            tip_lamports,
        )
        .await
        {
            Ok(encoded) => {
                launch_transaction = Some(decode_bags_versioned_transaction(&encoded)?);
                launch_error = None;
                break;
            }
            Err(error) => {
                launch_error = Some(error);
                if attempt < 4 {
                    sleep(Duration::from_millis(1200)).await;
                }
            }
        }
    }
    let launch_transaction = launch_transaction.ok_or_else(|| {
        launch_error.unwrap_or_else(|| "Failed to create Bags launch transaction.".to_string())
    })?;
    let ensured = ensure_tx_config_on_bags_versioned_transaction(
        rpc_url,
        &owner,
        launch_transaction,
        &tx_config,
        &setup_gate_commitment,
        blockhash_override.clone(),
    )
    .await?;
    let last_valid_block_height = if let Some((_, last_valid_block_height)) = blockhash_override {
        last_valid_block_height
    } else {
        fetch_latest_blockhash_cached(rpc_url, &setup_gate_commitment)
            .await?
            .1
    };
    let mut normalized = normalize_bags_versioned_transactions(
        &[ensured],
        "launch",
        last_valid_block_height,
        Some(tx_config.compute_unit_limit),
        Some(tx_config.compute_unit_price_micro_lamports),
        if tip_lamports > 0 {
            Some(tip_lamports)
        } else {
            None
        },
        if tx_config.tip_account.trim().is_empty() {
            None
        } else {
            Some(tx_config.tip_account.clone())
        },
    )?;
    let compiled_transaction = normalized
        .drain(..)
        .next()
        .ok_or_else(|| "Bags launch transaction normalization returned no transactions.".to_string())?;
    Ok(BagsLaunchTransactionArtifacts {
        compiled_transaction,
        launch_build_ms: Some(launch_build_started_at.elapsed().as_millis()),
    })
}

pub fn summarize_transactions(
    compiled_transactions: &[CompiledTransaction],
    dump_base64: bool,
) -> Vec<TransactionSummary> {
    build_transaction_summaries(compiled_transactions, dump_base64)
}

async fn load_local_damm_market(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<Option<(Pubkey, DecodedDammPool, Option<Pubkey>)>, String> {
    let cached = normalize_cached_bags_launch_hints(bags_launch);
    let Some((_dbc_pool_address, dbc_pool, config)) =
        load_canonical_dbc_market(rpc_url, mint, commitment, bags_launch).await?
    else {
        return Ok(None);
    };
    if !dbc_pool.is_migrated {
        return Ok(None);
    }
    let resolved = if let Some(resolved) = resolve_cached_damm_pool_address(mint, &cached)? {
        Some(resolved)
    } else if let Some(pool_address) = derive_canonical_damm_pool_address(mint, &config)? {
        let config_address = if config.migration_fee_option <= 6 {
            Some(
                Pubkey::from_str(DAMM_V2_MIGRATION_FEE_ADDRESS[config.migration_fee_option as usize])
                    .map_err(|error| format!("Invalid DAMM migration fee address: {error}"))?,
            )
        } else {
            None
        };
        Some((pool_address, config_address))
    } else {
        None
    };
    let Some((pool_address, config_address)) = resolved else {
        return Ok(None);
    };
    let Some(pool_bytes) =
        rpc_fetch_account_data(rpc_url, &pool_address, commitment, "damm-v2-pool").await?
    else {
        return Ok(None);
    };
    let pool = decode_damm_pool(&pool_bytes)?;
    Ok(Some((pool_address, pool, config_address)))
}

pub async fn load_follow_buy_context(
    rpc_url: &str,
    mint: &str,
    commitment: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<Option<BagsFollowBuyContext>, String> {
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bags mint address: {error}"))?;
    if let Some((pool_address, pool, config)) =
        load_canonical_dbc_market(rpc_url, &mint_pubkey, commitment, bags_launch).await?
    {
        if !pool.is_migrated && !is_completed_dbc_pool(&pool, &config) {
            let current_point = current_point_for_dbc_config(rpc_url, &config, commitment).await?;
            return Ok(Some(BagsFollowBuyContext::Dbc(BagsDbcFollowBuyContext {
                pool_address,
                pool,
                config,
                current_point,
            })));
        }
    }
    if let Some((pool_address, pool, _config_address)) =
        load_local_damm_market(rpc_url, &mint_pubkey, commitment, bags_launch).await?
    {
        let (current_slot, current_time) = current_time_for_damm(rpc_url, commitment).await?;
        let current_point = if pool.activation_type == 0 {
            current_slot
        } else {
            current_time
        };
        return Ok(Some(BagsFollowBuyContext::Damm(BagsDammFollowBuyContext {
            pool_address,
            pool,
            current_point,
        })));
    }
    Ok(None)
}

async fn native_fail_closed_bags_trade_error(
    rpc_url: &str,
    mint: &Pubkey,
    commitment: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
    action: &str,
) -> Result<String, String> {
    let dbc_failure_code = if action == "snapshot" {
        "dbc_snapshot_failed"
    } else {
        "dbc_quote_failed_boundary"
    };
    let damm_failure_code = if action == "snapshot" {
        "damm_snapshot_failed"
    } else {
        "damm_quote_failed"
    };
    let cached = normalize_cached_bags_launch_hints(bags_launch);
    let Some((pool_address, pool_bytes)) = rpc_fetch_first_dbc_pool_by_mint(rpc_url, mint, commitment).await?
    else {
        return Ok(build_local_trade_fail_closed_error(
            "dbc_pool_not_found",
            &format!("Canonical Bags {action} requires a local Meteora DBC pool, but none was found."),
            &[
                ("mint", mint.to_string()),
                (
                    "configKey",
                    cached.config_key.map(|value| value.to_string()).unwrap_or_default(),
                ),
                (
                    "expectedPool",
                    cached
                        .pre_migration_dbc_pool_address
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                ),
            ],
        ));
    };
    let pool = decode_dbc_virtual_pool(&pool_bytes)?;
    if let Some(expected_pool) = cached.pre_migration_dbc_pool_address {
        if pool_address != expected_pool {
            return Ok(build_local_trade_fail_closed_error(
                "dbc_pool_mismatch",
                &format!("Resolved DBC pool did not match the cached LaunchDeck Bags pool for {action}."),
                &[
                    ("mint", mint.to_string()),
                    ("resolvedPool", pool_address.to_string()),
                    ("expectedPool", expected_pool.to_string()),
                ],
            ));
        }
    }
    if let Some(expected_config) = cached.config_key {
        if pool.config != expected_config {
            return Ok(build_local_trade_fail_closed_error(
                "dbc_config_mismatch",
                &format!("Resolved DBC config did not match the cached LaunchDeck Bags config for {action}."),
                &[
                    ("mint", mint.to_string()),
                    ("resolvedConfig", pool.config.to_string()),
                    ("expectedConfig", expected_config.to_string()),
                ],
            ));
        }
    }
    let config_key = cached.config_key.unwrap_or(pool.config);
    let Some(config_bytes) =
        rpc_fetch_account_data(rpc_url, &config_key, commitment, "dbc-config").await?
    else {
        return Ok(build_local_trade_fail_closed_error(
            "dbc_config_not_found",
            &format!("Canonical Bags {action} could not load the expected local DBC config."),
            &[("mint", mint.to_string()), ("configKey", config_key.to_string())],
        ));
    };
    let config = decode_dbc_pool_config(&config_bytes)?;
    if config.quote_mint != bags_native_mint_pubkey()? {
        return Ok(build_local_trade_fail_closed_error(
            "dbc_config_not_found",
            &format!("Canonical Bags {action} could not load the expected local DBC config."),
            &[("mint", mint.to_string()), ("configKey", config_key.to_string())],
        ));
    }
    let derived_pool_address = derive_dbc_pool_address(&config.quote_mint, mint, &config_key)?;
    if let Some(expected_pool) = cached.pre_migration_dbc_pool_address {
        if derived_pool_address != expected_pool {
            return Ok(build_local_trade_fail_closed_error(
                "dbc_pool_not_derived",
                &format!("Cached LaunchDeck Bags DBC pool does not match deterministic derivation for {action}."),
                &[
                    ("mint", mint.to_string()),
                    ("derivedPool", derived_pool_address.to_string()),
                    ("expectedPool", expected_pool.to_string()),
                ],
            ));
        }
    }
    if pool_address != derived_pool_address {
        return Ok(build_local_trade_fail_closed_error(
            "dbc_pool_not_derived",
            &format!("Resolved DBC pool did not match deterministic derivation for canonical Bags {action}."),
            &[
                ("mint", mint.to_string()),
                ("resolvedPool", pool_address.to_string()),
                ("derivedPool", derived_pool_address.to_string()),
            ],
        ));
    }
    if !pool.is_migrated && !is_completed_dbc_pool(&pool, &config) {
        return Ok(build_local_trade_fail_closed_error(
            dbc_failure_code,
            &format!("Canonical Bags {action} stayed on the local DBC path but returned no usable result."),
            &[
                ("mint", mint.to_string()),
                ("pool", pool_address.to_string()),
                ("configKey", config_key.to_string()),
            ],
        ));
    }
    let Some((damm_pool_address, damm_config_address)) = resolve_cached_damm_pool_address(mint, &cached)? else {
        return Ok(build_local_trade_fail_closed_error(
            "migration_family_unresolved",
            &format!("Canonical Bags {action} could not resolve the migrated DAMM v2 family from cached launch metadata."),
            &[
                ("mint", mint.to_string()),
                (
                    "migrationFeeOption",
                    cached
                        .migration_fee_option
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                ),
                ("expectedMigrationFamily", cached.expected_migration_family.clone()),
                (
                    "expectedDammConfigKey",
                    cached
                        .expected_damm_config_key
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                ),
            ],
        ));
    };
    if !rpc_account_exists(rpc_url, &damm_pool_address, commitment, "damm-v2-pool").await? {
        return Ok(build_local_trade_fail_closed_error(
            "canonical_damm_pool_not_found",
            &format!("Canonical Bags {action} resolved to a migrated DAMM v2 pool that was not found on-chain."),
            &[
                ("mint", mint.to_string()),
                ("pool", damm_pool_address.to_string()),
                (
                    "configKey",
                    damm_config_address
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "customizable".to_string()),
                ),
            ],
        ));
    }
    let Some(pool_bytes) =
        rpc_fetch_account_data(rpc_url, &damm_pool_address, commitment, "damm-v2-pool").await?
    else {
        return Ok(build_local_trade_fail_closed_error(
            "canonical_damm_pool_not_found",
            &format!("Canonical Bags {action} resolved to a migrated DAMM v2 pool that could not be loaded."),
            &[("mint", mint.to_string()), ("pool", damm_pool_address.to_string())],
        ));
    };
    let _ = decode_damm_pool(&pool_bytes)?;
    Ok(build_local_trade_fail_closed_error(
        damm_failure_code,
        &format!("Canonical Bags {action} resolved to the local DAMM v2 pool but returned no usable result."),
        &[
            ("mint", mint.to_string()),
            ("pool", damm_pool_address.to_string()),
            (
                "configKey",
                damm_config_address
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "customizable".to_string()),
            ),
        ],
    ))
}

async fn native_try_build_local_dbc_follow_buy(
    rpc_url: &str,
    commitment: &str,
    owner: &Keypair,
    mint: &Pubkey,
    buy_amount_sol: &str,
    slippage_bps: u64,
    tx_config: &NativeFollowTxConfig,
    bags_launch: Option<&BagsLaunchMetadata>,
    context_override: Option<&BagsDbcFollowBuyContext>,
) -> Result<Option<CompiledTransaction>, String> {
    let (pool_address, pool, config, current_point) = if let Some(context) = context_override {
        let Some((pool_address, pool, config)) =
            load_canonical_dbc_market(rpc_url, mint, commitment, bags_launch).await?
        else {
            return Ok(None);
        };
        if pool_address != context.pool_address || pool.is_migrated || is_completed_dbc_pool(&pool, &config) {
            return Ok(None);
        }
        let current_point = current_point_for_dbc_config(rpc_url, &config, commitment).await?;
        (pool_address, pool, config, current_point)
    } else {
        let Some((pool_address, pool, config)) =
            load_canonical_dbc_market(rpc_url, mint, commitment, bags_launch).await?
        else {
            return Ok(None);
        };
        if pool.is_migrated || is_completed_dbc_pool(&pool, &config) {
            return Ok(None);
        }
        let current_point = current_point_for_dbc_config(rpc_url, &config, commitment).await?;
        (pool_address, pool, config, current_point)
    };
    let amount_in = parse_decimal_to_u128(buy_amount_sol, 9, "buy amount")?;
    if amount_in == 0 {
        return Ok(None);
    }
    let amount_in = u64::try_from(amount_in).map_err(|_| "buy amount is too large.".to_string())?;
    let (_out_amount, minimum_amount_out) =
        bags_dbc_swap_quote_exact_in(&pool, &config, false, amount_in, slippage_bps, current_point)?;
    let owner_pubkey = owner.pubkey();
    let input_token_program = token_program_for_flag(config.quote_token_flag)?;
    let output_token_program = token_program_for_flag(pool.pool_type)?;
    let (input_token_account, create_input_ata) = maybe_create_ata_instruction(
        rpc_url,
        commitment,
        &owner_pubkey,
        &owner_pubkey,
        &config.quote_mint,
        &input_token_program,
        "follow-buy-input-ata",
    )
    .await?;
    let (output_token_account, create_output_ata) = maybe_create_ata_instruction(
        rpc_url,
        commitment,
        &owner_pubkey,
        &owner_pubkey,
        mint,
        &output_token_program,
        "follow-buy-output-ata",
    )
    .await?;
    let mut instructions = Vec::new();
    if let Some(ix) = create_input_ata {
        instructions.push(ix);
    }
    if let Some(ix) = create_output_ata {
        instructions.push(ix);
    }
    instructions.extend(build_wrap_sol_instructions(&owner_pubkey, &input_token_account, amount_in)?);
    instructions.push(build_dbc_swap_instruction(
        &owner_pubkey,
        &pool_address,
        &pool,
        &config,
        &input_token_account,
        &output_token_account,
        false,
        amount_in,
        minimum_amount_out,
    )?);
    instructions.push(build_unwrap_sol_instruction(&owner_pubkey, &owner_pubkey)?);
    Ok(Some(
        compile_legacy_follow_transaction(
            "follow-buy",
            rpc_url,
            commitment,
            owner,
            tx_config,
            instructions,
        )
        .await?,
    ))
}

async fn native_try_build_local_dbc_follow_sell(
    rpc_url: &str,
    commitment: &str,
    owner: &Keypair,
    mint: &Pubkey,
    sell_percent: u8,
    slippage_bps: u64,
    tx_config: &NativeFollowTxConfig,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<Option<Option<CompiledTransaction>>, String> {
    let Some((pool_address, pool, config)) =
        load_canonical_dbc_market(rpc_url, mint, commitment, bags_launch).await?
    else {
        return Ok(None);
    };
    if pool.is_migrated || is_completed_dbc_pool(&pool, &config) {
        return Ok(None);
    }
    let owner_pubkey = owner.pubkey();
    let owner_token_account =
        get_associated_token_address_with_program_id(&owner_pubkey, mint, &bags_token_program_pubkey()?);
    let raw_amount = match fetch_bags_token_account_amount(rpc_url, &owner_token_account, commitment).await {
        Ok(value) => value,
        Err(_) => return Ok(Some(None)),
    };
    if raw_amount == 0 {
        return Ok(Some(None));
    }
    let sell_amount = ((u128::from(raw_amount) * u128::from(sell_percent)) / 100u128) as u64;
    if sell_amount == 0 {
        return Ok(Some(None));
    }
    let current_point = current_point_for_dbc_config(rpc_url, &config, commitment).await?;
    let (_out_amount, minimum_amount_out) =
        bags_dbc_swap_quote_exact_in(&pool, &config, true, sell_amount, slippage_bps, current_point)?;
    let input_token_program = token_program_for_flag(pool.pool_type)?;
    let output_token_program = token_program_for_flag(config.quote_token_flag)?;
    let (input_token_account, create_input_ata) = maybe_create_ata_instruction(
        rpc_url,
        commitment,
        &owner_pubkey,
        &owner_pubkey,
        mint,
        &input_token_program,
        "follow-sell-input-ata",
    )
    .await?;
    let (output_token_account, create_output_ata) = maybe_create_ata_instruction(
        rpc_url,
        commitment,
        &owner_pubkey,
        &owner_pubkey,
        &config.quote_mint,
        &output_token_program,
        "follow-sell-output-ata",
    )
    .await?;
    let mut instructions = Vec::new();
    if let Some(ix) = create_input_ata {
        instructions.push(ix);
    }
    if let Some(ix) = create_output_ata {
        instructions.push(ix);
    }
    instructions.push(build_dbc_swap_instruction(
        &owner_pubkey,
        &pool_address,
        &pool,
        &config,
        &input_token_account,
        &output_token_account,
        true,
        sell_amount,
        minimum_amount_out,
    )?);
    instructions.push(build_unwrap_sol_instruction(&owner_pubkey, &owner_pubkey)?);
    Ok(Some(Some(
        compile_legacy_follow_transaction(
            "follow-sell",
            rpc_url,
            commitment,
            owner,
            tx_config,
            instructions,
        )
        .await?,
    )))
}

async fn native_try_build_local_damm_follow_buy(
    rpc_url: &str,
    commitment: &str,
    owner: &Keypair,
    mint: &Pubkey,
    buy_amount_sol: &str,
    slippage_bps: u64,
    tx_config: &NativeFollowTxConfig,
    bags_launch: Option<&BagsLaunchMetadata>,
    context_override: Option<&BagsDammFollowBuyContext>,
) -> Result<Option<CompiledTransaction>, String> {
    let (pool_address, pool, current_point) = if let Some(context) = context_override {
        let Some((pool_address, pool, _config_address)) =
            load_local_damm_market(rpc_url, mint, commitment, bags_launch).await?
        else {
            return Ok(None);
        };
        if pool_address != context.pool_address {
            return Ok(None);
        }
        let (current_slot, current_time) = current_time_for_damm(rpc_url, commitment).await?;
        let current_point = if pool.activation_type == 0 {
            current_slot
        } else {
            current_time
        };
        (pool_address, pool, current_point)
    } else {
        let Some((pool_address, pool, _config_address)) =
            load_local_damm_market(rpc_url, mint, commitment, bags_launch).await?
        else {
            return Ok(None);
        };
        let (current_slot, current_time) = current_time_for_damm(rpc_url, commitment).await?;
        let current_point = if pool.activation_type == 0 {
            current_slot
        } else {
            current_time
        };
        (pool_address, pool, current_point)
    };
    let amount_in = parse_decimal_to_u128(buy_amount_sol, 9, "buy amount")?;
    if amount_in == 0 {
        return Ok(None);
    }
    let amount_in = u64::try_from(amount_in).map_err(|_| "buy amount is too large.".to_string())?;
    let out_amount = cpamm_swap_amount_out(&BigUint::from(amount_in), &bags_native_mint_pubkey()?, &pool, current_point)?
        .to_u64()
        .ok_or_else(|| "Bags DAMM follow quote overflowed u64.".to_string())?;
    let minimum_amount_out = helper_slippage_minimum_amount(out_amount, slippage_bps);
    let owner_pubkey = owner.pubkey();
    let input_token_program = if pool.token_a_mint == bags_native_mint_pubkey()? {
        token_program_for_flag(pool.token_a_flag)?
    } else {
        token_program_for_flag(pool.token_b_flag)?
    };
    let output_token_program = if pool.token_a_mint == *mint {
        token_program_for_flag(pool.token_a_flag)?
    } else {
        token_program_for_flag(pool.token_b_flag)?
    };
    let (input_token_account, create_input_ata) = maybe_create_ata_instruction(
        rpc_url,
        commitment,
        &owner_pubkey,
        &owner_pubkey,
        &bags_native_mint_pubkey()?,
        &input_token_program,
        "follow-buy-input-ata",
    )
    .await?;
    let (output_token_account, create_output_ata) = maybe_create_ata_instruction(
        rpc_url,
        commitment,
        &owner_pubkey,
        &owner_pubkey,
        mint,
        &output_token_program,
        "follow-buy-output-ata",
    )
    .await?;
    let mut instructions = Vec::new();
    if let Some(ix) = create_input_ata {
        instructions.push(ix);
    }
    if let Some(ix) = create_output_ata {
        instructions.push(ix);
    }
    instructions.extend(build_wrap_sol_instructions(&owner_pubkey, &input_token_account, amount_in)?);
    instructions.push(build_damm_swap_instruction(
        &owner_pubkey,
        &pool_address,
        &pool,
        &input_token_account,
        &output_token_account,
        amount_in,
        minimum_amount_out,
    )?);
    instructions.push(build_unwrap_sol_instruction(&owner_pubkey, &owner_pubkey)?);
    Ok(Some(
        compile_legacy_follow_transaction(
            "follow-buy",
            rpc_url,
            commitment,
            owner,
            tx_config,
            instructions,
        )
        .await?,
    ))
}

async fn native_try_build_local_damm_follow_sell(
    rpc_url: &str,
    commitment: &str,
    owner: &Keypair,
    mint: &Pubkey,
    sell_percent: u8,
    slippage_bps: u64,
    tx_config: &NativeFollowTxConfig,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<Option<Option<CompiledTransaction>>, String> {
    let Some((pool_address, pool, _config_address)) =
        load_local_damm_market(rpc_url, mint, commitment, bags_launch).await?
    else {
        return Ok(None);
    };
    let owner_pubkey = owner.pubkey();
    let owner_token_account =
        get_associated_token_address_with_program_id(&owner_pubkey, mint, &bags_token_program_pubkey()?);
    let raw_amount = match fetch_bags_token_account_amount(rpc_url, &owner_token_account, commitment).await {
        Ok(value) => value,
        Err(_) => return Ok(Some(None)),
    };
    if raw_amount == 0 {
        return Ok(Some(None));
    }
    let sell_amount = ((u128::from(raw_amount) * u128::from(sell_percent)) / 100u128) as u64;
    if sell_amount == 0 {
        return Ok(Some(None));
    }
    let (current_slot, current_time) = current_time_for_damm(rpc_url, commitment).await?;
    let current_point = if pool.activation_type == 0 { current_slot } else { current_time };
    let out_amount = cpamm_swap_amount_out(&BigUint::from(sell_amount), mint, &pool, current_point)?
        .to_u64()
        .ok_or_else(|| "Bags DAMM follow quote overflowed u64.".to_string())?;
    let minimum_amount_out = helper_slippage_minimum_amount(out_amount, slippage_bps);
    let input_token_program = if pool.token_a_mint == *mint {
        token_program_for_flag(pool.token_a_flag)?
    } else {
        token_program_for_flag(pool.token_b_flag)?
    };
    let output_token_program = if pool.token_a_mint == bags_native_mint_pubkey()? {
        token_program_for_flag(pool.token_a_flag)?
    } else {
        token_program_for_flag(pool.token_b_flag)?
    };
    let (input_token_account, create_input_ata) = maybe_create_ata_instruction(
        rpc_url,
        commitment,
        &owner_pubkey,
        &owner_pubkey,
        mint,
        &input_token_program,
        "follow-sell-input-ata",
    )
    .await?;
    let (output_token_account, create_output_ata) = maybe_create_ata_instruction(
        rpc_url,
        commitment,
        &owner_pubkey,
        &owner_pubkey,
        &bags_native_mint_pubkey()?,
        &output_token_program,
        "follow-sell-output-ata",
    )
    .await?;
    let mut instructions = Vec::new();
    if let Some(ix) = create_input_ata {
        instructions.push(ix);
    }
    if let Some(ix) = create_output_ata {
        instructions.push(ix);
    }
    instructions.push(build_damm_swap_instruction(
        &owner_pubkey,
        &pool_address,
        &pool,
        &input_token_account,
        &output_token_account,
        sell_amount,
        minimum_amount_out,
    )?);
    instructions.push(build_unwrap_sol_instruction(&owner_pubkey, &owner_pubkey)?);
    Ok(Some(Some(
        compile_legacy_follow_transaction(
            "follow-sell",
            rpc_url,
            commitment,
            owner,
            tx_config,
            instructions,
        )
        .await?,
    )))
}

pub async fn compile_follow_buy_transaction(
    rpc_url: &str,
    execution: &NormalizedExecution,
    _token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    _launch_creator: &str,
    buy_amount_sol: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
    context_override: Option<&BagsFollowBuyContext>,
) -> Result<CompiledTransaction, String> {
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bags mint address: {error}"))?;
    let owner = parse_owner_keypair(wallet_secret)?;
    let slippage_bps = slippage_bps_from_percent(&execution.buySlippagePercent)?;
    let tx_config = build_follow_buy_tx_config(execution, jito_tip_account)?;
    if let Some(compiled) = native_try_build_local_dbc_follow_buy(
        rpc_url,
        &execution.commitment,
        &owner,
        &mint_pubkey,
        buy_amount_sol,
        slippage_bps,
        &tx_config,
        bags_launch,
        match context_override {
            Some(BagsFollowBuyContext::Dbc(context)) => Some(context),
            _ => None,
        },
    )
    .await?
    {
        return Ok(compiled);
    }
    if let Some(compiled) = native_try_build_local_damm_follow_buy(
        rpc_url,
        &execution.commitment,
        &owner,
        &mint_pubkey,
        buy_amount_sol,
        slippage_bps,
        &tx_config,
        bags_launch,
        match context_override {
            Some(BagsFollowBuyContext::Damm(context)) => Some(context),
            _ => None,
        },
    )
    .await?
    {
        return Ok(compiled);
    }
    Err(native_fail_closed_bags_trade_error(
        rpc_url,
        &mint_pubkey,
        &execution.commitment,
        bags_launch,
        "buy",
    )
    .await?)
}

pub async fn compile_atomic_follow_buy_transaction(
    rpc_url: &str,
    _launch_mode: &str,
    _quote_asset: &str,
    execution: &NormalizedExecution,
    token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    buy_amount_sol: &str,
) -> Result<CompiledTransaction, String> {
    compile_follow_buy_transaction(
        rpc_url,
        execution,
        token_mayhem_mode,
        jito_tip_account,
        wallet_secret,
        mint,
        launch_creator,
        buy_amount_sol,
        None,
        None,
    )
    .await
}

pub async fn compile_follow_sell_transaction(
    rpc_url: &str,
    execution: &NormalizedExecution,
    _token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    _launch_creator: &str,
    sell_percent: u8,
    _prefer_post_setup_creator_vault: bool,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<Option<CompiledTransaction>, String> {
    let mint_pubkey =
        Pubkey::from_str(mint).map_err(|error| format!("Invalid Bags mint address: {error}"))?;
    let owner = parse_owner_keypair(wallet_secret)?;
    let slippage_bps = slippage_bps_from_percent(&execution.sellSlippagePercent)?;
    let tx_config = build_follow_sell_tx_config(execution, jito_tip_account)?;
    if let Some(local) = native_try_build_local_dbc_follow_sell(
        rpc_url,
        &execution.commitment,
        &owner,
        &mint_pubkey,
        sell_percent,
        slippage_bps,
        &tx_config,
        bags_launch,
    )
    .await?
    {
        return Ok(local);
    }
    if let Some(local) = native_try_build_local_damm_follow_sell(
        rpc_url,
        &execution.commitment,
        &owner,
        &mint_pubkey,
        sell_percent,
        slippage_bps,
        &tx_config,
        bags_launch,
    )
    .await?
    {
        return Ok(local);
    }
    Err(native_fail_closed_bags_trade_error(
        rpc_url,
        &mint_pubkey,
        &execution.commitment,
        bags_launch,
        "sell",
    )
    .await?)
}

pub async fn fetch_bags_market_snapshot(
    rpc_url: &str,
    mint: &str,
    bags_launch: Option<&BagsLaunchMetadata>,
) -> Result<BagsMarketSnapshot, String> {
    native_fetch_bags_market_snapshot(rpc_url, mint, bags_launch).await
}

pub async fn lookup_bags_fee_recipient(
    rpc_url: &str,
    provider: &str,
    username: &str,
    github_user_id: &str,
) -> Result<BagsFeeRecipientLookupResponse, String> {
    let api_key = read_active_bags_credentials().api_key;
    if api_key.trim().is_empty() {
        return Err("BAGS_API_KEY is required for Bagsapp integration.".to_string());
    }
    let normalized_type = match provider.trim().to_ascii_lowercase().as_str() {
        "x" => "twitter",
        "github" => "github",
        "twitter" => "twitter",
        "kick" => "kick",
        "tiktok" => "tiktok",
        "" => {
            return Err(
                "Unsupported Bags fee-share recipient type: (missing)".to_string()
            )
        }
        other => {
            return Err(format!(
                "Unsupported Bags fee-share recipient type: {other}"
            ))
        }
    };
    let social_handle = username.trim().trim_start_matches('@').to_string();
    let social_id = github_user_id.trim().to_string();
    let lookup_target = if normalized_type == "github" {
        if !social_handle.is_empty() {
            social_handle.clone()
        } else {
            social_id.clone()
        }
    } else {
        social_handle.clone()
    };
    if lookup_target.is_empty() {
        return Err(if normalized_type == "github" {
            "Bags GitHub fee-share rows require a GitHub username or user id.".to_string()
        } else {
            format!("Bags {normalized_type} fee-share rows require a username.")
        });
    }
    if rpc_url.trim().is_empty() {
        return Err("SOLANA_RPC_URL is required for Bagsapp integration.".to_string());
    }
    let build_lookup_error_response = |message: String| BagsFeeRecipientLookupResponse {
        found: false,
        provider: normalized_type.to_string(),
        lookupTarget: lookup_target.clone(),
        wallet: String::new(),
        resolvedUsername: social_handle.clone(),
        githubUserId: if normalized_type == "github" {
            social_id.clone()
        } else {
            String::new()
        },
        notFound: false,
        error: message,
    };
    let response = bags_fee_http_client()
        .get(format!(
            "{}/token-launch/fee-share/wallet/v2",
            bags_api_base_url()
        ))
        .header("x-api-key", api_key)
        .query(&[
            ("username", lookup_target.as_str()),
            ("provider", normalized_type),
        ])
        .send()
        .await;
    let response = match response {
        Ok(value) => value,
        Err(error) => {
            return Ok(build_lookup_error_response(format!(
                "Failed to query Bags fee-share wallet lookup: {error}"
            )));
        }
    };
    if response.status().is_success() {
        let payload = response
            .json()
            .await
            .map_err(|error| format!("Failed to parse Bags fee-share wallet lookup: {error}"));
        let payload: BagsApiEnvelope<BagsLookupWalletResponse> = match payload {
            Ok(value) => value,
            Err(error) => return Ok(build_lookup_error_response(error)),
        };
        if !payload.success {
            return Ok(BagsFeeRecipientLookupResponse {
                found: false,
                provider: normalized_type.to_string(),
                lookupTarget: lookup_target,
                wallet: String::new(),
                resolvedUsername: social_handle,
                githubUserId: if normalized_type == "github" {
                    social_id
                } else {
                    String::new()
                },
                notFound: payload.error.to_ascii_lowercase().contains("not found"),
                error: payload.error,
            });
        }
        let wallet = payload
            .response
            .as_ref()
            .map(|value| value.wallet.trim().to_string())
            .unwrap_or_default();
        return Ok(BagsFeeRecipientLookupResponse {
            found: !wallet.is_empty(),
            provider: payload
                .response
                .as_ref()
                .map(|value| value.provider.clone())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| normalized_type.to_string()),
            lookupTarget: lookup_target,
            wallet,
            resolvedUsername: social_handle,
            githubUserId: if normalized_type == "github" {
                social_id
            } else {
                String::new()
            },
            notFound: false,
            error: String::new(),
        });
    }
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let message = match serde_json::from_str::<Value>(&body) {
        Ok(value) => value
            .get("error")
            .and_then(Value::as_str)
            .or_else(|| value.get("message").and_then(Value::as_str))
            .map(|value| value.to_string())
            .unwrap_or_else(|| format!("Request failed with status {status}")),
        Err(_) => {
            let trimmed = body.trim();
            if trimmed.is_empty() {
                format!("Request failed with status {status}")
            } else {
                trimmed.to_string()
            }
        }
    };
    Ok(BagsFeeRecipientLookupResponse {
        found: false,
        provider: normalized_type.to_string(),
        lookupTarget: lookup_target,
        wallet: String::new(),
        resolvedUsername: social_handle,
        githubUserId: if normalized_type == "github" {
            social_id
        } else {
            String::new()
        },
        notFound: status == reqwest::StatusCode::NOT_FOUND
            || message.to_ascii_lowercase().contains("not found"),
        error: message,
    })
}

pub async fn detect_bags_import_context(
    rpc_url: &str,
    mint: &str,
) -> Result<Option<BagsImportContext>, String> {
    native_detect_bags_import_context(rpc_url, mint).await
}

pub async fn poll_bags_market_cap_lamports(
    rpc_url: &str,
    mint: &str,
) -> Result<Option<u64>, String> {
    let snapshot = fetch_bags_market_snapshot(rpc_url, mint, None).await?;
    let value = snapshot
        .marketCapLamports
        .parse::<u64>()
        .map_err(|error| format!("Invalid Bags market cap response: {error}"))?;
    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{RawConfig, normalize_raw_config};
    use serde_json::{Value, json};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sample_bags_config() -> NormalizedConfig {
        let raw: RawConfig = serde_json::from_value(json!({
            "mode": "regular",
            "launchpad": "bagsapp",
            "quoteAsset": "sol",
            "token": {
                "name": "LaunchDeck",
                "symbol": "LDECK",
                "uri": "ipfs://test",
                "description": "demo"
            },
            "agent": {
                "splitAgentInit": false,
                "feeRecipients": []
            },
            "tx": {
                "computeUnitPriceMicroLamports": 1,
                "jitoTipLamports": 200000,
                "jitoTipAccount": "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE",
                "useDefaultLookupTables": true,
                "dumpBase64": false,
                "writeReport": true
            },
            "feeSharing": {
                "generateLaterSetup": false,
                "recipients": [{
                    "type": "github",
                    "address": "",
                    "githubUsername": "",
                    "githubUserId": "123456",
                    "shareBps": 10000
                }]
            },
            "creatorFee": {
                "mode": "deployer"
            },
            "execution": {
                "simulate": false,
                "send": false,
                "txFormat": "auto",
                "commitment": "confirmed",
                "skipPreflight": true,
                "provider": "helius-sender",
                "endpointProfile": "global",
                "autoGas": true,
                "autoMode": "launchAuto",
                "buyProvider": "helius-sender",
                "buyEndpointProfile": "global",
                "buyAutoGas": true,
                "buyAutoMode": "buyAuto",
                "sellAutoGas": false,
                "sellAutoMode": "sellAuto",
                "sellProvider": "helius-sender",
                "sellEndpointProfile": "global"
            },
            "postLaunch": {
                "strategy": "none",
                "snipeOwnLaunch": { "buyAmountSol": "" },
                "automaticDevSell": {
                    "enabled": false,
                    "percent": 0,
                    "delaySeconds": 0
                }
            },
            "presets": {}
        }))
        .expect("sample bags raw config");
        normalize_raw_config(raw).expect("sample bags normalized config")
    }

    #[test]
    fn bags_setup_tip_percentile_follows_shared_auto_fee_setting() {
        let _guard = env_lock().lock().expect("lock env");
        unsafe {
            std::env::set_var("LAUNCHDECK_AUTO_FEE_JITO_TIP_PERCENTILE", "p95");
        }
        assert_eq!(bags_setup_jito_tip_percentile(), "p95");
        unsafe {
            std::env::remove_var("LAUNCHDECK_AUTO_FEE_JITO_TIP_PERCENTILE");
        }
    }

    #[test]
    fn bags_setup_tip_percentile_defaults_to_shared_p99() {
        let _guard = env_lock().lock().expect("lock env");
        unsafe {
            std::env::remove_var("LAUNCHDECK_AUTO_FEE_JITO_TIP_PERCENTILE");
        }
        assert_eq!(bags_setup_jito_tip_percentile(), "p99");
    }

    #[test]
    fn bags_fee_estimate_cache_key_tracks_request_tip_policy() {
        let base = bags_fee_estimate_cache_key("https://rpc.example", 100, 1_000, 10, "p99");
        let changed_tip = bags_fee_estimate_cache_key("https://rpc.example", 200, 1_000, 10, "p99");
        let changed_cap = bags_fee_estimate_cache_key("https://rpc.example", 100, 2_000, 10, "p99");
        let changed_min = bags_fee_estimate_cache_key("https://rpc.example", 100, 1_000, 20, "p99");
        assert_ne!(base, changed_tip);
        assert_ne!(base, changed_cap);
        assert_ne!(base, changed_min);
    }

    #[test]
    fn slippage_percent_maps_to_expected_bps() {
        assert_eq!(slippage_bps_from_percent("20").expect("20%"), 2_000);
        assert_eq!(slippage_bps_from_percent("0.5").expect("0.5%"), 50);
        assert_eq!(slippage_bps_from_percent("100").expect("100%"), 10_000);
    }

    #[test]
    fn summarize_bags_api_failure_prefers_response_message_when_error_blank() {
        let message = summarize_bags_api_failure(
            "Failed to create Bags launch transaction",
            reqwest::StatusCode::BAD_REQUEST,
            "",
            Some(&json!({ "message": "configKey is invalid" })),
            r#"{"success":false,"response":{"message":"configKey is invalid"}}"#,
        );
        assert_eq!(message, "configKey is invalid");
    }

    #[test]
    fn summarize_bags_api_failure_falls_back_to_body_preview() {
        let message = summarize_bags_api_failure(
            "Failed to create Bags launch transaction",
            reqwest::StatusCode::BAD_REQUEST,
            "",
            None,
            r#"{"success":false,"unexpected":"shape"}"#,
        );
        assert!(message.contains("status 400 Bad Request"));
        assert!(message.contains(r#"{"success":false,"unexpected":"shape"}"#));
    }

    #[test]
    fn prepare_launch_helper_request_preserves_github_user_id_fee_recipients() {
        let request = build_prepare_launch_helper_request(
            &sample_bags_config(),
            &[1_u8, 2_u8, 3_u8],
            "https://rpc.example",
            123_456,
            None,
        )
        .expect("helper request");
        let fee_sharing = request
            .get("feeSharing")
            .and_then(Value::as_array)
            .expect("fee sharing array");
        assert_eq!(fee_sharing.len(), 1);
        let first = &fee_sharing[0];
        assert_eq!(
            first.get("githubUserId").and_then(Value::as_str),
            Some("123456")
        );
        assert_eq!(
            first.get("githubUsername").and_then(Value::as_str),
            Some("")
        );
    }

    #[test]
    fn bags_mode_mapping_accepts_known_fee_pairs() {
        assert_eq!(bags_mode_from_fee_values(200, 200), "bags-2-2");
        assert_eq!(bags_mode_from_fee_values(25, 100), "bags-025-1");
        assert_eq!(bags_mode_from_fee_values(100, 25), "bags-1-025");
        assert_eq!(bags_mode_from_fee_values(7, 9), "");
    }

    fn sample_initial_dbc_follow_state(mode: &str) -> (DecodedDbcVirtualPool, DecodedDbcPoolConfig) {
        let initial_sqrt = bags_initial_sqrt_price()
            .to_u128()
            .expect("initial sqrt price should fit in u128");
        (
            DecodedDbcVirtualPool {
                config: Pubkey::new_unique(),
                creator: Pubkey::new_unique(),
                base_mint: Pubkey::new_unique(),
                base_vault: Pubkey::new_unique(),
                quote_vault: Pubkey::new_unique(),
                sqrt_price: initial_sqrt,
                base_reserve: 0,
                quote_reserve: 0,
                volatility_accumulator: 0,
                activation_point: 0,
                pool_type: 0,
                is_migrated: false,
            },
            DecodedDbcPoolConfig {
                quote_mint: bags_native_mint_pubkey().expect("native mint"),
                collect_fee_mode: 0,
                activation_type: 0,
                quote_token_flag: 0,
                migration_fee_option: 0,
                creator_trading_fee_percentage: 0,
                creator_migration_fee_percentage: 0,
                migration_quote_threshold: BAGS_MIGRATION_QUOTE_THRESHOLD_STR
                    .parse()
                    .expect("migration threshold"),
                sqrt_start_price: initial_sqrt,
                curve: bags_curve_points(),
                base_fee: DecodedDbcBaseFeeConfig {
                    cliff_fee_numerator: bags_cliff_fee_numerator_for_mode(mode),
                    first_factor: 0,
                    second_factor: 0,
                    third_factor: 0,
                    base_fee_mode: 0,
                },
                dynamic_fee: DecodedDbcDynamicFeeConfig {
                    initialized: false,
                    variable_fee_control: 0,
                    bin_step: 0,
                    volatility_accumulator: 0,
                },
            },
        )
    }

    #[test]
    fn generic_dbc_follow_buy_quote_matches_initial_launch_math() {
        let (pool, config) = sample_initial_dbc_follow_state("bags-2-2");
        let amount_in = 500_000_000u64;
        let expected = bags_get_quote_to_base_output(&bags_get_fee_amount_excluded(
            &BigUint::from(amount_in),
            bags_cliff_fee_numerator_for_mode("bags-2-2"),
        ))
        .expect("expected launch math")
        .to_u64()
        .expect("expected output");
        let (actual, minimum) =
            bags_dbc_swap_quote_exact_in(&pool, &config, false, amount_in, 0, 0).expect("follow quote");
        assert_eq!(actual, expected);
        assert_eq!(minimum, expected);
    }

    #[test]
    fn generic_dbc_follow_sell_quote_matches_initial_curve_math() {
        let (pool, config) = sample_initial_dbc_follow_state("bags-2-2");
        let amount_in = 1_000_000_000u64;
        let raw_out = bags_dbc_swap_amount_from_base_to_quote(
            &biguint_from_u128(pool.sqrt_price),
            &config.curve,
            &BigUint::from(amount_in),
        )
        .expect("raw output");
        let expected = bags_get_fee_amount_excluded(
            &raw_out,
            bags_cliff_fee_numerator_for_mode("bags-2-2"),
        )
        .to_u64()
        .expect("expected output");
        let (actual, minimum) =
            bags_dbc_swap_quote_exact_in(&pool, &config, true, amount_in, 0, 0).expect("follow quote");
        assert_eq!(actual, expected);
        assert_eq!(minimum, expected);
    }

    #[test]
    fn token_account_balance_parser_accepts_standard_rpc_shape() {
        let parsed: RpcResponse<RpcTokenAccountBalanceResult> = serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "result": {
                "context": {
                    "slot": 123
                },
                "value": {
                    "amount": "33792190511826",
                    "decimals": 9,
                    "uiAmount": 33792.190511826,
                    "uiAmountString": "33792.190511826"
                }
            },
            "id": "x"
        }))
        .expect("parse token account balance");
        assert_eq!(parsed.result.value.amount, "33792190511826");
    }

    #[tokio::test]
    async fn native_bags_sol_quote_matches_helper() {
        let _guard = env_lock().lock().expect("lock env");
        unsafe {
            std::env::set_var("LAUNCHDECK_BAGSAPP_QUOTE_BACKEND", "helper");
        }
        let helper = quote_launch("https://rpc.example", "bags-2-2", "sol", "0.5")
            .await
            .expect("helper quote");
        unsafe {
            std::env::set_var("LAUNCHDECK_BAGSAPP_QUOTE_BACKEND", "rust-native");
        }
        let native = quote_launch("https://rpc.example", "bags-2-2", "sol", "0.5")
            .await
            .expect("native quote");
        unsafe {
            std::env::remove_var("LAUNCHDECK_BAGSAPP_QUOTE_BACKEND");
        }
        assert_eq!(native.as_ref().map(|quote| &quote.mode), helper.as_ref().map(|quote| &quote.mode));
        assert_eq!(
            native.as_ref().map(|quote| &quote.estimatedTokens),
            helper.as_ref().map(|quote| &quote.estimatedTokens)
        );
        assert_eq!(
            native.as_ref().map(|quote| &quote.estimatedSol),
            helper.as_ref().map(|quote| &quote.estimatedSol)
        );
        assert_eq!(
            native.as_ref().map(|quote| &quote.estimatedSupplyPercent),
            helper.as_ref().map(|quote| &quote.estimatedSupplyPercent)
        );
    }

    #[tokio::test]
    async fn native_bags_token_quote_matches_helper() {
        let _guard = env_lock().lock().expect("lock env");
        unsafe {
            std::env::set_var("LAUNCHDECK_BAGSAPP_QUOTE_BACKEND", "helper");
        }
        let helper = quote_launch("https://rpc.example", "bags-025-1", "tokens", "250000")
            .await
            .expect("helper quote");
        unsafe {
            std::env::set_var("LAUNCHDECK_BAGSAPP_QUOTE_BACKEND", "rust-native");
        }
        let native = quote_launch("https://rpc.example", "bags-025-1", "tokens", "250000")
            .await
            .expect("native quote");
        unsafe {
            std::env::remove_var("LAUNCHDECK_BAGSAPP_QUOTE_BACKEND");
        }
        assert_eq!(native.as_ref().map(|quote| &quote.mode), helper.as_ref().map(|quote| &quote.mode));
        assert_eq!(
            native.as_ref().map(|quote| &quote.estimatedTokens),
            helper.as_ref().map(|quote| &quote.estimatedTokens)
        );
        assert_eq!(
            native.as_ref().map(|quote| &quote.estimatedSol),
            helper.as_ref().map(|quote| &quote.estimatedSol)
        );
        assert_eq!(
            native.as_ref().map(|quote| &quote.estimatedSupplyPercent),
            helper.as_ref().map(|quote| &quote.estimatedSupplyPercent)
        );
    }

    #[test]
    fn cached_customizable_damm_resolution_uses_customizable_pool() {
        let mint = Pubkey::from_str("So11111111111111111111111111111111111111111")
            .expect("test mint");
        let hints = CachedBagsLaunchHints {
            migration_fee_option: Some(6),
            expected_migration_family: "customizable".to_string(),
            ..Default::default()
        };
        let (pool, config) = resolve_cached_damm_pool_address(&mint, &hints)
            .expect("resolve cached damm")
            .expect("customizable pool");
        assert_eq!(config, None);
        assert_eq!(
            pool,
            derive_damm_customizable_pool_address(&mint, &bags_native_mint_pubkey().expect("native mint"))
                .expect("derive customizable"),
        );
    }

    #[test]
    fn bags_launch_payload_matches_current_api_shape() {
        let token_mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let config_key = Pubkey::new_unique();
        let payload = build_bags_launch_transaction_payload(
            "ipfs://metadata",
            &token_mint,
            &owner,
            500_000_000,
            &config_key,
            None,
            0,
        );

        assert!(payload.get("slippageBps").is_none());
        assert_eq!(payload["initialBuyLamports"], json!(500_000_000u64));
        assert_eq!(payload["configKey"], json!(config_key.to_string()));
    }
}
