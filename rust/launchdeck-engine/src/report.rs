#![allow(non_snake_case, dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

use crate::{
    config::{
        NormalizedConfig, NormalizedCreatorFee, NormalizedFollowLaunch, NormalizedRecipient,
        has_launch_follow_up, launch_follow_up_label,
    },
    transport::TransportPlan,
    wallet::list_solana_env_wallets,
};

const DEFAULT_LOOKUP_TABLES: [&str; 2] = [
    "AXVvmhWaaPtV52jqYuTNqp1xRrkbxhfJfeHQKxq5cbvZ",
    "BckPpoRV4h329qAuhTCNoWdWAy2pZSJ89Qu3nuCU1zsj",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionSummary {
    pub index: usize,
    pub programId: String,
    pub keyCount: usize,
    pub writableKeys: usize,
    pub signerKeys: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeSettings {
    pub computeUnitLimit: Option<i64>,
    pub computeUnitPriceMicroLamports: Option<i64>,
    pub jitoTipLamports: i64,
    pub jitoTipAccount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub label: String,
    pub instructionSummary: Vec<InstructionSummary>,
    pub legacyLength: Option<usize>,
    pub legacyBase64Length: Option<usize>,
    pub v0Length: Option<usize>,
    pub v0Base64Length: Option<usize>,
    pub v0AltLength: Option<usize>,
    pub v0AltBase64Length: Option<usize>,
    pub legacyError: Option<String>,
    pub v0Error: Option<String>,
    pub v0AltError: Option<String>,
    pub lookupTablesUsed: Vec<String>,
    pub fitsWithAlts: bool,
    pub exceedsPacketLimit: bool,
    pub feeSettings: FeeSettings,
    pub base64: Option<serde_json::Value>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionItem {
    pub label: String,
    pub format: String,
    pub err: Option<String>,
    pub unitsConsumed: Option<i64>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentItem {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmationSource: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submittedAtMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firstObservedStatus: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firstObservedSlot: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firstObservedAtMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmedAtMs: Option<u128>,
    #[serde(default, alias = "sendObservedBlockHeight")]
    pub sendObservedSlot: Option<u64>,
    #[serde(default, alias = "confirmedObservedBlockHeight")]
    pub confirmedObservedSlot: Option<u64>,
    pub confirmedSlot: Option<u64>,
    pub computeUnitLimit: Option<u64>,
    pub computeUnitPriceMicroLamports: Option<u64>,
    pub inlineTipLamports: Option<u64>,
    pub inlineTipAccount: Option<String>,
    pub bundleId: Option<String>,
    pub attemptedBundleIds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionTimings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmarkMode: Option<String>,
    pub totalElapsedMs: Option<u128>,
    pub backendTotalElapsedMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executionTotalMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observedWallClockMs: Option<u128>,
    pub clientPreRequestMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepareRequestPayloadMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apiRoundTripOverheadMs: Option<u128>,
    pub formToRawConfigMs: Option<u128>,
    pub normalizeConfigMs: Option<u128>,
    pub walletLoadMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transportPlanBuildMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autoFeeResolveMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sameTimeFeeGuardMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followDaemonReadyMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followDaemonReserveMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followDaemonArmMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followDaemonStatusRefreshMs: Option<u128>,
    pub reportBuildMs: Option<u128>,
    pub compileTransactionsMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compileLaunchCreatorPrepMs: Option<u128>,
    pub compileAltLoadMs: Option<u128>,
    pub compileBlockhashFetchMs: Option<u128>,
    pub compileGlobalFetchMs: Option<u128>,
    pub compileFollowUpPrepMs: Option<u128>,
    pub compileTxSerializeMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compileLaunchSerializeMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compileFollowUpSerializeMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compileTipSerializeMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bagsPrepareLaunchMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bagsMetadataUploadMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bagsFeeRecipientResolveMs: Option<u128>,
    pub simulateMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulateLaunchMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulateFollowUpMs: Option<u128>,
    pub sendMs: Option<u128>,
    pub sendSubmitMs: Option<u128>,
    pub sendConfirmMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sendTransportSubmitMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sendTransportConfirmMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sendBundleStatusMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sendWatchFallbackMs: Option<u128>,
    pub bagsSetupSubmitMs: Option<u128>,
    pub bagsSetupConfirmMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bagsSetupGateMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bagsLaunchBuildMs: Option<u128>,
    pub persistReportMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistInitialSnapshotMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistFinalReportUpdateMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followSnapshotFlushMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reportRenderMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reportListRefreshMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backendOtherMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executionOtherMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endToEndOtherMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reportingOverheadMs: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FollowActionTimings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watcherWaitMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligibilityMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postEligibilityToSubmitMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preSignedExpiryCheckMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compileMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submitMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executionTotalMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reportSyncMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reportingOverheadMs: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FollowJobTimings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmarkMode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserveMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cachePrepMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watcherWaitMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligibilityMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triggerCompilePrepMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postEligibilityToSubmitMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preSignedExpiryCheckMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compileMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submitMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executionTotalMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reportSyncMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followSnapshotFlushMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reportingOverheadMs: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkTimingItem {
    pub key: String,
    pub label: String,
    pub valueMs: Option<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default)]
    pub inclusive: bool,
    #[serde(default)]
    pub remainder: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkTimingGroup {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub items: Vec<BenchmarkTimingItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkSentItem {
    pub label: String,
    pub signature: Option<String>,
    pub confirmationStatus: Option<String>,
    #[serde(default, alias = "sendBlockHeight")]
    pub sendSlot: Option<u64>,
    #[serde(default, alias = "confirmedBlockHeight")]
    pub confirmedObservedSlot: Option<u64>,
    #[serde(default, alias = "blocksToConfirm")]
    pub slotsToConfirm: Option<u64>,
    pub confirmedSlot: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkSummary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    pub timings: ExecutionTimings,
    #[serde(default)]
    pub timingGroups: Vec<BenchmarkTimingGroup>,
    pub sent: Vec<BenchmarkSentItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkMode {
    Off,
    Light,
    Full,
}

impl BenchmarkMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Light => "light",
            Self::Full => "full",
        }
    }

    pub fn from_value(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" => Self::Off,
            "light" | "basic" => Self::Light,
            "full" => Self::Full,
            "" => Self::Full,
            _ => Self::Light,
        }
    }
}

pub fn configured_benchmark_mode() -> BenchmarkMode {
    BenchmarkMode::from_value(
        &env::var("LAUNCHDECK_BENCHMARK_MODE").unwrap_or_else(|_| "full".to_string()),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub provider: String,
    pub resolvedProvider: String,
    pub endpointProfile: String,
    pub resolvedEndpointProfile: String,
    pub executionClass: String,
    pub transportType: String,
    pub ordering: String,
    pub autoGas: bool,
    pub launchAutoMode: String,
    pub activePresetId: String,
    pub buyProvider: String,
    pub buyEndpointProfile: String,
    pub sellProvider: String,
    pub sellEndpointProfile: String,
    pub buyAutoGas: bool,
    pub buyAutoMode: String,
    pub requestedSummary: String,
    pub txFormat: String,
    pub commitment: String,
    pub skipPreflight: bool,
    pub trackSendBlockHeight: bool,
    pub maxRetries: u32,
    pub requiresInlineTip: bool,
    pub requiresPriorityFee: bool,
    pub separateTipTransaction: bool,
    pub heliusSenderEndpoint: Option<String>,
    #[serde(default)]
    pub timings: ExecutionTimings,
    pub simulation: Vec<ExecutionItem>,
    pub sent: Vec<SentItem>,
    #[serde(default)]
    pub notes: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedBagsReportConfig {
    pub configType: String,
    pub identityMode: String,
    pub agentUsername: String,
    pub identityVerifiedWallet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonkUsd1LaunchSummary {
    pub compilePath: String,
    pub currentQuoteAmount: String,
    pub requiredQuoteAmount: String,
    pub shortfallQuoteAmount: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputSol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expectedQuoteOut: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minQuoteOut: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atomicFallbackReason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchReport {
    pub builtAt: String,
    pub configPath: Option<String>,
    pub mode: String,
    pub launchpad: String,
    pub rpcUrl: String,
    pub creator: String,
    pub agentAuthority: Option<String>,
    pub mint: String,
    pub cashbackEnabled: bool,
    pub agentEnabled: bool,
    pub feeSharingStatus: String,
    pub creatorFeeReceiver: String,
    pub buybackBps: Option<i64>,
    pub devBuyDescription: String,
    pub metadataUri: Option<String>,
    pub requestedLookupTables: Vec<String>,
    pub loadedLookupTables: Vec<String>,
    pub bundleJitoTip: bool,
    pub transactions: Vec<TransactionSummary>,
    pub execution: ExecutionSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savedSelectedWalletKey: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savedQuoteAsset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savedFollowLaunch: Option<NormalizedFollowLaunch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savedBags: Option<SavedBagsReportConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savedFeeSharingRecipients: Option<Vec<NormalizedRecipient>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savedAgentFeeRecipients: Option<Vec<NormalizedRecipient>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savedCreatorFee: Option<NormalizedCreatorFee>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bonkUsd1Launch: Option<BonkUsd1LaunchSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followDaemon: Option<Value>,
    #[serde(default)]
    pub benchmark: Option<BenchmarkSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outPath: Option<String>,
}

fn summarize_recipients(
    entries: &[NormalizedRecipient],
    fallback_wallet: &str,
    buyback_bps: Option<i64>,
) -> String {
    if !entries.is_empty() {
        return entries
            .iter()
            .map(|entry| {
                let share = entry.shareBps as f64 / 100.0;
                match entry.r#type.as_deref() {
                    Some("agent") => format!("agent buyback ({share}%)"),
                    Some("github") if !entry.githubUsername.is_empty() => {
                        format!("GitHub @{} ({share}%)", entry.githubUsername)
                    }
                    Some("twitter") | Some("x") if !entry.githubUsername.is_empty() => {
                        format!("X @{} ({share}%)", entry.githubUsername)
                    }
                    Some("kick") if !entry.githubUsername.is_empty() => {
                        format!("Kick @{} ({share}%)", entry.githubUsername)
                    }
                    Some("tiktok") if !entry.githubUsername.is_empty() => {
                        format!("TikTok @{} ({share}%)", entry.githubUsername)
                    }
                    _ if !entry.githubUsername.is_empty() => {
                        format!("@{} ({share}%)", entry.githubUsername)
                    }
                    _ if !entry.address.is_empty() => {
                        format!("{} ({share}%)", short_address(&entry.address, 4, 4))
                    }
                    _ => format!("unknown ({share}%)"),
                }
            })
            .collect::<Vec<_>>()
            .join(" + ");
    }
    let buyback = buyback_bps.unwrap_or_default();
    let wallet_share = 10_000 - buyback;
    let mut parts = Vec::new();
    if buyback > 0 {
        parts.push(format!("agent buyback ({}%)", buyback as f64 / 100.0));
    }
    if wallet_share > 0 {
        parts.push(format!(
            "{} ({}%)",
            short_address(fallback_wallet, 4, 4),
            wallet_share as f64 / 100.0
        ));
    }
    parts.join(" + ")
}

fn report_wallet_label(wallet_env_key: &str) -> String {
    let key = wallet_env_key.trim();
    if key.is_empty() {
        return "Wallet".to_string();
    }
    let suffix = key.strip_prefix("SOLANA_PRIVATE_KEY").unwrap_or(key);
    let base = if suffix.is_empty() {
        "Wallet #1".to_string()
    } else if suffix.chars().all(|ch| ch.is_ascii_digit()) {
        format!("Wallet #{suffix}")
    } else {
        key.to_string()
    };
    let custom_name = list_solana_env_wallets()
        .into_iter()
        .find(|wallet| wallet.envKey == key)
        .and_then(|wallet| wallet.customName)
        .unwrap_or_default();
    if custom_name.is_empty() {
        base
    } else {
        format!("{base} {custom_name}")
    }
}

fn render_follow_action_summary(action: &Value) -> Option<String> {
    let kind = action
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let state = action
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let wallet = report_wallet_label(
        action
            .get("walletEnvKey")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    );
    let title = match kind {
        "sniper-buy" => "Sniper buy",
        "sniper-sell" => "Sniper sell",
        "dev-auto-sell" => "Dev auto-sell",
        _ => return None,
    };
    let mut parts = vec![format!("{title}: {wallet}")];
    if let Some(amount) = action.get("buyAmountSol").and_then(Value::as_str)
        && !amount.trim().is_empty()
    {
        parts.push(format!("buy {}", amount.trim()));
    }
    if let Some(percent) = action.get("sellPercent").and_then(Value::as_u64) {
        parts.push(format!("sell {}%", percent));
    }
    if let Some(delay_ms) = action.get("submitDelayMs").and_then(Value::as_u64) {
        parts.push(format!("submit delay={}ms", delay_ms));
    }
    if let Some(delay_ms) = action.get("delayMs").and_then(Value::as_u64) {
        parts.push(format!("delay={}ms", delay_ms));
    }
    if let Some(block_offset) = action.get("targetBlockOffset").and_then(Value::as_u64) {
        parts.push(format!("slot+{}", block_offset));
    }
    if let Some(trigger_key) = action.get("triggerKey").and_then(Value::as_str)
        && !trigger_key.trim().is_empty()
    {
        let trigger_label = if trigger_key.starts_with("market:") {
            "trigger=market-cap"
        } else if trigger_key.starts_with("slot:") {
            "trigger=block-offset"
        } else if trigger_key.starts_with("delay:") {
            "trigger=delay"
        } else if trigger_key.starts_with("submit:") {
            "trigger=submit"
        } else if trigger_key == "confirm" {
            "trigger=confirm"
        } else {
            "trigger=unknown"
        };
        parts.push(trigger_label.to_string());
    }
    if let Some(market_cap) = action.get("marketCap").and_then(Value::as_object) {
        let threshold = market_cap
            .get("threshold")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !threshold.trim().is_empty() {
            parts.push(format!(
                "market ${}",
                format_market_cap_threshold_for_display(threshold)
            ));
        }
        if let Some(timeout_seconds) = market_cap.get("scanTimeoutSeconds").and_then(Value::as_u64)
        {
            parts.push(format!("scan={}s", timeout_seconds));
        }
        if let Some(timeout_action) = market_cap.get("timeoutAction").and_then(Value::as_str)
            && !timeout_action.trim().is_empty()
        {
            parts.push(format!("timeout={}", timeout_action.trim()));
        }
    }
    parts.push(format!("status={state}"));
    if let Some(mode) = action.get("watcherMode").and_then(Value::as_str)
        && !mode.trim().is_empty()
    {
        parts.push(format!("watcher={}", mode.trim()));
    }
    if let Some(signature) = action.get("signature").and_then(Value::as_str)
        && !signature.trim().is_empty()
    {
        parts.push(format!("sig={}", short_address(signature.trim(), 8, 8)));
    }
    if let Some(slots) = action
        .get("slotsToConfirm")
        .or_else(|| action.get("blocksToConfirm"))
        .and_then(Value::as_u64)
    {
        parts.push(format!("slots={slots}"));
    }
    if let Some(error) = action.get("lastError").and_then(Value::as_str)
        && !error.trim().is_empty()
    {
        if error.contains("market-cap scan stopped") {
            parts.push("timeout=stop".to_string());
        }
        parts.push(format!("error={}", error.trim()));
    }
    if let Some(reason) = action.get("watcherFallbackReason").and_then(Value::as_str)
        && !reason.trim().is_empty()
    {
        parts.push(format!("watcher-note={}", reason.trim()));
    }
    Some(parts.join(" | "))
}

fn short_address(value: &str, left: usize, right: usize) -> String {
    if value.is_empty() {
        return "(unknown)".to_string();
    }
    if value.len() <= left + right {
        return value.to_string();
    }
    format!("{}...{}", &value[..left], &value[value.len() - right..])
}

fn format_market_cap_threshold_for_display(value: &str) -> String {
    let trimmed = value.trim();
    if !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return trimmed.to_string();
    }
    let Ok(micros) = trimmed.parse::<u128>() else {
        return trimmed.to_string();
    };
    if micros < 1_000_000 {
        return trimmed.to_string();
    }
    let whole_usd = micros / 1_000_000;
    let fractional_micros = micros % 1_000_000;
    let format_with_suffix = |suffix_value: u128, suffix_label: &str| {
        let whole = whole_usd / suffix_value;
        let remainder = whole_usd % suffix_value;
        if whole >= 100 || remainder == 0 {
            return format!("{whole}{suffix_label}");
        }
        let decimal = (remainder * 10) / suffix_value;
        if decimal == 0 {
            return format!("{whole}{suffix_label}");
        }
        format!("{whole}.{decimal}{suffix_label}")
    };
    if fractional_micros == 0 {
        if whole_usd >= 1_000_000_000_000 {
            return format_with_suffix(1_000_000_000_000, "t");
        }
        if whole_usd >= 1_000_000_000 {
            return format_with_suffix(1_000_000_000, "b");
        }
        if whole_usd >= 1_000_000 {
            return format_with_suffix(1_000_000, "m");
        }
        if whole_usd >= 1_000 {
            return format_with_suffix(1_000, "k");
        }
        return whole_usd.to_string();
    }
    let mut fractional = format!("{fractional_micros:06}");
    while fractional.ends_with('0') {
        fractional.pop();
    }
    if fractional.is_empty() {
        whole_usd.to_string()
    } else {
        format!("{whole_usd}.{fractional}")
    }
}

fn format_duration_ms(value_ms: u128) -> String {
    if value_ms < 1_000 {
        format!("{value_ms}ms")
    } else {
        format!("{:.3}s", value_ms as f64 / 1_000.0)
    }
}

fn parse_lookup_table_addresses(config: &NormalizedConfig) -> Vec<String> {
    let mut values = Vec::new();
    if config.tx.useDefaultLookupTables {
        values.extend(DEFAULT_LOOKUP_TABLES.iter().map(|entry| entry.to_string()));
    }
    values.extend(config.tx.lookupTables.clone());
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.iter().any(|entry| entry == &value) {
            deduped.push(value);
        }
    }
    deduped
}

fn display_transaction_label(label: &str) -> String {
    match label.trim() {
        "follow-up" => "fee-sharing setup".to_string(),
        "agent-setup" => "agent fee setup".to_string(),
        other if other.is_empty() => "transaction".to_string(),
        other => other.to_string(),
    }
}

fn planned_transactions(
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    lookup_tables: &[String],
) -> Vec<TransactionSummary> {
    let mut transactions = vec![TransactionSummary {
        label: "launch".to_string(),
        instructionSummary: vec![],
        legacyLength: None,
        legacyBase64Length: None,
        v0Length: None,
        v0Base64Length: None,
        v0AltLength: None,
        v0AltBase64Length: None,
        legacyError: None,
        v0Error: None,
        v0AltError: None,
        lookupTablesUsed: lookup_tables.to_vec(),
        fitsWithAlts: false,
        exceedsPacketLimit: false,
        feeSettings: FeeSettings {
            computeUnitLimit: config.tx.computeUnitLimit,
            computeUnitPriceMicroLamports: config.tx.computeUnitPriceMicroLamports,
            jitoTipLamports: if transport_plan.separateTipTransaction {
                0
            } else {
                config.tx.jitoTipLamports
            },
            jitoTipAccount: if transport_plan.separateTipTransaction
                || config.tx.jitoTipAccount.is_empty()
            {
                None
            } else {
                Some(config.tx.jitoTipAccount.clone())
            },
        },
        base64: None,
        warnings: vec![],
    }];

    let follow_up = launch_follow_up_label(config);
    if let Some(label) = follow_up {
        transactions.push(TransactionSummary {
            label: display_transaction_label(label),
            instructionSummary: vec![],
            legacyLength: None,
            legacyBase64Length: None,
            v0Length: None,
            v0Base64Length: None,
            v0AltLength: None,
            v0AltBase64Length: None,
            legacyError: None,
            v0Error: None,
            v0AltError: None,
            lookupTablesUsed: lookup_tables.to_vec(),
            fitsWithAlts: false,
            exceedsPacketLimit: false,
            feeSettings: FeeSettings {
                computeUnitLimit: config.tx.computeUnitLimit,
                computeUnitPriceMicroLamports: config.tx.computeUnitPriceMicroLamports,
                jitoTipLamports: if transport_plan.requiresInlineTip {
                    config.tx.jitoTipLamports
                } else {
                    0
                },
                jitoTipAccount: if transport_plan.requiresInlineTip
                    && !config.tx.jitoTipAccount.is_empty()
                {
                    Some(config.tx.jitoTipAccount.clone())
                } else {
                    None
                },
            },
            base64: None,
            warnings: vec![],
        });
    }
    if transport_plan.separateTipTransaction {
        transactions.push(TransactionSummary {
            label: "jito-tip".to_string(),
            instructionSummary: vec![],
            legacyLength: None,
            legacyBase64Length: None,
            v0Length: None,
            v0Base64Length: None,
            v0AltLength: None,
            v0AltBase64Length: None,
            legacyError: None,
            v0Error: None,
            v0AltError: None,
            lookupTablesUsed: lookup_tables.to_vec(),
            fitsWithAlts: false,
            exceedsPacketLimit: false,
            feeSettings: FeeSettings {
                computeUnitLimit: None,
                computeUnitPriceMicroLamports: None,
                jitoTipLamports: config.tx.jitoTipLamports,
                jitoTipAccount: if config.tx.jitoTipAccount.is_empty() {
                    None
                } else {
                    Some(config.tx.jitoTipAccount.clone())
                },
            },
            base64: None,
            warnings: vec![],
        });
    }
    transactions
}

fn requested_summary(config: &NormalizedConfig, transport_plan: &TransportPlan) -> String {
    if config.execution.send {
        if config.execution.simulate {
            return format!(
                "simulate + send ({}, provider={}, class={}, transport={})",
                config.execution.txFormat,
                transport_plan.resolvedProvider,
                transport_plan.executionClass,
                transport_plan.transportType
            );
        }
        return format!(
            "send ({}, provider={}, class={}, transport={})",
            config.execution.txFormat,
            transport_plan.resolvedProvider,
            transport_plan.executionClass,
            transport_plan.transportType
        );
    }
    if config.execution.simulate {
        return format!(
            "simulate ({}, provider={}, class={}, transport={})",
            config.execution.txFormat,
            transport_plan.resolvedProvider,
            transport_plan.executionClass,
            transport_plan.transportType
        );
    }
    "build only".to_string()
}

pub fn build_report(
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    built_at: String,
    rpc_url: String,
    creator: String,
    mint: String,
    agent_authority: Option<String>,
    config_path: Option<String>,
    loaded_lookup_tables: Vec<String>,
) -> LaunchReport {
    let requested_lookup_tables = parse_lookup_table_addresses(config);
    let transactions = planned_transactions(config, transport_plan, &requested_lookup_tables);
    let creator_fee_receiver = match config.mode.as_str() {
        "cashback" => "cashback to traders".to_string(),
        "agent-locked" => "agent buyback escrow (locked after launch)".to_string(),
        "agent-custom" if has_launch_follow_up(config) => summarize_recipients(
            &config.agent.feeRecipients,
            &creator,
            config.agent.buybackBps,
        ),
        _ if config.creatorFee.mode == "github" && !config.creatorFee.githubUsername.is_empty() => {
            format!("GitHub @{}", config.creatorFee.githubUsername)
        }
        _ if config.creatorFee.mode == "github" && !config.creatorFee.githubUserId.is_empty() => {
            format!("GitHub user {}", config.creatorFee.githubUserId)
        }
        _ if config.creatorFee.mode == "wallet" && !config.creatorFee.address.is_empty() => {
            config.creatorFee.address.clone()
        }
        _ => creator.clone(),
    };
    let fee_sharing_status = match config.mode.as_str() {
        "agent-custom" if has_launch_follow_up(config) => {
            "agent custom split bundled post-launch (final)".to_string()
        }
        "agent-custom" => "untouched on launch (configure later manually)".to_string(),
        "agent-unlocked" => "untouched on launch (configure later once)".to_string(),
        "agent-locked" => "locked to agent escrow".to_string(),
        _ if config.launchpad == "bagsapp" && !config.feeSharing.recipients.is_empty() => {
            "configured during Bags setup".to_string()
        }
        _ if has_launch_follow_up(config) => "deferred one-time setup artifact".to_string(),
        _ => "untouched".to_string(),
    };
    let notes = vec![match config.launchpad.as_str() {
        "bonk" => {
            "Rust engine owns validation, runtime state, and API contracts. Native Bonk assembly covers LaunchDeck's Bonk launch modes end-to-end."
        }
        "bagsapp" => {
            "Rust engine owns validation, runtime state, and API contracts. Native Bags assembly covers LaunchDeck's Bags launch flows end-to-end."
        }
        _ => {
            "Rust engine owns validation, runtime state, and API contracts. Native Pump assembly covers LaunchDeck's Pump launch modes end-to-end."
        }
    }
    .to_string()];
    let mut warnings = vec![];
    let has_unverified_provider =
        !crate::providers::get_provider_meta(&transport_plan.resolvedProvider).verified;
    if has_unverified_provider {
        warnings.push(format!(
            "Provider {} is currently marked unverified in this environment.",
            transport_plan.resolvedProvider
        ));
    }

    LaunchReport {
        builtAt: built_at,
        configPath: config_path,
        mode: config.mode.clone(),
        launchpad: config.launchpad.clone(),
        rpcUrl: rpc_url,
        creator: creator.clone(),
        agentAuthority: agent_authority,
        mint,
        cashbackEnabled: config.mode == "cashback",
        agentEnabled: config.mode != "regular" && config.mode != "cashback",
        feeSharingStatus: fee_sharing_status,
        creatorFeeReceiver: creator_fee_receiver,
        buybackBps: if config.mode == "regular" || config.mode == "cashback" {
            None
        } else {
            config.agent.buybackBps
        },
        devBuyDescription: config
            .devBuy
            .as_ref()
            .map(|entry| format!("{}:{}", entry.mode, entry.amount))
            .unwrap_or_else(|| "none".to_string()),
        metadataUri: if config.token.uri.trim().is_empty() {
            None
        } else {
            Some(config.token.uri.clone())
        },
        requestedLookupTables: requested_lookup_tables.clone(),
        loadedLookupTables: loaded_lookup_tables,
        bundleJitoTip: transport_plan.separateTipTransaction,
        transactions,
        execution: ExecutionSummary {
            provider: config.execution.provider.clone(),
            resolvedProvider: transport_plan.resolvedProvider.clone(),
            endpointProfile: config.execution.endpointProfile.clone(),
            resolvedEndpointProfile: transport_plan.resolvedEndpointProfile.clone(),
            executionClass: transport_plan.executionClass.clone(),
            transportType: transport_plan.transportType.clone(),
            ordering: transport_plan.ordering.clone(),
            autoGas: config.execution.autoGas,
            launchAutoMode: config.execution.autoMode.clone(),
            activePresetId: config.presets.activePresetId.clone(),
            buyProvider: config.execution.buyProvider.clone(),
            buyEndpointProfile: config.execution.buyEndpointProfile.clone(),
            sellProvider: config.execution.sellProvider.clone(),
            sellEndpointProfile: config.execution.sellEndpointProfile.clone(),
            buyAutoGas: config.execution.buyAutoGas,
            buyAutoMode: config.execution.buyAutoMode.clone(),
            requestedSummary: requested_summary(config, transport_plan),
            txFormat: config.execution.txFormat.clone(),
            commitment: config.execution.commitment.clone(),
            skipPreflight: transport_plan.skipPreflight,
            trackSendBlockHeight: config.execution.trackSendBlockHeight,
            maxRetries: transport_plan.maxRetries,
            requiresInlineTip: transport_plan.requiresInlineTip,
            requiresPriorityFee: transport_plan.requiresPriorityFee,
            separateTipTransaction: transport_plan.separateTipTransaction,
            heliusSenderEndpoint: transport_plan.heliusSenderEndpoint.clone(),
            timings: ExecutionTimings::default(),
            simulation: vec![],
            sent: vec![],
            notes,
            warnings,
        },
        savedSelectedWalletKey: if config.selectedWalletKey.trim().is_empty() {
            None
        } else {
            Some(config.selectedWalletKey.clone())
        },
        savedQuoteAsset: if config.quoteAsset.trim().is_empty() {
            None
        } else {
            Some(config.quoteAsset.clone())
        },
        savedFollowLaunch: if config.followLaunch.enabled {
            Some(config.followLaunch.clone())
        } else {
            None
        },
        savedBags: if config.launchpad == "bagsapp" {
            Some(SavedBagsReportConfig {
                configType: config.bags.configType.clone(),
                identityMode: config.bags.identityMode.clone(),
                agentUsername: config.bags.agentUsername.clone(),
                identityVerifiedWallet: config.bags.identityVerifiedWallet.clone(),
            })
        } else {
            None
        },
        savedFeeSharingRecipients: if config.feeSharing.recipients.is_empty() {
            None
        } else {
            Some(config.feeSharing.recipients.clone())
        },
        savedAgentFeeRecipients: if config.agent.feeRecipients.is_empty() {
            None
        } else {
            Some(config.agent.feeRecipients.clone())
        },
        savedCreatorFee: Some(config.creatorFee.clone()),
        bonkUsd1Launch: None,
        followDaemon: None,
        benchmark: None,
        outPath: None,
    }
}

fn timing_metric(
    key: &str,
    label: &str,
    value_ms: Option<u128>,
    detail: Option<&str>,
    inclusive: bool,
    remainder: bool,
) -> BenchmarkTimingItem {
    BenchmarkTimingItem {
        key: key.to_string(),
        label: label.to_string(),
        valueMs: value_ms,
        detail: detail.map(str::to_string),
        inclusive,
        remainder,
    }
}

fn sum_known(values: &[Option<u128>]) -> Option<u128> {
    let mut total = 0u128;
    let mut has_any = false;
    for value in values.iter().flatten() {
        total = total.saturating_add(*value);
        has_any = true;
    }
    has_any.then_some(total)
}

fn remaining_time(total: Option<u128>, children: &[Option<u128>]) -> Option<u128> {
    let total = total?;
    let child_sum = sum_known(children)?;
    Some(total.saturating_sub(child_sum))
}

fn pick_first(values: &[Option<u128>]) -> Option<u128> {
    values.iter().flatten().copied().next()
}

pub fn sanitize_execution_timings_for_mode(
    timings: &ExecutionTimings,
    mode: BenchmarkMode,
) -> ExecutionTimings {
    let mut sanitized = timings.clone();
    sanitized.benchmarkMode = Some(mode.as_str().to_string());
    match mode {
        BenchmarkMode::Full => sanitized,
        BenchmarkMode::Light => {
            sanitized.apiRoundTripOverheadMs = None;
            sanitized.transportPlanBuildMs = None;
            sanitized.autoFeeResolveMs = None;
            sanitized.sameTimeFeeGuardMs = None;
            sanitized.followDaemonReadyMs = None;
            sanitized.followDaemonReserveMs = None;
            sanitized.followDaemonArmMs = None;
            sanitized.followDaemonStatusRefreshMs = None;
            sanitized.compileLaunchCreatorPrepMs = None;
            sanitized.compileLaunchSerializeMs = None;
            sanitized.compileFollowUpSerializeMs = None;
            sanitized.compileTipSerializeMs = None;
            sanitized.bagsPrepareLaunchMs = None;
            sanitized.bagsMetadataUploadMs = None;
            sanitized.bagsFeeRecipientResolveMs = None;
            sanitized.simulateLaunchMs = None;
            sanitized.simulateFollowUpMs = None;
            sanitized.sendTransportSubmitMs = None;
            sanitized.sendTransportConfirmMs = None;
            sanitized.sendBundleStatusMs = None;
            sanitized.sendWatchFallbackMs = None;
            sanitized.bagsSetupGateMs = None;
            sanitized.bagsLaunchBuildMs = None;
            sanitized.persistFinalReportUpdateMs = None;
            sanitized.followSnapshotFlushMs = None;
            sanitized.reportRenderMs = None;
            sanitized.reportListRefreshMs = None;
            sanitized.reportingOverheadMs = None;
            sanitized
        }
        BenchmarkMode::Off => ExecutionTimings::default(),
    }
}

pub fn build_benchmark_timing_groups(
    timings: &ExecutionTimings,
    follow_timings: Option<&FollowJobTimings>,
) -> Vec<BenchmarkTimingGroup> {
    let prep_total = sum_known(&[
        timings.prepareRequestPayloadMs,
        timings.formToRawConfigMs,
        timings.normalizeConfigMs,
        timings.walletLoadMs,
        timings.transportPlanBuildMs,
        timings.autoFeeResolveMs,
        timings.sameTimeFeeGuardMs,
        timings.followDaemonReadyMs,
        timings.followDaemonReserveMs,
        timings.followDaemonArmMs,
        timings.followDaemonStatusRefreshMs,
        timings.reportBuildMs,
    ]);
    let compile_other = remaining_time(
        timings.compileTransactionsMs,
        &[
            timings.compileLaunchCreatorPrepMs,
            timings.compileAltLoadMs,
            timings.compileBlockhashFetchMs,
            timings.compileGlobalFetchMs,
            timings.compileFollowUpPrepMs,
            timings.compileLaunchSerializeMs,
            timings.compileFollowUpSerializeMs,
            timings.compileTipSerializeMs,
            timings.compileTxSerializeMs,
            timings.bagsPrepareLaunchMs,
            timings.bagsMetadataUploadMs,
            timings.bagsFeeRecipientResolveMs,
        ],
    );
    let simulate_other = remaining_time(
        timings.simulateMs,
        &[timings.simulateLaunchMs, timings.simulateFollowUpMs],
    );
    let send_other = remaining_time(
        timings.sendMs,
        &[timings.sendSubmitMs, timings.sendConfirmMs],
    );
    let submit_other = remaining_time(
        timings.sendSubmitMs,
        &[
            timings.bagsSetupSubmitMs,
            timings.bagsLaunchBuildMs,
            timings.sendTransportSubmitMs,
            pick_first(&[
                timings.compileLaunchSerializeMs,
                timings.compileFollowUpSerializeMs,
                timings.compileTipSerializeMs,
            ]),
        ],
    );
    let confirm_other = remaining_time(
        timings.sendConfirmMs,
        &[
            pick_first(&[timings.bagsSetupGateMs, timings.bagsSetupConfirmMs]),
            timings.sendTransportConfirmMs,
            timings.sendBundleStatusMs,
            timings.sendWatchFallbackMs,
        ],
    );
    let reporting_total = timings.reportingOverheadMs.or_else(|| {
        sum_known(&[
            timings.persistInitialSnapshotMs.or(timings.persistReportMs),
            timings.persistFinalReportUpdateMs,
            timings.followSnapshotFlushMs,
            timings.reportRenderMs,
            timings.reportListRefreshMs,
        ])
    });
    let execution_total = timings
        .backendTotalElapsedMs
        .map(|backend| backend.saturating_sub(reporting_total.unwrap_or_default()))
        .or(timings.executionTotalMs);
    let backend_other = timings.backendOtherMs.or_else(|| {
        remaining_time(
            execution_total,
            &[
                prep_total,
                timings.compileTransactionsMs,
                timings.simulateMs,
                timings.sendMs,
            ],
        )
    });
    let end_to_end_other = timings.endToEndOtherMs.or_else(|| {
        remaining_time(
            timings.totalElapsedMs,
            &[timings.clientPreRequestMs, timings.backendTotalElapsedMs],
        )
    });
    let top_level = BenchmarkTimingGroup {
        key: "topLevel".to_string(),
        label: "Top-Level Timings".to_string(),
        items: vec![
            timing_metric(
                "totalElapsedMs",
                "End-to-end",
                timings.totalElapsedMs,
                Some("client + backend inclusive"),
                true,
                false,
            ),
            timing_metric(
                "executionTotalMs",
                "Execution total",
                execution_total,
                Some("core execution without reporting overhead"),
                true,
                false,
            ),
            timing_metric(
                "reportingOverheadMs",
                "Reporting overhead",
                reporting_total,
                Some("persist + render + follow snapshot writes"),
                true,
                false,
            ),
            timing_metric(
                "clientPreRequestMs",
                "Client overhead",
                timings.clientPreRequestMs,
                Some("before engine work starts"),
                false,
                false,
            ),
            timing_metric(
                "backendTotalElapsedMs",
                "Backend total",
                timings.backendTotalElapsedMs,
                Some("all engine work observed by the backend"),
                true,
                false,
            ),
            timing_metric(
                "backendOtherMs",
                "Backend remainder",
                backend_other,
                Some("known backend groups do not fully explain this remainder"),
                false,
                true,
            ),
            timing_metric(
                "endToEndOtherMs",
                "End-to-end remainder",
                end_to_end_other,
                Some("known client and backend totals do not fully explain this remainder"),
                false,
                true,
            ),
            timing_metric(
                "observedWallClockMs",
                "Observed wall clock",
                timings.observedWallClockMs,
                Some("reference only, not a speed benchmark"),
                false,
                false,
            ),
        ],
    };
    let prep = BenchmarkTimingGroup {
        key: "prep".to_string(),
        label: "Preparation".to_string(),
        items: vec![
            timing_metric(
                "prepareRequestPayloadMs",
                "Prepare request payload",
                timings.prepareRequestPayloadMs,
                Some("client payload assembly before POST"),
                false,
                false,
            ),
            timing_metric(
                "formToRawConfigMs",
                "Form to raw config",
                timings.formToRawConfigMs,
                Some("UI payload to engine config"),
                false,
                false,
            ),
            timing_metric(
                "normalizeConfigMs",
                "Normalize config",
                timings.normalizeConfigMs,
                Some("validation + normalization"),
                false,
                false,
            ),
            timing_metric(
                "walletLoadMs",
                "Wallet load",
                timings.walletLoadMs,
                Some("wallet/env hydration"),
                false,
                false,
            ),
            timing_metric(
                "transportPlanBuildMs",
                "Transport plan",
                timings.transportPlanBuildMs,
                Some("provider + endpoint routing selection"),
                false,
                false,
            ),
            timing_metric(
                "autoFeeResolveMs",
                "Auto-fee resolve",
                timings.autoFeeResolveMs,
                Some("priority fee + tip estimation and caps"),
                false,
                false,
            ),
            timing_metric(
                "sameTimeFeeGuardMs",
                "Same-time fee guard",
                timings.sameTimeFeeGuardMs,
                Some("same-time launch/snipe safety adjustments"),
                false,
                false,
            ),
            timing_metric(
                "followDaemonReadyMs",
                "Follow ready check",
                timings.followDaemonReadyMs,
                Some("follow daemon readiness validation"),
                false,
                false,
            ),
            timing_metric(
                "followDaemonReserveMs",
                "Follow reserve",
                timings.followDaemonReserveMs,
                Some("follow daemon reservation request"),
                false,
                false,
            ),
            timing_metric(
                "followDaemonArmMs",
                "Follow arm",
                timings.followDaemonArmMs,
                Some("follow daemon arm request"),
                false,
                false,
            ),
            timing_metric(
                "followDaemonStatusRefreshMs",
                "Follow status refresh",
                timings.followDaemonStatusRefreshMs,
                Some("final follow daemon status read"),
                false,
                false,
            ),
            timing_metric(
                "reportBuildMs",
                "Report build",
                timings.reportBuildMs,
                Some("initial report object assembly"),
                false,
                false,
            ),
        ],
    };
    let compile = BenchmarkTimingGroup {
        key: "compile".to_string(),
        label: "Compile Breakdown".to_string(),
        items: vec![
            timing_metric(
                "compileTransactionsMs",
                "Compile total",
                timings.compileTransactionsMs,
                Some("inclusive compile stage total"),
                true,
                false,
            ),
            timing_metric(
                "compileLaunchCreatorPrepMs",
                "Launch creator prep",
                timings.compileLaunchCreatorPrepMs,
                Some("launch creator + pre-instruction resolution"),
                false,
                false,
            ),
            timing_metric(
                "compileAltLoadMs",
                "ALT load",
                timings.compileAltLoadMs,
                Some("lookup table fetch"),
                false,
                false,
            ),
            timing_metric(
                "compileBlockhashFetchMs",
                "Blockhash fetch",
                timings.compileBlockhashFetchMs,
                Some("latest blockhash fetch"),
                false,
                false,
            ),
            timing_metric(
                "compileGlobalFetchMs",
                "Global fetch",
                timings.compileGlobalFetchMs,
                Some("shared launch context"),
                false,
                false,
            ),
            timing_metric(
                "compileFollowUpPrepMs",
                "Follow-up prep",
                timings.compileFollowUpPrepMs,
                Some("follow action planning"),
                false,
                false,
            ),
            timing_metric(
                "compileLaunchSerializeMs",
                "Launch serialize",
                timings.compileLaunchSerializeMs,
                Some("launch transaction serialization"),
                false,
                false,
            ),
            timing_metric(
                "compileFollowUpSerializeMs",
                "Follow-up serialize",
                timings.compileFollowUpSerializeMs,
                Some("follow-up transaction serialization"),
                false,
                false,
            ),
            timing_metric(
                "compileTipSerializeMs",
                "Tip serialize",
                timings.compileTipSerializeMs,
                Some("tip transaction serialization"),
                false,
                false,
            ),
            timing_metric(
                "compileTxSerializeMs",
                "Serialize total",
                timings.compileTxSerializeMs,
                Some("all transaction serialization work"),
                false,
                false,
            ),
            timing_metric(
                "bagsPrepareLaunchMs",
                "Bags prepare",
                timings.bagsPrepareLaunchMs,
                Some("native/helper prepare-launch total"),
                false,
                false,
            ),
            timing_metric(
                "bagsMetadataUploadMs",
                "Bags metadata upload",
                timings.bagsMetadataUploadMs,
                Some("native/helper metadata upload only"),
                false,
                false,
            ),
            timing_metric(
                "bagsFeeRecipientResolveMs",
                "Bags recipient resolve",
                timings.bagsFeeRecipientResolveMs,
                Some("native/helper fee-recipient resolution"),
                false,
                false,
            ),
            timing_metric(
                "compileOtherMs",
                "Compile remainder",
                compile_other,
                Some("known compile children do not fully explain this remainder"),
                false,
                true,
            ),
        ],
    };
    let simulate = BenchmarkTimingGroup {
        key: "simulate".to_string(),
        label: "Simulate Breakdown".to_string(),
        items: vec![
            timing_metric(
                "simulateMs",
                "Simulate total",
                timings.simulateMs,
                Some("inclusive simulation stage total"),
                true,
                false,
            ),
            timing_metric(
                "simulateLaunchMs",
                "Launch simulate",
                timings.simulateLaunchMs,
                Some("launch simulation only"),
                false,
                false,
            ),
            timing_metric(
                "simulateFollowUpMs",
                "Follow-up simulate",
                timings.simulateFollowUpMs,
                Some("follow-up simulation only"),
                false,
                false,
            ),
            timing_metric(
                "simulateOtherMs",
                "Simulate remainder",
                simulate_other,
                Some("known simulate children do not fully explain this remainder"),
                false,
                true,
            ),
        ],
    };
    let send = BenchmarkTimingGroup {
        key: "send".to_string(),
        label: "Send Breakdown".to_string(),
        items: vec![
            timing_metric(
                "sendMs",
                "Send total",
                timings.sendMs,
                Some("inclusive stage total"),
                true,
                false,
            ),
            timing_metric(
                "sendSubmitMs",
                "Submit total",
                timings.sendSubmitMs,
                Some("all transaction submissions"),
                true,
                false,
            ),
            timing_metric(
                "sendTransportSubmitMs",
                "Transport submit",
                timings.sendTransportSubmitMs,
                Some("send RPC/provider submit only"),
                false,
                false,
            ),
            timing_metric(
                "bagsSetupSubmitMs",
                "Setup submit",
                timings.bagsSetupSubmitMs,
                Some("setup transaction submit"),
                false,
                false,
            ),
            timing_metric(
                "bagsLaunchBuildMs",
                "Bags launch build",
                timings.bagsLaunchBuildMs,
                Some("build final launch transaction after setup"),
                false,
                false,
            ),
            timing_metric(
                "sendSubmitOtherMs",
                "Submit remainder",
                submit_other,
                Some("known submit children do not fully explain this remainder"),
                false,
                true,
            ),
            timing_metric(
                "sendConfirmMs",
                "Confirm total",
                timings.sendConfirmMs,
                Some("all confirmation waits"),
                true,
                false,
            ),
            timing_metric(
                "sendTransportConfirmMs",
                "Transport confirm",
                timings.sendTransportConfirmMs,
                Some("provider confirmation path only"),
                false,
                false,
            ),
            timing_metric(
                "sendBundleStatusMs",
                "Bundle status",
                timings.sendBundleStatusMs,
                Some("Jito bundle status polling"),
                false,
                false,
            ),
            timing_metric(
                "sendWatchFallbackMs",
                "Watch fallback",
                timings.sendWatchFallbackMs,
                Some("websocket fallback or poll wait"),
                false,
                false,
            ),
            timing_metric(
                "bagsSetupGateMs",
                "Setup gate",
                pick_first(&[timings.bagsSetupGateMs, timings.bagsSetupConfirmMs]),
                Some("wait before final launch build"),
                false,
                false,
            ),
            timing_metric(
                "sendConfirmOtherMs",
                "Confirm remainder",
                confirm_other,
                Some("known confirm children do not fully explain this remainder"),
                false,
                true,
            ),
            timing_metric(
                "sendOtherMs",
                "Send remainder",
                send_other,
                Some("known send children do not fully explain this remainder"),
                false,
                true,
            ),
        ],
    };
    let reporting = BenchmarkTimingGroup {
        key: "reporting".to_string(),
        label: "Reporting Overhead".to_string(),
        items: vec![
            timing_metric(
                "persistInitialSnapshotMs",
                "Persist first snapshot",
                timings.persistInitialSnapshotMs.or(timings.persistReportMs),
                Some("first crash-safe snapshot write"),
                false,
                false,
            ),
            timing_metric(
                "persistFinalReportUpdateMs",
                "Persist final update",
                timings.persistFinalReportUpdateMs,
                Some("final report enrichment write"),
                false,
                false,
            ),
            timing_metric(
                "followSnapshotFlushMs",
                "Follow snapshot flush",
                timings.followSnapshotFlushMs,
                Some("follow daemon snapshot write"),
                false,
                false,
            ),
            timing_metric(
                "reportRenderMs",
                "Report render",
                timings.reportRenderMs,
                Some("text report rendering"),
                false,
                false,
            ),
            timing_metric(
                "reportListRefreshMs",
                "Report list refresh",
                timings.reportListRefreshMs,
                Some("report index/list refresh"),
                false,
                false,
            ),
        ],
    };
    let mut groups = vec![top_level, prep, compile, simulate, send, reporting];
    if let Some(follow) = follow_timings {
        groups.push(BenchmarkTimingGroup {
            key: "followDaemon".to_string(),
            label: "Follow Daemon Timing".to_string(),
            items: vec![
                timing_metric(
                    "followReserveMs",
                    "Reserve",
                    follow.reserveMs,
                    Some("initial follow reservation"),
                    false,
                    false,
                ),
                timing_metric(
                    "followArmMs",
                    "Arm",
                    follow.armMs,
                    Some("follow arm request"),
                    false,
                    false,
                ),
                timing_metric(
                    "followCachePrepMs",
                    "Cache prep",
                    follow.cachePrepMs,
                    Some("follow buy cache preparation"),
                    false,
                    false,
                ),
                timing_metric(
                    "followWatcherWaitMs",
                    "Watcher wait",
                    follow.watcherWaitMs,
                    Some("slot/signature/market watcher waits"),
                    false,
                    false,
                ),
                timing_metric(
                    "followEligibilityMs",
                    "Eligibility",
                    follow.eligibilityMs,
                    Some("time to become runnable"),
                    false,
                    false,
                ),
                timing_metric(
                    "followTriggerCompilePrepMs",
                    "Trigger compile prep",
                    follow.triggerCompilePrepMs,
                    Some("shared trigger-time warm/compile preparation"),
                    false,
                    false,
                ),
                timing_metric(
                    "followPostEligibilityToSubmitMs",
                    "Eligible to submit",
                    follow.postEligibilityToSubmitMs,
                    Some("gap between eligible and submit start"),
                    false,
                    false,
                ),
                timing_metric(
                    "followPreSignedExpiryCheckMs",
                    "Pre-signed expiry check",
                    follow.preSignedExpiryCheckMs,
                    Some("block-height safety check before sending pre-signed actions"),
                    false,
                    false,
                ),
                timing_metric(
                    "followCompileMs",
                    "Compile",
                    follow.compileMs,
                    Some("follow compile work"),
                    false,
                    false,
                ),
                timing_metric(
                    "followSubmitMs",
                    "Submit",
                    follow.submitMs,
                    Some("follow action submission"),
                    false,
                    false,
                ),
                timing_metric(
                    "followConfirmMs",
                    "Confirm",
                    follow.confirmMs,
                    Some("follow action confirmation"),
                    false,
                    false,
                ),
                timing_metric(
                    "followExecutionTotalMs",
                    "Execution total",
                    follow.executionTotalMs,
                    Some("aggregated follow execution timing"),
                    true,
                    false,
                ),
                timing_metric(
                    "followReportSyncMs",
                    "Report sync",
                    follow.reportSyncMs,
                    Some("follow snapshot/report sync work"),
                    false,
                    false,
                ),
                timing_metric(
                    "followSnapshotFlushMs",
                    "Snapshot flush",
                    follow.followSnapshotFlushMs,
                    Some("coalesced snapshot file writes"),
                    false,
                    false,
                ),
                timing_metric(
                    "followReportingOverheadMs",
                    "Reporting overhead",
                    follow.reportingOverheadMs,
                    Some("follow reporting overhead total"),
                    false,
                    false,
                ),
            ],
        });
    }
    groups
}

pub fn render_report(report: &LaunchReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Mode: {}", report.mode));
    lines.push(format!(
        "Config: {}",
        report.configPath.as_deref().unwrap_or("(cli only)")
    ));
    lines.push(format!("RPC: {}", report.rpcUrl));
    lines.push(format!("Creator: {}", report.creator));
    lines.push(format!(
        "Agent authority: {}",
        report.agentAuthority.as_deref().unwrap_or("(not used)")
    ));
    lines.push(format!("Mint: {}", report.mint));
    lines.push(format!("Cashback enabled: {}", report.cashbackEnabled));
    lines.push(format!("Tokenized Agent enabled: {}", report.agentEnabled));
    lines.push(format!(
        "Creator fee receiver: {}",
        report.creatorFeeReceiver
    ));
    lines.push(format!("Fee sharing: {}", report.feeSharingStatus));
    lines.push(format!(
        "Buyback BPS: {}",
        report
            .buybackBps
            .map(|value| value.to_string())
            .unwrap_or_else(|| "(not used)".to_string())
    ));
    lines.push(format!("Dev buy: {}", report.devBuyDescription));
    if let Some(metadata_uri) = &report.metadataUri {
        lines.push(format!("Metadata URI: {}", metadata_uri));
    }
    lines.push(format!("Launchpad: {}", report.launchpad));
    lines.push(format!("Provider: {}", report.execution.resolvedProvider));
    if !report.execution.resolvedEndpointProfile.is_empty() {
        lines.push(format!(
            "Endpoint profile: {}",
            report.execution.resolvedEndpointProfile
        ));
    }
    lines.push(format!(
        "Execution class: {}",
        report.execution.executionClass
    ));
    lines.push(format!("Transport: {}", report.execution.transportType));
    lines.push(format!("Ordering: {}", report.execution.ordering));
    lines.push(format!(
        "Skip preflight: {} | max retries: {}",
        report.execution.skipPreflight, report.execution.maxRetries
    ));
    lines.push(format!("Track send slot: {}", report.execution.trackSendBlockHeight));
    if let Some(endpoint) = &report.execution.heliusSenderEndpoint {
        lines.push(format!("Helius Sender responder: {}", endpoint));
    }
    if report.bundleJitoTip {
        lines.push("Jito tip handling: separate bundled tip transaction".to_string());
    }
    lines.push(format!("Execution: {}", report.execution.requestedSummary));
    if let Some(follow) = report.followDaemon.as_ref().and_then(Value::as_object) {
        let enabled = follow
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if enabled {
            lines.push("Follow daemon:".to_string());
            lines.push(
                "  Advisory only: timing suggestions do not change configured follow settings."
                    .to_string(),
            );
            if let Some(transport) = follow.get("transport").and_then(Value::as_str)
                && !transport.is_empty()
            {
                lines.push(format!("  Transport: {transport}"));
            }
            if let Some(job) = follow.get("job").and_then(Value::as_object) {
                if let Some(state) = job.get("state").and_then(Value::as_str) {
                    lines.push(format!("  Job state: {state}"));
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
                    let configured_actions = actions
                        .iter()
                        .filter_map(render_follow_action_summary)
                        .collect::<Vec<_>>();
                    if !configured_actions.is_empty() {
                        lines.push("  Configured actions:".to_string());
                        for summary in configured_actions {
                            lines.push(format!("  - {}", summary));
                        }
                    }
                }
            }
        }
    }
    if let Some(benchmark) = &report.benchmark {
        lines.push("Benchmark:".to_string());
        let timings = &benchmark.timings;
        let mut timing_parts = Vec::new();
        if let Some(value) = timings.totalElapsedMs {
            timing_parts.push(format!("endToEnd={value}ms"));
        }
        if let Some(value) = timings.backendTotalElapsedMs {
            timing_parts.push(format!("backendTotal={value}ms"));
        }
        if let Some(value) = timings.clientPreRequestMs {
            timing_parts.push(format!("clientOverhead={value}ms"));
        }
        if let Some(value) = timings.formToRawConfigMs {
            timing_parts.push(format!("formToRaw={value}ms"));
        }
        if let Some(value) = timings.normalizeConfigMs {
            timing_parts.push(format!("normalize={value}ms"));
        }
        if let Some(value) = timings.walletLoadMs {
            timing_parts.push(format!("wallet={value}ms"));
        }
        if let Some(value) = timings.reportBuildMs {
            timing_parts.push(format!("reportBuild={value}ms"));
        }
        if let Some(value) = timings.compileTransactionsMs {
            timing_parts.push(format!("compileTotal={value}ms"));
        }
        if let Some(value) = timings.compileAltLoadMs {
            timing_parts.push(format!("altLoad={value}ms"));
        }
        if let Some(value) = timings.compileBlockhashFetchMs {
            timing_parts.push(format!("blockhash={value}ms"));
        }
        if let Some(value) = timings.compileGlobalFetchMs {
            timing_parts.push(format!("global={value}ms"));
        }
        if let Some(value) = timings.compileFollowUpPrepMs {
            timing_parts.push(format!("followUpPrep={value}ms"));
        }
        if let Some(value) = timings.compileTxSerializeMs {
            timing_parts.push(format!("serializeTx={value}ms"));
        }
        if let Some(value) = timings.simulateMs {
            timing_parts.push(format!("simulate={value}ms"));
        }
        if let Some(value) = timings.sendMs {
            timing_parts.push(format!("sendTotal={value}ms"));
        }
        if let Some(value) = timings.sendSubmitMs {
            timing_parts.push(format!("submitTotal={value}ms"));
        }
        if let Some(value) = timings.sendConfirmMs {
            timing_parts.push(format!("confirmTotal={value}ms"));
        }
        if let Some(value) = timings.bagsSetupSubmitMs {
            timing_parts.push(format!("setupSubmit={value}ms"));
        }
        if let Some(value) = timings.bagsSetupConfirmMs {
            timing_parts.push(format!("setupConfirm={value}ms"));
        }
        if let Some(value) = timings.bagsSetupGateMs {
            timing_parts.push(format!("setupGate={value}ms"));
        }
        if let Some(value) = timings.bagsLaunchBuildMs {
            timing_parts.push(format!("bagsLaunchBuild={value}ms"));
        }
        if let Some(value) = timings.persistReportMs {
            timing_parts.push(format!("persistReport={value}ms"));
        }
        if !timing_parts.is_empty() {
            lines.push(format!("  Timings: {}", timing_parts.join(" | ")));
        }
        for sent in &benchmark.sent {
            let mut sent_parts = Vec::new();
            if let Some(value) = sent.sendSlot {
                sent_parts.push(format!("observed send slot={value}"));
            }
            if let Some(value) = sent.confirmedSlot {
                sent_parts.push(format!("confirmed slot={value}"));
            }
            if let Some(value) = sent.confirmedObservedSlot {
                sent_parts.push(format!("observed confirm slot={value}"));
            }
            if let Some(value) = sent.slotsToConfirm {
                sent_parts.push(format!("observed slots to confirm={value}"));
            }
            if !sent_parts.is_empty() {
                lines.push(format!(
                    "  {}: {}",
                    display_transaction_label(&sent.label),
                    sent_parts.join(" | ")
                ));
            }
        }
    }
    lines.push(String::new());
    lines.push("Transactions:".to_string());
    for tx in &report.transactions {
        lines.push(format!(
            "- {}: {} instructions | legacy={} | v0={} | v0+alt={}",
            display_transaction_label(&tx.label),
            tx.instructionSummary.len(),
            tx.legacyLength
                .map(|value| match tx.legacyBase64Length {
                    Some(encoded) => format!("{value} bytes / {encoded} b64"),
                    None => format!("{value} bytes"),
                })
                .unwrap_or_else(|| "n/a".to_string()),
            tx.v0Length
                .map(|value| match tx.v0Base64Length {
                    Some(encoded) => format!("{value} bytes / {encoded} b64"),
                    None => format!("{value} bytes"),
                })
                .unwrap_or_else(|| "n/a".to_string()),
            tx.v0AltLength
                .map(|value| match tx.v0AltBase64Length {
                    Some(encoded) => format!("{value} bytes / {encoded} b64"),
                    None => format!("{value} bytes"),
                })
                .unwrap_or_else(|| "n/a".to_string())
        ));
        if !tx.lookupTablesUsed.is_empty() {
            lines.push(format!(
                "  lookup tables: {}",
                tx.lookupTablesUsed.join(", ")
            ));
        }
        for warning in &tx.warnings {
            lines.push(format!("  note: {warning}"));
        }
    }
    if !report.execution.sent.is_empty() {
        lines.push(String::new());
        lines.push("Sent:".to_string());
        for sent in &report.execution.sent {
            let mut summary = format!(
                "- {}: signature={} | status={}",
                display_transaction_label(&sent.label),
                sent.signature.as_deref().unwrap_or("(missing)"),
                sent.confirmationStatus.as_deref().unwrap_or("(pending)")
            );
            if let Some(source) = sent.confirmationSource.as_deref() {
                summary.push_str(&format!(" | via={source}"));
            }
            if let Some(slot) = sent.sendObservedSlot {
                summary.push_str(&format!(" | observed send slot={slot}"));
            }
            if let Some(slot) = sent.confirmedSlot {
                summary.push_str(&format!(" | confirmed slot={slot}"));
            }
            if let Some(slot) = sent.confirmedObservedSlot {
                summary.push_str(&format!(" | observed confirm slot={slot}"));
            }
            if let (Some(send_slot), Some(confirmed_slot)) = (
                sent.sendObservedSlot,
                sent.confirmedObservedSlot,
            ) {
                summary.push_str(&format!(
                    " | observed slots to confirm={}",
                    confirmed_slot.saturating_sub(send_slot)
                ));
            }
            if let (Some(submitted_at), Some(first_seen_at)) =
                (sent.submittedAtMs, sent.firstObservedAtMs)
            {
                summary.push_str(&format!(
                    " | first seen {} after submit",
                    format_duration_ms(first_seen_at.saturating_sub(submitted_at))
                ));
            }
            if let Some(status) = sent.firstObservedStatus.as_deref() {
                summary.push_str(&format!(" | first status={status}"));
            }
            if let Some(slot) = sent.firstObservedSlot {
                summary.push_str(&format!(" | first slot={slot}"));
            }
            if let (Some(submitted_at), Some(confirmed_at)) =
                (sent.submittedAtMs, sent.confirmedAtMs)
            {
                summary.push_str(&format!(
                    " | confirmed seen {} after submit",
                    format_duration_ms(confirmed_at.saturating_sub(submitted_at))
                ));
            }
            lines.push(summary);
        }
    }
    if !report.execution.warnings.is_empty() {
        lines.push(String::new());
        lines.push("Warnings:".to_string());
        for warning in &report.execution.warnings {
            lines.push(format!("- {}", warning));
        }
    }
    if !report.execution.notes.is_empty() {
        lines.push(String::new());
        lines.push("Notes:".to_string());
        for note in &report.execution.notes {
            lines.push(format!("- {}", note));
        }
    }
    if let Some(usd1) = &report.bonkUsd1Launch {
        lines.push(String::new());
        lines.push("Bonk USD1 Launch:".to_string());
        lines.push(format!("  Compile path: {}", usd1.compilePath));
        lines.push(format!(
            "  Wallet USD1: current={} | required={} | shortfall={}",
            usd1.currentQuoteAmount, usd1.requiredQuoteAmount, usd1.shortfallQuoteAmount
        ));
        if let Some(input_sol) = usd1.inputSol.as_deref() {
            let mut quote_parts = vec![format!("input SOL={input_sol}")];
            if let Some(expected) = usd1.expectedQuoteOut.as_deref() {
                quote_parts.push(format!("expected USD1 out={expected}"));
            }
            if let Some(min_out) = usd1.minQuoteOut.as_deref() {
                quote_parts.push(format!("min USD1 out={min_out}"));
            }
            lines.push(format!("  Top-up quote: {}", quote_parts.join(" | ")));
        }
        if let Some(reason) = usd1.atomicFallbackReason.as_deref() {
            lines.push(format!("  Fallback: {reason}"));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{RawConfig, normalize_raw_config};

    fn regular_config() -> crate::config::NormalizedConfig {
        let mut raw = RawConfig {
            mode: "regular".to_string(),
            launchpad: "pump".to_string(),
            ..RawConfig::default()
        };
        raw.token.name = "LaunchDeck".to_string();
        raw.token.symbol = "LDECK".to_string();
        raw.token.uri = "ipfs://fixture".to_string();
        raw.tx.computeUnitPriceMicroLamports = Some(serde_json::Value::from(1));
        raw.tx.jitoTipLamports = Some(serde_json::Value::from(200_000));
        raw.tx.jitoTipAccount = "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE".to_string();
        raw.execution.skipPreflight = Some(serde_json::Value::Bool(true));
        raw.execution.provider = "helius-sender".to_string();
        raw.execution.buyProvider = "helius-sender".to_string();
        raw.execution.sellProvider = "helius-sender".to_string();
        normalize_raw_config(raw).expect("normalized config")
    }

    #[test]
    fn agent_unlocked_plans_single_launch_transaction() {
        let mut config = regular_config();
        config.mode = "agent-unlocked".to_string();
        config.agent.buybackBps = Some(2_500);

        let report = build_report(
            &config,
            &crate::transport::build_transport_plan(&config.execution, 1),
            "2026-03-27T00:00:00Z".to_string(),
            "https://rpc.example".to_string(),
            "creator".to_string(),
            "mint".to_string(),
            Some("creator".to_string()),
            None,
            vec![],
        );

        assert_eq!(report.transactions.len(), 1);
        assert_eq!(report.transactions[0].label, "launch");
    }

    #[test]
    fn report_uses_actual_loaded_lookup_tables() {
        let mut config = regular_config();
        config.tx.useDefaultLookupTables = true;
        config.tx.lookupTables = vec!["CustomLookup1111111111111111111111111111111".to_string()];

        let report = build_report(
            &config,
            &crate::transport::build_transport_plan(&config.execution, 2),
            "2026-03-27T00:00:00Z".to_string(),
            "https://rpc.example".to_string(),
            "creator".to_string(),
            "mint".to_string(),
            Some("creator".to_string()),
            None,
            vec!["LoadedLookup2222222222222222222222222222222".to_string()],
        );

        assert_eq!(
            report.requestedLookupTables,
            vec![
                "AXVvmhWaaPtV52jqYuTNqp1xRrkbxhfJfeHQKxq5cbvZ".to_string(),
                "BckPpoRV4h329qAuhTCNoWdWAy2pZSJ89Qu3nuCU1zsj".to_string(),
                "CustomLookup1111111111111111111111111111111".to_string()
            ]
        );
        assert_eq!(
            report.loadedLookupTables,
            vec!["LoadedLookup2222222222222222222222222222222".to_string()]
        );
    }

    #[test]
    fn benchmark_groups_reconcile_execution_totals() {
        let timings = ExecutionTimings {
            benchmarkMode: Some("full".to_string()),
            totalElapsedMs: Some(818),
            backendTotalElapsedMs: Some(700),
            executionTotalMs: Some(660),
            clientPreRequestMs: Some(118),
            prepareRequestPayloadMs: Some(12),
            formToRawConfigMs: Some(18),
            normalizeConfigMs: Some(20),
            walletLoadMs: Some(30),
            reportBuildMs: Some(20),
            compileTransactionsMs: Some(210),
            compileLaunchCreatorPrepMs: Some(20),
            compileAltLoadMs: Some(40),
            compileBlockhashFetchMs: Some(25),
            compileGlobalFetchMs: Some(15),
            compileFollowUpPrepMs: Some(10),
            compileLaunchSerializeMs: Some(30),
            compileFollowUpSerializeMs: Some(20),
            compileTipSerializeMs: Some(10),
            compileTxSerializeMs: Some(60),
            simulateMs: Some(40),
            sendMs: Some(250),
            sendSubmitMs: Some(90),
            sendTransportSubmitMs: Some(75),
            sendConfirmMs: Some(160),
            sendTransportConfirmMs: Some(120),
            sendWatchFallbackMs: Some(20),
            persistInitialSnapshotMs: Some(12),
            persistFinalReportUpdateMs: Some(18),
            reportRenderMs: Some(10),
            reportingOverheadMs: Some(40),
            ..ExecutionTimings::default()
        };
        let groups = build_benchmark_timing_groups(&timings, None);
        let top_level = groups
            .iter()
            .find(|group| group.key == "topLevel")
            .expect("top level group");
        let execution_total = top_level
            .items
            .iter()
            .find(|item| item.key == "executionTotalMs")
            .and_then(|item| item.valueMs)
            .expect("execution total");
        let reporting_overhead = top_level
            .items
            .iter()
            .find(|item| item.key == "reportingOverheadMs")
            .and_then(|item| item.valueMs)
            .expect("reporting overhead");
        let backend_remainder = top_level
            .items
            .iter()
            .find(|item| item.key == "backendOtherMs")
            .and_then(|item| item.valueMs)
            .expect("backend remainder");
        assert_eq!(execution_total, 660);
        assert_eq!(reporting_overhead, 40);
        assert_eq!(backend_remainder, 60);
    }

    #[test]
    fn benchmark_mode_off_strips_detailed_timings() {
        let timings = ExecutionTimings {
            benchmarkMode: Some("full".to_string()),
            totalElapsedMs: Some(500),
            backendTotalElapsedMs: Some(420),
            executionTotalMs: Some(400),
            clientPreRequestMs: Some(80),
            compileTransactionsMs: Some(140),
            sendTransportSubmitMs: Some(40),
            persistInitialSnapshotMs: Some(12),
            ..ExecutionTimings::default()
        };
        let sanitized = sanitize_execution_timings_for_mode(&timings, BenchmarkMode::Off);
        assert_eq!(sanitized.benchmarkMode, None);
        assert_eq!(sanitized.totalElapsedMs, None);
        assert_eq!(sanitized.backendTotalElapsedMs, None);
        assert_eq!(sanitized.executionTotalMs, None);
        assert_eq!(sanitized.clientPreRequestMs, None);
        assert_eq!(sanitized.persistInitialSnapshotMs, None);
        assert_eq!(sanitized.compileTransactionsMs, None);
        assert_eq!(sanitized.sendTransportSubmitMs, None);
    }

    #[test]
    fn formats_market_cap_thresholds_for_display() {
        assert_eq!(
            format_market_cap_threshold_for_display("100000000000"),
            "100k"
        );
        assert_eq!(
            format_market_cap_threshold_for_display("250000000"),
            "250"
        );
    }
}
