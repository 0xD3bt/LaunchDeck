#![allow(non_snake_case, dead_code)]

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    process::Stdio,
    sync::{Arc, OnceLock},
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
        configured_default_launch_compute_unit_limit,
        configured_default_launch_usd1_topup_compute_unit_limit,
        configured_default_sniper_buy_compute_unit_limit, validate_launchpad_support,
    },
    helper_worker::{
        HelperWorkerClient, HelperWorkerConfig, HelperWorkerError, helper_worker_enabled,
    },
    report::{
        BonkUsd1LaunchSummary, FeeSettings, InstructionSummary, TransactionSummary, build_report,
        render_report,
    },
    rpc::CompiledTransaction,
    transport::TransportPlan,
};

use crate::pump_native::{LaunchQuote, NativeCompileTimings};

const PACKET_LIMIT_BYTES: usize = 1232;
const PRIORITY_FEE_PRICE_BASE_COMPUTE_UNIT_LIMIT: u64 = 1_000_000;
const DEFAULT_HELPER_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_HELPER_MAX_CONCURRENCY: usize = 4;

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

#[derive(Debug, Serialize, Deserialize)]
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
    Ok(project_root()?.join("scripts/bonk-launchpad.js"))
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
) -> HelperTxConfig<'_> {
    HelperTxConfig {
        computeUnitLimit: compute_unit_limit.unwrap_or_else(configured_default_launch_compute_unit_limit),
        computeUnitPriceMicroLamports: compute_unit_price_micro_lamports,
        tipLamports: tip_lamports,
        tipAccount: tip_account,
    }
}

fn provider_uses_follow_tip(provider: &str) -> bool {
    matches!(
        provider.trim().to_ascii_lowercase().as_str(),
        "helius-sender" | "jito-bundle"
    )
}

fn resolve_follow_tip_lamports(
    provider: &str,
    tip_sol: &str,
    label: &str,
) -> Result<u64, String> {
    if !provider_uses_follow_tip(provider) {
        return Ok(0);
    }
    parse_decimal_u64(tip_sol, 9, label)
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
    BASE64.encode(secret)
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
    let quote: LaunchQuote = run_helper(&json!({
        "action": "quote",
        "rpcUrl": rpc_url,
        "quoteAsset": quote_asset,
        "launchMode": launch_mode,
        "mode": if trimmed_mode.is_empty() { "sol" } else { trimmed_mode.as_str() },
        "amount": amount,
        "commitment": "confirmed",
    }))
    .await?;
    Ok(Some(quote))
}

