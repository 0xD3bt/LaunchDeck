#![allow(non_snake_case, dead_code)]

use crate::{
    config::{NormalizedExecution, NormalizedFollowLaunch},
    transport::TransportPlan,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex, RwLock},
    time::sleep,
};

pub const FOLLOW_SCHEMA_VERSION: u32 = 1;
pub const FOLLOW_JOB_SCHEMA_VERSION: u32 = 1;
pub const FOLLOW_TELEMETRY_SCHEMA_VERSION: u32 = 1;
pub const FOLLOW_RESPONSE_SCHEMA_VERSION: u32 = 1;
const DEFAULT_LOCAL_AUTH_TOKEN: &str = "4815927603149027";

fn follow_job_schema_version() -> u32 {
    FOLLOW_JOB_SCHEMA_VERSION
}

fn follow_telemetry_schema_version() -> u32 {
    FOLLOW_TELEMETRY_SCHEMA_VERSION
}

fn follow_response_schema_version() -> u32 {
    FOLLOW_RESPONSE_SCHEMA_VERSION
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
    pub submitStartedAtMs: Option<u128>,
    pub submittedAtMs: Option<u128>,
    pub confirmedAtMs: Option<u128>,
    #[serde(default)]
    pub sendObservedBlockHeight: Option<u64>,
    #[serde(default)]
    pub confirmedObservedBlockHeight: Option<u64>,
    #[serde(default)]
    pub blocksToConfirm: Option<u64>,
    pub signature: Option<String>,
    pub explorerUrl: Option<String>,
    pub endpoint: Option<String>,
    pub bundleId: Option<String>,
    pub lastError: Option<String>,
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
    pub selectedWalletKey: String,
    pub execution: NormalizedExecution,
    pub tokenMayhemMode: bool,
    pub jitoTipAccount: String,
    #[serde(default)]
    pub preferPostSetupCreatorVaultForSell: bool,
    pub mint: Option<String>,
    pub launchCreator: Option<String>,
    pub launchSignature: Option<String>,
    pub submitAtMs: Option<u128>,
    pub sendObservedBlockHeight: Option<u64>,
    pub reportPath: Option<String>,
    pub transportPlan: Option<TransportPlan>,
    pub followLaunch: NormalizedFollowLaunch,
    pub actions: Vec<FollowActionRecord>,
    pub cancelRequested: bool,
    pub lastError: Option<String>,
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
    pub maxActiveJobs: usize,
    pub maxConcurrentCompiles: usize,
    pub maxConcurrentSends: usize,
    pub availableCompileSlots: usize,
    pub availableSendSlots: usize,
    pub slotWatcher: FollowWatcherHealth,
    pub signatureWatcher: FollowWatcherHealth,
    pub marketWatcher: FollowWatcherHealth,
    pub lastError: Option<String>,
    pub watchEndpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowTelemetrySample {
    #[serde(default = "follow_telemetry_schema_version")]
    pub schemaVersion: u32,
    pub traceId: String,
    pub actionId: String,
    pub actionType: String,
    #[serde(default)]
    pub phase: String,
    pub provider: String,
    pub endpointProfile: String,
    pub transportType: String,
    #[serde(default)]
    pub attemptCount: u32,
    #[serde(default)]
    pub triggerType: String,
    pub delaySettingMs: Option<u64>,
    pub jitterMs: Option<u64>,
    pub feeJitterBps: Option<u16>,
    pub submitLatencyMs: Option<u128>,
    pub confirmLatencyMs: Option<u128>,
    pub launchToActionMs: Option<u128>,
    pub launchToActionBlocks: Option<u64>,
    pub scheduleSlipMs: Option<u64>,
    pub outcome: String,
    #[serde(default)]
    pub qualityLabel: String,
    pub qualityWeight: u8,
    pub detail: Option<String>,
    pub writtenAtMs: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowTimingRecommendation {
    pub suggestedSubmitDelayMs: Option<u64>,
    pub suggestedJitterMs: Option<u64>,
    pub confidence: String,
    pub successRate: f64,
    pub weightedQualityScore: f64,
    pub sampleCount: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowTimingProfile {
    #[serde(default = "follow_telemetry_schema_version")]
    pub schemaVersion: u32,
    pub provider: String,
    pub endpointProfile: String,
    pub actionType: String,
    pub sampleCount: usize,
    pub successCount: usize,
    pub weightedQualityScore: f64,
    pub p50SubmitMs: Option<u64>,
    pub p75SubmitMs: Option<u64>,
    pub p90SubmitMs: Option<u64>,
    pub recommendation: FollowTimingRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowDaemonStateFile {
    pub schemaVersion: u32,
    pub health: FollowDaemonHealth,
    pub jobs: Vec<FollowJobRecord>,
    pub telemetrySamples: Vec<FollowTelemetrySample>,
    pub timingProfiles: Vec<FollowTimingProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowReserveRequest {
    pub traceId: String,
    pub launchpad: String,
    pub selectedWalletKey: String,
    pub followLaunch: NormalizedFollowLaunch,
    pub execution: NormalizedExecution,
    pub tokenMayhemMode: bool,
    pub jitoTipAccount: String,
    #[serde(default)]
    pub preferPostSetupCreatorVaultForSell: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowArmRequest {
    pub traceId: String,
    pub mint: String,
    pub launchCreator: String,
    pub launchSignature: String,
    pub submitAtMs: u128,
    pub sendObservedBlockHeight: Option<u64>,
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
    pub timingProfiles: Vec<FollowTimingProfile>,
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

fn percentile(sorted: &[u128], percentile: f64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted
        .get(rank)
        .and_then(|value| u64::try_from(*value).ok())
}

fn outcome_success_weight(outcome: &str) -> (bool, f64) {
    match outcome {
        "success" => (true, 1.0),
        "degraded-success" => (true, 0.8),
        "late" => (true, 0.5),
        "missed-window" => (false, 0.2),
        "reverted" | "provider-rejected" | "insufficient-funds" => (false, 0.0),
        "cancelled" => (false, 0.0),
        _ => (false, 0.1),
    }
}

fn recommendation_confidence(sample_count: usize, success_rate: f64) -> String {
    if sample_count >= 25 && success_rate >= 0.75 {
        "high".to_string()
    } else if sample_count >= 10 && success_rate >= 0.5 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn build_action_records(follow: &NormalizedFollowLaunch) -> Vec<FollowActionRecord> {
    let mut actions = follow
        .snipes
        .iter()
        .map(|snipe| FollowActionRecord {
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
        })
        .collect::<Vec<_>>();
    if let Some(dev_auto_sell) = &follow.devAutoSell {
        actions.push(FollowActionRecord {
            actionId: dev_auto_sell.actionId.clone(),
            kind: FollowActionKind::DevAutoSell,
            walletEnvKey: dev_auto_sell.walletEnvKey.clone(),
            state: FollowActionState::Queued,
            buyAmountSol: None,
            sellPercent: Some(dev_auto_sell.percent),
            submitDelayMs: None,
            targetBlockOffset: dev_auto_sell.targetBlockOffset,
            delayMs: Some(dev_auto_sell.delayMs),
            marketCap: dev_auto_sell
                .marketCap
                .as_ref()
                .map(|trigger| FollowMarketCapTrigger {
                    direction: trigger.direction.clone(),
                    threshold: trigger.threshold.clone(),
                }),
            jitterMs: None,
            feeJitterBps: None,
            precheckRequired: dev_auto_sell.precheckRequired,
            requireConfirmation: dev_auto_sell.requireConfirmation,
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
        });
    }
    for snipe in &follow.snipes {
        if let Some(sell) = &snipe.postBuySell {
            actions.push(FollowActionRecord {
                actionId: sell.actionId.clone(),
                kind: FollowActionKind::SniperSell,
                walletEnvKey: sell.walletEnvKey.clone(),
                state: FollowActionState::Queued,
                buyAmountSol: None,
                sellPercent: Some(sell.percent),
                submitDelayMs: None,
                targetBlockOffset: sell.targetBlockOffset,
                delayMs: Some(sell.delayMs),
                marketCap: sell
                    .marketCap
                    .as_ref()
                    .map(|trigger| FollowMarketCapTrigger {
                        direction: trigger.direction.clone(),
                        threshold: trigger.threshold.clone(),
                    }),
                jitterMs: None,
                feeJitterBps: None,
                precheckRequired: sell.precheckRequired,
                requireConfirmation: sell.requireConfirmation,
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
            });
        }
    }
    actions
}

impl FollowDaemonStore {
    pub fn load_or_default(state_path: PathBuf) -> Self {
        let state = match fs::read_to_string(&state_path) {
            Ok(raw) => serde_json::from_str::<FollowDaemonStateFile>(&raw)
                .unwrap_or_else(|_| Self::default_state(state_path.clone())),
            Err(_) => Self::default_state(state_path.clone()),
        };
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
                maxActiveJobs: 0,
                maxConcurrentCompiles: 0,
                maxConcurrentSends: 0,
                availableCompileSlots: 0,
                availableSendSlots: 0,
                slotWatcher: FollowWatcherHealth::Healthy,
                signatureWatcher: FollowWatcherHealth::Healthy,
                marketWatcher: FollowWatcherHealth::Healthy,
                lastError: None,
                watchEndpoint: None,
            },
            jobs: vec![],
            telemetrySamples: vec![],
            timingProfiles: vec![],
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

    fn refresh_counts(state: &mut FollowDaemonStateFile) {
        state.health.queueDepth = state.jobs.len();
        state.health.activeJobs = Self::active_job_count(&state.jobs);
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
            match fs::write(&self.state_path, &payload) {
                Ok(()) => return Ok(()),
                Err(error) if is_retryable_persist_error(&error) && attempt < 4 => {
                    last_error = Some(error);
                    sleep(Duration::from_millis(20 * (attempt + 1) as u64)).await;
                }
                Err(error) => return Err(error.to_string()),
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

    pub async fn reserve_job(
        &self,
        request: FollowReserveRequest,
    ) -> Result<FollowJobRecord, String> {
        let mut state = self.inner.write().await;
        let now = now_ms();
        if let Some(existing) = state
            .jobs
            .iter_mut()
            .find(|job| job.traceId == request.traceId)
        {
            if existing.launchpad != request.launchpad
                || existing.selectedWalletKey != request.selectedWalletKey
                || existing.execution.provider != request.execution.provider
                || existing.execution.endpointProfile != request.execution.endpointProfile
                || existing.preferPostSetupCreatorVaultForSell
                    != request.preferPostSetupCreatorVaultForSell
                || existing.followLaunch.snipes.len() != request.followLaunch.snipes.len()
                || existing.followLaunch.devAutoSell.is_some()
                    != request.followLaunch.devAutoSell.is_some()
            {
                return Err(format!(
                    "Conflicting follow reserve request for traceId {}. Reused traceIds must keep the same follow-launch payload.",
                    request.traceId
                ));
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
        let actions = build_action_records(&request.followLaunch);
        let job = FollowJobRecord {
            schemaVersion: FOLLOW_JOB_SCHEMA_VERSION,
            traceId: request.traceId.clone(),
            jobId: format!("follow-{}", request.traceId.replace('-', "")),
            state: FollowJobState::Reserved,
            createdAtMs: now,
            updatedAtMs: now,
            launchpad: request.launchpad,
            selectedWalletKey: request.selectedWalletKey,
            execution: request.execution,
            tokenMayhemMode: request.tokenMayhemMode,
            jitoTipAccount: request.jitoTipAccount,
            preferPostSetupCreatorVaultForSell: request.preferPostSetupCreatorVaultForSell,
            mint: None,
            launchCreator: None,
            launchSignature: None,
            submitAtMs: None,
            sendObservedBlockHeight: None,
            reportPath: None,
            transportPlan: None,
            followLaunch: request.followLaunch,
            actions,
            cancelRequested: false,
            lastError: None,
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
        if job.submitAtMs.is_none() {
            job.submitAtMs = Some(request.submitAtMs);
        }
        if job.sendObservedBlockHeight.is_none() {
            job.sendObservedBlockHeight = request.sendObservedBlockHeight;
        }
        if job.reportPath.is_none() {
            job.reportPath = request.reportPath;
        }
        if job.transportPlan.is_none() {
            job.transportPlan = Some(request.transportPlan);
        }
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
        job.cancelRequested = true;
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
            job.state = FollowJobState::Cancelled;
            for action in &mut job.actions {
                if !matches!(
                    action.state,
                    FollowActionState::Confirmed
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
        signature_watcher: FollowWatcherHealth,
        market_watcher: FollowWatcherHealth,
        last_error: Option<String>,
    ) -> Result<(), String> {
        let mut state = self.inner.write().await;
        state.health.updatedAtMs = now_ms();
        state.health.watchEndpoint = watch_endpoint;
        state.health.slotWatcher = slot_watcher;
        state.health.signatureWatcher = signature_watcher;
        state.health.marketWatcher = market_watcher;
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

    pub async fn record_sample(&self, sample: FollowTelemetrySample) -> Result<(), String> {
        let mut state = self.inner.write().await;
        state.telemetrySamples.push(sample);
        let mut grouped: HashMap<(String, String, String), Vec<&FollowTelemetrySample>> =
            HashMap::new();
        for sample in &state.telemetrySamples {
            grouped
                .entry((
                    sample.provider.clone(),
                    sample.endpointProfile.clone(),
                    sample.actionType.clone(),
                ))
                .or_default()
                .push(sample);
        }
        state.timingProfiles = grouped
            .into_iter()
            .map(|((provider, endpoint_profile, action_type), samples)| {
                let mut submit_latencies = samples
                    .iter()
                    .filter(|sample| outcome_success_weight(&sample.outcome).0)
                    .filter_map(|sample| sample.submitLatencyMs)
                    .collect::<Vec<_>>();
                submit_latencies.sort_unstable();
                let success_count = samples
                    .iter()
                    .filter(|sample| outcome_success_weight(&sample.outcome).0)
                    .count();
                let success_rate = if samples.is_empty() {
                    0.0
                } else {
                    success_count as f64 / samples.len() as f64
                };
                let weighted_quality_score = if samples.is_empty() {
                    0.0
                } else {
                    samples
                        .iter()
                        .map(|sample| {
                            let (_, outcome_weight) = outcome_success_weight(&sample.outcome);
                            outcome_weight * f64::from(sample.qualityWeight)
                        })
                        .sum::<f64>()
                        / samples.len() as f64
                };
                let p50_submit_ms = percentile(&submit_latencies, 0.50);
                let recommended_delay = if weighted_quality_score >= 80.0 {
                    p50_submit_ms
                } else if weighted_quality_score >= 60.0 {
                    percentile(&submit_latencies, 0.75)
                } else {
                    percentile(&submit_latencies, 0.90)
                };
                let suggested_jitter_ms = recommended_delay.map(|value| (value / 10).max(5));
                let recommendation = FollowTimingRecommendation {
                    suggestedSubmitDelayMs: recommended_delay,
                    suggestedJitterMs: suggested_jitter_ms,
                    confidence: recommendation_confidence(samples.len(), success_rate),
                    successRate: success_rate,
                    weightedQualityScore: weighted_quality_score,
                    sampleCount: samples.len(),
                };
                FollowTimingProfile {
                    schemaVersion: FOLLOW_TELEMETRY_SCHEMA_VERSION,
                    provider,
                    endpointProfile: endpoint_profile,
                    actionType: action_type,
                    sampleCount: samples.len(),
                    successCount: success_count,
                    weightedQualityScore: weighted_quality_score,
                    p50SubmitMs: p50_submit_ms,
                    p75SubmitMs: percentile(&submit_latencies, 0.75),
                    p90SubmitMs: percentile(&submit_latencies, 0.90),
                    recommendation,
                }
            })
            .collect();
        drop(state);
        self.persist().await
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
        Self {
            baseUrl: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
            authToken: configured_follow_daemon_auth_token(),
        }
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
        if let Some(token) = &self.authToken {
            request = request.header("x-launchdeck-engine-auth", token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.map_err(|error| error.to_string())?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Follow daemon request failed with status {}: {}",
                status, body
            ));
        }
        response
            .json::<TResponse>()
            .await
            .map_err(|error| error.to_string())
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
    timing_profiles: Vec<FollowTimingProfile>,
) -> FollowJobResponse {
    FollowJobResponse {
        schemaVersion: FOLLOW_RESPONSE_SCHEMA_VERSION,
        ok: true,
        job,
        jobs,
        health,
        timingProfiles: timing_profiles,
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
