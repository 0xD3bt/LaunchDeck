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
        configured_default_sniper_buy_compute_unit_limit, validate_launchpad_support,
    },
    helper_worker::{
        HelperWorkerClient, HelperWorkerConfig, HelperWorkerError, helper_worker_enabled,
    },
    pump_native::{LaunchQuote, NativeCompileTimings},
    report::{InstructionSummary, LaunchReport, TransactionSummary, build_report, render_report},
    rpc::CompiledTransaction,
    transport::TransportPlan,
};

const PACKET_LIMIT_BYTES: usize = 1232;
const PRIORITY_FEE_PRICE_BASE_COMPUTE_UNIT_LIMIT: u64 = 1_000_000;
const DEFAULT_HELPER_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_HELPER_MAX_CONCURRENCY: usize = 4;
const DEFAULT_BAGS_SETUP_JITO_TIP_CAP_LAMPORTS: u64 = 1_000_000;
const DEFAULT_BAGS_SETUP_JITO_TIP_MIN_LAMPORTS: u64 = 1_000;
const DEFAULT_AUTO_FEE_JITO_TIP_PERCENTILE: &str = "p99";

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
pub struct NativeBagsArtifacts {
    pub compiled_transactions: Vec<CompiledTransaction>,
    pub report: Value,
    pub text: String,
    pub compile_timings: NativeCompileTimings,
    pub mint: String,
    pub launch_creator: String,
}

#[derive(Debug, Clone)]
pub struct PreparedBagsSendArtifacts {
    pub native_artifacts: NativeBagsArtifacts,
    pub config_key: String,
    pub metadata_uri: String,
    pub setup_bundles: Vec<Vec<CompiledTransaction>>,
    pub setup_transactions: Vec<CompiledTransaction>,
    pub fee_estimate: BagsFeeEstimateSnapshot,
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
    setupBundles: Vec<HelperBundleResponse>,
    #[serde(default)]
    setupTransactions: Vec<HelperCompiledTransaction>,
    compiledTransactions: Vec<HelperCompiledTransaction>,
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
}

#[derive(Debug, Deserialize)]
struct HelperFollowBuyResponse {
    compiledTransaction: HelperCompiledTransaction,
}

