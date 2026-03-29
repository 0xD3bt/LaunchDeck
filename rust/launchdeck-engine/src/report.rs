#![allow(non_snake_case, dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    config::{NormalizedConfig, NormalizedRecipient},
    transport::TransportPlan,
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
    pub v0Length: Option<usize>,
    pub v0AltLength: Option<usize>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionTimings {
    pub totalElapsedMs: Option<u128>,
    pub backendTotalElapsedMs: Option<u128>,
    pub clientPreRequestMs: Option<u128>,
    pub formToRawConfigMs: Option<u128>,
    pub normalizeConfigMs: Option<u128>,
    pub walletLoadMs: Option<u128>,
    pub reportBuildMs: Option<u128>,
    pub compileTransactionsMs: Option<u128>,
    pub compileAltLoadMs: Option<u128>,
    pub compileBlockhashFetchMs: Option<u128>,
    pub compileGlobalFetchMs: Option<u128>,
    pub compileFollowUpPrepMs: Option<u128>,
    pub compileTxSerializeMs: Option<u128>,
    pub simulateMs: Option<u128>,
    pub sendMs: Option<u128>,
    pub sendSubmitMs: Option<u128>,
    pub sendConfirmMs: Option<u128>,
    pub persistReportMs: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkSentItem {
    pub label: String,
    pub signature: Option<String>,
    pub confirmationStatus: Option<String>,
    pub sendBlockHeight: Option<u64>,
    pub confirmedBlockHeight: Option<u64>,
    pub blocksToConfirm: Option<u64>,
    pub confirmedSlot: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkSummary {
    pub timings: ExecutionTimings,
    pub sent: Vec<BenchmarkSentItem>,
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
    pub warnings: Vec<String>,
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

fn short_address(value: &str, left: usize, right: usize) -> String {
    if value.is_empty() {
        return "(unknown)".to_string();
    }
    if value.len() <= left + right {
        return value.to_string();
    }
    format!("{}...{}", &value[..left], &value[value.len() - right..])
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

fn planned_transactions(
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    lookup_tables: &[String],
) -> Vec<TransactionSummary> {
    let mut transactions = vec![TransactionSummary {
        label: "launch".to_string(),
        instructionSummary: vec![],
        legacyLength: None,
        v0Length: None,
        v0AltLength: None,
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

    let follow_up = match config.mode.as_str() {
        "regular" | "cashback" if config.feeSharing.generateLaterSetup => Some("follow-up"),
        "agent-custom" | "agent-locked" => Some("agent-setup"),
        _ => None,
    };
    if let Some(label) = follow_up {
        transactions.push(TransactionSummary {
            label: label.to_string(),
            instructionSummary: vec![],
            legacyLength: None,
            v0Length: None,
            v0AltLength: None,
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
            v0Length: None,
            v0AltLength: None,
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
        "agent-custom" => summarize_recipients(
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
        "agent-custom" => "agent custom split bundled post-launch (final)".to_string(),
        "agent-unlocked" => "untouched on launch (configure later once)".to_string(),
        "agent-locked" => "locked to agent escrow".to_string(),
        _ if config.feeSharing.generateLaterSetup => "deferred one-time setup artifact".to_string(),
        _ => "untouched".to_string(),
    };
    let mut warnings = vec![
        "Rust engine owns validation, runtime state, and API contracts. Native Pump assembly covers LaunchDeck's Pump launch modes end-to-end."
            .to_string(),
    ];
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
            warnings,
        },
        followDaemon: None,
        benchmark: None,
        outPath: None,
    }
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
    lines.push(format!(
        "Provider: {}",
        report.execution.resolvedProvider
    ));
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
    lines.push(format!(
        "Track send block height: {}",
        report.execution.trackSendBlockHeight
    ));
    if let Some(endpoint) = &report.execution.heliusSenderEndpoint {
        lines.push(format!("Helius Sender endpoint: {}", endpoint));
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
                }
            }
            if let Some(profiles) = follow.get("timingProfiles").and_then(Value::as_array)
                && !profiles.is_empty()
            {
                lines.push("  Timing advice:".to_string());
                for profile in profiles {
                    let action_type = profile
                        .get("actionType")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let sample_count = profile
                        .get("sampleCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    let weighted_quality_score = profile
                        .get("weightedQualityScore")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0);
                    let recommendation = profile
                        .get("recommendation")
                        .cloned()
                        .unwrap_or(Value::Null);
                    let confidence = recommendation
                        .get("confidence")
                        .and_then(Value::as_str)
                        .unwrap_or("low");
                    let success_rate = recommendation
                        .get("successRate")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0);
                    let suggested_delay = recommendation
                        .get("suggestedSubmitDelayMs")
                        .and_then(Value::as_u64);
                    let suggested_jitter = recommendation
                        .get("suggestedJitterMs")
                        .and_then(Value::as_u64);
                    let mut summary = format!(
                        "  - {}: samples={} | confidence={} | success={:.0}% | quality={:.1}",
                        action_type,
                        sample_count,
                        confidence,
                        success_rate * 100.0,
                        weighted_quality_score
                    );
                    if let Some(delay) = suggested_delay {
                        summary.push_str(&format!(" | suggest delay={}ms", delay));
                    }
                    if let Some(jitter) = suggested_jitter {
                        summary.push_str(&format!(" | suggest jitter={}ms", jitter));
                    }
                    lines.push(summary);
                }
            }
        }
    }
    if let Some(benchmark) = &report.benchmark {
        lines.push("Benchmark:".to_string());
        let timings = &benchmark.timings;
        let mut timing_parts = Vec::new();
        if let Some(value) = timings.totalElapsedMs {
            timing_parts.push(format!("total={value}ms"));
        }
        if let Some(value) = timings.backendTotalElapsedMs {
            timing_parts.push(format!("backendTotal={value}ms"));
        }
        if let Some(value) = timings.clientPreRequestMs {
            timing_parts.push(format!("preRequest={value}ms"));
        }
        if let Some(value) = timings.formToRawConfigMs {
            timing_parts.push(format!("form={value}ms"));
        }
        if let Some(value) = timings.normalizeConfigMs {
            timing_parts.push(format!("normalize={value}ms"));
        }
        if let Some(value) = timings.walletLoadMs {
            timing_parts.push(format!("wallet={value}ms"));
        }
        if let Some(value) = timings.reportBuildMs {
            timing_parts.push(format!("report={value}ms"));
        }
        if let Some(value) = timings.compileTransactionsMs {
            timing_parts.push(format!("compile={value}ms"));
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
            timing_parts.push(format!("serialize={value}ms"));
        }
        if let Some(value) = timings.simulateMs {
            timing_parts.push(format!("simulate={value}ms"));
        }
        if let Some(value) = timings.sendMs {
            timing_parts.push(format!("send={value}ms"));
        }
        if let Some(value) = timings.sendSubmitMs {
            timing_parts.push(format!("submit={value}ms"));
        }
        if let Some(value) = timings.sendConfirmMs {
            timing_parts.push(format!("confirm={value}ms"));
        }
        if let Some(value) = timings.persistReportMs {
            timing_parts.push(format!("persist={value}ms"));
        }
        if !timing_parts.is_empty() {
            lines.push(format!("  Timings: {}", timing_parts.join(" | ")));
        }
        for sent in &benchmark.sent {
            let mut sent_parts = Vec::new();
            if let Some(value) = sent.sendBlockHeight {
                sent_parts.push(format!("send block height={value}"));
            }
            if let Some(value) = sent.confirmedBlockHeight {
                sent_parts.push(format!("confirmed block height={value}"));
            }
            if let Some(value) = sent.blocksToConfirm {
                sent_parts.push(format!("blocks to confirm={value}"));
            }
            if let Some(value) = sent.confirmedSlot {
                sent_parts.push(format!("confirmed slot={value}"));
            }
            if !sent_parts.is_empty() {
                lines.push(format!("  {}: {}", sent.label, sent_parts.join(" | ")));
            }
        }
    }
    lines.push(String::new());
    lines.push("Transactions:".to_string());
    for tx in &report.transactions {
        lines.push(format!(
            "- {}: {} instructions | legacy={} | v0={} | v0+alt={}",
            tx.label,
            tx.instructionSummary.len(),
            tx.legacyLength
                .map(|value| format!("{value} bytes"))
                .unwrap_or_else(|| "n/a".to_string()),
            tx.v0Length
                .map(|value| format!("{value} bytes"))
                .unwrap_or_else(|| "n/a".to_string()),
            tx.v0AltLength
                .map(|value| format!("{value} bytes"))
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
                sent.label,
                sent.signature.as_deref().unwrap_or("(missing)"),
                sent.confirmationStatus.as_deref().unwrap_or("(pending)")
            );
            if let Some(block_height) = sent.sendObservedBlockHeight {
                summary.push_str(&format!(" | send block height={block_height}"));
            }
            if let Some(block_height) = sent.confirmedObservedBlockHeight {
                summary.push_str(&format!(" | confirmed block height={block_height}"));
            }
            if let (Some(send_height), Some(confirmed_height)) = (
                sent.sendObservedBlockHeight,
                sent.confirmedObservedBlockHeight,
            ) {
                summary.push_str(&format!(
                    " | blocks to confirm={}",
                    confirmed_height.saturating_sub(send_height)
                ));
            }
            if let Some(slot) = sent.confirmedSlot {
                summary.push_str(&format!(" | confirmed slot={slot}"));
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
}