pub async fn compile_sol_to_usd1_topup_transaction(
    rpc_url: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    required_quote_amount: &str,
    label_prefix: &str,
) -> Result<Option<CompiledTransaction>, String> {
    let tip_lamports = parse_decimal_u64(&execution.buyTipSol, 9, "buy tip")?;
    let response: HelperUsd1TopupResponse = run_helper(&json!({
        "action": "compile-sol-to-usd1-topup",
        "rpcUrl": rpc_url,
        "quoteAsset": "usd1",
        "allowAtaCreation": true,
        "commitment": execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
        "requiredQuoteAmount": required_quote_amount,
        "labelPrefix": label_prefix,
        "txFormat": execution.txFormat,
        "slippageBps": slippage_bps_from_percent(&execution.buySlippagePercent)?,
        "txConfig": helper_tx_config(
            Some(configured_default_launch_usd1_topup_compute_unit_limit()),
            priority_fee_sol_to_micro_lamports(&execution.buyPriorityFeeSol)?,
            tip_lamports,
            jito_tip_account,
        ),
    }))
    .await?;
    Ok(response
        .compiledTransaction
        .map(convert_compiled_transaction))
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
    let tip_lamports = u64::try_from(config.tx.jitoTipLamports.max(0)).unwrap_or_default();
    let response: HelperLaunchResponse = run_helper(&json!({
        "action": "build-launch",
        "mode": config.mode,
        "quoteAsset": config.quoteAsset,
        "rpcUrl": rpc_url,
        "commitment": config.execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
        "allowAtaCreation": allow_ata_creation,
        "vanitySecret": config.vanityPrivateKey,
        "txFormat": config.execution.txFormat,
        "slippageBps": slippage_bps_from_percent(&config.execution.buySlippagePercent)?,
        "txConfig": helper_tx_config(
            config
                .tx
                .computeUnitLimit
                .and_then(|value| u64::try_from(value).ok())
                .or_else(|| Some(configured_default_launch_compute_unit_limit())),
            u64::try_from(config.tx.computeUnitPriceMicroLamports.unwrap_or_default().max(0))
                .unwrap_or_default(),
            tip_lamports,
            &config.tx.jitoTipAccount,
        ),
        "token": {
            "name": config.token.name,
            "symbol": config.token.symbol,
            "uri": config.token.uri,
        },
        "devBuy": config.devBuy.as_ref().map(|dev_buy| json!({
            "mode": dev_buy.mode,
            "amount": dev_buy.amount,
            "quoteAsset": config.quoteAsset,
        })),
    }))
    .await?;
    let compiled_transactions = response
        .compiledTransactions
        .into_iter()
        .map(convert_compiled_transaction)
        .collect::<Vec<_>>();
    let mut report = build_report(
        config,
        transport_plan,
        built_at,
        rpc_url.to_string(),
        creator_public_key,
        response.mint.clone(),
        None,
        config_path,
        vec![],
    );
    report.execution.notes.push(
        "Bonk launch assembly uses the Raydium LaunchLab SDK-backed compile bridge.".to_string(),
    );
    if response.atomicCombined {
        report.execution.notes.push(
            "USD1 dev buy was assembled atomically with the launch transaction.".to_string(),
        );
    } else if let Some(reason) = response.atomicFallbackReason.as_ref() {
        report.execution.notes.push(format!(
            "USD1 dev buy uses split launch transactions: {reason}"
        ));
    }
    if let Some(details) = response.usd1LaunchDetails.as_ref() {
        report.bonkUsd1Launch = Some(BonkUsd1LaunchSummary {
            compilePath: details.compilePath.clone(),
            currentQuoteAmount: details.currentQuoteAmount.clone(),
            requiredQuoteAmount: details.requiredQuoteAmount.clone(),
            shortfallQuoteAmount: details.shortfallQuoteAmount.clone(),
            inputSol: details.inputSol.clone(),
            expectedQuoteOut: details.expectedQuoteOut.clone(),
            minQuoteOut: details.minQuoteOut.clone(),
            atomicFallbackReason: response.atomicFallbackReason.clone(),
        });
    }
    if let Some(metrics) = response.usd1QuoteMetrics.as_ref() {
        if let Some(note) = render_usd1_quote_metrics_note(metrics) {
            report.execution.notes.push(note);
        }
    }
    report.transactions = build_transaction_summaries(&compiled_transactions, config.tx.dumpBase64);
    let text = render_report(&report);
    let report = serde_json::to_value(report).map_err(|error| error.to_string())?;
    Ok(Some(NativeBonkArtifacts {
        creation_transactions: compiled_transactions.clone(),
        deferred_setup_transactions: vec![],
        compiled_transactions,
        report,
        text,
        compile_timings: NativeCompileTimings::default(),
        mint: response.mint,
        launch_creator: response.launchCreator,
    }))
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
) -> Result<CompiledTransaction, String> {
    let tip_lamports = resolve_follow_tip_lamports(&execution.buyProvider, &execution.buyTipSol, "buy tip")?;
    let response: HelperFollowBuyResponse = run_helper(&json!({
        "action": "compile-follow-buy",
        "rpcUrl": rpc_url,
        "quoteAsset": quote_asset,
        "commitment": execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
        "allowAtaCreation": allow_ata_creation,
        "mint": mint,
        "buyAmountSol": buy_amount_sol,
        "txFormat": execution.txFormat,
        "slippageBps": slippage_bps_from_percent(&execution.buySlippagePercent)?,
        "txConfig": helper_tx_config(
            Some(configured_default_sniper_buy_compute_unit_limit()),
            priority_fee_sol_to_micro_lamports(&execution.buyPriorityFeeSol)?,
            tip_lamports,
            jito_tip_account,
        ),
    }))
    .await?;
    Ok(convert_compiled_transaction(response.compiledTransaction))
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
) -> Result<CompiledTransaction, String> {
    let tip_lamports = resolve_follow_tip_lamports(&execution.buyProvider, &execution.buyTipSol, "buy tip")?;
    let response: HelperFollowBuyResponse = run_helper(&json!({
        "action": "compile-follow-buy-atomic",
        "mode": launch_mode,
        "rpcUrl": rpc_url,
        "quoteAsset": quote_asset,
        "commitment": execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
        "allowAtaCreation": allow_ata_creation,
        "mint": mint,
        "launchCreator": launch_creator,
        "buyAmountSol": buy_amount_sol,
        "txFormat": execution.txFormat,
        "slippageBps": slippage_bps_from_percent(&execution.buySlippagePercent)?,
        "txConfig": helper_tx_config(
            Some(configured_default_sniper_buy_compute_unit_limit()),
            priority_fee_sol_to_micro_lamports(&execution.buyPriorityFeeSol)?,
            tip_lamports,
            jito_tip_account,
        ),
    }))
    .await?;
    Ok(convert_compiled_transaction(response.compiledTransaction))
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
    let tip_lamports =
        resolve_follow_tip_lamports(&execution.sellProvider, &execution.sellTipSol, "sell tip")?;
    let response: HelperFollowSellResponse = run_helper(&json!({
        "action": "compile-follow-sell",
        "rpcUrl": rpc_url,
        "commitment": execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
        "mint": mint,
        "quoteAsset": quote_asset,
        "sellPercent": sell_percent,
        "exactTokenAmountRaw": token_amount_override.map(|value| value.to_string()),
        "poolId": pool_id_override,
        "mode": launch_mode_override,
        "launchCreator": launch_creator_override,
        "txFormat": execution.txFormat,
        "slippageBps": slippage_bps_from_percent(&execution.sellSlippagePercent)?,
        "txConfig": helper_tx_config(
            Some(configured_default_dev_auto_sell_compute_unit_limit()),
            priority_fee_sol_to_micro_lamports(&execution.sellPriorityFeeSol)?,
            tip_lamports,
            jito_tip_account,
        ),
    }))
    .await?;
    Ok(response
        .compiledTransaction
        .map(convert_compiled_transaction))
}

