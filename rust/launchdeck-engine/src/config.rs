#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

const TOKEN_NAME_MAX_LENGTH: usize = 32;
const TOKEN_SYMBOL_MAX_LENGTH: usize = 10;
const MAX_FEE_SPLIT_RECIPIENTS: usize = 10;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawConfig {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub launchpad: String,
    #[serde(default)]
    pub token: RawToken,
    #[serde(default)]
    pub signer: RawSigner,
    #[serde(default)]
    pub agent: RawAgent,
    #[serde(default)]
    pub tx: RawTx,
    #[serde(default)]
    pub feeSharing: RawFeeSharing,
    #[serde(default)]
    pub creatorFee: RawCreatorFee,
    #[serde(default)]
    pub execution: RawExecution,
    #[serde(default)]
    pub initialBuySol: String,
    #[serde(default)]
    pub initialBuyTokens: String,
    #[serde(default)]
    pub devBuy: Option<RawDevBuy>,
    #[serde(default)]
    pub postLaunch: RawPostLaunch,
    #[serde(default)]
    pub followLaunch: RawFollowLaunch,
    #[serde(default)]
    pub presets: RawPresets,
    #[serde(default)]
    pub imageLocalPath: String,
    #[serde(default)]
    pub selectedWalletKey: String,
    #[serde(default)]
    pub vanityPrivateKey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawToken {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub uri: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub twitter: String,
    #[serde(default)]
    pub telegram: String,
    #[serde(default)]
    pub mayhemMode: Option<Value>,
    #[serde(default)]
    pub cashback: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawSigner {
    #[serde(default)]
    pub keypairFile: String,
    #[serde(default)]
    pub secretKey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawAgent {
    #[serde(default)]
    pub authority: String,
    #[serde(default)]
    pub buybackBps: Option<Value>,
    #[serde(default)]
    pub splitAgentInit: Option<Value>,
    #[serde(default)]
    pub feeReceiver: String,
    #[serde(default)]
    pub feeRecipients: Vec<RawRecipient>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawTx {
    #[serde(default)]
    pub computeUnitLimit: Option<Value>,
    #[serde(default)]
    pub computeUnitPriceMicroLamports: Option<Value>,
    #[serde(default)]
    pub jitoTipLamports: Option<Value>,
    #[serde(default)]
    pub jitoTipAccount: String,
    #[serde(default)]
    pub lookupTables: Vec<String>,
    #[serde(default)]
    pub useDefaultLookupTables: Option<Value>,
    #[serde(default)]
    pub dumpBase64: Option<Value>,
    #[serde(default)]
    pub writeReport: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawFeeSharing {
    #[serde(default)]
    pub generateLaterSetup: Option<Value>,
    #[serde(default)]
    pub recipients: Vec<RawRecipient>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawCreatorFee {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub githubUsername: String,
    #[serde(default)]
    pub githubUserId: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawExecution {
    #[serde(default)]
    pub simulate: Option<Value>,
    #[serde(default)]
    pub send: Option<Value>,
    #[serde(default)]
    pub txFormat: String,
    #[serde(default)]
    pub commitment: String,
    #[serde(default)]
    pub skipPreflight: Option<Value>,
    #[serde(default)]
    pub trackSendBlockHeight: Option<Value>,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub endpointProfile: String,
    #[serde(default)]
    pub policy: String,
    #[serde(default)]
    pub autoGas: Option<Value>,
    #[serde(default)]
    pub autoMode: String,
    #[serde(default)]
    pub priorityFeeSol: String,
    #[serde(default)]
    pub tipSol: String,
    #[serde(default)]
    pub maxPriorityFeeSol: String,
    #[serde(default)]
    pub maxTipSol: String,
    #[serde(default)]
    pub buyProvider: String,
    #[serde(default)]
    pub buyEndpointProfile: String,
    #[serde(default)]
    pub buyPolicy: String,
    #[serde(default)]
    pub buyAutoGas: Option<Value>,
    #[serde(default)]
    pub buyAutoMode: String,
    #[serde(default)]
    pub buyPriorityFeeSol: String,
    #[serde(default)]
    pub buyTipSol: String,
    #[serde(default)]
    pub buySlippagePercent: String,
    #[serde(default)]
    pub buyMaxPriorityFeeSol: String,
    #[serde(default)]
    pub buyMaxTipSol: String,
    #[serde(default)]
    pub sellProvider: String,
    #[serde(default)]
    pub sellEndpointProfile: String,
    #[serde(default)]
    pub sellPolicy: String,
    #[serde(default)]
    pub sellPriorityFeeSol: String,
    #[serde(default)]
    pub sellTipSol: String,
    #[serde(default)]
    pub sellSlippagePercent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawDevBuy {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawPostLaunch {
    #[serde(default)]
    pub strategy: String,
    #[serde(default)]
    pub snipeOwnLaunch: RawSnipeOwnLaunch,
    #[serde(default)]
    pub automaticDevSell: RawAutomaticDevSell,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawSnipeOwnLaunch {
    #[serde(default)]
    pub buyAmountSol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawAutomaticDevSell {
    #[serde(default)]
    pub enabled: Option<Value>,
    #[serde(default)]
    pub percent: Option<Value>,
    #[serde(default)]
    pub delaySeconds: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawFollowLaunch {
    #[serde(default)]
    pub enabled: Option<Value>,
    #[serde(default)]
    pub schemaVersion: Option<Value>,
    #[serde(default)]
    pub snipes: Vec<RawFollowLaunchSnipe>,
    #[serde(default)]
    pub devAutoSell: Option<RawFollowLaunchSell>,
    #[serde(default)]
    pub constraints: RawFollowLaunchConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawFollowLaunchSnipe {
    #[serde(default)]
    pub actionId: String,
    #[serde(default)]
    pub enabled: Option<Value>,
    #[serde(default)]
    pub walletEnvKey: String,
    #[serde(default)]
    pub buyAmountSol: String,
    #[serde(default)]
    pub submitWithLaunch: Option<Value>,
    #[serde(default)]
    pub retryOnFailure: Option<Value>,
    #[serde(default)]
    pub submitDelayMs: Option<Value>,
    #[serde(default)]
    pub targetBlockOffset: Option<Value>,
    #[serde(default)]
    pub jitterMs: Option<Value>,
    #[serde(default)]
    pub feeJitterBps: Option<Value>,
    #[serde(default)]
    pub postBuySell: Option<RawFollowLaunchSell>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawFollowLaunchSell {
    #[serde(default)]
    pub actionId: String,
    #[serde(default)]
    pub enabled: Option<Value>,
    #[serde(default)]
    pub walletEnvKey: String,
    #[serde(default)]
    pub percent: Option<Value>,
    #[serde(default)]
    pub delayMs: Option<Value>,
    #[serde(default)]
    pub targetBlockOffset: Option<Value>,
    #[serde(default)]
    pub marketCap: RawFollowLaunchMarketCapTrigger,
    #[serde(default)]
    pub precheckRequired: Option<Value>,
    #[serde(default)]
    pub requireConfirmation: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawFollowLaunchMarketCapTrigger {
    #[serde(default)]
    pub enabled: Option<Value>,
    #[serde(default)]
    pub direction: String,
    #[serde(default)]
    pub threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawFollowLaunchConstraints {
    #[serde(default)]
    pub pumpOnly: Option<Value>,
    #[serde(default)]
    pub retryBudget: Option<Value>,
    #[serde(default)]
    pub requireDaemonReadiness: Option<Value>,
    #[serde(default)]
    pub blockOnRequiredPrechecks: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawPresets {
    #[serde(default)]
    pub activePresetId: String,
    #[serde(default)]
    pub selectedLaunchPresetId: String,
    #[serde(default)]
    pub selectedSniperPresetId: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawRecipient {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub githubUsername: String,
    #[serde(default)]
    pub githubUserId: String,
    #[serde(default)]
    pub shareBps: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedConfig {
    pub mode: String,
    pub launchpad: String,
    pub token: NormalizedToken,
    pub signer: NormalizedSigner,
    pub agent: NormalizedAgent,
    pub tx: NormalizedTx,
    pub feeSharing: NormalizedFeeSharing,
    pub creatorFee: NormalizedCreatorFee,
    pub execution: NormalizedExecution,
    pub devBuy: Option<NormalizedDevBuy>,
    pub postLaunch: NormalizedPostLaunch,
    pub followLaunch: NormalizedFollowLaunch,
    pub presets: NormalizedPresets,
    pub imageLocalPath: String,
    pub selectedWalletKey: String,
    pub vanityPrivateKey: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedToken {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub description: String,
    pub website: String,
    pub twitter: String,
    pub telegram: String,
    pub mayhemMode: bool,
    pub cashback: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedSigner {
    pub keypairFile: String,
    pub secretKey: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedAgent {
    pub authority: String,
    pub buybackBps: Option<i64>,
    pub splitAgentInit: bool,
    pub feeReceiver: String,
    pub feeRecipients: Vec<NormalizedRecipient>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedTx {
    pub computeUnitLimit: Option<i64>,
    pub computeUnitPriceMicroLamports: Option<i64>,
    pub jitoTipLamports: i64,
    pub jitoTipAccount: String,
    pub lookupTables: Vec<String>,
    pub useDefaultLookupTables: bool,
    pub dumpBase64: bool,
    pub writeReport: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedFeeSharing {
    pub generateLaterSetup: bool,
    pub recipients: Vec<NormalizedRecipient>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedCreatorFee {
    pub mode: String,
    pub address: String,
    pub githubUsername: String,
    pub githubUserId: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedExecution {
    pub simulate: bool,
    pub send: bool,
    pub txFormat: String,
    pub commitment: String,
    pub skipPreflight: bool,
    pub trackSendBlockHeight: bool,
    pub provider: String,
    pub endpointProfile: String,
    pub autoGas: bool,
    pub autoMode: String,
    pub priorityFeeSol: String,
    pub tipSol: String,
    pub maxPriorityFeeSol: String,
    pub maxTipSol: String,
    pub buyProvider: String,
    pub buyEndpointProfile: String,
    pub buyAutoGas: bool,
    pub buyAutoMode: String,
    pub buyPriorityFeeSol: String,
    pub buyTipSol: String,
    pub buySlippagePercent: String,
    pub buyMaxPriorityFeeSol: String,
    pub buyMaxTipSol: String,
    pub sellProvider: String,
    pub sellEndpointProfile: String,
    pub sellPriorityFeeSol: String,
    pub sellTipSol: String,
    pub sellSlippagePercent: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedDevBuy {
    pub mode: String,
    pub amount: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedPostLaunch {
    pub strategy: String,
    pub snipeOwnLaunch: NormalizedSnipeOwnLaunch,
    pub automaticDevSell: NormalizedAutomaticDevSell,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedSnipeOwnLaunch {
    pub enabled: bool,
    pub buyAmountSol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedAutomaticDevSell {
    pub enabled: bool,
    pub percent: i64,
    pub delaySeconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFollowLaunch {
    pub enabled: bool,
    pub source: String,
    pub schemaVersion: u32,
    pub snipes: Vec<NormalizedFollowLaunchSnipe>,
    pub devAutoSell: Option<NormalizedFollowLaunchSell>,
    pub constraints: NormalizedFollowLaunchConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFollowLaunchSnipe {
    pub actionId: String,
    pub enabled: bool,
    pub walletEnvKey: String,
    pub buyAmountSol: String,
    pub submitWithLaunch: bool,
    #[serde(default)]
    pub retryOnFailure: bool,
    pub submitDelayMs: u64,
    pub targetBlockOffset: Option<u8>,
    pub jitterMs: u64,
    pub feeJitterBps: u16,
    #[serde(default)]
    pub skipIfTokenBalancePositive: bool,
    pub postBuySell: Option<NormalizedFollowLaunchSell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFollowLaunchSell {
    pub actionId: String,
    pub enabled: bool,
    pub walletEnvKey: String,
    pub percent: u8,
    pub delayMs: u64,
    pub targetBlockOffset: Option<u8>,
    pub marketCap: Option<NormalizedFollowLaunchMarketCapTrigger>,
    pub precheckRequired: bool,
    pub requireConfirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFollowLaunchMarketCapTrigger {
    pub direction: String,
    pub threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFollowLaunchConstraints {
    pub pumpOnly: bool,
    pub retryBudget: u32,
    pub requireDaemonReadiness: bool,
    pub blockOnRequiredPrechecks: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedPresets {
    pub activePresetId: String,
    pub selectedLaunchPresetId: String,
    pub selectedSniperPresetId: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedRecipient {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub address: String,
    pub githubUserId: String,
    pub githubUsername: String,
    pub shareBps: i64,
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn parse_bool(value: &Option<Value>, fallback: bool) -> bool {
    match value {
        None => fallback,
        Some(Value::Bool(v)) => *v,
        Some(Value::String(v)) => match v.trim().to_lowercase().as_str() {
            "true" => true,
            "false" => false,
            _ => !v.trim().is_empty(),
        },
        Some(Value::Number(n)) => n.as_i64().unwrap_or_default() != 0,
        Some(Value::Null) => fallback,
        Some(_) => fallback,
    }
}

fn parse_int(
    value: &Option<Value>,
    label: &str,
    min: Option<i64>,
    max: Option<i64>,
    fallback: Option<i64>,
) -> Result<Option<i64>, ConfigError> {
    let Some(value) = value else {
        return Ok(fallback);
    };
    if matches!(value, Value::Null) {
        return Ok(fallback);
    }
    if let Value::String(s) = value
        && s.trim().is_empty()
    {
        return Ok(fallback);
    }
    let parsed = match value {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    };
    let Some(parsed) = parsed else {
        return Err(ConfigError::Message(format!(
            "{label} must be an integer. Got: {value}"
        )));
    };
    if let Some(min) = min
        && parsed < min
    {
        return Err(ConfigError::Message(format!(
            "{label} must be >= {min}. Got: {parsed}"
        )));
    }
    if let Some(max) = max
        && parsed > max
    {
        return Err(ConfigError::Message(format!(
            "{label} must be <= {max}. Got: {parsed}"
        )));
    }
    Ok(Some(parsed))
}

fn parse_choice(
    value: &str,
    label: &str,
    allowed: &[&str],
    fallback: &str,
) -> Result<String, ConfigError> {
    let normalized = if is_blank(value) { fallback } else { value }
        .trim()
        .to_lowercase();
    if allowed.iter().any(|entry| *entry == normalized) {
        Ok(normalized)
    } else {
        Err(ConfigError::Message(format!(
            "{label} must be one of {}. Got: {value}",
            allowed.join(", ")
        )))
    }
}

fn parse_limited_string(
    value: &str,
    label: &str,
    max_length: usize,
    required: bool,
) -> Result<String, ConfigError> {
    let normalized = value.trim().to_string();
    if required && normalized.is_empty() {
        return Err(ConfigError::Message(format!("{label} is required.")));
    }
    if normalized.len() > max_length {
        return Err(ConfigError::Message(format!(
            "{label} must be at most {max_length} characters."
        )));
    }
    Ok(normalized)
}

fn normalize_recipients(
    entries: &[RawRecipient],
    label_prefix: &str,
    allow_agent: bool,
) -> Result<Vec<NormalizedRecipient>, ConfigError> {
    if entries.len() > MAX_FEE_SPLIT_RECIPIENTS {
        return Err(ConfigError::Message(format!(
            "{}s support at most {} recipients.",
            label_prefix, MAX_FEE_SPLIT_RECIPIENTS
        )));
    }
    let mut normalized = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.shareBps.is_none() {
            continue;
        }
        let share_bps = parse_int(
            &entry.shareBps,
            &format!("{label_prefix} shareBps at index {index}"),
            Some(1),
            Some(10_000),
            None,
        )?
        .unwrap_or_default();
        let entry_type = if entry.r#type.trim().is_empty() {
            "wallet".to_string()
        } else {
            entry.r#type.trim().to_lowercase()
        };
        let address = entry.address.trim().to_string();
        let github_user_id = entry.githubUserId.trim().to_string();
        let github_username = entry.githubUsername.trim().to_string();

        if allow_agent && entry_type == "agent" {
            normalized.push(NormalizedRecipient {
                r#type: Some(entry_type),
                address: String::new(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: share_bps,
            });
            continue;
        }

        if address.is_empty() && github_user_id.is_empty() {
            return Err(ConfigError::Message(format!(
                "{label_prefix} at index {index} must provide either address or githubUserId."
            )));
        }

        normalized.push(NormalizedRecipient {
            r#type: if allow_agent { Some(entry_type) } else { None },
            address,
            githubUserId: github_user_id,
            githubUsername: github_username,
            shareBps: share_bps,
        });
    }

    if normalized.is_empty() {
        return Ok(normalized);
    }
    let total: i64 = normalized.iter().map(|entry| entry.shareBps).sum();
    if total != 10_000 {
        return Err(ConfigError::Message(format!(
            "{}s must total 10000 bps. Got: {}",
            label_prefix, total
        )));
    }

    Ok(normalized)
}

fn normalize_creator_fee(
    raw: &RawCreatorFee,
    mode: &str,
) -> Result<NormalizedCreatorFee, ConfigError> {
    match mode {
        "agent-locked" => {
            return Ok(NormalizedCreatorFee {
                mode: "agent-escrow".to_string(),
                address: String::new(),
                githubUsername: String::new(),
                githubUserId: String::new(),
            });
        }
        "agent-custom" | "agent-unlocked" => {
            return Ok(NormalizedCreatorFee {
                mode: "deployer".to_string(),
                address: String::new(),
                githubUsername: String::new(),
                githubUserId: String::new(),
            });
        }
        "cashback" => {
            return Ok(NormalizedCreatorFee {
                mode: "cashback".to_string(),
                address: String::new(),
                githubUsername: String::new(),
                githubUserId: String::new(),
            });
        }
        _ => {}
    }

    let normalized_mode = if is_blank(&raw.mode) {
        "deployer".to_string()
    } else {
        raw.mode.trim().to_lowercase()
    };
    if !["deployer", "wallet", "github"].contains(&normalized_mode.as_str()) {
        return Err(ConfigError::Message(format!(
            "creatorFee.mode must be one of deployer, wallet, github. Got: {}",
            normalized_mode
        )));
    }
    let normalized = NormalizedCreatorFee {
        mode: normalized_mode,
        address: raw.address.trim().to_string(),
        githubUsername: raw.githubUsername.trim().to_string(),
        githubUserId: raw.githubUserId.trim().to_string(),
    };
    if normalized.mode == "wallet" && normalized.address.is_empty() {
        return Err(ConfigError::Message(
            "creatorFee.address is required when creatorFee.mode is wallet.".to_string(),
        ));
    }
    if normalized.mode == "github" && normalized.githubUserId.is_empty() {
        return Err(ConfigError::Message(
            "creatorFee.githubUserId is required when creatorFee.mode is github.".to_string(),
        ));
    }
    Ok(normalized)
}

fn normalize_dev_buy(raw: &RawConfig) -> Result<Option<NormalizedDevBuy>, ConfigError> {
    let object_mode = raw
        .devBuy
        .as_ref()
        .map(|entry| entry.mode.trim().to_string())
        .unwrap_or_default();
    let object_amount = raw
        .devBuy
        .as_ref()
        .map(|entry| entry.amount.trim().to_string())
        .unwrap_or_default();
    let shorthand_sol = raw.initialBuySol.trim().to_string();
    let shorthand_tokens = raw.initialBuyTokens.trim().to_string();

    if !shorthand_sol.is_empty() && !shorthand_tokens.is_empty() {
        return Err(ConfigError::Message(
            "Use only one of initialBuySol or initialBuyTokens.".to_string(),
        ));
    }
    if !object_mode.is_empty() || !object_amount.is_empty() {
        if !shorthand_sol.is_empty() || !shorthand_tokens.is_empty() {
            return Err(ConfigError::Message(
                "Use either devBuy object form or shorthand initialBuySol/initialBuyTokens, not both."
                    .to_string(),
            ));
        }
        if object_mode.is_empty() || object_amount.is_empty() {
            return Err(ConfigError::Message(
                "devBuy.mode and devBuy.amount must both be set.".to_string(),
            ));
        }
        let mode = object_mode.to_lowercase();
        if mode != "sol" && mode != "tokens" {
            return Err(ConfigError::Message(format!(
                "devBuy.mode must be 'sol' or 'tokens'. Got: {}",
                object_mode
            )));
        }
        return Ok(Some(NormalizedDevBuy {
            mode,
            amount: object_amount,
            source: "object".to_string(),
        }));
    }
    if !shorthand_sol.is_empty() {
        return Ok(Some(NormalizedDevBuy {
            mode: "sol".to_string(),
            amount: shorthand_sol,
            source: "shorthand".to_string(),
        }));
    }
    if !shorthand_tokens.is_empty() {
        return Ok(Some(NormalizedDevBuy {
            mode: "tokens".to_string(),
            amount: shorthand_tokens,
            source: "shorthand".to_string(),
        }));
    }
    Ok(None)
}

fn normalize_follow_sell(
    raw: &RawFollowLaunchSell,
    fallback_wallet_env_key: &str,
    fallback_action_id: &str,
) -> Result<Option<NormalizedFollowLaunchSell>, ConfigError> {
    let enabled = parse_bool(&raw.enabled, false);
    let market_cap_enabled =
        parse_bool(&raw.marketCap.enabled, false) && !raw.marketCap.threshold.trim().is_empty();
    let has_payload = enabled
        || raw.percent.is_some()
        || raw.delayMs.is_some()
        || raw.targetBlockOffset.is_some()
        || market_cap_enabled
        || !raw.walletEnvKey.trim().is_empty();
    if !has_payload {
        return Ok(None);
    }
    let percent = parse_int(
        &raw.percent,
        &format!("{fallback_action_id}.percent"),
        Some(0),
        Some(100),
        Some(100),
    )?
    .unwrap_or(100) as u8;
    let delay_ms = parse_int(
        &raw.delayMs,
        &format!("{fallback_action_id}.delayMs"),
        Some(0),
        None,
        Some(0),
    )?
    .unwrap_or(0) as u64;
    let target_block_offset = parse_int(
        &raw.targetBlockOffset,
        &format!("{fallback_action_id}.targetBlockOffset"),
        Some(0),
        Some(32),
        None,
    )?
    .map(|value| value as u8);
    let direction = if raw.marketCap.direction.trim().is_empty() {
        "gte".to_string()
    } else {
        parse_choice(
            &raw.marketCap.direction,
            &format!("{fallback_action_id}.marketCap.direction"),
            &["gte", "lte"],
            "gte",
        )?
    };
    Ok(Some(NormalizedFollowLaunchSell {
        actionId: if raw.actionId.trim().is_empty() {
            fallback_action_id.to_string()
        } else {
            raw.actionId.trim().to_string()
        },
        enabled: enabled || percent > 0,
        walletEnvKey: if raw.walletEnvKey.trim().is_empty() {
            fallback_wallet_env_key.trim().to_string()
        } else {
            raw.walletEnvKey.trim().to_string()
        },
        percent,
        delayMs: delay_ms,
        targetBlockOffset: target_block_offset,
        marketCap: if market_cap_enabled {
            Some(NormalizedFollowLaunchMarketCapTrigger {
                direction,
                threshold: raw.marketCap.threshold.trim().to_string(),
            })
        } else {
            None
        },
        precheckRequired: parse_bool(&raw.precheckRequired, false),
        requireConfirmation: parse_bool(&raw.requireConfirmation, true),
    }))
}

fn follow_sell_has_payload(raw: &RawFollowLaunchSell) -> bool {
    parse_bool(&raw.enabled, false)
        || raw.percent.is_some()
        || raw.delayMs.is_some()
        || raw.targetBlockOffset.is_some()
        || (parse_bool(&raw.marketCap.enabled, false) && !raw.marketCap.threshold.trim().is_empty())
        || !raw.walletEnvKey.trim().is_empty()
}

fn legacy_follow_launch(
    raw: &RawConfig,
    post_launch_strategy: &str,
) -> Result<NormalizedFollowLaunch, ConfigError> {
    let mut snipes = Vec::new();
    if post_launch_strategy == "snipe-own-launch"
        && !raw.postLaunch.snipeOwnLaunch.buyAmountSol.trim().is_empty()
    {
        snipes.push(NormalizedFollowLaunchSnipe {
            actionId: "legacy-snipe-1".to_string(),
            enabled: true,
            walletEnvKey: raw.selectedWalletKey.trim().to_string(),
            buyAmountSol: raw
                .postLaunch
                .snipeOwnLaunch
                .buyAmountSol
                .trim()
                .to_string(),
            submitWithLaunch: false,
            retryOnFailure: false,
            submitDelayMs: 0,
            targetBlockOffset: None,
            jitterMs: 0,
            feeJitterBps: 0,
            skipIfTokenBalancePositive: false,
            postBuySell: None,
        });
    }
    let dev_auto_sell = if post_launch_strategy == "automatic-dev-sell"
        || parse_bool(&raw.postLaunch.automaticDevSell.enabled, false)
    {
        Some(NormalizedFollowLaunchSell {
            actionId: "legacy-dev-auto-sell".to_string(),
            enabled: true,
            walletEnvKey: raw.selectedWalletKey.trim().to_string(),
            percent: parse_int(
                &raw.postLaunch.automaticDevSell.percent,
                "postLaunch.automaticDevSell.percent",
                Some(0),
                Some(100),
                Some(100),
            )?
            .unwrap_or(100) as u8,
            delayMs: (parse_int(
                &raw.postLaunch.automaticDevSell.delaySeconds,
                "postLaunch.automaticDevSell.delaySeconds",
                Some(0),
                Some(10),
                Some(0),
            )?
            .unwrap_or(0) as u64)
                * 1000,
            targetBlockOffset: None,
            marketCap: None,
            precheckRequired: false,
            requireConfirmation: true,
        })
    } else {
        None
    };
    Ok(NormalizedFollowLaunch {
        enabled: !snipes.is_empty() || dev_auto_sell.is_some(),
        source: "legacy-postLaunch".to_string(),
        schemaVersion: 1,
        snipes,
        devAutoSell: dev_auto_sell,
        constraints: NormalizedFollowLaunchConstraints {
            pumpOnly: true,
            retryBudget: 1,
            requireDaemonReadiness: true,
            blockOnRequiredPrechecks: true,
        },
    })
}

fn normalize_follow_launch(
    raw: &RawConfig,
    post_launch_strategy: &str,
) -> Result<NormalizedFollowLaunch, ConfigError> {
    let follow = &raw.followLaunch;
    let explicit_schema = parse_int(
        &follow.schemaVersion,
        "followLaunch.schemaVersion",
        Some(1),
        Some(i64::from(u32::MAX)),
        Some(1),
    )?
    .unwrap_or(1) as u32;
    let mut snipes = Vec::new();
    for (index, entry) in follow.snipes.iter().enumerate() {
        let enabled = parse_bool(&entry.enabled, true);
        let amount = entry.buyAmountSol.trim().to_string();
        let wallet_env_key = entry.walletEnvKey.trim().to_string();
        let has_payload = enabled || !amount.is_empty() || !wallet_env_key.is_empty();
        if !has_payload {
            continue;
        }
        let target_block_offset = parse_int(
            &entry.targetBlockOffset,
            &format!("followLaunch.snipes[{index}].targetBlockOffset"),
            Some(0),
            Some(5),
            None,
        )?
        .map(|value| value as u8);
        let submit_with_launch = parse_bool(&entry.submitWithLaunch, false);
        let submit_delay_ms = parse_int(
            &entry.submitDelayMs,
            &format!("followLaunch.snipes[{index}].submitDelayMs"),
            Some(0),
            None,
            Some(0),
        )?
        .unwrap_or(0) as u64;
        if submit_with_launch && (submit_delay_ms > 0 || target_block_offset.is_some()) {
            return Err(ConfigError::Message(format!(
                "followLaunch.snipes[{index}] cannot use submitWithLaunch together with submitDelayMs or targetBlockOffset."
            )));
        }
        if let Some(sell) = &entry.postBuySell {
            if follow_sell_has_payload(sell) {
                return Err(ConfigError::Message(format!(
                    "followLaunch.snipes[{index}].postBuySell is not shipped yet. Phase 1 supports multi-sniper buys plus dev auto-sell only."
                )));
            }
        }
        let post_buy_sell = match &entry.postBuySell {
            Some(sell) => {
                normalize_follow_sell(sell, &wallet_env_key, &format!("snipe-{}-sell", index + 1))?
            }
            None => None,
        };
        snipes.push(NormalizedFollowLaunchSnipe {
            actionId: if entry.actionId.trim().is_empty() {
                format!("snipe-{}-buy", index + 1)
            } else {
                entry.actionId.trim().to_string()
            },
            enabled,
            walletEnvKey: wallet_env_key,
            buyAmountSol: amount,
            submitWithLaunch: submit_with_launch,
            retryOnFailure: parse_bool(&entry.retryOnFailure, false),
            submitDelayMs: submit_delay_ms,
            targetBlockOffset: target_block_offset,
            jitterMs: parse_int(
                &entry.jitterMs,
                &format!("followLaunch.snipes[{index}].jitterMs"),
                Some(0),
                None,
                Some(0),
            )?
            .unwrap_or(0) as u64,
            feeJitterBps: parse_int(
                &entry.feeJitterBps,
                &format!("followLaunch.snipes[{index}].feeJitterBps"),
                Some(0),
                Some(10_000),
                Some(0),
            )?
            .unwrap_or(0) as u16,
            skipIfTokenBalancePositive: false,
            postBuySell: post_buy_sell,
        });
    }
    let dev_auto_sell = match &follow.devAutoSell {
        Some(sell) => normalize_follow_sell(sell, raw.selectedWalletKey.trim(), "dev-auto-sell")?,
        None => None,
    };
    let explicit_enabled = parse_bool(&follow.enabled, false);
    let has_explicit_payload = explicit_enabled || !snipes.is_empty() || dev_auto_sell.is_some();
    if !has_explicit_payload {
        return legacy_follow_launch(raw, post_launch_strategy);
    }
    Ok(NormalizedFollowLaunch {
        enabled: explicit_enabled || !snipes.is_empty() || dev_auto_sell.is_some(),
        source: "followLaunch".to_string(),
        schemaVersion: explicit_schema,
        snipes,
        devAutoSell: dev_auto_sell,
        constraints: NormalizedFollowLaunchConstraints {
            pumpOnly: parse_bool(&follow.constraints.pumpOnly, true),
            retryBudget: parse_int(
                &follow.constraints.retryBudget,
                "followLaunch.constraints.retryBudget",
                Some(0),
                Some(32),
                Some(1),
            )?
            .unwrap_or(1) as u32,
            requireDaemonReadiness: parse_bool(&follow.constraints.requireDaemonReadiness, true),
            blockOnRequiredPrechecks: parse_bool(
                &follow.constraints.blockOnRequiredPrechecks,
                true,
            ),
        },
    })
}

pub fn normalize_raw_config(raw: RawConfig) -> Result<NormalizedConfig, ConfigError> {
    let requested_mode = if raw.mode.trim().is_empty() {
        "regular".to_string()
    } else {
        raw.mode.trim().to_lowercase()
    };
    let mode = match requested_mode.as_str() {
        "custom" | "agent-custom-live" => "agent-custom".to_string(),
        _ => requested_mode,
    };
    if ![
        "regular",
        "cashback",
        "agent-custom",
        "agent-unlocked",
        "agent-locked",
    ]
    .contains(&mode.as_str())
    {
        return Err(ConfigError::Message(format!(
            "mode must be one of regular, cashback, agent-custom, agent-unlocked, agent-locked. Got: {mode}"
        )));
    }

    let launchpad = parse_choice(
        &raw.launchpad,
        "launchpad",
        &["pump", "bonk", "bagsapp"],
        "pump",
    )?;
    let post_launch_strategy = parse_choice(
        &raw.postLaunch.strategy,
        "postLaunch.strategy",
        &["none", "dev-buy", "snipe-own-launch", "automatic-dev-sell"],
        "none",
    )?;

    let fee_recipients =
        normalize_recipients(&raw.feeSharing.recipients, "feeSharing recipient", false)?;
    let mut agent_fee_recipients =
        normalize_recipients(&raw.agent.feeRecipients, "agent recipient", true)?;

    let mut normalized = NormalizedConfig {
        mode: mode.clone(),
        launchpad,
        token: NormalizedToken {
            name: parse_limited_string(&raw.token.name, "token.name", TOKEN_NAME_MAX_LENGTH, true)?,
            symbol: parse_limited_string(
                &raw.token.symbol,
                "token.symbol",
                TOKEN_SYMBOL_MAX_LENGTH,
                true,
            )?,
            uri: raw.token.uri.trim().to_string(),
            description: raw.token.description.trim().to_string(),
            website: raw.token.website.trim().to_string(),
            twitter: raw.token.twitter.trim().to_string(),
            telegram: raw.token.telegram.trim().to_string(),
            mayhemMode: parse_bool(&raw.token.mayhemMode, false),
            cashback: if mode == "cashback" {
                true
            } else {
                parse_bool(&raw.token.cashback, false)
            },
        },
        signer: NormalizedSigner {
            keypairFile: raw.signer.keypairFile.trim().to_string(),
            secretKey: raw.signer.secretKey.trim().to_string(),
        },
        agent: NormalizedAgent {
            authority: raw.agent.authority.trim().to_string(),
            buybackBps: parse_int(
                &raw.agent.buybackBps,
                "buybackBps",
                Some(0),
                Some(10_000),
                None,
            )?,
            splitAgentInit: parse_bool(&raw.agent.splitAgentInit, false),
            feeReceiver: raw.agent.feeReceiver.trim().to_string(),
            feeRecipients: agent_fee_recipients.clone(),
        },
        tx: NormalizedTx {
            computeUnitLimit: parse_int(
                &raw.tx.computeUnitLimit,
                "computeUnitLimit",
                Some(1),
                None,
                None,
            )?,
            computeUnitPriceMicroLamports: parse_int(
                &raw.tx.computeUnitPriceMicroLamports,
                "computeUnitPriceMicroLamports",
                Some(0),
                None,
                None,
            )?,
            jitoTipLamports: parse_int(
                &raw.tx.jitoTipLamports,
                "jitoTipLamports",
                Some(0),
                None,
                Some(0),
            )?
            .unwrap_or(0),
            jitoTipAccount: raw.tx.jitoTipAccount.trim().to_string(),
            lookupTables: raw
                .tx
                .lookupTables
                .iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect(),
            useDefaultLookupTables: parse_bool(&raw.tx.useDefaultLookupTables, true),
            dumpBase64: parse_bool(&raw.tx.dumpBase64, false),
            writeReport: parse_bool(&raw.tx.writeReport, false),
        },
        feeSharing: NormalizedFeeSharing {
            generateLaterSetup: parse_bool(&raw.feeSharing.generateLaterSetup, false),
            recipients: fee_recipients,
        },
        creatorFee: normalize_creator_fee(&raw.creatorFee, &mode)?,
        execution: NormalizedExecution {
            simulate: parse_bool(&raw.execution.simulate, false),
            send: parse_bool(&raw.execution.send, false),
            txFormat: parse_choice(
                &raw.execution.txFormat,
                "txFormat",
                &["auto", "v0-alt", "v0", "legacy"],
                "auto",
            )?,
            commitment: parse_choice(
                &raw.execution.commitment,
                "commitment",
                &["processed", "confirmed", "finalized"],
                "confirmed",
            )?,
            skipPreflight: parse_bool(&raw.execution.skipPreflight, false),
            trackSendBlockHeight: parse_bool(&raw.execution.trackSendBlockHeight, false),
            provider: parse_choice(
                &raw.execution.provider,
                "execution.provider",
                &["standard-rpc", "helius-sender", "jito-bundle"],
                "helius-sender",
            )?,
            endpointProfile: if is_blank(&raw.execution.endpointProfile) {
                String::new()
            } else {
                parse_choice(
                    &raw.execution.endpointProfile,
                    "execution.endpointProfile",
                    &["global", "us", "eu", "west", "asia"],
                    "global",
                )?
            },
            autoGas: parse_bool(&raw.execution.autoGas, true),
            autoMode: if is_blank(&raw.execution.autoMode) {
                "launchAuto".to_string()
            } else {
                raw.execution.autoMode.trim().to_string()
            },
            priorityFeeSol: raw.execution.priorityFeeSol.trim().to_string(),
            tipSol: raw.execution.tipSol.trim().to_string(),
            maxPriorityFeeSol: raw.execution.maxPriorityFeeSol.trim().to_string(),
            maxTipSol: raw.execution.maxTipSol.trim().to_string(),
            buyProvider: parse_choice(
                &raw.execution.buyProvider,
                "execution.buyProvider",
                &["standard-rpc", "helius-sender", "jito-bundle"],
                "helius-sender",
            )?,
            buyEndpointProfile: if is_blank(&raw.execution.buyEndpointProfile) {
                String::new()
            } else {
                parse_choice(
                    &raw.execution.buyEndpointProfile,
                    "execution.buyEndpointProfile",
                    &["global", "us", "eu", "west", "asia"],
                    "global",
                )?
            },
            buyAutoGas: parse_bool(&raw.execution.buyAutoGas, true),
            buyAutoMode: if is_blank(&raw.execution.buyAutoMode) {
                "buyAuto".to_string()
            } else {
                raw.execution.buyAutoMode.trim().to_string()
            },
            buyPriorityFeeSol: raw.execution.buyPriorityFeeSol.trim().to_string(),
            buyTipSol: raw.execution.buyTipSol.trim().to_string(),
            buySlippagePercent: raw.execution.buySlippagePercent.trim().to_string(),
            buyMaxPriorityFeeSol: raw.execution.buyMaxPriorityFeeSol.trim().to_string(),
            buyMaxTipSol: raw.execution.buyMaxTipSol.trim().to_string(),
            sellProvider: parse_choice(
                &raw.execution.sellProvider,
                "execution.sellProvider",
                &["standard-rpc", "helius-sender", "jito-bundle"],
                "helius-sender",
            )?,
            sellEndpointProfile: if is_blank(&raw.execution.sellEndpointProfile) {
                String::new()
            } else {
                parse_choice(
                    &raw.execution.sellEndpointProfile,
                    "execution.sellEndpointProfile",
                    &["global", "us", "eu", "west", "asia"],
                    "global",
                )?
            },
            sellPriorityFeeSol: raw.execution.sellPriorityFeeSol.trim().to_string(),
            sellTipSol: raw.execution.sellTipSol.trim().to_string(),
            sellSlippagePercent: raw.execution.sellSlippagePercent.trim().to_string(),
        },
        devBuy: normalize_dev_buy(&raw)?,
        postLaunch: NormalizedPostLaunch {
            strategy: post_launch_strategy.clone(),
            snipeOwnLaunch: NormalizedSnipeOwnLaunch {
                enabled: post_launch_strategy == "snipe-own-launch",
                buyAmountSol: raw
                    .postLaunch
                    .snipeOwnLaunch
                    .buyAmountSol
                    .trim()
                    .to_string(),
            },
            automaticDevSell: NormalizedAutomaticDevSell {
                enabled: post_launch_strategy == "automatic-dev-sell"
                    || parse_bool(&raw.postLaunch.automaticDevSell.enabled, false),
                percent: parse_int(
                    &raw.postLaunch.automaticDevSell.percent,
                    "postLaunch.automaticDevSell.percent",
                    Some(0),
                    Some(100),
                    Some(0),
                )?
                .unwrap_or(0),
                delaySeconds: parse_int(
                    &raw.postLaunch.automaticDevSell.delaySeconds,
                    "postLaunch.automaticDevSell.delaySeconds",
                    Some(0),
                    Some(10),
                    Some(0),
                )?
                .unwrap_or(0),
            },
        },
        followLaunch: normalize_follow_launch(&raw, &post_launch_strategy)?,
        presets: NormalizedPresets {
            activePresetId: raw.presets.activePresetId.trim().to_string(),
            selectedLaunchPresetId: raw.presets.selectedLaunchPresetId.trim().to_string(),
            selectedSniperPresetId: raw.presets.selectedSniperPresetId.trim().to_string(),
        },
        imageLocalPath: raw.imageLocalPath.trim().to_string(),
        selectedWalletKey: raw.selectedWalletKey.trim().to_string(),
        vanityPrivateKey: raw.vanityPrivateKey.trim().to_string(),
    };

    if normalized.token.uri.is_empty() {
        return Err(ConfigError::Message("token.uri is required.".to_string()));
    }

    if mode == "regular" || mode == "cashback" {
        normalized.agent.buybackBps = None;
        normalized.agent.feeRecipients = vec![];
    }
    if ["agent-custom", "agent-unlocked", "agent-locked"].contains(&mode.as_str())
        && normalized.agent.buybackBps.is_none()
    {
        return Err(ConfigError::Message(format!(
            "buybackBps is required for {mode} mode."
        )));
    }
    if mode == "agent-unlocked" || mode == "agent-locked" {
        normalized.agent.feeRecipients = vec![];
    }
    if (mode == "regular" || mode == "cashback") && normalized.agent.splitAgentInit {
        normalized.agent.splitAgentInit = false;
    }
    if mode != "regular" && normalized.feeSharing.generateLaterSetup {
        return Err(ConfigError::Message(
            "feeSharing.generateLaterSetup is only supported in regular mode.".to_string(),
        ));
    }
    if mode == "regular"
        && normalized.feeSharing.generateLaterSetup
        && normalized.creatorFee.mode != "deployer"
    {
        return Err(ConfigError::Message(
            "Later fee-sharing setup is only supported when the regular-mode creator fee receiver is the deployer."
                .to_string(),
        ));
    }
    if normalized.feeSharing.generateLaterSetup && normalized.feeSharing.recipients.is_empty() {
        return Err(ConfigError::Message(
            "feeSharing.recipients is required when feeSharing.generateLaterSetup is true."
                .to_string(),
        ));
    }
    if normalized.tx.jitoTipLamports > 0 && normalized.tx.jitoTipAccount.trim().is_empty() {
        return Err(ConfigError::Message(
            "jitoTipAccount is required when jitoTipLamports is set.".to_string(),
        ));
    }
    if normalized.execution.provider == "helius-sender" {
        if !normalized.execution.skipPreflight {
            return Err(ConfigError::Message(
                "execution.skipPreflight must be true when execution.provider is helius-sender."
                    .to_string(),
            ));
        }
        if normalized.tx.computeUnitPriceMicroLamports.unwrap_or(0) <= 0 {
            return Err(ConfigError::Message(
                "tx.computeUnitPriceMicroLamports must be greater than 0 when execution.provider is helius-sender."
                    .to_string(),
            ));
        }
        if normalized.tx.jitoTipLamports < 200_000 {
            return Err(ConfigError::Message(
                "tx.jitoTipLamports must be at least 200000 when execution.provider is helius-sender."
                    .to_string(),
            ));
        }
    }

    // Keep variable live for future parity extensions.
    agent_fee_recipients.clear();

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_raw_config() -> RawConfig {
        serde_json::from_value(json!({
            "mode": "regular",
            "launchpad": "pump",
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
                "recipients": []
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
        .expect("sample config should deserialize")
    }

    #[test]
    fn normalizes_minimal_regular_config() {
        let normalized =
            normalize_raw_config(sample_raw_config()).expect("config should normalize");
        assert_eq!(normalized.mode, "regular");
        assert_eq!(normalized.launchpad, "pump");
        assert_eq!(normalized.token.name, "LaunchDeck");
        assert_eq!(normalized.execution.provider, "helius-sender");
        assert_eq!(normalized.execution.endpointProfile, "global");
        assert_eq!(normalized.execution.buyProvider, "helius-sender");
        assert_eq!(normalized.execution.buyEndpointProfile, "global");
        assert!(normalized.devBuy.is_none());
        assert!(!normalized.followLaunch.enabled);
    }

    #[test]
    fn ignores_legacy_policy_fields_during_normalization() {
        let mut raw = sample_raw_config();
        raw.execution.policy = "not-a-valid-policy-anymore".to_string();
        raw.execution.buyPolicy = "fast".to_string();
        raw.execution.sellPolicy = "safe".to_string();

        let normalized =
            normalize_raw_config(raw).expect("legacy policy fields should be ignored");

        assert_eq!(normalized.execution.provider, "helius-sender");
        assert_eq!(normalized.execution.buyProvider, "helius-sender");
        assert_eq!(normalized.execution.sellProvider, "helius-sender");
    }

    #[test]
    fn requires_jito_tip_account_when_tip_is_set() {
        let mut raw = sample_raw_config();
        raw.tx.jitoTipLamports = Some(json!(200000));
        raw.tx.jitoTipAccount = String::new();
        let error = normalize_raw_config(raw).expect_err("jito tip account should be required");
        assert_eq!(
            error.to_string(),
            "jitoTipAccount is required when jitoTipLamports is set."
        );
    }

    #[test]
    fn helius_sender_requires_priority_tip_and_skip_preflight() {
        let mut raw = sample_raw_config();
        raw.tx.jitoTipLamports = Some(json!(200_000));
        raw.tx.jitoTipAccount = "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE".to_string();
        raw.tx.computeUnitPriceMicroLamports = Some(json!(1));
        raw.execution.skipPreflight = Some(json!(false));
        let error = normalize_raw_config(raw).expect_err("sender should require skipPreflight");
        assert_eq!(
            error.to_string(),
            "execution.skipPreflight must be true when execution.provider is helius-sender."
        );
    }

    #[test]
    fn rejects_removed_auto_provider_values() {
        let mut raw = sample_raw_config();
        raw.execution.provider = "auto".to_string();
        let error = normalize_raw_config(raw).expect_err("auto should be rejected");
        assert!(error.to_string().contains(
            "execution.provider must be one of standard-rpc, helius-sender, jito-bundle"
        ));
    }

    #[test]
    fn migrates_legacy_post_launch_into_follow_launch() {
        let mut raw = sample_raw_config();
        raw.selectedWalletKey = "SOLANA_PRIVATE_KEY2".to_string();
        raw.postLaunch.strategy = "automatic-dev-sell".to_string();
        raw.postLaunch.automaticDevSell.enabled = Some(json!(true));
        raw.postLaunch.automaticDevSell.percent = Some(json!(75));
        raw.postLaunch.automaticDevSell.delaySeconds = Some(json!(2));
        let normalized = normalize_raw_config(raw).expect("legacy postLaunch should migrate");
        assert!(normalized.followLaunch.enabled);
        let dev_auto_sell = normalized
            .followLaunch
            .devAutoSell
            .expect("dev auto sell should be present");
        assert_eq!(dev_auto_sell.walletEnvKey, "SOLANA_PRIVATE_KEY2");
        assert_eq!(dev_auto_sell.percent, 75);
        assert_eq!(dev_auto_sell.delayMs, 2000);
    }

    #[test]
    fn normalizes_explicit_follow_launch_snipes() {
        let mut raw = sample_raw_config();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": true,
            "schemaVersion": 1,
            "snipes": [
                {
                    "walletEnvKey": "SOLANA_PRIVATE_KEY2",
                    "buyAmountSol": "0.25",
                    "submitDelayMs": 30,
                    "targetBlockOffset": 1,
                    "jitterMs": 5,
                    "feeJitterBps": 250
                }
            ]
        }))
        .expect("follow launch raw");
        let normalized = normalize_raw_config(raw).expect("follow launch should normalize");
        assert!(normalized.followLaunch.enabled);
        assert_eq!(normalized.followLaunch.snipes.len(), 1);
        let snipe = &normalized.followLaunch.snipes[0];
        assert_eq!(snipe.walletEnvKey, "SOLANA_PRIVATE_KEY2");
        assert_eq!(snipe.buyAmountSol, "0.25");
        assert_eq!(snipe.submitDelayMs, 30);
        assert_eq!(snipe.targetBlockOffset, Some(1));
        assert_eq!(snipe.jitterMs, 5);
        assert_eq!(snipe.feeJitterBps, 250);
    }

    #[test]
    fn rejects_phase_two_post_buy_sell_in_follow_launch() {
        let mut raw = sample_raw_config();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": true,
            "schemaVersion": 1,
            "snipes": [
                {
                    "walletEnvKey": "SOLANA_PRIVATE_KEY2",
                    "buyAmountSol": "0.25",
                    "postBuySell": {
                        "enabled": true,
                        "percent": 100,
                        "delayMs": 2500
                    }
                }
            ]
        }))
        .expect("follow launch raw");
        let error =
            normalize_raw_config(raw).expect_err("phase two post buy sell should be rejected");
        assert!(
            error
                .to_string()
                .contains("followLaunch.snipes[0].postBuySell is not shipped yet")
        );
    }
}
