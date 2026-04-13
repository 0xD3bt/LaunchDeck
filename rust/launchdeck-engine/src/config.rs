#![allow(non_snake_case)]

use crate::provider_tip::{provider_min_tip_sol_label, provider_required_tip_lamports};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use thiserror::Error;

const TOKEN_NAME_MAX_LENGTH: usize = 32;
const TOKEN_SYMBOL_MAX_LENGTH: usize = 10;
const MAX_FEE_SPLIT_RECIPIENTS: usize = 10;
const DEFAULT_LAUNCH_COMPUTE_UNIT_LIMIT: u64 = 340_000;
const DEFAULT_AGENT_SETUP_COMPUTE_UNIT_LIMIT: u64 = 180_000;
const DEFAULT_FOLLOW_UP_COMPUTE_UNIT_LIMIT: u64 = 175_000;
const DEFAULT_SNIPER_BUY_COMPUTE_UNIT_LIMIT: u64 = 120_000;
const DEFAULT_DEV_AUTO_SELL_COMPUTE_UNIT_LIMIT: u64 = 145_000;
const DEFAULT_LAUNCH_USD1_TOPUP_COMPUTE_UNIT_LIMIT: u64 = 90_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchpadActionBackendMode {
    Auto,
    Helper,
    Rust,
}

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
    pub quoteAsset: String,
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
    pub bags: RawBags,
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
pub struct RawBags {
    #[serde(default)]
    pub identityMode: String,
    #[serde(default)]
    pub agentUsername: String,
    #[serde(default)]
    pub authToken: String,
    #[serde(default)]
    pub identityVerifiedWallet: String,
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
    pub mevProtect: Option<Value>,
    #[serde(default)]
    pub mevMode: Option<Value>,
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
    pub buyMevProtect: Option<Value>,
    #[serde(default)]
    pub buyMevMode: Option<Value>,
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
    pub sellAutoGas: Option<Value>,
    #[serde(default)]
    pub sellAutoMode: String,
    #[serde(default)]
    pub sellProvider: String,
    #[serde(default)]
    pub sellEndpointProfile: String,
    #[serde(default)]
    pub sellPolicy: String,
    #[serde(default)]
    pub sellMevProtect: Option<Value>,
    #[serde(default)]
    pub sellMevMode: Option<Value>,
    #[serde(default)]
    pub sellPriorityFeeSol: String,
    #[serde(default)]
    pub sellTipSol: String,
    #[serde(default)]
    pub sellSlippagePercent: String,
    #[serde(default)]
    pub sellMaxPriorityFeeSol: String,
    #[serde(default)]
    pub sellMaxTipSol: String,
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
    #[serde(default)]
    pub scanTimeoutSeconds: Option<Value>,
    #[serde(default)]
    pub timeoutAction: String,
    #[serde(
        default,
        rename = "scanTimeoutMinutes",
        skip_serializing_if = "Option::is_none"
    )]
    pub legacyScanTimeoutMinutes: Option<Value>,
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
    pub quoteAsset: String,
    pub token: NormalizedToken,
    pub signer: NormalizedSigner,
    pub agent: NormalizedAgent,
    pub tx: NormalizedTx,
    pub feeSharing: NormalizedFeeSharing,
    pub creatorFee: NormalizedCreatorFee,
    pub bags: NormalizedBags,
    pub execution: NormalizedExecution,
    pub devBuy: Option<NormalizedDevBuy>,
    pub postLaunch: NormalizedPostLaunch,
    pub followLaunch: NormalizedFollowLaunch,
    pub presets: NormalizedPresets,
    pub imageLocalPath: String,
    pub selectedWalletKey: String,
    pub vanityPrivateKey: String,
}

