#![allow(non_snake_case, dead_code)]

use crate::{
    app_logs::{record_error, record_info, record_warn},
    config::{NormalizedExecution, NormalizedFollowLaunch},
    fs_utils::{atomic_write, quarantine_corrupt_file},
    report::{FollowActionTimings, FollowJobTimings, configured_benchmark_mode},
    rpc::CompiledTransaction,
    transport::TransportPlan,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex, RwLock},
    time::sleep,
};

pub const FOLLOW_SCHEMA_VERSION: u32 = 1;
pub const FOLLOW_JOB_SCHEMA_VERSION: u32 = 2;
pub const FOLLOW_RESPONSE_SCHEMA_VERSION: u32 = 1;
const DEFAULT_LOCAL_AUTH_TOKEN: &str = "4815927603149027";
const RESTART_STALE_JOB_MAX_AGE_MS: u128 = 5 * 60 * 1000;

fn follow_job_schema_version() -> u32 {
    FOLLOW_JOB_SCHEMA_VERSION
}

fn follow_response_schema_version() -> u32 {
    FOLLOW_RESPONSE_SCHEMA_VERSION
}

fn json_values_equal<T: Serialize>(left: &T, right: &T) -> bool {
    match (serde_json::to_value(left), serde_json::to_value(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

#[derive(Debug, Clone, Serialize)]
struct StableFollowActionIdentity {
    actionId: String,
    kind: FollowActionKind,
    walletEnvKey: String,
    buyAmountSol: Option<String>,
    sellPercent: Option<u8>,
    submitDelayMs: Option<u64>,
    targetBlockOffset: Option<u8>,
    delayMs: Option<u64>,
    marketCap: Option<FollowMarketCapTrigger>,
    jitterMs: Option<u64>,
    feeJitterBps: Option<u16>,
    precheckRequired: bool,
    requireConfirmation: bool,
    skipIfTokenBalancePositive: bool,
    triggerKey: Option<String>,
    orderIndex: u32,
    poolId: Option<String>,
    preSignedTransactions: Vec<CompiledTransaction>,
}

#[derive(Debug, Clone, Serialize)]
struct StableFollowReservePayloadIdentity {
    actions: Vec<StableFollowActionIdentity>,
    deferredSetupTransactions: Vec<CompiledTransaction>,
}

fn stable_identity_trigger_key(action: &FollowActionRecord) -> Option<String> {
    if matches!(action.kind, FollowActionKind::SniperSell) {
        return Some(trigger_key_for_action(action));
    }
    normalized_trigger_key_value(action.triggerKey.as_deref(), action.targetBlockOffset)
}

fn stable_action_identity(action: &FollowActionRecord) -> StableFollowActionIdentity {
    StableFollowActionIdentity {
        actionId: action.actionId.clone(),
        kind: action.kind.clone(),
        walletEnvKey: action.walletEnvKey.clone(),
        buyAmountSol: action.buyAmountSol.clone(),
        sellPercent: action.sellPercent,
        submitDelayMs: action.submitDelayMs,
        targetBlockOffset: action.targetBlockOffset,
        delayMs: action.delayMs,
        marketCap: action.marketCap.clone(),
        jitterMs: action.jitterMs,
        feeJitterBps: action.feeJitterBps,
        precheckRequired: action.precheckRequired,
        requireConfirmation: action.requireConfirmation,
        skipIfTokenBalancePositive: action.skipIfTokenBalancePositive,
        triggerKey: stable_identity_trigger_key(action),
        orderIndex: action.orderIndex,
        poolId: action.poolId.clone(),
        preSignedTransactions: action.preSignedTransactions.clone(),
    }
}

fn stable_reserve_payload_fingerprint(
    action_identity: &[StableFollowActionIdentity],
    deferred_setup_transactions: &[CompiledTransaction],
) -> Result<String, String> {
    serde_json::to_string(&StableFollowReservePayloadIdentity {
        actions: action_identity.to_vec(),
        deferredSetupTransactions: deferred_setup_transactions.to_vec(),
    })
    .map_err(|error| format!("Failed to serialize follow reserve payload identity: {error}"))
}

fn should_prune_job_on_restart(job: &FollowJobRecord, now: u128) -> bool {
    let freshest_ms = job.updatedAtMs.max(job.createdAtMs);
    now.saturating_sub(freshest_ms) > RESTART_STALE_JOB_MAX_AGE_MS
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FollowJobState {
    Reserved,
    Armed,
    Running,
    Completed,
    CompletedWithFailures,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FollowActionState {
    Queued,
    Armed,
    Eligible,
    Running,
    Sent,
    Confirmed,
    Stopped,
    Failed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FollowActionKind {
    SniperBuy,
    DevAutoSell,
    SniperSell,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BagsLaunchMetadata {
    #[serde(default)]
    pub configKey: String,
    #[serde(default)]
    pub migrationFeeOption: Option<i64>,
    #[serde(default)]
    pub expectedMigrationFamily: String,
    #[serde(default)]
    pub expectedDammConfigKey: String,
    #[serde(default)]
    pub expectedDammDerivationMode: String,
    #[serde(default)]
    pub preMigrationDbcPoolAddress: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FollowWatcherHealth {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowMarketCapTrigger {
    pub direction: String,
    pub threshold: String,
    pub scanTimeoutSeconds: u64,
    pub timeoutAction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DeferredSetupState {
    Queued,
    Running,
    Sent,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredSetupRecord {
    pub transactions: Vec<CompiledTransaction>,
    pub state: DeferredSetupState,
    #[serde(default)]
    pub attemptCount: u32,
    #[serde(default)]
    pub signatures: Vec<String>,
    #[serde(default)]
    pub submittedAtMs: Option<u128>,
    #[serde(default)]
    pub confirmedAtMs: Option<u128>,
    #[serde(default)]
    pub lastError: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowActionRecord {
    pub actionId: String,
    pub kind: FollowActionKind,
    pub walletEnvKey: String,
    pub state: FollowActionState,
    pub buyAmountSol: Option<String>,
    pub sellPercent: Option<u8>,
    pub submitDelayMs: Option<u64>,
    pub targetBlockOffset: Option<u8>,
    pub delayMs: Option<u64>,
    pub marketCap: Option<FollowMarketCapTrigger>,
    pub jitterMs: Option<u64>,
    pub feeJitterBps: Option<u16>,
    pub precheckRequired: bool,
    pub requireConfirmation: bool,
    #[serde(default)]
    pub skipIfTokenBalancePositive: bool,
    pub attemptCount: u32,
    pub scheduledForMs: Option<u128>,
    #[serde(default)]
    pub eligibleAtMs: Option<u128>,
    pub submitStartedAtMs: Option<u128>,
    pub submittedAtMs: Option<u128>,
    pub confirmedAtMs: Option<u128>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub endpointProfile: Option<String>,
    #[serde(default)]
    pub transportType: Option<String>,
    #[serde(default)]
    pub watcherMode: Option<String>,
    #[serde(default)]
    pub watcherFallbackReason: Option<String>,
    #[serde(default, alias = "sendObservedBlockHeight")]
    pub sendObservedSlot: Option<u64>,
    #[serde(default, alias = "confirmedObservedBlockHeight")]
    pub confirmedObservedSlot: Option<u64>,
    #[serde(default)]
    pub confirmedTokenBalanceRaw: Option<String>,
    #[serde(default, alias = "eligibilityObservedBlockHeight")]
    pub eligibilityObservedSlot: Option<u64>,
    #[serde(default, alias = "blocksToConfirm")]
    pub slotsToConfirm: Option<u64>,
    pub signature: Option<String>,
    pub explorerUrl: Option<String>,
    pub endpoint: Option<String>,
    pub bundleId: Option<String>,
    pub lastError: Option<String>,
    #[serde(default)]
    pub triggerKey: Option<String>,
    #[serde(default)]
    pub orderIndex: u32,
    #[serde(default)]
    pub preSignedTransactions: Vec<CompiledTransaction>,
    #[serde(default)]
    pub poolId: Option<String>,
    #[serde(default)]
    pub timings: FollowActionTimings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowJobRecord {
    #[serde(default = "follow_job_schema_version")]
    pub schemaVersion: u32,
    pub traceId: String,
    pub jobId: String,
    pub state: FollowJobState,
    pub createdAtMs: u128,
    pub updatedAtMs: u128,
    pub launchpad: String,
    pub quoteAsset: String,
    #[serde(default)]
    pub launchMode: String,
    pub selectedWalletKey: String,
    pub execution: NormalizedExecution,
    pub tokenMayhemMode: bool,
    pub jitoTipAccount: String,
    #[serde(default)]
    pub buyTipAccount: String,
    #[serde(default)]
    pub sellTipAccount: String,
    #[serde(default)]
    pub preferPostSetupCreatorVaultForSell: bool,
    pub mint: Option<String>,
    pub launchCreator: Option<String>,
    pub launchSignature: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub launchTransactionSubscribeAccountRequired: Vec<String>,
    pub submitAtMs: Option<u128>,
    #[serde(default, alias = "sendObservedBlockHeight")]
    pub sendObservedSlot: Option<u64>,
    #[serde(default, alias = "confirmedObservedBlockHeight")]
    pub confirmedObservedSlot: Option<u64>,
    pub reportPath: Option<String>,
    pub transportPlan: Option<TransportPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bagsLaunch: Option<BagsLaunchMetadata>,
    pub followLaunch: NormalizedFollowLaunch,
    pub actions: Vec<FollowActionRecord>,
    #[serde(default)]
    pub reservedPayloadFingerprint: String,
    #[serde(default)]
    pub deferredSetup: Option<DeferredSetupRecord>,
    pub cancelRequested: bool,
    pub lastError: Option<String>,
    #[serde(default)]
    pub timings: FollowJobTimings,
}

fn should_use_post_setup_creator_vault(
    job_prefers_post_setup_creator_vault: bool,
    action: &FollowActionRecord,
    mev_mode: &str,
) -> bool {
    job_prefers_post_setup_creator_vault
        && (action.targetBlockOffset.unwrap_or_default() > 0
            || mev_mode.trim().eq_ignore_ascii_case("secure"))
}

pub fn should_use_post_setup_creator_vault_for_sell(
    job_prefers_post_setup_creator_vault: bool,
    action: &FollowActionRecord,
    mev_mode: &str,
) -> bool {
    should_use_post_setup_creator_vault(job_prefers_post_setup_creator_vault, action, mev_mode)
}

pub fn should_use_post_setup_creator_vault_for_buy(
    job_prefers_post_setup_creator_vault: bool,
    action: &FollowActionRecord,
    mev_mode: &str,
) -> bool {
    should_use_post_setup_creator_vault(job_prefers_post_setup_creator_vault, action, mev_mode)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowDaemonHealth {
    pub running: bool,
    pub statePath: PathBuf,
    pub version: String,
    pub pid: Option<u32>,
    pub startedAtMs: Option<u128>,
    pub controlTransport: String,
    pub controlUrl: Option<String>,
    pub updatedAtMs: u128,
    pub queueDepth: usize,
    pub activeJobs: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxActiveJobs: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxConcurrentCompiles: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxConcurrentSends: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub availableCompileSlots: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub availableSendSlots: Option<usize>,
    pub slotWatcher: FollowWatcherHealth,
    #[serde(default)]
    pub slotWatcherMode: Option<String>,
    pub signatureWatcher: FollowWatcherHealth,
    #[serde(default)]
    pub signatureWatcherMode: Option<String>,
    pub marketWatcher: FollowWatcherHealth,
    #[serde(default)]
    pub marketWatcherMode: Option<String>,
    pub lastError: Option<String>,
    pub watchEndpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowDaemonStateFile {
    pub schemaVersion: u32,
    pub health: FollowDaemonHealth,
    pub jobs: Vec<FollowJobRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowReserveRequest {
    pub traceId: String,
    pub launchpad: String,
    pub quoteAsset: String,
    #[serde(default)]
    pub launchMode: String,
    pub selectedWalletKey: String,
    pub followLaunch: NormalizedFollowLaunch,
    pub execution: NormalizedExecution,
    pub tokenMayhemMode: bool,
    pub jitoTipAccount: String,
    #[serde(default)]
    pub buyTipAccount: String,
    #[serde(default)]
    pub sellTipAccount: String,
    #[serde(default)]
    pub preferPostSetupCreatorVaultForSell: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bagsLaunch: Option<BagsLaunchMetadata>,
    #[serde(default)]
    pub prebuiltActions: Vec<FollowActionRecord>,
    #[serde(default)]
    pub deferredSetupTransactions: Vec<CompiledTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowArmRequest {
    pub traceId: String,
    pub mint: String,
    pub launchCreator: String,
    pub launchSignature: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub launchTransactionSubscribeAccountRequired: Vec<String>,
    pub submitAtMs: u128,
    #[serde(default, alias = "sendObservedBlockHeight")]
    pub sendObservedSlot: Option<u64>,
    #[serde(default, alias = "confirmedObservedBlockHeight")]
    pub confirmedObservedSlot: Option<u64>,
    pub reportPath: Option<String>,
    pub transportPlan: TransportPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowCancelRequest {
    pub traceId: String,
    pub actionId: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FollowStopAllRequest {
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowReadyRequest {
    pub followLaunch: NormalizedFollowLaunch,
    pub quoteAsset: String,
    pub execution: NormalizedExecution,
    pub watchEndpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowReadyResponse {
    pub ok: bool,
    pub ready: bool,
    pub watchEndpoint: Option<String>,
    pub requiredWebsocket: bool,
    pub reason: Option<String>,
    pub health: FollowDaemonHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowJobResponse {
    #[serde(default = "follow_response_schema_version")]
    pub schemaVersion: u32,
    pub ok: bool,
    pub job: Option<FollowJobRecord>,
    pub jobs: Vec<FollowJobRecord>,
    pub health: FollowDaemonHealth,
}

#[derive(Debug, Clone)]
pub struct FollowDaemonStore {
    pub state_path: PathBuf,
    pub inner: Arc<RwLock<FollowDaemonStateFile>>,
    persist_lock: Arc<Mutex<()>>,
}

#[derive(Debug, Clone)]
pub struct FollowDaemonClient {
    pub baseUrl: String,
    client: Client,
    authToken: Option<String>,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn base_trigger_key_for_action(action: &FollowActionRecord) -> String {
    if let Some(trigger) = &action.marketCap {
        return format!(
            "market:{}:{}:{}:{}",
            trigger.direction, trigger.threshold, trigger.scanTimeoutSeconds, trigger.timeoutAction
        );
    }
    if let Some(offset) = action.targetBlockOffset {
        return format!("slot:{offset}");
    }
    if action.requireConfirmation {
        return "confirm".to_string();
    }
    if let Some(delay_ms) = action.delayMs {
        return format!("delay:{delay_ms}");
    }
    if let Some(delay_ms) = action.submitDelayMs {
        return format!("submit:{delay_ms}");
    }
    "submit:0".to_string()
}

fn trigger_key_for_action(action: &FollowActionRecord) -> String {
    let base = base_trigger_key_for_action(action);
    if matches!(action.kind, FollowActionKind::SniperSell) {
        return format!("sniper-sell:{}:{base}", action.actionId);
    }
    base
}

fn normalized_trigger_key_value(
    trigger_key: Option<&str>,
    target_block_offset: Option<u8>,
) -> Option<String> {
    if let Some(trigger_key) = trigger_key {
        let trimmed = trigger_key.trim();
        if let Some(offset) = trimmed.strip_prefix("block:") {
            return Some(format!("slot:{offset}"));
        }
        if trimmed.is_empty() {
            return target_block_offset.map(|offset| format!("slot:{offset}"));
        }
        return Some(trimmed.to_string());
    }
    target_block_offset.map(|offset| format!("slot:{offset}"))
}

fn canonicalize_action_trigger_key(action: &mut FollowActionRecord) {
    action.triggerKey = if matches!(action.kind, FollowActionKind::SniperSell) {
        Some(trigger_key_for_action(action))
    } else {
        normalized_trigger_key_value(action.triggerKey.as_deref(), action.targetBlockOffset)
            .or_else(|| Some(trigger_key_for_action(action)))
    };
}

fn canonicalize_job_trigger_keys(job: &mut FollowJobRecord) {
    for action in &mut job.actions {
        canonicalize_action_trigger_key(action);
    }
}

fn is_retryable_persist_error(error: &std::io::Error) -> bool {
    matches!(error.raw_os_error(), Some(5 | 32 | 33 | 1224))
        || matches!(
            error.kind(),
            std::io::ErrorKind::Interrupted
                | std::io::ErrorKind::WouldBlock
                | std::io::ErrorKind::PermissionDenied
        )
}

fn configured_follow_daemon_auth_token() -> Option<String> {
    let token = std::env::var("LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN")
        .unwrap_or_else(|_| DEFAULT_LOCAL_AUTH_TOKEN.to_string());
    let trimmed = token.trim();
    if trimmed.is_empty() {
        Some(DEFAULT_LOCAL_AUTH_TOKEN.to_string())
    } else {
        Some(trimmed.to_string())
    }
}

pub fn build_action_records(follow: &NormalizedFollowLaunch) -> Vec<FollowActionRecord> {
    let mut actions = follow
        .snipes
        .iter()
        .enumerate()
        .filter(|(_, snipe)| snipe.enabled)
        .map(|(index, snipe)| {
            let mut action = FollowActionRecord {
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
                precheckRequired: follow.constraints.blockOnRequiredPrechecks,
                requireConfirmation: false,
                skipIfTokenBalancePositive: snipe.skipIfTokenBalancePositive,
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
                orderIndex: index as u32,
                preSignedTransactions: vec![],
                poolId: None,
                timings: FollowActionTimings::default(),
            };
            action.triggerKey = Some(trigger_key_for_action(&action));
            action
        })
        .collect::<Vec<_>>();
    if let Some(dev_auto_sell) = &follow.devAutoSell
        && dev_auto_sell.enabled
    {
        let mut action = FollowActionRecord {
            actionId: dev_auto_sell.actionId.clone(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: dev_auto_sell.walletEnvKey.clone(),
            state: FollowActionState::Queued,
            buyAmountSol: None,
            sellPercent: Some(dev_auto_sell.percent),
            submitDelayMs: None,
            targetBlockOffset: dev_auto_sell.targetBlockOffset,
            delayMs: dev_auto_sell.delayMs,
            marketCap: dev_auto_sell
                .marketCap
                .as_ref()
                .map(|trigger| FollowMarketCapTrigger {
                    direction: trigger.direction.clone(),
                    threshold: trigger.threshold.clone(),
                    scanTimeoutSeconds: trigger.scanTimeoutSeconds,
                    timeoutAction: trigger.timeoutAction.clone(),
                }),
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: dev_auto_sell.precheckRequired,
            requireConfirmation: dev_auto_sell.requireConfirmation,
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
        action.triggerKey = Some(trigger_key_for_action(&action));
        actions.push(action);
    }
    for (index, snipe) in follow.snipes.iter().enumerate() {
        if !snipe.enabled {
            continue;
        }
        if let Some(sell) = &snipe.postBuySell
            && sell.enabled
        {
            let mut action = FollowActionRecord {
                actionId: sell.actionId.clone(),
                kind: FollowActionKind::SniperSell,
                walletEnvKey: sell.walletEnvKey.clone(),
                state: FollowActionState::Queued,
                buyAmountSol: None,
                sellPercent: Some(sell.percent),
                submitDelayMs: None,
                targetBlockOffset: sell.targetBlockOffset,
                delayMs: sell.delayMs,
                marketCap: sell
                    .marketCap
                    .as_ref()
                    .map(|trigger| FollowMarketCapTrigger {
                        direction: trigger.direction.clone(),
                        threshold: trigger.threshold.clone(),
                        scanTimeoutSeconds: trigger.scanTimeoutSeconds,
                        timeoutAction: trigger.timeoutAction.clone(),
                    }),
                jitterMs: None,
                feeJitterBps: None,
                precheckRequired: sell.precheckRequired,
                requireConfirmation: sell.requireConfirmation,
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
                orderIndex: index as u32,
                preSignedTransactions: vec![],
                poolId: None,
                timings: FollowActionTimings::default(),
            };
            action.triggerKey = Some(trigger_key_for_action(&action));
            actions.push(action);
        }
    }
    actions
}

impl FollowDaemonStore {
    pub fn load_or_default(state_path: PathBuf) -> Self {
        let mut state = match fs::read_to_string(&state_path) {
            Ok(raw) => serde_json::from_str::<FollowDaemonStateFile>(&raw).unwrap_or_else(|_| {
                let _ = quarantine_corrupt_file(&state_path, "follow daemon state");
                Self::default_state(state_path.clone())
            }),
            Err(_) => Self::default_state(state_path.clone()),
        };
        for job in &mut state.jobs {
            canonicalize_job_trigger_keys(job);
        }
        Self {
            state_path,
            inner: Arc::new(RwLock::new(state)),
            persist_lock: Arc::new(Mutex::new(())),
        }
    }

    fn default_state(state_path: PathBuf) -> FollowDaemonStateFile {
        FollowDaemonStateFile {
            schemaVersion: FOLLOW_SCHEMA_VERSION,
            health: FollowDaemonHealth {
                running: false,
                statePath: state_path,
                version: env!("CARGO_PKG_VERSION").to_string(),
                pid: None,
                startedAtMs: None,
                controlTransport: "local-http".to_string(),
                controlUrl: None,
                updatedAtMs: now_ms(),
                queueDepth: 0,
                activeJobs: 0,
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
            },
            jobs: vec![],
        }
    }

    fn active_job_count(jobs: &[FollowJobRecord]) -> usize {
        jobs.iter()
            .filter(|job| {
                matches!(
                    job.state,
                    FollowJobState::Reserved | FollowJobState::Armed | FollowJobState::Running
                )
            })
            .count()
    }

    fn has_live_watcher_work(jobs: &[FollowJobRecord]) -> bool {
        jobs.iter()
            .any(|job| matches!(job.state, FollowJobState::Armed | FollowJobState::Running))
    }

    fn normalize_idle_health(state: &mut FollowDaemonStateFile) {
        if Self::has_live_watcher_work(&state.jobs) {
            return;
        }
        state.health.slotWatcher = FollowWatcherHealth::Healthy;
        state.health.slotWatcherMode = None;
        state.health.signatureWatcher = FollowWatcherHealth::Healthy;
        state.health.signatureWatcherMode = None;
        state.health.marketWatcher = FollowWatcherHealth::Healthy;
        state.health.marketWatcherMode = None;
        state.health.watchEndpoint = None;
        state.health.lastError = None;
    }

    fn refresh_counts(state: &mut FollowDaemonStateFile) {
        state.health.queueDepth = state.jobs.len();
        state.health.activeJobs = Self::active_job_count(&state.jobs);
        Self::normalize_idle_health(state);
    }

    pub async fn persist(&self) -> Result<(), String> {
        let state = self.inner.read().await.clone();
        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let payload = serde_json::to_vec_pretty(&state).map_err(|error| error.to_string())?;
        let _persist_guard = self.persist_lock.lock().await;
        let mut last_error = None;
        for attempt in 0..5 {
            match atomic_write(&self.state_path, &payload) {
                Ok(()) => return Ok(()),
                Err(error)
                    if is_retryable_persist_error(&std::io::Error::other(error.clone()))
                        && attempt < 4 =>
                {
                    last_error = Some(std::io::Error::other(error));
                    sleep(Duration::from_millis(20 * (attempt + 1) as u64)).await;
                }
                Err(error) => return Err(error),
            }
        }
        Err(last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "Failed to persist follow daemon state.".to_string()))
    }

    pub async fn health(&self) -> FollowDaemonHealth {
        self.inner.read().await.health.clone()
    }

    pub async fn list_jobs(&self) -> Vec<FollowJobRecord> {
        self.inner.read().await.jobs.clone()
    }

    pub async fn get_job(&self, trace_id: &str) -> Option<FollowJobRecord> {
        self.inner
            .read()
            .await
            .jobs
            .iter()
            .find(|job| job.traceId == trace_id)
            .cloned()
    }

    pub async fn reserve_job(
        &self,
        request: FollowReserveRequest,
    ) -> Result<FollowJobRecord, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        let mut requested_actions = if request.prebuiltActions.is_empty() {
            build_action_records(&request.followLaunch)
        } else {
            request.prebuiltActions.clone()
        };
        for action in &mut requested_actions {
            canonicalize_action_trigger_key(action);
        }
        let requested_action_identity = requested_actions
            .iter()
            .map(stable_action_identity)
            .collect::<Vec<_>>();
        let requested_payload_fingerprint = stable_reserve_payload_fingerprint(
            &requested_action_identity,
            &request.deferredSetupTransactions,
        )?;
        if let Some(existing) = state
            .jobs
            .iter_mut()
            .find(|job| job.traceId == request.traceId)
        {
            let existing_action_identity = existing
                .actions
                .iter()
                .map(stable_action_identity)
                .collect::<Vec<_>>();
            let existing_payload_fingerprint = if existing.reservedPayloadFingerprint.is_empty() {
                let existing_deferred_setup_transactions = existing
                    .deferredSetup
                    .as_ref()
                    .map(|setup| setup.transactions.clone())
                    .unwrap_or_default();
                stable_reserve_payload_fingerprint(
                    &existing_action_identity,
                    &existing_deferred_setup_transactions,
                )?
            } else {
                existing.reservedPayloadFingerprint.clone()
            };
            if existing.launchpad != request.launchpad
                || existing.quoteAsset != request.quoteAsset
                || existing.launchMode != request.launchMode
                || existing.selectedWalletKey != request.selectedWalletKey
                || existing.tokenMayhemMode != request.tokenMayhemMode
                || existing.jitoTipAccount != request.jitoTipAccount
                || existing.buyTipAccount != request.buyTipAccount
                || existing.sellTipAccount != request.sellTipAccount
                || existing.preferPostSetupCreatorVaultForSell
                    != request.preferPostSetupCreatorVaultForSell
                || !json_values_equal(&existing.followLaunch, &request.followLaunch)
                || !json_values_equal(&existing.execution, &request.execution)
                || !json_values_equal(&existing.bagsLaunch, &request.bagsLaunch)
                || existing_payload_fingerprint != requested_payload_fingerprint
            {
                return Err(format!(
                    "Conflicting follow reserve request for traceId {}. Reused traceIds must keep the same follow-launch payload.",
                    request.traceId
                ));
            }
            if existing.reservedPayloadFingerprint.is_empty() {
                existing.reservedPayloadFingerprint = requested_payload_fingerprint;
            }
            existing.updatedAtMs = now;
            state.health.updatedAtMs = now;
            Self::refresh_counts(&mut state);
            drop(state);
            self.persist().await?;
            return Ok(self
                .inner
                .read()
                .await
                .jobs
                .iter()
                .find(|job| job.traceId == request.traceId)
                .cloned()
                .expect("reserved job should exist"));
        }
        let actions = requested_actions;
        let job = FollowJobRecord {
            schemaVersion: FOLLOW_JOB_SCHEMA_VERSION,
            traceId: request.traceId.clone(),
            jobId: format!("follow-{}", request.traceId.replace('-', "")),
            state: FollowJobState::Reserved,
            createdAtMs: now,
            updatedAtMs: now,
            launchpad: request.launchpad,
            quoteAsset: request.quoteAsset,
            launchMode: request.launchMode,
            selectedWalletKey: request.selectedWalletKey,
            execution: request.execution,
            tokenMayhemMode: request.tokenMayhemMode,
            jitoTipAccount: request.jitoTipAccount,
            buyTipAccount: request.buyTipAccount,
            sellTipAccount: request.sellTipAccount,
            preferPostSetupCreatorVaultForSell: request.preferPostSetupCreatorVaultForSell,
            mint: None,
            launchCreator: None,
            launchSignature: None,
            launchTransactionSubscribeAccountRequired: vec![],
            submitAtMs: None,
            sendObservedSlot: None,
            confirmedObservedSlot: None,
            reportPath: None,
            transportPlan: None,
            bagsLaunch: request.bagsLaunch,
            followLaunch: request.followLaunch,
            actions,
            reservedPayloadFingerprint: requested_payload_fingerprint,
            deferredSetup: if request.deferredSetupTransactions.is_empty() {
                None
            } else {
                Some(DeferredSetupRecord {
                    transactions: request.deferredSetupTransactions,
                    state: DeferredSetupState::Queued,
                    attemptCount: 0,
                    signatures: vec![],
                    submittedAtMs: None,
                    confirmedAtMs: None,
                    lastError: None,
                })
            },
            cancelRequested: false,
            lastError: None,
            timings: FollowJobTimings {
                benchmarkMode: Some(configured_benchmark_mode().as_str().to_string()),
                ..FollowJobTimings::default()
            },
        };
        state.jobs.push(job.clone());
        state.health.updatedAtMs = now;
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(job)
    }

    pub async fn arm_job(&self, request: FollowArmRequest) -> Result<FollowJobRecord, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.traceId == request.traceId)
            .ok_or_else(|| format!("Unknown follow job traceId: {}", request.traceId))?;
        if let Some(existing_mint) = &job.mint
            && existing_mint != &request.mint
        {
            return Err(format!(
                "Conflicting follow arm request for traceId {}: mint changed from {} to {}.",
                request.traceId, existing_mint, request.mint
            ));
        }
        if let Some(existing_signature) = &job.launchSignature
            && existing_signature != &request.launchSignature
        {
            return Err(format!(
                "Conflicting follow arm request for traceId {}: signature changed from {} to {}.",
                request.traceId, existing_signature, request.launchSignature
            ));
        }
        if let Some(existing_launch_creator) = &job.launchCreator
            && existing_launch_creator != &request.launchCreator
        {
            return Err(format!(
                "Conflicting follow arm request for traceId {}: launch creator changed from {} to {}.",
                request.traceId, existing_launch_creator, request.launchCreator
            ));
        }
        if matches!(
            job.state,
            FollowJobState::Completed
                | FollowJobState::CompletedWithFailures
                | FollowJobState::Cancelled
                | FollowJobState::Failed
        ) {
            let snapshot = job.clone();
            drop(state);
            return Ok(snapshot);
        }
        job.state = if matches!(job.state, FollowJobState::Running) {
            FollowJobState::Running
        } else {
            FollowJobState::Armed
        };
        job.updatedAtMs = now;
        if job.mint.is_none() {
            job.mint = Some(request.mint);
        }
        if job.launchCreator.is_none() {
            job.launchCreator = Some(request.launchCreator);
        }
        if job.launchSignature.is_none() {
            job.launchSignature = Some(request.launchSignature);
        }
        if !request.launchTransactionSubscribeAccountRequired.is_empty() {
            job.launchTransactionSubscribeAccountRequired =
                request.launchTransactionSubscribeAccountRequired;
        }
        if job.submitAtMs.is_none() {
            job.submitAtMs = Some(request.submitAtMs);
        }
        if let Some(send_observed_slot) = request.sendObservedSlot {
            job.sendObservedSlot = Some(send_observed_slot);
        }
        if let Some(confirmed_observed_slot) = request.confirmedObservedSlot {
            job.confirmedObservedSlot = Some(confirmed_observed_slot);
        }
        if let Some(report_path) = request.reportPath {
            job.reportPath = Some(report_path);
        }
        job.transportPlan = Some(request.transportPlan);
        for action in &mut job.actions {
            if matches!(
                action.state,
                FollowActionState::Queued | FollowActionState::Armed
            ) {
                action.state = FollowActionState::Armed;
            }
            if action.scheduledForMs.is_none() {
                if let Some(delay_ms) = action.submitDelayMs {
                    action.scheduledForMs =
                        Some(request.submitAtMs.saturating_add(u128::from(delay_ms)));
                } else if let Some(delay_ms) = action.delayMs {
                    action.scheduledForMs =
                        Some(request.submitAtMs.saturating_add(u128::from(delay_ms)));
                }
            }
        }
        state.health.updatedAtMs = now;
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(self
            .inner
            .read()
            .await
            .jobs
            .iter()
            .find(|job| job.traceId == request.traceId)
            .cloned()
            .expect("armed job should exist"))
    }

    pub async fn cancel_job(
        &self,
        request: FollowCancelRequest,
    ) -> Result<FollowJobRecord, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.traceId == request.traceId)
            .ok_or_else(|| format!("Unknown follow job traceId: {}", request.traceId))?;
        job.updatedAtMs = now;
        if let Some(action_id) = request.actionId.as_deref() {
            if let Some(action) = job
                .actions
                .iter_mut()
                .find(|action| action.actionId == action_id)
            {
                action.state = FollowActionState::Cancelled;
                action.lastError = request.note.clone();
            }
        } else {
            job.cancelRequested = true;
            job.state = FollowJobState::Cancelled;
            for action in &mut job.actions {
                if !matches!(
                    action.state,
                    FollowActionState::Confirmed
                        | FollowActionState::Stopped
                        | FollowActionState::Failed
                        | FollowActionState::Cancelled
                ) {
                    action.state = FollowActionState::Cancelled;
                    action.lastError = request.note.clone();
                }
            }
        }
        state.health.updatedAtMs = now;
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(self
            .inner
            .read()
            .await
            .jobs
            .iter()
            .find(|job| job.traceId == request.traceId)
            .cloned()
            .expect("cancelled job should exist"))
    }

    pub async fn cancel_all_jobs(
        &self,
        note: Option<String>,
    ) -> Result<Vec<FollowJobRecord>, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        let mut snapshots = Vec::new();
        for job in &mut state.jobs {
            if matches!(
                job.state,
                FollowJobState::Completed
                    | FollowJobState::CompletedWithFailures
                    | FollowJobState::Cancelled
                    | FollowJobState::Failed
            ) {
                continue;
            }
            job.cancelRequested = true;
            job.state = FollowJobState::Cancelled;
            job.updatedAtMs = now;
            for action in &mut job.actions {
                if !matches!(
                    action.state,
                    FollowActionState::Confirmed
                        | FollowActionState::Stopped
                        | FollowActionState::Failed
                        | FollowActionState::Cancelled
                ) {
                    action.state = FollowActionState::Cancelled;
                    action.lastError = note.clone();
                }
            }
            snapshots.push(job.clone());
        }
        state.health.updatedAtMs = now;
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(snapshots)
    }

    pub async fn update_health(
        &self,
        watch_endpoint: Option<String>,
        slot_watcher: FollowWatcherHealth,
        slot_watcher_mode: Option<String>,
        signature_watcher: FollowWatcherHealth,
        signature_watcher_mode: Option<String>,
        market_watcher: FollowWatcherHealth,
        market_watcher_mode: Option<String>,
        last_error: Option<String>,
    ) -> Result<(), String> {
        let mut state = self.inner.write().await;
        state.health.updatedAtMs = now_ms();
        state.health.watchEndpoint = watch_endpoint;
        state.health.slotWatcher = slot_watcher;
        state.health.slotWatcherMode = slot_watcher_mode;
        state.health.signatureWatcher = signature_watcher;
        state.health.signatureWatcherMode = signature_watcher_mode;
        state.health.marketWatcher = market_watcher;
        state.health.marketWatcherMode = market_watcher_mode;
        state.health.lastError = last_error;
        drop(state);
        self.persist().await
    }

    pub async fn update_supervision(
        &self,
        running: bool,
        pid: Option<u32>,
        control_transport: Option<String>,
        control_url: Option<String>,
        last_error: Option<String>,
    ) -> Result<(), String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        state.health.running = running;
        state.health.pid = pid;
        if running {
            state.health.startedAtMs.get_or_insert(now);
        }
        if let Some(value) = control_transport {
            state.health.controlTransport = value;
        }
        if control_url.is_some() {
            state.health.controlUrl = control_url;
        }
        state.health.lastError = last_error;
        state.health.updatedAtMs = now;
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await
    }

    pub async fn recover_jobs_for_restart(&self) -> Result<Vec<FollowJobRecord>, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        state
            .jobs
            .retain(|job| !should_prune_job_on_restart(job, now));
        let mut recovered = Vec::new();
        for job in &mut state.jobs {
            if matches!(job.state, FollowJobState::Running) {
                job.state = FollowJobState::Armed;
            }
            let mut touched = false;
            for action in &mut job.actions {
                if matches!(
                    action.state,
                    FollowActionState::Running | FollowActionState::Sent
                ) {
                    action.state = FollowActionState::Failed;
                    action.lastError = Some(
                        "Follow daemon restarted while the action was in flight; automatic resend was skipped to avoid duplication."
                            .to_string(),
                    );
                    touched = true;
                }
            }
            if touched {
                job.updatedAtMs = now;
                recovered.push(job.clone());
            }
        }
        state.health.updatedAtMs = now;
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(recovered)
    }

    pub async fn clear_jobs_for_restart(&self) -> Result<usize, String> {
        let mut state = self.inner.write().await;
        let removed = state.jobs.len();
        let now = now_ms();
        state.jobs.clear();
        state.health.updatedAtMs = now;
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(removed)
    }

    pub async fn update_action(
        &self,
        trace_id: &str,
        action_id: &str,
        mutator: impl FnOnce(&mut FollowActionRecord),
    ) -> Result<FollowJobRecord, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        state.health.updatedAtMs = now;
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.traceId == trace_id)
            .ok_or_else(|| format!("Unknown follow job traceId: {trace_id}"))?;
        let action = job
            .actions
            .iter_mut()
            .find(|action| action.actionId == action_id)
            .ok_or_else(|| format!("Unknown follow action: {action_id}"))?;
        mutator(action);
        job.updatedAtMs = now;
        let snapshot = job.clone();
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(snapshot)
    }

    pub async fn update_job(
        &self,
        trace_id: &str,
        mutator: impl FnOnce(&mut FollowJobRecord),
    ) -> Result<FollowJobRecord, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        state.health.updatedAtMs = now;
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.traceId == trace_id)
            .ok_or_else(|| format!("Unknown follow job traceId: {trace_id}"))?;
        mutator(job);
        job.updatedAtMs = now;
        let snapshot = job.clone();
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(snapshot)
    }

    pub async fn finalize_job_state(
        &self,
        trace_id: &str,
        state_value: FollowJobState,
        last_error: Option<String>,
    ) -> Result<FollowJobRecord, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        state.health.updatedAtMs = now;
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.traceId == trace_id)
            .ok_or_else(|| format!("Unknown follow job traceId: {trace_id}"))?;
        job.state = state_value;
        job.lastError = last_error;
        job.updatedAtMs = now;
        let snapshot = job.clone();
        Self::refresh_counts(&mut state);
        drop(state);
        self.persist().await?;
        Ok(snapshot)
    }
}

impl FollowDaemonClient {
    pub fn new(base_url: &str) -> Self {
        static CLIENT: OnceLock<Client> = OnceLock::new();
        Self {
            baseUrl: base_url.trim_end_matches('/').to_string(),
            client: CLIENT.get_or_init(Client::new).clone(),
            authToken: configured_follow_daemon_auth_token(),
        }
    }

    fn is_readonly_poll_path(path: &str) -> bool {
        matches!(path, "/health" | "/jobs") || path.starts_with("/jobs/")
    }

    async fn request_json<TRequest, TResponse>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&TRequest>,
    ) -> Result<TResponse, String>
    where
        TRequest: Serialize + ?Sized,
        TResponse: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.baseUrl, path.trim_start_matches('/'));
        let mut request = self.client.request(method, url);
        let method_name = request
            .try_clone()
            .map(|request| {
                request
                    .build()
                    .map(|built| built.method().to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        if let Some(token) = &self.authToken {
            request = request.header("x-launchdeck-engine-auth", token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.map_err(|error| {
            let message = error.to_string();
            let details = Some(serde_json::json!({
                "baseUrl": self.baseUrl,
                "path": path,
                "method": method_name,
                "message": message,
            }));
            if Self::is_readonly_poll_path(path) {
                record_warn(
                    "follow-client",
                    format!("Follow daemon request temporarily failed: {}", path),
                    details,
                );
            } else {
                record_error(
                    "follow-client",
                    format!("Follow daemon request failed: {}", path),
                    details,
                );
            }
            message
        })?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let message = format!(
                "Follow daemon request failed with status {}: {}",
                status, body
            );
            record_error(
                "follow-client",
                format!("Follow daemon request rejected: {}", path),
                Some(serde_json::json!({
                    "baseUrl": self.baseUrl,
                    "path": path,
                    "method": method_name,
                    "status": status.as_u16(),
                    "body": body,
                })),
            );
            return Err(message);
        }
        let parsed = response.json::<TResponse>().await.map_err(|error| {
            let message = error.to_string();
            record_error(
                "follow-client",
                format!("Follow daemon response decode failed: {}", path),
                Some(serde_json::json!({
                    "baseUrl": self.baseUrl,
                    "path": path,
                    "method": method_name,
                    "message": message,
                })),
            );
            message
        })?;
        if matches!(
            path,
            "/jobs/reserve" | "/jobs/arm" | "/jobs/cancel" | "/jobs/stop-all"
        ) {
            record_info(
                "follow-client",
                format!("Follow daemon request succeeded: {}", path),
                Some(serde_json::json!({
                    "baseUrl": self.baseUrl,
                    "path": path,
                    "method": method_name,
                })),
            );
        }
        Ok(parsed)
    }

    pub async fn health(&self) -> Result<FollowDaemonHealth, String> {
        self.request_json::<Value, FollowDaemonHealth>(reqwest::Method::GET, "/health", None)
            .await
    }

    pub async fn ready(&self, payload: &FollowReadyRequest) -> Result<FollowReadyResponse, String> {
        self.request_json(reqwest::Method::POST, "/ready", Some(payload))
            .await
    }

    pub async fn reserve(
        &self,
        payload: &FollowReserveRequest,
    ) -> Result<FollowJobResponse, String> {
        self.request_json(reqwest::Method::POST, "/jobs/reserve", Some(payload))
            .await
    }

    pub async fn arm(&self, payload: &FollowArmRequest) -> Result<FollowJobResponse, String> {
        self.request_json(reqwest::Method::POST, "/jobs/arm", Some(payload))
            .await
    }

    pub async fn cancel(&self, payload: &FollowCancelRequest) -> Result<FollowJobResponse, String> {
        self.request_json(reqwest::Method::POST, "/jobs/cancel", Some(payload))
            .await
    }

    pub async fn list(&self) -> Result<FollowJobResponse, String> {
        self.request_json::<Value, FollowJobResponse>(reqwest::Method::GET, "/jobs", None)
            .await
    }

    pub async fn stop_all(
        &self,
        payload: &FollowStopAllRequest,
    ) -> Result<FollowJobResponse, String> {
        self.request_json(reqwest::Method::POST, "/jobs/stop-all", Some(payload))
            .await
    }

    pub async fn status(&self, trace_id: &str) -> Result<FollowJobResponse, String> {
        self.request_json::<Value, FollowJobResponse>(
            reqwest::Method::GET,
            &format!("/jobs/{trace_id}"),
            None,
        )
        .await
    }
}

pub fn follow_job_response(
    health: FollowDaemonHealth,
    job: Option<FollowJobRecord>,
    jobs: Vec<FollowJobRecord>,
) -> FollowJobResponse {
    FollowJobResponse {
        schemaVersion: FOLLOW_RESPONSE_SCHEMA_VERSION,
        ok: true,
        job,
        jobs,
        health,
    }
}

pub fn follow_ready_response(
    health: FollowDaemonHealth,
    watch_endpoint: Option<String>,
    required_websocket: bool,
    ready: bool,
    reason: Option<String>,
) -> FollowReadyResponse {
    FollowReadyResponse {
        ok: true,
        ready,
        watchEndpoint: watch_endpoint,
        requiredWebsocket: required_websocket,
        reason,
        health,
    }
}

pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        NormalizedFollowLaunchConstraints, NormalizedFollowLaunchSell, NormalizedFollowLaunchSnipe,
    };
    use crate::transport::TransportPlan;
    use serde_json::json;

    fn sample_compiled_transaction(label: &str, blockhash: &str) -> CompiledTransaction {
        CompiledTransaction {
            label: label.to_string(),
            format: "v0".to_string(),
            blockhash: blockhash.to_string(),
            lastValidBlockHeight: 123,
            serializedBase64: format!("base64-{label}-{blockhash}"),
            signature: None,
            lookupTablesUsed: vec![],
            computeUnitLimit: None,
            computeUnitPriceMicroLamports: None,
            inlineTipLamports: None,
            inlineTipAccount: None,
        }
    }

    fn sample_follow_launch() -> NormalizedFollowLaunch {
        NormalizedFollowLaunch {
            enabled: true,
            source: "test".to_string(),
            schemaVersion: 1,
            snipes: vec![NormalizedFollowLaunchSnipe {
                actionId: "snipe-a".to_string(),
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
                postBuySell: None,
            }],
            devAutoSell: Some(NormalizedFollowLaunchSell {
                actionId: "dev-sell".to_string(),
                enabled: true,
                walletEnvKey: "WALLET_A".to_string(),
                percent: 100,
                delayMs: None,
                targetBlockOffset: Some(0),
                marketCap: None,
                precheckRequired: false,
                requireConfirmation: false,
            }),
            constraints: NormalizedFollowLaunchConstraints {
                pumpOnly: false,
                retryBudget: 1,
                requireDaemonReadiness: false,
                blockOnRequiredPrechecks: true,
            },
        }
    }

    fn sample_follow_launch_with_sniper_sell() -> NormalizedFollowLaunch {
        let mut follow = sample_follow_launch();
        follow.snipes[0].postBuySell = Some(NormalizedFollowLaunchSell {
            actionId: "snipe-a-sell".to_string(),
            enabled: true,
            walletEnvKey: "WALLET_A".to_string(),
            percent: 50,
            delayMs: None,
            targetBlockOffset: Some(1),
            marketCap: None,
            precheckRequired: false,
            requireConfirmation: true,
        });
        follow
    }

    fn sample_execution() -> NormalizedExecution {
        serde_json::from_value(json!({
            "simulate": false,
            "send": true,
            "txFormat": "v0",
            "commitment": "confirmed",
            "skipPreflight": false,
            "trackSendBlockHeight": true,
            "provider": "standard-rpc",
            "endpointProfile": "default",
            "mevProtect": false,
            "mevMode": "reduced",
            "jitodontfront": false,
            "autoGas": false,
            "autoMode": "manual",
            "priorityFeeSol": "0",
            "tipSol": "0",
            "maxPriorityFeeSol": "0",
            "maxTipSol": "0",
            "buyProvider": "standard-rpc",
            "buyEndpointProfile": "default",
            "buyMevProtect": false,
            "buyMevMode": "reduced",
            "buyJitodontfront": false,
            "buyAutoGas": false,
            "buyAutoMode": "manual",
            "buyPriorityFeeSol": "0",
            "buyTipSol": "0",
            "buySlippagePercent": "5",
            "buyMaxPriorityFeeSol": "0",
            "buyMaxTipSol": "0",
            "sellAutoGas": false,
            "sellAutoMode": "manual",
            "sellProvider": "standard-rpc",
            "sellEndpointProfile": "default",
            "sellMevProtect": false,
            "sellMevMode": "reduced",
            "sellJitodontfront": false,
            "sellPriorityFeeSol": "0",
            "sellTipSol": "0",
            "sellSlippagePercent": "5",
            "sellMaxPriorityFeeSol": "0",
            "sellMaxTipSol": "0"
        }))
        .expect("sample execution")
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

    #[test]
    fn build_action_records_sets_trigger_keys_and_order() {
        let actions = build_action_records(&sample_follow_launch());
        let snipe = actions
            .iter()
            .find(|action| action.actionId == "snipe-a")
            .expect("snipe action");
        let dev_sell = actions
            .iter()
            .find(|action| action.actionId == "dev-sell")
            .expect("dev sell action");
        assert_eq!(snipe.triggerKey.as_deref(), Some("slot:0"));
        assert_eq!(dev_sell.triggerKey.as_deref(), Some("slot:0"));
        assert_eq!(snipe.orderIndex, 0);
        assert!(snipe.preSignedTransactions.is_empty());
    }

    #[test]
    fn stable_action_identity_normalizes_legacy_block_trigger_keys() {
        let mut legacy = build_action_records(&sample_follow_launch())
            .into_iter()
            .find(|action| action.actionId == "dev-sell")
            .expect("dev sell action");
        let mut current = legacy.clone();
        legacy.triggerKey = Some("block:0".to_string());
        current.triggerKey = Some("slot:0".to_string());
        assert!(json_values_equal(
            &stable_action_identity(&legacy),
            &stable_action_identity(&current)
        ));
    }

    #[test]
    fn sniper_sell_trigger_keys_are_action_specific() {
        let sell = build_action_records(&sample_follow_launch_with_sniper_sell())
            .into_iter()
            .find(|action| action.kind == FollowActionKind::SniperSell)
            .expect("sniper sell action");
        assert_eq!(
            sell.triggerKey.as_deref(),
            Some("sniper-sell:snipe-a-sell:slot:1")
        );
    }

    #[test]
    fn stable_action_identity_rewrites_legacy_sniper_sell_trigger_keys() {
        let mut legacy = build_action_records(&sample_follow_launch_with_sniper_sell())
            .into_iter()
            .find(|action| action.kind == FollowActionKind::SniperSell)
            .expect("sniper sell action");
        let mut current = legacy.clone();
        legacy.triggerKey = Some("slot:1".to_string());
        current.triggerKey = Some("sniper-sell:snipe-a-sell:slot:1".to_string());
        assert!(json_values_equal(
            &stable_action_identity(&legacy),
            &stable_action_identity(&current)
        ));
    }

    #[test]
    fn trigger_key_uses_market_cap_shape_when_present() {
        let action = FollowActionRecord {
            actionId: "sell-a".to_string(),
            kind: FollowActionKind::SniperSell,
            walletEnvKey: "WALLET_A".to_string(),
            state: FollowActionState::Queued,
            buyAmountSol: None,
            sellPercent: Some(50),
            submitDelayMs: None,
            targetBlockOffset: None,
            delayMs: None,
            marketCap: Some(FollowMarketCapTrigger {
                direction: "above".to_string(),
                threshold: "100".to_string(),
                scanTimeoutSeconds: 30,
                timeoutAction: "cancel".to_string(),
            }),
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
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        assert_eq!(
            trigger_key_for_action(&action),
            "sniper-sell:sell-a:market:above:100:30:cancel".to_string()
        );
    }

    #[test]
    fn creator_vault_rule_keeps_confirmation_zero_on_deployer_path() {
        let action = FollowActionRecord {
            actionId: "sell-a".to_string(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: "WALLET_A".to_string(),
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
    fn creator_vault_rule_allows_post_setup_path_after_confirmation_zero() {
        let action = FollowActionRecord {
            actionId: "sell-b".to_string(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: "WALLET_A".to_string(),
            state: FollowActionState::Queued,
            buyAmountSol: None,
            sellPercent: Some(100),
            submitDelayMs: None,
            targetBlockOffset: Some(1),
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
        assert!(should_use_post_setup_creator_vault_for_sell(
            true, &action, "reduced"
        ));
    }

    #[test]
    fn creator_vault_rule_uses_post_setup_path_immediately_for_secure_buy() {
        let action = FollowActionRecord {
            actionId: "buy-a".to_string(),
            kind: FollowActionKind::SniperBuy,
            walletEnvKey: "WALLET_A".to_string(),
            state: FollowActionState::Queued,
            buyAmountSol: Some("0.001".to_string()),
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
        assert!(should_use_post_setup_creator_vault_for_buy(
            true, &action, "secure"
        ));
    }

    #[test]
    fn creator_vault_rule_uses_post_setup_path_immediately_for_secure_sell() {
        let action = FollowActionRecord {
            actionId: "sell-secure".to_string(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: "WALLET_A".to_string(),
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
            orderIndex: 0,
            preSignedTransactions: vec![],
            poolId: None,
            timings: FollowActionTimings::default(),
        };
        assert!(should_use_post_setup_creator_vault_for_sell(
            true, &action, "secure"
        ));
    }

    #[test]
    fn creator_vault_rule_keeps_non_secure_buy_on_deployer_path_at_zero_offset() {
        let action = FollowActionRecord {
            actionId: "buy-b".to_string(),
            kind: FollowActionKind::SniperBuy,
            walletEnvKey: "WALLET_A".to_string(),
            state: FollowActionState::Queued,
            buyAmountSol: Some("0.001".to_string()),
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
        assert!(!should_use_post_setup_creator_vault_for_buy(
            true, &action, "reduced"
        ));
    }

    #[tokio::test]
    async fn reserve_job_remains_idempotent_after_arm_and_runtime_mutation() {
        let state_path = std::env::temp_dir().join(format!(
            "launchdeck-follow-store-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let store = FollowDaemonStore::load_or_default(state_path.clone());
        let reserve_request = FollowReserveRequest {
            traceId: "trace-123".to_string(),
            launchpad: "pump".to_string(),
            quoteAsset: "sol".to_string(),
            launchMode: "launch".to_string(),
            selectedWalletKey: "WALLET_A".to_string(),
            followLaunch: sample_follow_launch(),
            execution: sample_execution(),
            tokenMayhemMode: false,
            jitoTipAccount: String::new(),
            buyTipAccount: String::new(),
            sellTipAccount: String::new(),
            preferPostSetupCreatorVaultForSell: false,
            bagsLaunch: None,
            prebuiltActions: vec![],
            deferredSetupTransactions: vec![],
        };
        store
            .reserve_job(reserve_request.clone())
            .await
            .expect("initial reserve");
        store
            .arm_job(FollowArmRequest {
                traceId: reserve_request.traceId.clone(),
                mint: "mint".to_string(),
                launchCreator: "creator".to_string(),
                launchSignature: "sig".to_string(),
                launchTransactionSubscribeAccountRequired: vec!["payer".to_string()],
                submitAtMs: 1,
                sendObservedSlot: Some(100),
                confirmedObservedSlot: Some(101),
                reportPath: None,
                transportPlan: sample_transport_plan(),
            })
            .await
            .expect("arm");
        store
            .update_action(&reserve_request.traceId, "snipe-a", |record| {
                record.state = FollowActionState::Running;
                record.attemptCount = 1;
                record.provider = Some("standard-rpc".to_string());
                record.transportType = Some("standard-rpc".to_string());
                record.signature = Some("sig-follow".to_string());
                record.sendObservedSlot = Some(102);
            })
            .await
            .expect("runtime mutation");
        store
            .reserve_job(reserve_request)
            .await
            .expect("reserve should remain idempotent");
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn repeat_arm_updates_confirm_slot_and_report_path() {
        let state_path = std::env::temp_dir().join(format!(
            "launchdeck-follow-store-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let store = FollowDaemonStore::load_or_default(state_path.clone());
        let reserve_request = FollowReserveRequest {
            traceId: "trace-arm-update".to_string(),
            launchpad: "pump".to_string(),
            quoteAsset: "sol".to_string(),
            launchMode: "launch".to_string(),
            selectedWalletKey: "WALLET_A".to_string(),
            followLaunch: sample_follow_launch(),
            execution: sample_execution(),
            tokenMayhemMode: false,
            jitoTipAccount: String::new(),
            buyTipAccount: String::new(),
            sellTipAccount: String::new(),
            preferPostSetupCreatorVaultForSell: false,
            bagsLaunch: None,
            prebuiltActions: vec![],
            deferredSetupTransactions: vec![],
        };
        store
            .reserve_job(reserve_request.clone())
            .await
            .expect("reserve");
        store
            .arm_job(FollowArmRequest {
                traceId: reserve_request.traceId.clone(),
                mint: "mint".to_string(),
                launchCreator: "creator".to_string(),
                launchSignature: "sig".to_string(),
                launchTransactionSubscribeAccountRequired: vec!["payer".to_string()],
                submitAtMs: 1,
                sendObservedSlot: Some(100),
                confirmedObservedSlot: None,
                reportPath: None,
                transportPlan: sample_transport_plan(),
            })
            .await
            .expect("initial arm");
        let updated = store
            .arm_job(FollowArmRequest {
                traceId: reserve_request.traceId.clone(),
                mint: "mint".to_string(),
                launchCreator: "creator".to_string(),
                launchSignature: "sig".to_string(),
                launchTransactionSubscribeAccountRequired: vec![
                    "payer".to_string(),
                    "mint".to_string(),
                ],
                submitAtMs: 1,
                sendObservedSlot: Some(100),
                confirmedObservedSlot: Some(101),
                reportPath: Some("report.json".to_string()),
                transportPlan: sample_transport_plan(),
            })
            .await
            .expect("repeat arm");
        assert_eq!(updated.confirmedObservedSlot, Some(101));
        assert_eq!(updated.reportPath.as_deref(), Some("report.json"));
        assert_eq!(
            updated.launchTransactionSubscribeAccountRequired,
            vec!["payer".to_string(), "mint".to_string()]
        );
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn reserve_job_rejects_changed_presigned_payloads_for_same_trace() {
        let state_path = std::env::temp_dir().join(format!(
            "launchdeck-follow-store-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let store = FollowDaemonStore::load_or_default(state_path.clone());
        let mut initial_request = FollowReserveRequest {
            traceId: "trace-presigned".to_string(),
            launchpad: "pump".to_string(),
            quoteAsset: "sol".to_string(),
            launchMode: "launch".to_string(),
            selectedWalletKey: "WALLET_A".to_string(),
            followLaunch: sample_follow_launch(),
            execution: sample_execution(),
            tokenMayhemMode: false,
            jitoTipAccount: String::new(),
            buyTipAccount: String::new(),
            sellTipAccount: String::new(),
            preferPostSetupCreatorVaultForSell: false,
            bagsLaunch: None,
            prebuiltActions: build_action_records(&sample_follow_launch()),
            deferredSetupTransactions: vec![],
        };
        initial_request.prebuiltActions[0].preSignedTransactions =
            vec![sample_compiled_transaction("snipe-a", "blockhash-a")];
        store
            .reserve_job(initial_request.clone())
            .await
            .expect("initial reserve");
        let mut conflicting_request = initial_request.clone();
        conflicting_request.prebuiltActions[0].preSignedTransactions =
            vec![sample_compiled_transaction("snipe-a", "blockhash-b")];
        let error = store
            .reserve_job(conflicting_request)
            .await
            .expect_err("changed presigned payload should conflict");
        assert!(error.contains("Conflicting follow reserve request"));
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn reserve_job_rejects_changed_deferred_setup_transactions_for_same_trace() {
        let state_path = std::env::temp_dir().join(format!(
            "launchdeck-follow-store-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let store = FollowDaemonStore::load_or_default(state_path.clone());
        let initial_request = FollowReserveRequest {
            traceId: "trace-deferred-setup".to_string(),
            launchpad: "pump".to_string(),
            quoteAsset: "sol".to_string(),
            launchMode: "launch".to_string(),
            selectedWalletKey: "WALLET_A".to_string(),
            followLaunch: sample_follow_launch(),
            execution: sample_execution(),
            tokenMayhemMode: false,
            jitoTipAccount: String::new(),
            buyTipAccount: String::new(),
            sellTipAccount: String::new(),
            preferPostSetupCreatorVaultForSell: false,
            bagsLaunch: None,
            prebuiltActions: vec![],
            deferredSetupTransactions: vec![sample_compiled_transaction(
                "deferred-setup",
                "blockhash-a",
            )],
        };
        store
            .reserve_job(initial_request.clone())
            .await
            .expect("initial reserve");
        let mut conflicting_request = initial_request.clone();
        conflicting_request.deferredSetupTransactions =
            vec![sample_compiled_transaction("deferred-setup", "blockhash-b")];
        let error = store
            .reserve_job(conflicting_request)
            .await
            .expect_err("changed deferred setup payload should conflict");
        assert!(error.contains("Conflicting follow reserve request"));
        let _ = std::fs::remove_file(state_path);
    }

    #[tokio::test]
    async fn repeat_arm_rejects_changed_launch_creator() {
        let state_path = std::env::temp_dir().join(format!(
            "launchdeck-follow-store-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let store = FollowDaemonStore::load_or_default(state_path.clone());
        let reserve_request = FollowReserveRequest {
            traceId: "trace-arm-creator".to_string(),
            launchpad: "pump".to_string(),
            quoteAsset: "sol".to_string(),
            launchMode: "launch".to_string(),
            selectedWalletKey: "WALLET_A".to_string(),
            followLaunch: sample_follow_launch(),
            execution: sample_execution(),
            tokenMayhemMode: false,
            jitoTipAccount: String::new(),
            buyTipAccount: String::new(),
            sellTipAccount: String::new(),
            preferPostSetupCreatorVaultForSell: false,
            bagsLaunch: None,
            prebuiltActions: vec![],
            deferredSetupTransactions: vec![],
        };
        store
            .reserve_job(reserve_request.clone())
            .await
            .expect("reserve");
        store
            .arm_job(FollowArmRequest {
                traceId: reserve_request.traceId.clone(),
                mint: "mint".to_string(),
                launchCreator: "creator-a".to_string(),
                launchSignature: "sig".to_string(),
                launchTransactionSubscribeAccountRequired: vec!["payer".to_string()],
                submitAtMs: 1,
                sendObservedSlot: Some(100),
                confirmedObservedSlot: None,
                reportPath: None,
                transportPlan: sample_transport_plan(),
            })
            .await
            .expect("initial arm");
        let error = store
            .arm_job(FollowArmRequest {
                traceId: reserve_request.traceId.clone(),
                mint: "mint".to_string(),
                launchCreator: "creator-b".to_string(),
                launchSignature: "sig".to_string(),
                launchTransactionSubscribeAccountRequired: vec!["payer".to_string()],
                submitAtMs: 1,
                sendObservedSlot: Some(100),
                confirmedObservedSlot: Some(101),
                reportPath: None,
                transportPlan: sample_transport_plan(),
            })
            .await
            .expect_err("launch creator changes should conflict");
        assert!(error.contains("launch creator changed"));
        let _ = std::fs::remove_file(state_path);
    }
}
