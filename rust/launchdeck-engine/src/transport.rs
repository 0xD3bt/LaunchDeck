#![allow(non_snake_case, dead_code)]

use serde::{Deserialize, Serialize};
use std::env;

use crate::{
    config::{NormalizedConfig, NormalizedExecution, has_launch_follow_up},
    providers::get_provider_meta,
};

const DEFAULT_HELIUS_SENDER_ENDPOINT: &str = "https://sender.helius-rpc.com/fast";
const DEFAULT_HELIUS_SENDER_REGIONAL_ENDPOINTS: [(&str, &str); 7] = [
    ("slc", "http://slc-sender.helius-rpc.com/fast"),
    ("ewr", "http://ewr-sender.helius-rpc.com/fast"),
    ("lon", "http://lon-sender.helius-rpc.com/fast"),
    ("fra", "http://fra-sender.helius-rpc.com/fast"),
    ("ams", "http://ams-sender.helius-rpc.com/fast"),
    ("sg", "http://sg-sender.helius-rpc.com/fast"),
    ("tyo", "http://tyo-sender.helius-rpc.com/fast"),
];
const DEFAULT_JITO_BUNDLE_BASE_URLS: [&str; 9] = [
    "https://ny.mainnet.block-engine.jito.wtf",
    "https://frankfurt.mainnet.block-engine.jito.wtf",
    "https://amsterdam.mainnet.block-engine.jito.wtf",
    "https://london.mainnet.block-engine.jito.wtf",
    "https://slc.mainnet.block-engine.jito.wtf",
    "https://mainnet.block-engine.jito.wtf",
    "https://singapore.mainnet.block-engine.jito.wtf",
    "https://tokyo.mainnet.block-engine.jito.wtf",
    "https://dublin.mainnet.block-engine.jito.wtf",
];
const DEFAULT_ENDPOINT_PROFILE: &str = "global";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoBundleEndpoint {
    pub name: String,
    pub send: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportPlan {
    pub requestedProvider: String,
    pub resolvedProvider: String,
    pub requestedEndpointProfile: String,
    pub resolvedEndpointProfile: String,
    pub executionClass: String,
    pub transportType: String,
    pub ordering: String,
    pub verified: bool,
    pub supportsBundle: bool,
    pub requiresInlineTip: bool,
    pub requiresPriorityFee: bool,
    pub separateTipTransaction: bool,
    pub skipPreflight: bool,
    pub maxRetries: u32,
    pub standardRpcSubmitEndpoints: Vec<String>,
    pub heliusSenderEndpoint: Option<String>,
    pub heliusSenderEndpoints: Vec<String>,
    pub watchEndpoint: Option<String>,
    pub watchEndpoints: Vec<String>,
    pub jitoBundleEndpoints: Vec<JitoBundleEndpoint>,
    pub warnings: Vec<String>,
}

fn normalize_provider(provider: &str) -> String {
    if provider.trim().is_empty() {
        "helius-sender".to_string()
    } else {
        provider.trim().to_lowercase()
    }
}

fn normalized_supported_region(region: &str) -> Option<String> {
    match region.trim().to_lowercase().as_str() {
        "global" | "us" | "eu" | "west" | "asia" => Some(region.trim().to_lowercase()),
        _ => None,
    }
}

fn first_non_empty_env(keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| {
            let value = env::var(key).unwrap_or_default();
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_default()
}

fn provider_region_env_key(provider: &str) -> Option<&'static str> {
    match normalize_provider(provider).as_str() {
        "helius-sender" => Some("USER_REGION_HELIUS_SENDER"),
        "jito-bundle" => Some("USER_REGION_JITO_BUNDLE"),
        _ => None,
    }
}

fn default_endpoint_profile_from_user_region(user_region: &str) -> String {
    normalized_supported_region(user_region).unwrap_or_else(|| DEFAULT_ENDPOINT_PROFILE.to_string())
}

fn resolve_default_endpoint_profile_for_provider(
    provider: &str,
    provider_region: &str,
    shared_region: &str,
) -> String {
    if normalize_provider(provider) == "standard-rpc" {
        return String::new();
    }
    normalized_supported_region(provider_region)
        .or_else(|| normalized_supported_region(shared_region))
        .unwrap_or_else(|| DEFAULT_ENDPOINT_PROFILE.to_string())
}

pub fn configured_shared_region() -> String {
    first_non_empty_env(&["USER_REGION"])
}

pub fn configured_provider_region(provider: &str) -> String {
    provider_region_env_key(provider)
        .map(|key| first_non_empty_env(&[key]))
        .unwrap_or_default()
}

pub fn default_endpoint_profile() -> String {
    default_endpoint_profile_from_user_region(&configured_shared_region())
}

pub fn default_endpoint_profile_for_provider(provider: &str) -> String {
    resolve_default_endpoint_profile_for_provider(
        provider,
        &configured_provider_region(provider),
        &configured_shared_region(),
    )
}

fn normalize_endpoint_profile(provider: &str, endpoint_profile: &str) -> String {
    let normalized_provider = normalize_provider(provider);
    if normalized_provider == "standard-rpc" {
        return String::new();
    }
    match endpoint_profile.trim().to_lowercase().as_str() {
        "" => default_endpoint_profile_for_provider(provider),
        "global" | "us" | "eu" | "west" | "asia" => endpoint_profile.trim().to_lowercase(),
        _ => default_endpoint_profile_for_provider(provider),
    }
}

fn configured_helius_sender_override() -> Option<String> {
    let explicit = env::var("HELIUS_SENDER_ENDPOINT").unwrap_or_default();
    let trimmed_explicit = explicit.trim();
    if !trimmed_explicit.is_empty() {
        return Some(trimmed_explicit.to_string());
    }

    let base = env::var("HELIUS_SENDER_BASE_URL")
        .unwrap_or_default()
        .trim()
        .to_string();
    if !base.is_empty() {
        return Some(format!("{}/fast", base.trim_end_matches('/')));
    }

    None
}

pub fn configured_standard_rpc_submit_endpoints() -> Vec<String> {
    env::var("LAUNCHDECK_STANDARD_RPC_SEND_URLS")
        .unwrap_or_default()
        .split(',')
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn helius_sender_endpoint_override_active() -> bool {
    configured_helius_sender_override().is_some()
}

pub fn configured_helius_sender_endpoints_for_profile(endpoint_profile: &str) -> Vec<String> {
    if let Some(override_endpoint) = configured_helius_sender_override() {
        return vec![override_endpoint];
    }

    let resolved_endpoint_profile = normalize_endpoint_profile("helius-sender", endpoint_profile);
    let global_endpoint = DEFAULT_HELIUS_SENDER_ENDPOINT.to_string();
    let regional = |codes: &[&str]| {
        DEFAULT_HELIUS_SENDER_REGIONAL_ENDPOINTS
            .iter()
            .filter(|(code, _)| codes.contains(code))
            .map(|(_, endpoint)| endpoint.to_string())
            .collect::<Vec<_>>()
    };
    match resolved_endpoint_profile.as_str() {
        "us" => regional(&["slc", "ewr"]),
        "eu" => regional(&["lon", "fra", "ams"]),
        "asia" => regional(&["sg", "tyo"]),
        "west" => regional(&["slc", "ewr", "lon", "fra", "ams"]),
        _ => vec![global_endpoint],
    }
}

pub fn configured_helius_sender_endpoint() -> String {
    configured_helius_sender_endpoints_for_profile(&default_endpoint_profile_for_provider(
        "helius-sender",
    ))
    .into_iter()
    .next()
    .unwrap_or_else(|| DEFAULT_HELIUS_SENDER_ENDPOINT.to_string())
}

pub fn configured_watch_endpoints_for_provider(
    provider: &str,
    endpoint_profile: &str,
) -> Vec<String> {
    let _ = normalize_provider(provider);
    let _ = normalize_endpoint_profile(provider, endpoint_profile);
    let explicit_ws = env::var("SOLANA_WS_URL").unwrap_or_default();
    if !explicit_ws.trim().is_empty() {
        return vec![explicit_ws.trim().to_string()];
    }
    vec![]
}

pub fn supports_helius_transaction_subscribe(
    provider: &str,
    endpoint_profile: &str,
    watch_endpoint: Option<&str>,
) -> bool {
    let _ = normalize_provider(provider);
    let _ = normalize_endpoint_profile(provider, endpoint_profile);
    watch_endpoint
        .map(|endpoint| endpoint.trim().to_ascii_lowercase().contains("helius"))
        .unwrap_or(false)
}

pub fn resolved_provider(execution: &NormalizedExecution, transaction_count: usize) -> String {
    let _ = transaction_count;
    normalize_provider(&execution.provider)
}

pub fn execution_class(execution: &NormalizedExecution, transaction_count: usize) -> String {
    let provider = resolved_provider(execution, transaction_count);
    if provider == "jito-bundle" {
        return "bundle".to_string();
    }
    if transaction_count <= 1 {
        return "single".to_string();
    }
    "sequential".to_string()
}

pub fn transport_type(execution: &NormalizedExecution, transaction_count: usize) -> String {
    let provider = resolved_provider(execution, transaction_count);
    match provider.as_str() {
        "standard-rpc" => "standard-rpc-fanout".to_string(),
        "helius-sender" => "helius-sender".to_string(),
        "jito-bundle" => "jito-bundle".to_string(),
        _ => "standard-rpc-fanout".to_string(),
    }
}

pub fn transport_ordering(execution: &NormalizedExecution, transaction_count: usize) -> String {
    match execution_class(execution, transaction_count).as_str() {
        "bundle" => "bundle".to_string(),
        "single" => "single".to_string(),
        _ => "sequential".to_string(),
    }
}

fn jito_endpoint_matches_profile(endpoint: &JitoBundleEndpoint, endpoint_profile: &str) -> bool {
    let name = endpoint.name.to_lowercase();
    match endpoint_profile {
        "us" => name.contains("ny.") || name.contains("slc."),
        "eu" => {
            name.contains("frankfurt.")
                || name.contains("amsterdam.")
                || name.contains("london.")
                || name.contains("dublin.")
        }
        "asia" => name.contains("singapore.") || name.contains("tokyo."),
        "west" => {
            name.contains("ny.")
                || name.contains("slc.")
                || name.contains("frankfurt.")
                || name.contains("amsterdam.")
                || name.contains("london.")
                || name.contains("dublin.")
        }
        _ => true,
    }
}

pub fn configured_jito_bundle_endpoints_for_profile(
    endpoint_profile: &str,
) -> Vec<JitoBundleEndpoint> {
    let resolved_endpoint_profile = normalize_endpoint_profile("jito-bundle", endpoint_profile);
    let explicit_send = env::var("JITO_SEND_BUNDLE_ENDPOINT")
        .unwrap_or_default()
        .trim()
        .to_string();
    let explicit_status = env::var("JITO_BUNDLE_STATUS_ENDPOINT")
        .unwrap_or_default()
        .trim()
        .to_string();
    if !explicit_send.is_empty() || !explicit_status.is_empty() {
        if !explicit_send.is_empty() && !explicit_status.is_empty() {
            return vec![JitoBundleEndpoint {
                name: "custom".to_string(),
                send: explicit_send,
                status: explicit_status,
            }];
        }
        return vec![];
    }

    let configured_bases = env::var("JITO_BUNDLE_BASE_URLS").unwrap_or_default();
    let bases: Vec<String> = if configured_bases.trim().is_empty() {
        DEFAULT_JITO_BUNDLE_BASE_URLS
            .iter()
            .map(|entry| entry.to_string())
            .collect()
    } else {
        configured_bases
            .split(',')
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect()
    };
    bases
        .into_iter()
        .map(|base| JitoBundleEndpoint {
            name: base
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .to_string(),
            send: format!("{base}/api/v1/bundles"),
            status: format!("{base}/api/v1/getBundleStatuses"),
        })
        .filter(|endpoint| jito_endpoint_matches_profile(endpoint, &resolved_endpoint_profile))
        .collect()
}

pub fn jito_bundle_endpoint_override_active() -> bool {
    let explicit_send = env::var("JITO_SEND_BUNDLE_ENDPOINT")
        .unwrap_or_default()
        .trim()
        .to_string();
    let explicit_status = env::var("JITO_BUNDLE_STATUS_ENDPOINT")
        .unwrap_or_default()
        .trim()
        .to_string();
    !explicit_send.is_empty() || !explicit_status.is_empty()
}

pub fn configured_jito_bundle_endpoints() -> Vec<JitoBundleEndpoint> {
    configured_jito_bundle_endpoints_for_profile(&default_endpoint_profile_for_provider(
        "jito-bundle",
    ))
}

pub fn build_transport_plan(
    execution: &NormalizedExecution,
    transaction_count: usize,
) -> TransportPlan {
    let requested = normalize_provider(&execution.provider);
    let resolved = resolved_provider(execution, transaction_count);
    let requested_endpoint_profile =
        normalize_endpoint_profile(&execution.provider, &execution.endpointProfile);
    let resolved_endpoint_profile =
        normalize_endpoint_profile(&resolved, &execution.endpointProfile);
    let class = execution_class(execution, transaction_count);
    let transport = transport_type(execution, transaction_count);
    let ordering = transport_ordering(execution, transaction_count);
    let meta = get_provider_meta(&resolved);
    let helius_sender_endpoints = if transport == "helius-sender" {
        configured_helius_sender_endpoints_for_profile(&resolved_endpoint_profile)
    } else {
        vec![]
    };
    let standard_rpc_submit_endpoints = if resolved == "standard-rpc" {
        configured_standard_rpc_submit_endpoints()
    } else {
        vec![]
    };
    let jito_bundle_endpoints = if transport == "jito-bundle" {
        configured_jito_bundle_endpoints_for_profile(&resolved_endpoint_profile)
    } else {
        vec![]
    };
    let watch_endpoints =
        configured_watch_endpoints_for_provider(&resolved, &resolved_endpoint_profile);
    let mut warnings = Vec::new();
    if !meta.verified {
        warnings.push(format!(
            "Provider {} is currently marked unverified in this environment.",
            resolved
        ));
    }
    if class == "bundle" && jito_bundle_endpoints.is_empty() {
        warnings.push(
            "Bundle execution selected but no Jito bundle endpoints are configured.".to_string(),
        );
    }
    if resolved == "jito-bundle" && transaction_count > 5 {
        warnings.push("Jito bundle transport supports at most 5 transactions.".to_string());
    }
    if resolved == "helius-sender" && !execution.skipPreflight {
        warnings.push(
            "Helius Sender requires skipPreflight=true and will hard-fail if it is disabled."
                .to_string(),
        );
    }
    if resolved == "helius-sender" {
        if let Some(override_endpoint) = configured_helius_sender_override() {
            warnings.push(format!(
                "HELIUS_SENDER endpoint override is active ({override_endpoint}); endpoint profile fanout is bypassed."
            ));
        }
    }

    TransportPlan {
        requestedProvider: requested,
        resolvedProvider: resolved,
        requestedEndpointProfile: requested_endpoint_profile,
        resolvedEndpointProfile: resolved_endpoint_profile,
        executionClass: class,
        transportType: transport.clone(),
        ordering,
        verified: meta.verified,
        supportsBundle: meta.supportsBundle,
        requiresInlineTip: transport == "helius-sender",
        requiresPriorityFee: transport == "helius-sender",
        separateTipTransaction: transport == "jito-bundle",
        skipPreflight: matches!(transport.as_str(), "helius-sender" | "standard-rpc-fanout")
            || execution.skipPreflight,
        maxRetries: if matches!(transport.as_str(), "helius-sender" | "standard-rpc-fanout") {
            0
        } else {
            3
        },
        standardRpcSubmitEndpoints: standard_rpc_submit_endpoints,
        heliusSenderEndpoint: if transport == "helius-sender" {
            helius_sender_endpoints.first().cloned()
        } else {
            None
        },
        heliusSenderEndpoints: helius_sender_endpoints,
        watchEndpoint: watch_endpoints.first().cloned(),
        watchEndpoints: watch_endpoints,
        jitoBundleEndpoints: jito_bundle_endpoints,
        warnings,
    }
}

pub fn estimate_transaction_count(config: &NormalizedConfig) -> usize {
    let mut transaction_count = 1usize;
    if has_launch_follow_up(config) {
        transaction_count += 1;
    }
    if normalize_provider(&config.execution.provider) == "jito-bundle"
        && config.tx.jitoTipLamports > 0
    {
        transaction_count += 1;
    }
    transaction_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{RawConfig, normalize_raw_config};
    use serde_json::json;

    fn sample_config(provider: &str) -> NormalizedConfig {
        let raw: RawConfig = serde_json::from_value(json!({
            "mode": "regular",
            "launchpad": "pump",
            "token": {
                "name": "LaunchDeck",
                "symbol": "LDECK",
                "uri": "ipfs://test"
            },
            "tx": {
                "computeUnitPriceMicroLamports": 1,
                "jitoTipLamports": 200000,
                "jitoTipAccount": "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE"
            },
            "execution": {
                "provider": provider,
                "buyProvider": provider,
                "sellProvider": provider,
                "skipPreflight": true
            }
        }))
        .expect("sample config");
        normalize_raw_config(raw).expect("normalized config")
    }

    #[test]
    fn standard_rpc_resolves_to_sequential_transport() {
        let config = sample_config("standard-rpc");
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.transportType, "standard-rpc-fanout");
        assert_eq!(plan.executionClass, "sequential");
        assert!(!plan.requiresInlineTip);
        assert_eq!(plan.maxRetries, 0);
        assert!(plan.skipPreflight);
    }

    #[test]
    fn helius_sender_resolves_to_sender_transport() {
        let config = sample_config("helius-sender");
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.transportType, "helius-sender");
        assert_eq!(plan.executionClass, "sequential");
        assert!(plan.requiresInlineTip);
        assert_eq!(plan.maxRetries, 0);
    }

    #[test]
    fn jito_bundle_resolves_to_bundle_transport() {
        let config = sample_config("jito-bundle");
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.transportType, "jito-bundle");
        assert_eq!(plan.executionClass, "bundle");
        assert!(plan.separateTipTransaction);
    }

    #[test]
    fn helius_sender_eu_profile_filters_endpoints() {
        let mut config = sample_config("helius-sender");
        config.execution.endpointProfile = "eu".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "eu");
        assert!(!plan.heliusSenderEndpoints.is_empty());
        assert!(plan.heliusSenderEndpoints.iter().all(|entry| {
            entry.contains("lon-") || entry.contains("fra-") || entry.contains("ams-")
        }));
        assert!(
            plan.heliusSenderEndpoints
                .iter()
                .all(|entry| !entry.contains("https://sender.helius-rpc.com/fast"))
        );
        assert!(
            plan.heliusSenderEndpoints
                .iter()
                .all(|entry| entry.starts_with("http://"))
        );
    }

    #[test]
    fn standard_rpc_ignores_endpoint_profile() {
        let mut config = sample_config("standard-rpc");
        config.execution.endpointProfile = "asia".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "");
        assert!(plan.heliusSenderEndpoints.is_empty());
        assert!(plan.jitoBundleEndpoints.is_empty());
    }

    #[test]
    fn user_region_defaults_endpoint_profile() {
        assert_eq!(default_endpoint_profile_from_user_region("EU"), "eu");
        assert_eq!(default_endpoint_profile_from_user_region("asia"), "asia");
        assert_eq!(default_endpoint_profile_from_user_region(""), "global");
        assert_eq!(default_endpoint_profile_from_user_region("nope"), "global");
    }

    #[test]
    fn provider_specific_region_overrides_shared_default() {
        assert_eq!(
            resolve_default_endpoint_profile_for_provider("helius-sender", "eu", "us"),
            "eu"
        );
        assert_eq!(
            resolve_default_endpoint_profile_for_provider("jito-bundle", "", "asia"),
            "asia"
        );
        assert_eq!(
            resolve_default_endpoint_profile_for_provider("jito-bundle", "invalid", "us"),
            "us"
        );
        assert_eq!(
            resolve_default_endpoint_profile_for_provider("helius-sender", "", ""),
            "global"
        );
    }

    #[test]
    fn standard_rpc_has_no_default_endpoint_profile() {
        assert_eq!(
            resolve_default_endpoint_profile_for_provider("standard-rpc", "eu", "us"),
            ""
        );
    }

    #[test]
    fn helius_transaction_subscribe_requires_helius_ws() {
        assert!(supports_helius_transaction_subscribe(
            "helius-sender",
            "global",
            Some("wss://mainnet.helius-rpc.com/?api-key=test")
        ));
        assert!(supports_helius_transaction_subscribe(
            "standard-rpc",
            "global",
            Some("wss://mainnet.helius-rpc.com/?api-key=test")
        ));
        assert!(!supports_helius_transaction_subscribe(
            "helius-sender",
            "global",
            Some("wss://example-rpc.com/ws")
        ));
    }
}