#[derive(Debug, Deserialize)]
struct HelperFollowSellResponse {
    compiledTransaction: Option<HelperCompiledTransaction>,
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
    Ok(project_root()?.join("scripts/bags-launchpad.js"))
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

fn follow_tip_lamports_for_provider(provider: &str, tip_sol: &str, label: &str) -> Result<u64, String> {
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

pub async fn estimate_bags_fee_market(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<BagsFeeEstimateSnapshot, String> {
    let requested_tip_lamports =
        u64::try_from(config.tx.jitoTipLamports.max(0)).unwrap_or_default();
    run_helper(&json!({
        "action": "estimate-fees",
        "rpcUrl": rpc_url,
        "commitment": config.execution.commitment,
        "requestedTipLamports": requested_tip_lamports,
        "tipPolicy": {
            "setupJitoTipCapLamports": bags_setup_jito_tip_cap_lamports(),
            "setupJitoTipMinLamports": bags_setup_jito_tip_min_lamports(),
            "setupJitoTipPercentile": bags_setup_jito_tip_percentile(),
        },
    }))
    .await
}

pub async fn quote_launch(
    rpc_url: &str,
    launch_mode: &str,
    mode: &str,
    amount: &str,
) -> Result<Option<LaunchQuote>, String> {
    if amount.trim().is_empty() {
        return Ok(None);
    }
    let quote: Option<LaunchQuote> = run_helper(&json!({
        "action": "quote",
        "rpcUrl": rpc_url,
        "launchMode": launch_mode,
        "mode": mode,
        "amount": amount,
        "commitment": "confirmed",
    }))
    .await?;
    Ok(quote)
}

pub async fn try_compile_native_bags(
    rpc_url: &str,
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    wallet_secret: &[u8],
    built_at: String,
    creator_public_key: String,
    config_path: Option<String>,
) -> Result<Option<NativeBagsArtifacts>, String> {
    if config.launchpad != "bagsapp" {
        return Ok(None);
    }
    validate_launchpad_support(config).map_err(|error| error.to_string())?;
    let fee_estimate = estimate_bags_fee_market(rpc_url, config).await?;
    let response: HelperLaunchResponse = run_helper(&json!({
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
            fee_estimate.setupJitoTipLamports,
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
            "shareBps": entry.shareBps,
        })).collect::<Vec<_>>(),
        "devBuy": config.devBuy.as_ref().map(|dev_buy| json!({
            "mode": dev_buy.mode,
            "amount": dev_buy.amount,
        })),
        "identityLabel": if config.bags.identityMode == "linked" {
            if config.bags.agentUsername.trim().is_empty() {
                "Linked Bags Identity".to_string()
            } else {
                format!("Linked Bags Identity (@{})", config.bags.agentUsername.trim())
            }
        } else {
            "Wallet Only".to_string()
        },
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
    report
        .execution
        .notes
        .push("Bags launch assembly uses the hosted Bags API / SDK compile bridge.".to_string());
    report.execution.notes.push(
        "Bags setup follows Bags-native behavior: setup bundles use Jito-style bundle submission and plain setup transactions use direct RPC, while optional setup tips are baked into the Bags API-generated transactions; the final launch transaction still uses the selected LaunchDeck transport."
            .to_string(),
    );
    append_bags_fee_estimate_notes(&mut report, &fee_estimate);
    if !response.configKey.trim().is_empty() {
        report.execution.notes.push(format!(
            "Bags fee-share config created for this launch: {}",
            response.configKey
        ));
    }
    if !response.identityLabel.trim().is_empty() {
        report
            .execution
            .notes
            .push(format!("Identity: {}", response.identityLabel.trim()));
    }
    report.transactions = build_transaction_summaries(&compiled_transactions, config.tx.dumpBase64);
    let text = render_report(&report);
    let mut report = serde_json::to_value(report).map_err(|error| error.to_string())?;
    if !response.metadataUri.trim().is_empty() {
        report["metadataUri"] = Value::String(response.metadataUri.clone());
    }
    report["bagsSetupFeeEstimate"] =
        serde_json::to_value(&fee_estimate).map_err(|error| error.to_string())?;
    Ok(Some(NativeBagsArtifacts {
        compiled_transactions,
        report,
        text,
        compile_timings: NativeCompileTimings::default(),
        mint: response.mint,
        launch_creator: response.launchCreator,
    }))
}

pub async fn prepare_native_bags_send(
    rpc_url: &str,
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    wallet_secret: &[u8],
    built_at: String,
    creator_public_key: String,
    config_path: Option<String>,
) -> Result<PreparedBagsSendArtifacts, String> {
    validate_launchpad_support(config).map_err(|error| error.to_string())?;
    let fee_estimate = estimate_bags_fee_market(rpc_url, config).await?;
    let response: HelperLaunchResponse = run_helper(&json!({
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
            fee_estimate.setupJitoTipLamports,
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
            "shareBps": entry.shareBps,
        })).collect::<Vec<_>>(),
        "devBuy": config.devBuy.as_ref().map(|dev_buy| json!({
            "mode": dev_buy.mode,
            "amount": dev_buy.amount,
        })),
        "identityLabel": if config.bags.identityMode == "linked" {
            if config.bags.agentUsername.trim().is_empty() {
                "Linked Bags Identity".to_string()
            } else {
                format!("Linked Bags Identity (@{})", config.bags.agentUsername.trim())
            }
        } else {
            "Wallet Only".to_string()
        },
    }))
    .await?;
    let compiled_transactions = response
        .compiledTransactions
        .clone()
        .into_iter()
        .map(convert_compiled_transaction)
        .collect::<Vec<_>>();
    let setup_bundles = response
        .setupBundles
        .into_iter()
        .map(|bundle| {
            let _ = bundle.label;
            bundle
                .compiledTransactions
                .into_iter()
                .map(convert_compiled_transaction)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let setup_transactions = response
        .setupTransactions
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
    report
        .execution
        .notes
        .push("Bags launch assembly uses the hosted Bags API / SDK compile bridge.".to_string());
    report.execution.notes.push(
        "Bags setup follows Bags-native behavior: setup bundles use Jito-style bundle submission and plain setup transactions use direct RPC, while optional setup tips are baked into the Bags API-generated transactions; the final launch transaction still uses the selected LaunchDeck transport."
            .to_string(),
    );
    append_bags_fee_estimate_notes(&mut report, &fee_estimate);
    if !response.configKey.trim().is_empty() {
        report.execution.notes.push(format!(
            "Bags fee-share config prepared for this launch: {}",
            response.configKey
        ));
    }
    report.execution.notes.push(
        "Bags launch is sent in two tracked phases: fee-share config setup first, then the token creation transaction."
            .to_string(),
    );
    if !response.identityLabel.trim().is_empty() {
        report
            .execution
            .notes
            .push(format!("Identity: {}", response.identityLabel.trim()));
    }
    report.transactions = build_transaction_summaries(&compiled_transactions, config.tx.dumpBase64);
    let text = render_report(&report);
    let mut report = serde_json::to_value(report).map_err(|error| error.to_string())?;
    if !response.metadataUri.trim().is_empty() {
        report["metadataUri"] = Value::String(response.metadataUri.clone());
    }
    report["bagsSetupFeeEstimate"] =
        serde_json::to_value(&fee_estimate).map_err(|error| error.to_string())?;
    Ok(PreparedBagsSendArtifacts {
        native_artifacts: NativeBagsArtifacts {
            compiled_transactions,
            report,
            text,
            compile_timings: NativeCompileTimings::default(),
            mint: response.mint.clone(),
            launch_creator: response.launchCreator.clone(),
        },
        config_key: response.configKey,
        metadata_uri: response.metadataUri,
        setup_bundles,
        setup_transactions,
        fee_estimate,
    })
}

pub async fn compile_launch_transaction(
    rpc_url: &str,
    config: &NormalizedConfig,
    wallet_secret: &[u8],
    mint: &str,
    config_key: &str,
    metadata_uri: &str,
) -> Result<CompiledTransaction, String> {
    let tip_lamports = u64::try_from(config.tx.jitoTipLamports.max(0)).unwrap_or_default();
    let response: HelperLaunchTransactionResponse = run_helper(&json!({
        "action": "build-launch-transaction",
        "rpcUrl": rpc_url,
        "commitment": config.execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
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
            config.execution.jitodontfront,
            uses_single_bundle_tip_last_tx(&config.execution.provider, &config.execution.mevMode),
        ),
        "metadataUri": metadata_uri,
        "mint": mint,
        "configKey": config_key,
        "devBuy": config.devBuy.as_ref().map(|dev_buy| json!({
            "mode": dev_buy.mode,
            "amount": dev_buy.amount,
        })),
    }))
    .await?;
    Ok(convert_compiled_transaction(response.compiledTransaction))
}

pub fn summarize_transactions(
    compiled_transactions: &[CompiledTransaction],
    dump_base64: bool,
) -> Vec<TransactionSummary> {
    build_transaction_summaries(compiled_transactions, dump_base64)
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
) -> Result<CompiledTransaction, String> {
    let response: HelperFollowBuyResponse = run_helper(&json!({
        "action": "compile-follow-buy",
        "rpcUrl": rpc_url,
        "commitment": execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
        "mint": mint,
        "buyAmountSol": buy_amount_sol,
        "txFormat": execution.txFormat,
        "slippageBps": slippage_bps_from_percent(&execution.buySlippagePercent)?,
        "txConfig": helper_tx_config(
            Some(configured_default_sniper_buy_compute_unit_limit()),
            priority_fee_sol_to_micro_lamports(&execution.buyPriorityFeeSol)?,
            follow_tip_lamports_for_provider(
                &execution.buyProvider,
                &execution.buyTipSol,
                "buy tip",
            )?,
            jito_tip_account,
            execution.buyJitodontfront,
            uses_single_bundle_tip_last_tx(&execution.buyProvider, &execution.buyMevMode),
        ),
    }))
    .await?;
    Ok(convert_compiled_transaction(response.compiledTransaction))
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
) -> Result<Option<CompiledTransaction>, String> {
    let response: HelperFollowSellResponse = run_helper(&json!({
        "action": "compile-follow-sell",
        "rpcUrl": rpc_url,
        "commitment": execution.commitment,
        "ownerSecret": decode_secret_base64(wallet_secret),
        "mint": mint,
        "sellPercent": sell_percent,
        "txFormat": execution.txFormat,
        "slippageBps": slippage_bps_from_percent(&execution.sellSlippagePercent)?,
        "txConfig": helper_tx_config(
            Some(configured_default_dev_auto_sell_compute_unit_limit()),
            priority_fee_sol_to_micro_lamports(&execution.sellPriorityFeeSol)?,
            follow_tip_lamports_for_provider(
                &execution.sellProvider,
                &execution.sellTipSol,
                "sell tip",
            )?,
            jito_tip_account,
            execution.sellJitodontfront,
            uses_single_bundle_tip_last_tx(&execution.sellProvider, &execution.sellMevMode),
        ),
    }))
    .await?;
    Ok(response
        .compiledTransaction
        .map(convert_compiled_transaction))
}

pub async fn fetch_bags_market_snapshot(
    rpc_url: &str,
    mint: &str,
) -> Result<BagsMarketSnapshot, String> {
    run_helper(&json!({
        "action": "fetch-market-snapshot",
        "rpcUrl": rpc_url,
        "commitment": "processed",
        "mint": mint,
    }))
    .await
}

pub async fn detect_bags_import_context(
    rpc_url: &str,
    mint: &str,
) -> Result<Option<BagsImportContext>, String> {
    run_helper(&json!({
        "action": "detect-import-context",
        "rpcUrl": rpc_url,
        "commitment": "processed",
        "mint": mint,
    }))
    .await
}

pub async fn poll_bags_market_cap_lamports(
    rpc_url: &str,
    mint: &str,
) -> Result<Option<u64>, String> {
    let snapshot = fetch_bags_market_snapshot(rpc_url, mint).await?;
    let value = snapshot
        .marketCapLamports
        .parse::<u64>()
        .map_err(|error| format!("Invalid Bags market cap response: {error}"))?;
    Ok(Some(value))
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
    fn slippage_percent_maps_to_expected_bps() {
        assert_eq!(slippage_bps_from_percent("20").expect("20%"), 2_000);
        assert_eq!(slippage_bps_from_percent("0.5").expect("0.5%"), 50);
        assert_eq!(slippage_bps_from_percent("100").expect("100%"), 10_000);
    }
}