pub fn validate_launchpad_support(config: &NormalizedConfig) -> Result<(), ConfigError> {
    match config.launchpad.as_str() {
        "bonk" => {
            if config.mode != "regular" && config.mode != "bonkers" {
                return Err(ConfigError::Message(format!(
                    "Bonk currently supports only regular and bonkers modes. Got mode={}.",
                    config.mode
                )));
            }
            if config.feeSharing.generateLaterSetup || !config.feeSharing.recipients.is_empty() {
                return Err(ConfigError::Message(
                    "Bonk does not support fee-sharing setup yet.".to_string(),
                ));
            }
            if config.token.mayhemMode {
                return Err(ConfigError::Message(
                    "Bonk does not support mayhem mode.".to_string(),
                ));
            }
            if config.token.cashback {
                return Err(ConfigError::Message(
                    "Bonk does not support cashback mode.".to_string(),
                ));
            }
            Ok(())
        }
        "bagsapp" => {
            if !matches!(
                config.mode.as_str(),
                "bags-2-2" | "bags-025-1" | "bags-1-025"
            ) {
                return Err(ConfigError::Message(format!(
                    "Bagsapp currently supports only bags-2-2, bags-025-1, and bags-1-025 modes. Got mode={}.",
                    config.mode
                )));
            }
            if config.quoteAsset != "sol" {
                return Err(ConfigError::Message(
                    "Bagsapp currently supports only SOL-denominated launch and trade flows."
                        .to_string(),
                ));
            }
            if config.token.mayhemMode {
                return Err(ConfigError::Message(
                    "Bagsapp does not support mayhem mode.".to_string(),
                ));
            }
            if config.token.cashback {
                return Err(ConfigError::Message(
                    "Bagsapp does not support cashback mode.".to_string(),
                ));
            }
            if config.agent.splitAgentInit
                || config.agent.buybackBps.is_some()
                || !config.agent.feeRecipients.is_empty()
            {
                return Err(ConfigError::Message(
                    "Bagsapp does not support Pump agent modes.".to_string(),
                ));
            }
            if config.creatorFee.mode != "deployer" {
                return Err(ConfigError::Message(
                    "Bagsapp creator fee receiver must remain the deployer wallet.".to_string(),
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
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

pub fn launch_follow_up_label(config: &NormalizedConfig) -> Option<&'static str> {
    match config.mode.as_str() {
        "regular" | "cashback"
            if config.feeSharing.generateLaterSetup && !config.feeSharing.recipients.is_empty() =>
        {
            Some("follow-up")
        }
        "agent-custom" if config.agent.splitAgentInit && !config.agent.feeRecipients.is_empty() => {
            Some("agent-setup")
        }
        "agent-locked" => Some("agent-setup"),
        _ => None,
    }
}

pub fn has_launch_follow_up(config: &NormalizedConfig) -> bool {
    launch_follow_up_label(config).is_some()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedCreatorFee {
    pub mode: String,
    pub address: String,
    pub githubUsername: String,
    pub githubUserId: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NormalizedBags {
    pub configType: String,
    pub identityMode: String,
    pub agentUsername: String,
    pub authToken: String,
    pub identityVerifiedWallet: String,
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
    #[serde(default)]
    pub mevProtect: bool,
    #[serde(default)]
    pub mevMode: String,
    #[serde(default)]
    pub jitodontfront: bool,
    pub autoGas: bool,
    pub autoMode: String,
    pub priorityFeeSol: String,
    pub tipSol: String,
    pub maxPriorityFeeSol: String,
    pub maxTipSol: String,
    pub buyProvider: String,
    pub buyEndpointProfile: String,
    #[serde(default)]
    pub buyMevProtect: bool,
    #[serde(default)]
    pub buyMevMode: String,
    #[serde(default)]
    pub buyJitodontfront: bool,
    pub buyAutoGas: bool,
    pub buyAutoMode: String,
    pub buyPriorityFeeSol: String,
    pub buyTipSol: String,
    pub buySlippagePercent: String,
    pub buyMaxPriorityFeeSol: String,
    pub buyMaxTipSol: String,
    pub sellAutoGas: bool,
    pub sellAutoMode: String,
    pub sellProvider: String,
    pub sellEndpointProfile: String,
    #[serde(default)]
    pub sellMevProtect: bool,
    #[serde(default)]
    pub sellMevMode: String,
    #[serde(default)]
    pub sellJitodontfront: bool,
    pub sellPriorityFeeSol: String,
    pub sellTipSol: String,
    pub sellSlippagePercent: String,
    pub sellMaxPriorityFeeSol: String,
    pub sellMaxTipSol: String,
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
    pub delayMs: Option<u64>,
    pub targetBlockOffset: Option<u8>,
    pub marketCap: Option<NormalizedFollowLaunchMarketCapTrigger>,
    pub precheckRequired: bool,
    pub requireConfirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFollowLaunchMarketCapTrigger {
    pub direction: String,
    pub threshold: String,
    pub scanTimeoutSeconds: u64,
    pub timeoutAction: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedRecipient {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub address: String,
    pub githubUserId: String,
    pub githubUsername: String,
    pub shareBps: i64,
}

fn is_supported_social_recipient_type(value: &str) -> bool {
    matches!(value, "github" | "twitter" | "x" | "kick" | "tiktok")
}

fn launchpad_supports_extended_social_recipients(launchpad: &str) -> bool {
    launchpad.trim().eq_ignore_ascii_case("bagsapp")
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn configured_track_send_block_height_env_enabled() -> bool {
    matches!(
        env::var("LAUNCHDECK_TRACK_SEND_BLOCK_HEIGHT")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn benchmark_mode_allows_track_send_block_height_default(mode: &str) -> bool {
    matches!(mode.trim().to_ascii_lowercase().as_str(), "" | "full")
}

fn configured_track_send_block_height_default() -> bool {
    benchmark_mode_allows_track_send_block_height_default(
        &env::var("LAUNCHDECK_BENCHMARK_MODE").unwrap_or_default(),
    ) && configured_track_send_block_height_env_enabled()
}

fn configured_hellomoon_mev_protect_default() -> bool {
    matches!(
        env::var("HELLOMOON_MEV_PROTECT")
            .or_else(|_| env::var("LUNAR_LANDER_MEV_PROTECT"))
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn configured_hellomoon_mev_mode_default() -> String {
    if configured_hellomoon_mev_protect_default() {
        "reduced".to_string()
    } else {
        "off".to_string()
    }
}

fn configured_compute_unit_limit_env(name: &str, fallback: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(fallback)
}

pub fn configured_default_launch_compute_unit_limit() -> u64 {
    configured_compute_unit_limit_env(
        "LAUNCHDECK_LAUNCH_COMPUTE_UNIT_LIMIT",
        DEFAULT_LAUNCH_COMPUTE_UNIT_LIMIT,
    )
}

pub fn configured_default_agent_setup_compute_unit_limit() -> u64 {
    configured_compute_unit_limit_env(
        "LAUNCHDECK_AGENT_SETUP_COMPUTE_UNIT_LIMIT",
        DEFAULT_AGENT_SETUP_COMPUTE_UNIT_LIMIT,
    )
}

pub fn configured_default_follow_up_compute_unit_limit() -> u64 {
    configured_compute_unit_limit_env(
        "LAUNCHDECK_FOLLOW_UP_COMPUTE_UNIT_LIMIT",
        DEFAULT_FOLLOW_UP_COMPUTE_UNIT_LIMIT,
    )
}

pub fn configured_default_sniper_buy_compute_unit_limit() -> u64 {
    configured_compute_unit_limit_env(
        "LAUNCHDECK_SNIPER_BUY_COMPUTE_UNIT_LIMIT",
        DEFAULT_SNIPER_BUY_COMPUTE_UNIT_LIMIT,
    )
}

pub fn configured_default_dev_auto_sell_compute_unit_limit() -> u64 {
    configured_compute_unit_limit_env(
        "LAUNCHDECK_DEV_AUTO_SELL_COMPUTE_UNIT_LIMIT",
        DEFAULT_DEV_AUTO_SELL_COMPUTE_UNIT_LIMIT,
    )
}

pub fn configured_default_launch_usd1_topup_compute_unit_limit() -> u64 {
    configured_compute_unit_limit_env(
        "LAUNCHDECK_LAUNCH_USD1_TOPUP_COMPUTE_UNIT_LIMIT",
        DEFAULT_LAUNCH_USD1_TOPUP_COMPUTE_UNIT_LIMIT,
    )
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

fn parse_mev_mode(value: &Option<Value>, fallback: &str) -> String {
    match value {
        None | Some(Value::Null) => fallback.to_string(),
        Some(Value::Bool(v)) => {
            if *v {
                "reduced".to_string()
            } else {
                "off".to_string()
            }
        }
        Some(Value::String(v)) => match v.trim().to_ascii_lowercase().as_str() {
            "off" => "off".to_string(),
            "reduced" => "reduced".to_string(),
            "secure" => "secure".to_string(),
            "true" => "reduced".to_string(),
            "false" => "off".to_string(),
            _ => fallback.to_string(),
        },
        Some(Value::Number(n)) => {
            if n.as_i64().unwrap_or_default() != 0 {
                "reduced".to_string()
            } else {
                "off".to_string()
            }
        }
        Some(_) => fallback.to_string(),
    }
}

fn mev_mode_enables_hellomoon_protect(mode: &str) -> bool {
    matches!(
        mode.trim().to_ascii_lowercase().as_str(),
        "reduced" | "secure"
    )
}

fn mev_mode_enables_jitodontfront(mode: &str) -> bool {
    matches!(
        mode.trim().to_ascii_lowercase().as_str(),
        "reduced" | "secure"
    )
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

fn parse_market_cap_threshold(value: &str, label: &str) -> Result<String, ConfigError> {
    let normalized = value.trim().replace(',', "").to_lowercase();
    if normalized.is_empty() {
        return Err(ConfigError::Message(format!("{label} is required.")));
    }
    let (number_part, multiplier) = match normalized.chars().last() {
        Some('k') => (&normalized[..normalized.len() - 1], 1_000_f64),
        Some('m') => (&normalized[..normalized.len() - 1], 1_000_000_f64),
        Some('b') => (&normalized[..normalized.len() - 1], 1_000_000_000_f64),
        Some('t') => (&normalized[..normalized.len() - 1], 1_000_000_000_000_f64),
        _ => (normalized.as_str(), 1_f64),
    };
    let parsed = number_part.parse::<f64>().map_err(|_| {
        ConfigError::Message(format!(
            "{label} must be a positive USD number like 100000 or 100k. Got: {value}"
        ))
    })?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(ConfigError::Message(format!(
            "{label} must be a positive USD number like 100000 or 100k. Got: {value}"
        )));
    }
    let expanded = (parsed * multiplier * 1_000_000_f64).round();
    if !expanded.is_finite() || expanded <= 0.0 || expanded > u64::MAX as f64 {
        return Err(ConfigError::Message(format!(
            "{label} is too large. Got: {value}"
        )));
    }
    Ok((expanded as u64).to_string())
}

fn parse_market_cap_scan_timeout_seconds(
    raw: &RawFollowLaunchMarketCapTrigger,
    label_prefix: &str,
) -> Result<u64, ConfigError> {
    if raw.scanTimeoutSeconds.is_some() {
        return Ok(parse_int(
            &raw.scanTimeoutSeconds,
            &format!("{label_prefix}.scanTimeoutSeconds"),
            Some(1),
            Some(86_400),
            Some(30),
        )?
        .unwrap_or(30) as u64);
    }
    if raw.legacyScanTimeoutMinutes.is_some() {
        let minutes = parse_int(
            &raw.legacyScanTimeoutMinutes,
            &format!("{label_prefix}.scanTimeoutMinutes"),
            Some(1),
            Some(1_440),
            Some(30),
        )?
        .unwrap_or(30) as u64;
        return Ok(minutes.saturating_mul(60));
    }
    Ok(30)
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

fn provider_requires_tip_and_priority(provider: &str) -> bool {
    matches!(
        provider.trim(),
        "helius-sender" | "hellomoon" | "jito-bundle"
    )
}

fn parse_sol_decimal_to_lamports(value: &str, label: &str) -> Result<u64, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }
    if !trimmed
        .chars()
        .all(|char| char.is_ascii_digit() || char == '.')
    {
        return Err(ConfigError::Message(format!(
            "{label} must be a positive decimal string. Got: {value}"
        )));
    }
    let mut parts = trimmed.split('.');
    let whole = parts.next().unwrap_or_default();
    let fractional = parts.next().unwrap_or_default();
    if parts.next().is_some() {
        return Err(ConfigError::Message(format!(
            "{label} must be a positive decimal string. Got: {value}"
        )));
    }
    if fractional.len() > 9 {
        return Err(ConfigError::Message(format!(
            "{label} supports at most 9 decimal places. Got: {value}"
        )));
    }
    let normalized = format!("{whole}{fractional:0<width$}", width = 9);
    let digits = normalized.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(0);
    }
    digits
        .parse::<u64>()
        .map_err(|error| ConfigError::Message(error.to_string()))
}

fn validate_manual_provider_fee_fields(
    provider: &str,
    auto_gas: bool,
    priority_fee_sol: &str,
    tip_sol: &str,
    label_prefix: &str,
) -> Result<(), ConfigError> {
    if auto_gas || !provider_requires_tip_and_priority(provider) {
        return Ok(());
    }
    if priority_fee_sol.trim().is_empty() && tip_sol.trim().is_empty() {
        return Err(ConfigError::Message(format!(
            "{label_prefix}PriorityFeeSol and {label_prefix}TipSol are required when {label_prefix}Provider is {provider} and {label_prefix}AutoGas is false."
        )));
    }
    let priority_lamports =
        parse_sol_decimal_to_lamports(priority_fee_sol, &format!("{label_prefix}PriorityFeeSol"))?;
    if priority_lamports == 0 {
        return Err(ConfigError::Message(format!(
            "{label_prefix}PriorityFeeSol must be greater than 0 when {label_prefix}Provider is {provider}."
        )));
    }
    let tip_lamports = parse_sol_decimal_to_lamports(tip_sol, &format!("{label_prefix}TipSol"))?;
    let minimum_tip_lamports = provider_required_tip_lamports(provider).unwrap_or(0);
    if tip_lamports < minimum_tip_lamports {
        return Err(ConfigError::Message(format!(
            "{label_prefix}TipSol must be at least {} SOL when {label_prefix}Provider is {provider}.",
            provider_min_tip_sol_label(provider)
        )));
    }
    Ok(())
}

fn normalize_bags_mode(mode: &str) -> String {
    match mode {
        "regular" | "bags-2-2" => "bags-2-2".to_string(),
        "bags-025-1" => "bags-025-1".to_string(),
        "bags-1-025" => "bags-1-025".to_string(),
        other => other.to_string(),
    }
}

fn bags_config_type_for_mode(mode: &str) -> &'static str {
    match mode {
        "bags-025-1" => "d16d3585-6488-4a6c-9a6f-e6c39ca0fda3",
        "bags-1-025" => "a7c8e1f2-3d4b-5a6c-9e0f-1b2c3d4e5f6a",
        _ => "fa29606e-5e48-4c37-827f-4b03d58ee23d",
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
    allow_extended_social: bool,
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

        if entry_type == "wallet" && address.is_empty() {
            return Err(ConfigError::Message(format!(
                "{label_prefix} at index {index} must provide a wallet address."
            )));
        }
        if entry_type == "github" && github_user_id.is_empty() && github_username.is_empty() {
            let provider_label = if entry_type == "x" {
                "X"
            } else if entry_type == "tiktok" {
                "TikTok"
            } else if entry_type == "kick" {
                "Kick"
            } else {
                "GitHub"
            };
            return Err(ConfigError::Message(format!(
                "{label_prefix} at index {index} must provide a {provider_label} username{}.",
                if entry_type == "github" {
                    " or user id"
                } else {
                    ""
                }
            )));
        }
        if matches!(entry_type.as_str(), "twitter" | "x" | "kick" | "tiktok")
            && github_username.is_empty()
        {
            let provider_label = if entry_type == "x" {
                "X"
            } else if entry_type == "tiktok" {
                "TikTok"
            } else if entry_type == "kick" {
                "Kick"
            } else {
                "Twitter"
            };
            return Err(ConfigError::Message(format!(
                "{label_prefix} at index {index} must provide a {provider_label} username."
            )));
        }
        if entry_type != "wallet"
            && entry_type != "agent"
            && !is_supported_social_recipient_type(&entry_type)
        {
            return Err(ConfigError::Message(format!(
                "{label_prefix} at index {index} has unsupported recipient type: {entry_type}."
            )));
        }
        if matches!(entry_type.as_str(), "twitter" | "x" | "kick" | "tiktok")
            && !allow_extended_social
        {
            return Err(ConfigError::Message(format!(
                "{label_prefix} at index {index} uses {entry_type}, which is only supported for Bags launches."
            )));
        }

        normalized.push(NormalizedRecipient {
            r#type: Some(entry_type),
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

fn normalize_quote_asset(raw: &RawConfig, launchpad: &str) -> Result<String, ConfigError> {
    let requested = if raw.quoteAsset.trim().is_empty() {
        "sol".to_string()
    } else {
        raw.quoteAsset.trim().to_lowercase()
    };
    match launchpad {
        "bonk" => parse_choice(&requested, "quoteAsset", &["sol", "usd1"], "sol"),
        _ => {
            if requested != "sol" {
                return Err(ConfigError::Message(format!(
                    "quoteAsset={} is only supported for bonk right now.",
                    requested
                )));
            }
            Ok("sol".to_string())
        }
    }
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

const DEV_AUTO_SELL_MAX_TARGET_BLOCK_OFFSET: i64 = 23;
const SNIPER_BUY_MAX_TARGET_BLOCK_OFFSET: i64 = 37;
const SNIPER_AUTO_SELL_MAX_TARGET_BLOCK_OFFSET: i64 = 23;

fn normalize_follow_sell(
    raw: &RawFollowLaunchSell,
    fallback_wallet_env_key: &str,
    fallback_action_id: &str,
    max_target_block_offset: i64,
) -> Result<Option<NormalizedFollowLaunchSell>, ConfigError> {
    let enabled = parse_bool(&raw.enabled, false);
    let market_cap_requested = parse_bool(&raw.marketCap.enabled, false);
    if market_cap_requested && raw.marketCap.threshold.trim().is_empty() {
        return Err(ConfigError::Message(format!(
            "{fallback_action_id}.marketCap.threshold is required when marketCap.enabled is true."
        )));
    }
    let market_cap_direction = raw.marketCap.direction.trim().to_lowercase();
    if !market_cap_direction.is_empty() && market_cap_direction != "gte" {
        return Err(ConfigError::Message(format!(
            "{fallback_action_id}.marketCap.direction must be gte. Legacy lte is no longer supported."
        )));
    }
    let market_cap_enabled = market_cap_requested && !raw.marketCap.threshold.trim().is_empty();
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
        None,
    )?
    .map(|value| value as u64);
    let target_block_offset = parse_int(
        &raw.targetBlockOffset,
        &format!("{fallback_action_id}.targetBlockOffset"),
        Some(0),
        Some(max_target_block_offset),
        None,
    )?
    .map(|value| value as u8);
    let direction = "gte".to_string();
    let scan_timeout_seconds = parse_market_cap_scan_timeout_seconds(
        &raw.marketCap,
        &format!("{fallback_action_id}.marketCap"),
    )?;
    let timeout_action = parse_choice(
        &raw.marketCap.timeoutAction,
        &format!("{fallback_action_id}.marketCap.timeoutAction"),
        &["stop", "sell"],
        "stop",
    )?;
    let normalized_market_cap_threshold = if market_cap_enabled {
        Some(parse_market_cap_threshold(
            &raw.marketCap.threshold,
            &format!("{fallback_action_id}.marketCap.threshold"),
        )?)
    } else {
        None
    };
    Ok(Some(NormalizedFollowLaunchSell {
        actionId: if raw.actionId.trim().is_empty() {
            fallback_action_id.to_string()
        } else {
            raw.actionId.trim().to_string()
        },
        enabled,
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
                threshold: normalized_market_cap_threshold.unwrap_or_default(),
                scanTimeoutSeconds: scan_timeout_seconds,
                timeoutAction: timeout_action,
            })
        } else {
            None
        },
        precheckRequired: parse_bool(&raw.precheckRequired, false),
        requireConfirmation: parse_bool(&raw.requireConfirmation, true),
    }))
}

fn legacy_follow_launch(
    raw: &RawConfig,
    post_launch_strategy: &str,
) -> Result<NormalizedFollowLaunch, ConfigError> {
    let launchpad_is_pump = raw.launchpad.trim().is_empty() || raw.launchpad.trim() == "pump";
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
            delayMs: Some(
                (parse_int(
                    &raw.postLaunch.automaticDevSell.delaySeconds,
                    "postLaunch.automaticDevSell.delaySeconds",
                    Some(0),
                    Some(10),
                    Some(0),
                )?
                .unwrap_or(0) as u64)
                    * 1000,
            ),
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
            pumpOnly: launchpad_is_pump,
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
            Some(SNIPER_BUY_MAX_TARGET_BLOCK_OFFSET),
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
        let post_buy_sell = match &entry.postBuySell {
            Some(sell) => {
                let normalized_sell =
                    normalize_follow_sell(
                        sell,
                        &wallet_env_key,
                        &format!("snipe-{}-sell", index + 1),
                        SNIPER_AUTO_SELL_MAX_TARGET_BLOCK_OFFSET,
                    )?;
                if let Some(sell) = &normalized_sell {
                    if sell.delayMs.is_some() {
                        return Err(ConfigError::Message(format!(
                            "followLaunch.snipes[{index}].postBuySell.delayMs is not supported. Use targetBlockOffset or marketCap trigger."
                        )));
                    }
                    if sell.targetBlockOffset.is_some() && sell.marketCap.is_some() {
                        return Err(ConfigError::Message(format!(
                            "followLaunch.snipes[{index}].postBuySell must choose either targetBlockOffset or marketCap, not both."
                        )));
                    }
                    if sell.targetBlockOffset.is_none() && sell.marketCap.is_none() {
                        return Err(ConfigError::Message(format!(
                            "followLaunch.snipes[{index}].postBuySell requires targetBlockOffset or marketCap trigger."
                        )));
                    }
                    if sell.percent == 0 {
                        return Err(ConfigError::Message(format!(
                            "followLaunch.snipes[{index}].postBuySell.percent must be between 1 and 100."
                        )));
                    }
                }
                normalized_sell
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
        Some(sell) => normalize_follow_sell(
            sell,
            raw.selectedWalletKey.trim(),
            "dev-auto-sell",
            DEV_AUTO_SELL_MAX_TARGET_BLOCK_OFFSET,
        )?,
        None => None,
    };
    let explicit_enabled = parse_bool(&follow.enabled, false);
    let has_enabled_snipes = snipes.iter().any(|snipe| snipe.enabled);
    let has_enabled_dev_auto_sell = dev_auto_sell.as_ref().is_some_and(|sell| sell.enabled);
    let has_explicit_payload = explicit_enabled || has_enabled_snipes || has_enabled_dev_auto_sell;
    if !has_explicit_payload {
        return legacy_follow_launch(raw, post_launch_strategy);
    }
    Ok(NormalizedFollowLaunch {
        enabled: explicit_enabled || has_enabled_snipes || has_enabled_dev_auto_sell,
        source: "followLaunch".to_string(),
        schemaVersion: explicit_schema,
        snipes,
        devAutoSell: dev_auto_sell,
        constraints: NormalizedFollowLaunchConstraints {
            pumpOnly: parse_bool(
                &follow.constraints.pumpOnly,
                raw.launchpad.trim().is_empty() || raw.launchpad.trim() == "pump",
            ),
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
        "bonkers",
        "cashback",
        "agent-custom",
        "agent-unlocked",
        "agent-locked",
        "bags-2-2",
        "bags-025-1",
        "bags-1-025",
    ]
    .contains(&mode.as_str())
    {
        return Err(ConfigError::Message(format!(
            "mode must be one of regular, bonkers, cashback, agent-custom, agent-unlocked, agent-locked, bags-2-2, bags-025-1, bags-1-025. Got: {mode}"
        )));
    }

    let launchpad = parse_choice(
        &raw.launchpad,
        "launchpad",
        &["pump", "bonk", "bagsapp"],
        "pump",
    )?;
    let mode = if launchpad == "bagsapp" {
        normalize_bags_mode(&mode)
    } else {
        mode
    };
    let quote_asset = normalize_quote_asset(&raw, &launchpad)?;
    let post_launch_strategy = parse_choice(
        &raw.postLaunch.strategy,
        "postLaunch.strategy",
        &["none", "dev-buy", "snipe-own-launch", "automatic-dev-sell"],
        "none",
    )?;

    let fee_recipients = normalize_recipients(
        &raw.feeSharing.recipients,
        "feeSharing recipient",
        false,
        launchpad_supports_extended_social_recipients(&launchpad),
    )?;
    let mut agent_fee_recipients = normalize_recipients(
        &raw.agent.feeRecipients,
        "agent recipient",
        true,
        launchpad_supports_extended_social_recipients(&launchpad),
    )?;
    let default_mev_mode = configured_hellomoon_mev_mode_default();
    let creation_mev_mode = parse_mev_mode(
        if raw.execution.mevMode.is_some() {
            &raw.execution.mevMode
        } else {
            &raw.execution.mevProtect
        },
        &default_mev_mode,
    );
    let buy_mev_mode = parse_mev_mode(
        if raw.execution.buyMevMode.is_some() {
            &raw.execution.buyMevMode
        } else {
            &raw.execution.buyMevProtect
        },
        &default_mev_mode,
    );
    let sell_mev_mode = parse_mev_mode(
        if raw.execution.sellMevMode.is_some() {
            &raw.execution.sellMevMode
        } else {
            &raw.execution.sellMevProtect
        },
        &default_mev_mode,
    );

    let execution_provider = parse_choice(
        &raw.execution.provider,
        "execution.provider",
        &["standard-rpc", "helius-sender", "hellomoon", "jito-bundle"],
        "helius-sender",
    )?;
    // All supported send paths either require skip preflight (Sender / Hello Moon), use bundle APIs
    // without Solana RPC preflight (Jito), or use standard-rpc fanout which sets skip in TransportPlan.
    // Keep execution.skipPreflight true so reports and follow jobs stay consistent.
    let execution_skip_preflight = true;

    let mut normalized = NormalizedConfig {
        mode: mode.clone(),
        launchpad,
        quoteAsset: quote_asset,
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
        bags: NormalizedBags {
            configType: bags_config_type_for_mode(&mode).to_string(),
            identityMode: "wallet-only".to_string(),
            agentUsername: String::new(),
            authToken: String::new(),
            identityVerifiedWallet: String::new(),
        },
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
            skipPreflight: execution_skip_preflight,
            trackSendBlockHeight: parse_bool(
                &raw.execution.trackSendBlockHeight,
                configured_track_send_block_height_default(),
            ),
            provider: execution_provider,
            endpointProfile: if is_blank(&raw.execution.endpointProfile) {
                String::new()
            } else {
                crate::endpoint_profile::parse_config_endpoint_profile(
                    &raw.execution.endpointProfile,
                )
                .map_err(|msg| ConfigError::Message(format!("execution.endpointProfile: {msg}")))?
            },
            mevProtect: mev_mode_enables_hellomoon_protect(&creation_mev_mode),
            mevMode: creation_mev_mode.clone(),
            jitodontfront: mev_mode_enables_jitodontfront(&creation_mev_mode),
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
                &["standard-rpc", "helius-sender", "hellomoon", "jito-bundle"],
                "helius-sender",
            )?,
            buyEndpointProfile: if is_blank(&raw.execution.buyEndpointProfile) {
                String::new()
            } else {
                crate::endpoint_profile::parse_config_endpoint_profile(
                    &raw.execution.buyEndpointProfile,
                )
                .map_err(|msg| {
                    ConfigError::Message(format!("execution.buyEndpointProfile: {msg}"))
                })?
            },
            buyMevProtect: mev_mode_enables_hellomoon_protect(&buy_mev_mode),
            buyMevMode: buy_mev_mode.clone(),
            buyJitodontfront: mev_mode_enables_jitodontfront(&buy_mev_mode),
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
            sellAutoGas: parse_bool(&raw.execution.sellAutoGas, false),
            sellAutoMode: if is_blank(&raw.execution.sellAutoMode) {
                "sellAuto".to_string()
            } else {
                raw.execution.sellAutoMode.trim().to_string()
            },
            sellProvider: parse_choice(
                &raw.execution.sellProvider,
                "execution.sellProvider",
                &["standard-rpc", "helius-sender", "hellomoon", "jito-bundle"],
                "helius-sender",
            )?,
            sellEndpointProfile: if is_blank(&raw.execution.sellEndpointProfile) {
                String::new()
            } else {
                crate::endpoint_profile::parse_config_endpoint_profile(
                    &raw.execution.sellEndpointProfile,
                )
                .map_err(|msg| {
                    ConfigError::Message(format!("execution.sellEndpointProfile: {msg}"))
                })?
            },
            sellMevProtect: mev_mode_enables_hellomoon_protect(&sell_mev_mode),
            sellMevMode: sell_mev_mode.clone(),
            sellJitodontfront: mev_mode_enables_jitodontfront(&sell_mev_mode),
            sellPriorityFeeSol: raw.execution.sellPriorityFeeSol.trim().to_string(),
            sellTipSol: raw.execution.sellTipSol.trim().to_string(),
            sellSlippagePercent: raw.execution.sellSlippagePercent.trim().to_string(),
            sellMaxPriorityFeeSol: raw.execution.sellMaxPriorityFeeSol.trim().to_string(),
            sellMaxTipSol: raw.execution.sellMaxTipSol.trim().to_string(),
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

    if normalized.token.uri.is_empty() && normalized.launchpad != "bagsapp" {
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
    if provider_requires_tip_and_priority(&normalized.execution.provider) {
        let minimum_tip_lamports =
            provider_required_tip_lamports(&normalized.execution.provider).unwrap_or(200_000);
        if normalized.tx.computeUnitPriceMicroLamports.unwrap_or(0) <= 0 {
            return Err(ConfigError::Message(format!(
                "tx.computeUnitPriceMicroLamports must be greater than 0 when execution.provider is {}.",
                normalized.execution.provider
            )));
        }
        if normalized.tx.jitoTipLamports < minimum_tip_lamports as i64 {
            return Err(ConfigError::Message(format!(
                "tx.jitoTipLamports must be at least {} when execution.provider is {}.",
                minimum_tip_lamports, normalized.execution.provider
            )));
        }
    }
    if normalized.execution.provider == "helius-sender"
        || normalized.execution.provider == "hellomoon"
    {
        if normalized.tx.computeUnitPriceMicroLamports.unwrap_or(0) <= 0 {
            return Err(ConfigError::Message(format!(
                "tx.computeUnitPriceMicroLamports must be greater than 0 when execution.provider is {}.",
                normalized.execution.provider
            )));
        }
        let minimum_tip_lamports =
            provider_required_tip_lamports(&normalized.execution.provider).unwrap_or(200_000);
        if normalized.tx.jitoTipLamports < minimum_tip_lamports as i64 {
            return Err(ConfigError::Message(format!(
                "tx.jitoTipLamports must be at least {} when execution.provider is {}.",
                minimum_tip_lamports, normalized.execution.provider
            )));
        }
    }
    if normalized
        .followLaunch
        .snipes
        .iter()
        .any(|snipe| snipe.enabled)
    {
        validate_manual_provider_fee_fields(
            &normalized.execution.buyProvider,
            normalized.execution.buyAutoGas,
            &normalized.execution.buyPriorityFeeSol,
            &normalized.execution.buyTipSol,
            "execution.buy",
        )?;
    }
    if normalized
        .followLaunch
        .devAutoSell
        .as_ref()
        .is_some_and(|sell| sell.enabled)
    {
        validate_manual_provider_fee_fields(
            &normalized.execution.sellProvider,
            normalized.execution.sellAutoGas,
            &normalized.execution.sellPriorityFeeSol,
            &normalized.execution.sellTipSol,
            "execution.sell",
        )?;
    }

    // Keep variable live for future parity extensions.
    agent_fee_recipients.clear();

    validate_launchpad_support(&normalized)?;

    Ok(normalized)
}

/// When unset or non-false: build `LaunchpadWarmContext` once per launch request (blockhash prime).
#[allow(dead_code)] // Server `main` uses via `launchpad_warm`; other binaries link `config` only.
pub fn configured_launchpad_warm_context_enabled() -> bool {
    match env::var("LAUNCHDECK_LAUNCHPAD_WARM_CONTEXT") {
        Ok(value) => {
            let trimmed = value.trim();
            trimmed != "0" && !trimmed.eq_ignore_ascii_case("false")
        }
        Err(_) => true,
    }
}

/// Opt-in: set `1` or `true` to enable parallel independent warm fetches once implemented.
/// Defaults to **false** so telemetry does not claim parallelism that the builder does not perform yet.
#[allow(dead_code)]
pub fn configured_warm_parallel_fetch_enabled() -> bool {
    match env::var("LAUNCHDECK_LAUNCHPAD_PARALLEL_WARM_FETCH") {
        Ok(value) => {
            let trimmed = value.trim();
            trimmed == "1" || trimmed.eq_ignore_ascii_case("true")
        }
        Err(_) => false,
    }
}

/// Upper bound for concurrent warm RPC operations in the warm-context builder.
#[allow(dead_code)]
pub fn launchpad_warm_max_parallel_fetches() -> usize {
    env::var("LAUNCHDECK_LAUNCHPAD_WARM_MAX_PARALLEL_FETCH")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .filter(|value| *value > 0)
        .unwrap_or(8)
}

/// When unset or non-false: pass Rust-cached blockhash into the Bags helper for prepare/build-launch.
pub fn configured_bags_rust_blockhash_for_helper() -> bool {
    match env::var("LAUNCHDECK_BAGS_HELPER_BLOCKHASH_FROM_RUST") {
        Ok(value) => {
            let trimmed = value.trim();
            trimmed != "0" && !trimmed.eq_ignore_ascii_case("false")
        }
        Err(_) => true,
    }
}

/// Wall-clock cap for confirming a Bags setup transaction batch after submit (websocket + polling).
/// Default 20 seconds; Bags setup should land quickly or fail so shared blockhashes do not expire mid-flight.
#[allow(dead_code)]
pub fn configured_bags_setup_confirm_timeout_secs() -> u64 {
    env::var("LAUNCHDECK_BAGS_SETUP_CONFIRM_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .filter(|value| *value > 0)
        .unwrap_or(20)
}

/// Commitment required before Bags builds the final launch transaction after setup.
/// Defaults to `confirmed`; set `LAUNCHDECK_BAGS_SETUP_GATE_COMMITMENT=processed` to opt into the faster gate.
pub fn configured_bags_setup_gate_commitment() -> String {
    match env::var("LAUNCHDECK_BAGS_SETUP_GATE_COMMITMENT") {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "processed" | "confirmed" | "finalized" => value.trim().to_ascii_lowercase(),
            _ => "confirmed".to_string(),
        },
        Err(_) => "confirmed".to_string(),
    }
}

pub fn configured_launchpad_action_backend_mode(
    launchpad: &str,
    action: &str,
) -> LaunchpadActionBackendMode {
    if launchpad.trim().eq_ignore_ascii_case("bonk")
        || launchpad.trim().eq_ignore_ascii_case("bagsapp")
    {
        return LaunchpadActionBackendMode::Rust;
    }
    let launchpad_key = launchpad
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    let action_key = action
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    let env_key = format!("LAUNCHDECK_{launchpad_key}_{action_key}_BACKEND");
    match env::var(env_key) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "helper" | "helper-backed" => LaunchpadActionBackendMode::Helper,
            "rust" | "rust-native" | "rust-only" => LaunchpadActionBackendMode::Rust,
            _ => LaunchpadActionBackendMode::Auto,
        },
        Err(_) => LaunchpadActionBackendMode::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sample_raw_config() -> RawConfig {
        serde_json::from_value(json!({
            "mode": "regular",
            "launchpad": "pump",
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
        .expect("sample config should deserialize")
    }

    #[test]
    fn launchpad_action_backend_mode_defaults_bonk_and_bags_to_rust() {
        let _guard = env_lock().lock().expect("env lock");
        unsafe {
            env::remove_var("LAUNCHDECK_BONK_STARTUP_WARM_BACKEND");
            env::remove_var("LAUNCHDECK_BAGSAPP_STARTUP_WARM_BACKEND");
        }
        assert_eq!(
            configured_launchpad_action_backend_mode("bonk", "startup-warm"),
            LaunchpadActionBackendMode::Rust
        );
        assert_eq!(
            configured_launchpad_action_backend_mode("bagsapp", "startup-warm"),
            LaunchpadActionBackendMode::Rust
        );
    }

    #[test]
    fn launchpad_action_backend_mode_keeps_bonk_and_bags_rust_only() {
        let _guard = env_lock().lock().expect("env lock");
        unsafe {
            env::set_var("LAUNCHDECK_BONK_STARTUP_WARM_BACKEND", "helper");
            env::set_var("LAUNCHDECK_BAGSAPP_STARTUP_WARM_BACKEND", "helper");
        }
        assert_eq!(
            configured_launchpad_action_backend_mode("bonk", "startup-warm"),
            LaunchpadActionBackendMode::Rust
        );
        assert_eq!(
            configured_launchpad_action_backend_mode("bagsapp", "startup-warm"),
            LaunchpadActionBackendMode::Rust
        );
        unsafe {
            env::remove_var("LAUNCHDECK_BONK_STARTUP_WARM_BACKEND");
            env::remove_var("LAUNCHDECK_BAGSAPP_STARTUP_WARM_BACKEND");
        }
    }

    #[test]
    fn normalizes_minimal_regular_config() {
        let normalized =
            normalize_raw_config(sample_raw_config()).expect("config should normalize");
        assert_eq!(normalized.mode, "regular");
        assert_eq!(normalized.launchpad, "pump");
        assert_eq!(normalized.quoteAsset, "sol");
        assert_eq!(normalized.token.name, "LaunchDeck");
        assert_eq!(normalized.execution.provider, "helius-sender");
        assert_eq!(normalized.execution.endpointProfile, "global");
        assert_eq!(normalized.execution.buyProvider, "helius-sender");
        assert_eq!(normalized.execution.buyEndpointProfile, "global");
        assert!(normalized.devBuy.is_none());
        assert!(!normalized.followLaunch.enabled);
    }

    #[test]
    fn rejects_west_endpoint_profile() {
        let mut raw = sample_raw_config();
        raw.execution.endpointProfile = "west".to_string();
        let err = normalize_raw_config(raw).expect_err("west should be rejected");
        assert!(
            err.to_string().to_lowercase().contains("west"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn accepts_comma_metro_endpoint_profile() {
        let mut raw = sample_raw_config();
        raw.execution.endpointProfile = "fra,ams".to_string();
        let normalized = normalize_raw_config(raw).expect("config should normalize");
        assert_eq!(normalized.execution.endpointProfile, "fra,ams");
    }

    #[test]
    fn normalizes_ny_endpoint_profile_to_ewr() {
        let mut raw = sample_raw_config();
        raw.execution.endpointProfile = "NY".to_string();
        let normalized = normalize_raw_config(raw).expect("config should normalize");
        assert_eq!(normalized.execution.endpointProfile, "ewr");
    }

    #[test]
    fn ignores_legacy_policy_fields_during_normalization() {
        let mut raw = sample_raw_config();
        raw.execution.policy = "not-a-valid-policy-anymore".to_string();
        raw.execution.buyPolicy = "fast".to_string();
        raw.execution.sellPolicy = "safe".to_string();

        let normalized = normalize_raw_config(raw).expect("legacy policy fields should be ignored");

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
    fn execution_skip_preflight_always_true_for_all_providers() {
        for provider in ["standard-rpc", "helius-sender", "hellomoon", "jito-bundle"] {
            let mut raw = sample_raw_config();
            raw.execution.provider = provider.to_string();
            raw.execution.skipPreflight = Some(json!(false));
            match provider {
                "hellomoon" => {
                    raw.tx.jitoTipLamports = Some(json!(1_000_000));
                    raw.tx.computeUnitPriceMicroLamports = Some(json!(1));
                }
                "jito-bundle" => {
                    raw.tx.jitoTipLamports = Some(json!(200_000));
                    raw.tx.computeUnitPriceMicroLamports = Some(json!(1));
                }
                _ => {}
            }
            let normalized = normalize_raw_config(raw).unwrap_or_else(|err| {
                panic!("normalize failed for provider {provider}: {err}");
            });
            assert!(
                normalized.execution.skipPreflight,
                "skipPreflight should be true for provider {provider}"
            );
        }
    }

    #[test]
    fn hellomoon_mev_modes_normalize_to_expected_flags() {
        let mut raw = sample_raw_config();
        raw.execution.provider = "hellomoon".to_string();
        raw.tx.jitoTipLamports = Some(json!(1_000_000));
        raw.execution.mevMode = Some(json!("off"));
        raw.execution.buyMevMode = Some(json!("reduced"));
        raw.execution.sellMevMode = Some(json!("secure"));

        let normalized = normalize_raw_config(raw).expect("hellomoon mev modes should normalize");

        assert_eq!(normalized.execution.mevMode, "off");
        assert!(!normalized.execution.mevProtect);
        assert!(!normalized.execution.jitodontfront);

        assert_eq!(normalized.execution.buyMevMode, "reduced");
        assert!(normalized.execution.buyMevProtect);
        assert!(normalized.execution.buyJitodontfront);

        assert_eq!(normalized.execution.sellMevMode, "secure");
        assert!(normalized.execution.sellMevProtect);
        assert!(normalized.execution.sellJitodontfront);
    }

    #[test]
    fn hellomoon_requires_priority_and_minimum_tip() {
        let mut raw = sample_raw_config();
        raw.execution.provider = "hellomoon".to_string();
        raw.tx.jitoTipLamports = Some(json!(1_000_000));
        raw.tx.computeUnitPriceMicroLamports = Some(json!(0));
        let error = normalize_raw_config(raw).expect_err("hellomoon should require priority");
        assert_eq!(
            error.to_string(),
            "tx.computeUnitPriceMicroLamports must be greater than 0 when execution.provider is hellomoon."
        );

        let mut raw = sample_raw_config();
        raw.execution.provider = "hellomoon".to_string();
        raw.tx.jitoTipLamports = Some(json!(999_999));
        raw.tx.computeUnitPriceMicroLamports = Some(json!(1));
        let error = normalize_raw_config(raw).expect_err("hellomoon should require minimum tip");
        assert_eq!(
            error.to_string(),
            "tx.jitoTipLamports must be at least 1000000 when execution.provider is hellomoon."
        );
    }

    #[test]
    fn jito_bundle_requires_priority_and_minimum_tip() {
        let mut raw = sample_raw_config();
        raw.execution.provider = "jito-bundle".to_string();
        raw.tx.jitoTipLamports = Some(json!(100_000));
        raw.tx.computeUnitPriceMicroLamports = Some(json!(0));
        let error = normalize_raw_config(raw).expect_err("jito bundle should require priority");
        assert_eq!(
            error.to_string(),
            "tx.computeUnitPriceMicroLamports must be greater than 0 when execution.provider is jito-bundle."
        );

        let mut raw = sample_raw_config();
        raw.execution.provider = "jito-bundle".to_string();
        raw.tx.jitoTipLamports = Some(json!(100_000));
        raw.tx.computeUnitPriceMicroLamports = Some(json!(1));
        let error = normalize_raw_config(raw).expect_err("jito bundle should require minimum tip");
        assert_eq!(
            error.to_string(),
            "tx.jitoTipLamports must be at least 200000 when execution.provider is jito-bundle."
        );
    }

    #[test]
    fn manual_buy_provider_fees_must_satisfy_provider_minimums() {
        let mut raw = sample_raw_config();
        raw.followLaunch.enabled = Some(json!(true));
        raw.followLaunch.snipes = vec![RawFollowLaunchSnipe {
            enabled: Some(json!(true)),
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            buyAmountSol: "0.1".to_string(),
            ..RawFollowLaunchSnipe::default()
        }];
        raw.execution.buyProvider = "jito-bundle".to_string();
        raw.execution.buyAutoGas = Some(json!(false));
        raw.execution.buyPriorityFeeSol = String::new();
        raw.execution.buyTipSol = "0.0001".to_string();
        let error = normalize_raw_config(raw).expect_err("manual buy fees should be validated");
        assert_eq!(
            error.to_string(),
            "execution.buyPriorityFeeSol must be greater than 0 when execution.buyProvider is jito-bundle."
        );

        let mut raw = sample_raw_config();
        raw.followLaunch.enabled = Some(json!(true));
        raw.followLaunch.snipes = vec![RawFollowLaunchSnipe {
            enabled: Some(json!(true)),
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            buyAmountSol: "0.1".to_string(),
            ..RawFollowLaunchSnipe::default()
        }];
        raw.execution.buyProvider = "jito-bundle".to_string();
        raw.execution.buyAutoGas = Some(json!(false));
        raw.execution.buyPriorityFeeSol = "0.001".to_string();
        raw.execution.buyTipSol = "0.0001".to_string();
        let error =
            normalize_raw_config(raw).expect_err("manual buy tip floor should be validated");
        assert_eq!(
            error.to_string(),
            "execution.buyTipSol must be at least 0.0002 SOL when execution.buyProvider is jito-bundle."
        );
    }

    #[test]
    fn manual_hellomoon_buy_fees_must_satisfy_provider_minimums() {
        let mut raw = sample_raw_config();
        raw.followLaunch.enabled = Some(json!(true));
        raw.followLaunch.snipes = vec![RawFollowLaunchSnipe {
            enabled: Some(json!(true)),
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            buyAmountSol: "0.1".to_string(),
            ..RawFollowLaunchSnipe::default()
        }];
        raw.execution.buyProvider = "hellomoon".to_string();
        raw.execution.buyAutoGas = Some(json!(false));
        raw.execution.buyPriorityFeeSol = String::new();
        raw.execution.buyTipSol = "0.001".to_string();
        let error = normalize_raw_config(raw)
            .expect_err("manual hellomoon buy priority should be required");
        assert_eq!(
            error.to_string(),
            "execution.buyPriorityFeeSol must be greater than 0 when execution.buyProvider is hellomoon."
        );

        let mut raw = sample_raw_config();
        raw.followLaunch.enabled = Some(json!(true));
        raw.followLaunch.snipes = vec![RawFollowLaunchSnipe {
            enabled: Some(json!(true)),
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            buyAmountSol: "0.1".to_string(),
            ..RawFollowLaunchSnipe::default()
        }];
        raw.execution.buyProvider = "hellomoon".to_string();
        raw.execution.buyAutoGas = Some(json!(false));
        raw.execution.buyPriorityFeeSol = "0.001".to_string();
        raw.execution.buyTipSol = "0.000999999".to_string();
        let error = normalize_raw_config(raw)
            .expect_err("manual hellomoon buy tip floor should be validated");
        assert_eq!(
            error.to_string(),
            "execution.buyTipSol must be at least 0.001 SOL when execution.buyProvider is hellomoon."
        );
    }

    #[test]
    fn manual_helius_follow_fees_require_explicit_buy_values() {
        let mut raw = sample_raw_config();
        raw.followLaunch.enabled = Some(json!(true));
        raw.followLaunch.snipes = vec![RawFollowLaunchSnipe {
            enabled: Some(json!(true)),
            walletEnvKey: "SOLANA_PRIVATE_KEY".to_string(),
            buyAmountSol: "0.1".to_string(),
            ..RawFollowLaunchSnipe::default()
        }];
        raw.execution.buyProvider = "helius-sender".to_string();
        raw.execution.buyAutoGas = Some(json!(false));
        raw.execution.buyPriorityFeeSol = String::new();
        raw.execution.buyTipSol = String::new();

        let error =
            normalize_raw_config(raw).expect_err("manual helius buy fees should be required");

        assert_eq!(
            error.to_string(),
            "execution.buyPriorityFeeSol and execution.buyTipSol are required when execution.buyProvider is helius-sender and execution.buyAutoGas is false."
        );
    }

    #[test]
    fn manual_helius_follow_fees_require_explicit_sell_values() {
        let mut raw = sample_raw_config();
        raw.postLaunch.strategy = "automatic-dev-sell".to_string();
        raw.postLaunch.automaticDevSell.enabled = Some(json!(true));
        raw.postLaunch.automaticDevSell.percent = Some(json!(75));
        raw.postLaunch.automaticDevSell.delaySeconds = Some(json!(0));
        raw.execution.sellProvider = "helius-sender".to_string();
        raw.execution.sellAutoGas = Some(json!(false));
        raw.execution.sellPriorityFeeSol = String::new();
        raw.execution.sellTipSol = String::new();

        let error =
            normalize_raw_config(raw).expect_err("manual helius sell fees should be required");

        assert_eq!(
            error.to_string(),
            "execution.sellPriorityFeeSol and execution.sellTipSol are required when execution.sellProvider is helius-sender and execution.sellAutoGas is false."
        );
    }

    #[test]
    fn rejects_removed_auto_provider_values() {
        let mut raw = sample_raw_config();
        raw.execution.provider = "auto".to_string();
        let error = normalize_raw_config(raw).expect_err("auto should be rejected");
        assert!(error.to_string().contains(
            "execution.provider must be one of standard-rpc, helius-sender, hellomoon, jito-bundle"
        ));
    }

    #[test]
    fn bonk_rejects_pump_only_modes() {
        let mut raw = sample_raw_config();
        raw.launchpad = "bonk".to_string();
        raw.mode = "cashback".to_string();

        let error = normalize_raw_config(raw).expect_err("bonk should reject cashback mode");
        assert_eq!(
            error.to_string(),
            "Bonk currently supports only regular and bonkers modes. Got mode=cashback."
        );
    }

    #[test]
    fn bonk_rejects_fee_sharing_setup() {
        let mut raw = sample_raw_config();
        raw.launchpad = "bonk".to_string();
        raw.feeSharing.generateLaterSetup = Some(json!(true));
        raw.feeSharing.recipients = vec![RawRecipient {
            r#type: "wallet".to_string(),
            address: "11111111111111111111111111111111".to_string(),
            githubUsername: String::new(),
            githubUserId: String::new(),
            shareBps: Some(json!(10_000)),
        }];

        let error = normalize_raw_config(raw).expect_err("bonk should reject fee-sharing setup");
        assert_eq!(
            error.to_string(),
            "Bonk does not support fee-sharing setup yet."
        );
    }

    #[test]
    fn bagsapp_accepts_supported_social_fee_recipients() {
        let mut raw = sample_raw_config();
        raw.launchpad = "bagsapp".to_string();
        raw.feeSharing.recipients = vec![
            RawRecipient {
                r#type: "twitter".to_string(),
                address: String::new(),
                githubUsername: "launchdeck".to_string(),
                githubUserId: String::new(),
                shareBps: Some(json!(5_000)),
            },
            RawRecipient {
                r#type: "wallet".to_string(),
                address: "11111111111111111111111111111111".to_string(),
                githubUsername: String::new(),
                githubUserId: String::new(),
                shareBps: Some(json!(5_000)),
            },
        ];

        let normalized =
            normalize_raw_config(raw).expect("bagsapp social recipients should normalize");
        assert_eq!(normalized.feeSharing.recipients.len(), 2);
        assert_eq!(
            normalized.feeSharing.recipients[0].r#type.as_deref(),
            Some("twitter")
        );
        assert_eq!(
            normalized.feeSharing.recipients[0].githubUsername,
            "launchdeck"
        );
    }

    #[test]
    fn bagsapp_allows_helper_managed_metadata_uri() {
        let mut raw = sample_raw_config();
        raw.launchpad = "bagsapp".to_string();
        raw.token.uri = String::new();

        let normalized =
            normalize_raw_config(raw).expect("bagsapp should allow helper-managed metadata uri");
        assert!(normalized.token.uri.is_empty());
        assert_eq!(normalized.launchpad, "bagsapp");
    }

    #[test]
    fn pump_rejects_extended_social_fee_recipients() {
        let mut raw = sample_raw_config();
        raw.launchpad = "pump".to_string();
        raw.feeSharing.recipients = vec![
            RawRecipient {
                r#type: "twitter".to_string(),
                address: String::new(),
                githubUsername: "launchdeck".to_string(),
                githubUserId: String::new(),
                shareBps: Some(json!(5_000)),
            },
            RawRecipient {
                r#type: "wallet".to_string(),
                address: "11111111111111111111111111111111".to_string(),
                githubUsername: String::new(),
                githubUserId: String::new(),
                shareBps: Some(json!(5_000)),
            },
        ];

        let error =
            normalize_raw_config(raw).expect_err("pump should reject extended social recipients");
        assert!(
            error
                .to_string()
                .contains("only supported for Bags launches"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn bonk_allows_bonkers_mode() {
        let mut raw = sample_raw_config();
        raw.launchpad = "bonk".to_string();
        raw.mode = "bonkers".to_string();
        raw.quoteAsset = "usd1".to_string();

        let normalized = normalize_raw_config(raw).expect("bonk bonkers mode should normalize");
        assert_eq!(normalized.launchpad, "bonk");
        assert_eq!(normalized.mode, "bonkers");
        assert_eq!(normalized.quoteAsset, "usd1");
    }

    #[test]
    fn pump_rejects_non_sol_quote_asset() {
        let mut raw = sample_raw_config();
        raw.launchpad = "pump".to_string();
        raw.quoteAsset = "usd1".to_string();
        let error = normalize_raw_config(raw).expect_err("pump should reject usd1 quote asset");
        assert_eq!(
            error.to_string(),
            "quoteAsset=usd1 is only supported for bonk right now."
        );
    }

    #[test]
    fn migrates_legacy_post_launch_into_follow_launch() {
        let mut raw = sample_raw_config();
        raw.selectedWalletKey = "SOLANA_PRIVATE_KEY2".to_string();
        raw.postLaunch.strategy = "automatic-dev-sell".to_string();
        raw.postLaunch.automaticDevSell.enabled = Some(json!(true));
        raw.postLaunch.automaticDevSell.percent = Some(json!(75));
        raw.postLaunch.automaticDevSell.delaySeconds = Some(json!(2));
        raw.execution.sellPriorityFeeSol = "0.000001".to_string();
        raw.execution.sellTipSol = "0.0002".to_string();
        let normalized = normalize_raw_config(raw).expect("legacy postLaunch should migrate");
        assert!(normalized.followLaunch.enabled);
        let dev_auto_sell = normalized
            .followLaunch
            .devAutoSell
            .expect("dev auto sell should be present");
        assert_eq!(dev_auto_sell.walletEnvKey, "SOLANA_PRIVATE_KEY2");
        assert_eq!(dev_auto_sell.percent, 75);
        assert_eq!(dev_auto_sell.delayMs, Some(2000));
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
                    "targetBlockOffset": 37,
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
        assert_eq!(snipe.targetBlockOffset, Some(37));
        assert_eq!(snipe.jitterMs, 5);
        assert_eq!(snipe.feeJitterBps, 250);
    }

    #[test]
    fn rejects_sniper_post_buy_sell_delay_mode() {
        let mut raw = sample_raw_config();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": true,
            "schemaVersion": 1,
            "snipes": [
                {
                    "walletEnvKey": "SOLANA_PRIVATE_KEY2",
                    "buyAmountSol": "0.25",
                    "postBuySell": { "enabled": true, "percent": 100, "delayMs": 2500 }
                }
            ]
        }))
        .expect("follow launch raw");
        let error = normalize_raw_config(raw).expect_err("sniper sell delay should be rejected");
        assert!(
            error
                .to_string()
                .contains("postBuySell.delayMs is not supported")
        );
    }

    #[test]
    fn normalizes_sniper_post_buy_sell_block_offset_trigger() {
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
                        "percent": 50,
                        "targetBlockOffset": 23
                    }
                }
            ]
        }))
        .expect("follow launch raw");
        let normalized = normalize_raw_config(raw).expect("sniper post buy sell should normalize");
        let snipe = &normalized.followLaunch.snipes[0];
        let sell = snipe
            .postBuySell
            .clone()
            .expect("post buy sell should be present");
        assert_eq!(sell.percent, 50);
        assert_eq!(sell.targetBlockOffset, Some(23));
        assert!(sell.marketCap.is_none());
    }

    #[test]
    fn disabled_follow_launch_entries_do_not_enable_follow_daemon() {
        let mut raw = sample_raw_config();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": false,
            "schemaVersion": 1,
            "snipes": [
                {
                    "enabled": false,
                    "walletEnvKey": "SOLANA_PRIVATE_KEY2",
                    "buyAmountSol": "0.25",
                    "submitDelayMs": 30
                }
            ],
            "devAutoSell": {
                "enabled": false,
                "percent": 50,
                "delayMs": 2000
            }
        }))
        .expect("follow launch raw");

        let normalized =
            normalize_raw_config(raw).expect("disabled follow launch should normalize");
        assert!(!normalized.followLaunch.enabled);
        assert!(normalized.followLaunch.snipes.is_empty());
        assert!(normalized.followLaunch.devAutoSell.is_none());
    }

    #[test]
    fn normalizes_market_cap_scan_timeout_for_follow_sell() {
        let mut raw = sample_raw_config();
        raw.execution.sellPriorityFeeSol = "0.000001".to_string();
        raw.execution.sellTipSol = "0.0002".to_string();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": true,
            "schemaVersion": 1,
            "devAutoSell": {
                "enabled": true,
                "walletEnvKey": "SOLANA_PRIVATE_KEY",
                "percent": 100,
                "targetBlockOffset": 1,
                "marketCap": {
                    "enabled": true,
                    "direction": "gte",
                    "threshold": "250000000",
                    "scanTimeoutSeconds": 42,
                    "timeoutAction": "sell"
                }
            }
        }))
        .expect("follow launch raw");

        let normalized =
            normalize_raw_config(raw).expect("market-cap follow sell should normalize");
        let dev_auto_sell = normalized
            .followLaunch
            .devAutoSell
            .expect("dev auto sell should be present");
        let trigger = dev_auto_sell
            .marketCap
            .expect("market-cap trigger should be present");
        assert_eq!(trigger.direction, "gte");
        assert_eq!(trigger.threshold, "250000000000000");
        assert_eq!(trigger.scanTimeoutSeconds, 42);
        assert_eq!(trigger.timeoutAction, "sell");
    }

    #[test]
    fn normalizes_market_cap_shorthand_threshold_for_follow_sell() {
        let mut raw = sample_raw_config();
        raw.execution.sellPriorityFeeSol = "0.000001".to_string();
        raw.execution.sellTipSol = "0.0002".to_string();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": true,
            "schemaVersion": 1,
            "devAutoSell": {
                "enabled": true,
                "walletEnvKey": "SOLANA_PRIVATE_KEY",
                "percent": 100,
                "targetBlockOffset": 1,
                "marketCap": {
                    "enabled": true,
                    "threshold": "100k",
                    "scanTimeoutSeconds": 15
                }
            }
        }))
        .expect("follow launch raw");

        let normalized = normalize_raw_config(raw).expect("market-cap shorthand should normalize");
        let dev_auto_sell = normalized
            .followLaunch
            .devAutoSell
            .expect("dev auto sell should be present");
        let trigger = dev_auto_sell
            .marketCap
            .expect("market-cap trigger should be present");
        assert_eq!(trigger.direction, "gte");
        assert_eq!(trigger.threshold, "100000000000");
        assert_eq!(trigger.scanTimeoutSeconds, 15);
        assert_eq!(trigger.timeoutAction, "stop");
    }

    #[test]
    fn defaults_market_cap_follow_sell_timeout_to_thirty_seconds() {
        let mut raw = sample_raw_config();
        raw.execution.sellPriorityFeeSol = "0.000001".to_string();
        raw.execution.sellTipSol = "0.0002".to_string();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": true,
            "schemaVersion": 1,
            "devAutoSell": {
                "enabled": true,
                "walletEnvKey": "SOLANA_PRIVATE_KEY",
                "percent": 100,
                "targetBlockOffset": 1,
                "marketCap": {
                    "enabled": true,
                    "threshold": "100k"
                }
            }
        }))
        .expect("follow launch raw");

        let normalized =
            normalize_raw_config(raw).expect("market-cap default timeout should normalize");
        let dev_auto_sell = normalized
            .followLaunch
            .devAutoSell
            .expect("dev auto sell should be present");
        let trigger = dev_auto_sell
            .marketCap
            .expect("market-cap trigger should be present");
        assert_eq!(trigger.scanTimeoutSeconds, 30);
        assert_eq!(trigger.timeoutAction, "stop");
    }

    #[test]
    fn rejects_legacy_follow_sell_market_cap_direction() {
        let mut raw = sample_raw_config();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": true,
            "schemaVersion": 1,
            "devAutoSell": {
                "enabled": true,
                "walletEnvKey": "SOLANA_PRIVATE_KEY",
                "percent": 100,
                "marketCap": {
                    "enabled": true,
                    "direction": "lte",
                    "threshold": "250000000",
                    "scanTimeoutSeconds": 42,
                    "timeoutAction": "sell"
                }
            }
        }))
        .expect("follow launch raw");

        let error = normalize_raw_config(raw).expect_err("legacy lte should be rejected");
        assert!(error.to_string().contains("marketCap.direction must be gte"));
    }

    #[test]
    fn rejects_market_cap_follow_sell_without_threshold_when_enabled() {
        let mut raw = sample_raw_config();
        raw.followLaunch = serde_json::from_value(json!({
            "enabled": true,
            "schemaVersion": 1,
            "devAutoSell": {
                "enabled": true,
                "walletEnvKey": "SOLANA_PRIVATE_KEY",
                "percent": 100,
                "marketCap": {
                    "enabled": true,
                    "threshold": ""
                }
            }
        }))
        .expect("follow launch raw");

        let error =
            normalize_raw_config(raw).expect_err("blank threshold should be rejected when enabled");
        assert!(error.to_string().contains("marketCap.threshold is required"));
    }

    #[test]
    fn track_send_block_height_default_only_runs_in_full_mode() {
        assert!(!benchmark_mode_allows_track_send_block_height_default(
            "off"
        ));
        assert!(!benchmark_mode_allows_track_send_block_height_default(
            "light"
        ));
        assert!(!benchmark_mode_allows_track_send_block_height_default(
            "basic"
        ));
        assert!(!benchmark_mode_allows_track_send_block_height_default(
            "unexpected"
        ));
        assert!(benchmark_mode_allows_track_send_block_height_default(
            "full"
        ));
        assert!(benchmark_mode_allows_track_send_block_height_default(""));
    }

    #[test]
    fn explicit_track_send_block_height_false_overrides_full_mode_default() {
        let resolved = parse_bool(&Some(json!(false)), true);
        assert!(!resolved);
    }
}