pub async fn predict_dev_buy_token_amount(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<Option<u64>, String> {
    let response: HelperPredictDevBuyResponse = run_helper(&json!({
        "action": "predict-dev-buy-token-amount",
        "mode": config.mode,
        "quoteAsset": config.quoteAsset,
        "rpcUrl": rpc_url,
        "commitment": config.execution.commitment,
        "slippageBps": slippage_bps_from_percent(&config.execution.buySlippagePercent)?,
        "devBuy": config.devBuy.as_ref().map(|dev_buy| json!({
            "mode": dev_buy.mode,
            "amount": dev_buy.amount,
            "quoteAsset": config.quoteAsset,
        })),
    }))
    .await?;
    response
        .predictedDevBuyTokenAmountRaw
        .map(|value| value.parse::<u64>().map_err(|error| format!("Invalid Bonk predicted dev buy token amount: {error}")))
        .transpose()
}

pub async fn derive_canonical_pool_id(
    quote_asset: &str,
    mint: &str,
) -> Result<String, String> {
    let response: HelperDerivePoolIdResponse = run_helper(&json!({
        "action": "derive-pool-id",
        "quoteAsset": quote_asset,
        "mint": mint,
    }))
    .await?;
    Ok(response.poolId)
}

pub async fn fetch_bonk_market_snapshot(
    rpc_url: &str,
    mint: &str,
    quote_asset: &str,
) -> Result<BonkMarketSnapshot, String> {
    run_helper(&json!({
        "action": "fetch-market-snapshot",
        "rpcUrl": rpc_url,
        "commitment": "processed",
        "mint": mint,
        "quoteAsset": quote_asset,
    }))
    .await
}

pub async fn detect_bonk_import_context(
    rpc_url: &str,
    mint: &str,
) -> Result<Option<BonkImportContext>, String> {
    run_helper(&json!({
        "action": "detect-import-context",
        "rpcUrl": rpc_url,
        "commitment": "processed",
        "mint": mint,
    }))
    .await
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
    let response: HelperWarmStateResponse = run_helper(&json!({
        "action": "warm-state",
        "rpcUrl": rpc_url,
        "commitment": "processed",
    }))
    .await?;
    let launch_defaults = response
        .warmedLaunchDefaults
        .iter()
        .map(|entry| {
            json!({
                "mode": entry.mode,
                "quoteAsset": entry.quoteAsset,
                "platformId": entry.platformId,
                "configId": entry.configId,
                "quoteMint": entry.quoteMint,
            })
        })
        .collect::<Vec<_>>();
    let mut payload = json!({
        "ok": true,
        "launchDefaults": launch_defaults,
        "usd1RoutePoolId": response.usd1RoutePoolId,
        "usd1RouteConfigId": response.usd1RouteConfigId,
    });
    if let Some(metrics) = response.usd1QuoteMetrics.as_ref() {
        payload["usd1QuoteMetrics"] = serde_json::to_value(metrics).unwrap_or(Value::Null);
    }
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
    fn slippage_percent_maps_to_expected_bps() {
        assert_eq!(slippage_bps_from_percent("20").expect("20%"), 2_000);
        assert_eq!(slippage_bps_from_percent("0.5").expect("0.5%"), 50);
        assert_eq!(slippage_bps_from_percent("100").expect("100%"), 10_000);
    }
}
