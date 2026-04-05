#![allow(non_snake_case, dead_code)]

use serde::{Deserialize, Serialize};
use std::env;

use crate::{
    config::{NormalizedConfig, NormalizedExecution, has_launch_follow_up},
    endpoint_profile::{
        metro_token_canonical, normalize_user_region, parse_config_endpoint_profile,
    },
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
const DEFAULT_HELLOMOON_GLOBAL_QUIC_ENDPOINT: &str = "lunar-lander.hellomoon.io:16888";
const DEFAULT_HELLOMOON_REGIONAL_QUIC_ENDPOINTS: [(&str, &str); 5] = [
    ("fra", "fra.lunar-lander.hellomoon.io:16888"),
    ("ams", "ams.lunar-lander.hellomoon.io:16888"),
    ("nyc", "nyc.lunar-lander.hellomoon.io:16888"),
    ("ash", "ash.lunar-lander.hellomoon.io:16888"),
    ("tyo", "tyo.lunar-lander.hellomoon.io:16888"),
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
pub struct HelloMoonEndpoint {
    pub name: String,
    pub quic: String,
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
    pub helloMoonApiKeyConfigured: bool,
    pub helloMoonMevProtect: bool,
    pub helloMoonQuicEndpoint: Option<String>,
    pub helloMoonQuicEndpoints: Vec<String>,
    #[serde(default)]
    pub helloMoonBundleEndpoint: Option<String>,
    #[serde(default)]
    pub helloMoonBundleEndpoints: Vec<String>,
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
        "hellomoon" => Some("USER_REGION_HELLOMOON"),
        "jito-bundle" => Some("USER_REGION_JITO_BUNDLE"),
        _ => None,
    }
}

fn default_endpoint_profile_from_user_region(user_region: &str) -> String {
    normalize_user_region(user_region).unwrap_or_else(|| DEFAULT_ENDPOINT_PROFILE.to_string())
}

fn resolve_default_endpoint_profile_for_provider(
    provider: &str,
    provider_region: &str,
    shared_region: &str,
) -> String {
    if normalize_provider(provider) == "standard-rpc" {
        return String::new();
    }
    normalize_user_region(provider_region)
        .or_else(|| normalize_user_region(shared_region))
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
    let trimmed = endpoint_profile.trim();
    if trimmed.is_empty() {
        return default_endpoint_profile_for_provider(provider);
    }
    parse_config_endpoint_profile(trimmed)
        .unwrap_or_else(|_| default_endpoint_profile_for_provider(provider))
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

fn configured_hellomoon_quic_override() -> Option<String> {
    let explicit = first_non_empty_env(&["HELLOMOON_QUIC_ENDPOINT", "LUNAR_LANDER_QUIC_ENDPOINT"]);
    let trimmed = explicit.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn configured_hellomoon_api_key() -> String {
    env::var("HELLOMOON_API_KEY").unwrap_or_default()
}

pub fn hellomoon_api_key_configured() -> bool {
    !configured_hellomoon_api_key().is_empty()
}

pub fn configured_hellomoon_mev_protect() -> bool {
    matches!(
        first_non_empty_env(&["HELLOMOON_MEV_PROTECT", "LUNAR_LANDER_MEV_PROTECT"])
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub fn configured_standard_rpc_submit_endpoints() -> Vec<String> {
    first_non_empty_env(&[
        "LAUNCHDECK_EXTRA_STANDARD_RPC_SEND_URLS",
        "LAUNCHDECK_STANDARD_RPC_SEND_URLS",
    ])
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
            .filter(|(code, _)| codes.iter().any(|c| *c == *code))
            .map(|(_, endpoint)| endpoint.to_string())
            .collect::<Vec<_>>()
    };
    if resolved_endpoint_profile.contains(',') {
        let codes: Vec<&str> = resolved_endpoint_profile
            .split(',')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .collect();
        let endpoints = regional(&codes);
        return if endpoints.is_empty() {
            vec![global_endpoint]
        } else {
            endpoints
        };
    }
    match resolved_endpoint_profile.as_str() {
        "us" => regional(&["slc", "ewr"]),
        "eu" => regional(&["fra", "ams"]),
        "asia" => regional(&["sg", "tyo"]),
        code if metro_token_canonical(code).is_some() => regional(&[code]),
        _ => vec![global_endpoint],
    }
}

fn hellomoon_profile_tokens(endpoint_profile: &str) -> Vec<String> {
    let resolved = normalize_endpoint_profile("hellomoon", endpoint_profile);
    let map_token = |token: &str| match token {
        "global" => vec!["global".to_string()],
        "us" => vec!["nyc".to_string(), "ash".to_string()],
        "eu" => vec!["fra".to_string(), "ams".to_string()],
        // Hello Moon only exposes Tokyo in Asia, so both the Asia group and Singapore fall back
        // to the Tokyo endpoint.
        "asia" => vec!["tyo".to_string()],
        // Hello Moon does not expose exact Newark/SLC regional endpoints, so keep US metro
        // selections on the dual-US fanout across New York and Ashburn.
        "ewr" => vec!["nyc".to_string(), "ash".to_string()],
        "slc" => vec!["nyc".to_string(), "ash".to_string()],
        "fra" => vec!["fra".to_string()],
        "ams" => vec!["ams".to_string()],
        // Hello Moon does not expose a London endpoint, so use the dual-EU fanout set.
        "lon" => vec!["fra".to_string(), "ams".to_string()],
        "sg" => vec!["tyo".to_string()],
        "tyo" => vec!["tyo".to_string()],
        _ => vec!["global".to_string()],
    };
    if resolved.contains(',') {
        let mut out = Vec::new();
        for token in resolved.split(',').map(|value| value.trim()) {
            for mapped in map_token(token) {
                if !out.iter().any(|existing| existing == &mapped) {
                    out.push(mapped);
                }
            }
        }
        return out;
    }
    map_token(&resolved)
}

pub fn configured_hellomoon_quic_endpoints_for_profile(endpoint_profile: &str) -> Vec<String> {
    if let Some(override_endpoint) = configured_hellomoon_quic_override() {
        return vec![override_endpoint];
    }
    let profile_tokens = hellomoon_profile_tokens(endpoint_profile);
    if profile_tokens.iter().any(|token| token == "global") {
        return vec![DEFAULT_HELLOMOON_GLOBAL_QUIC_ENDPOINT.to_string()];
    }
    let mut endpoints = Vec::new();
    for token in profile_tokens {
        if let Some((_, endpoint)) = DEFAULT_HELLOMOON_REGIONAL_QUIC_ENDPOINTS
            .iter()
            .find(|(name, _)| *name == token)
        {
            if !endpoints.iter().any(|existing| existing == endpoint) {
                endpoints.push((*endpoint).to_string());
            }
        }
    }
    if endpoints.is_empty() {
        vec![DEFAULT_HELLOMOON_GLOBAL_QUIC_ENDPOINT.to_string()]
    } else {
        endpoints
    }
}

pub fn configured_hellomoon_bundle_endpoints_for_profile(endpoint_profile: &str) -> Vec<String> {
    let profile_tokens = hellomoon_profile_tokens(endpoint_profile);
    if profile_tokens.iter().any(|token| token == "global") {
        return vec!["http://lunar-lander.hellomoon.io/sendBundle".to_string()];
    }
    let mut endpoints = Vec::new();
    for token in profile_tokens {
        let endpoint = match token.as_str() {
            "fra" => Some("http://fra.lunar-lander.hellomoon.io/sendBundle"),
            "ams" => Some("http://ams.lunar-lander.hellomoon.io/sendBundle"),
            "nyc" => Some("http://nyc.lunar-lander.hellomoon.io/sendBundle"),
            "ash" => Some("http://ash.lunar-lander.hellomoon.io/sendBundle"),
            "tyo" => Some("http://tyo.lunar-lander.hellomoon.io/sendBundle"),
            _ => None,
        };
        if let Some(endpoint) = endpoint {
            if !endpoints.iter().any(|existing| existing == endpoint) {
                endpoints.push(endpoint.to_string());
            }
        }
    }
    if endpoints.is_empty() {
        vec!["http://lunar-lander.hellomoon.io/sendBundle".to_string()]
    } else {
        endpoints
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

/// Optional dedicated Helius HTTP JSON-RPC URL (e.g. for `getPriorityFeeEstimate`) when
/// `SOLANA_RPC_URL` points at a non-Helius provider.
pub fn configured_helius_rpc_url_trimmed() -> Option<String> {
    let trimmed = env::var("HELIUS_RPC_URL").unwrap_or_default();
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// JSON-RPC URL used for Helius priority-fee API calls. Uses `HELIUS_RPC_URL` when set.
pub fn resolved_helius_priority_fee_rpc_url(primary_solana_rpc: &str) -> String {
    configured_helius_rpc_url_trimmed().unwrap_or_else(|| primary_solana_rpc.to_string())
}

/// Optional dedicated Helius websocket URL for `transactionSubscribe` when `SOLANA_WS_URL` is not Helius.
pub fn configured_helius_ws_url_trimmed() -> Option<String> {
    let trimmed = env::var("HELIUS_WS_URL").unwrap_or_default();
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// WebSocket URL for Helius `transactionSubscribe`. Prefers `HELIUS_WS_URL` when set; otherwise a
/// `SOLANA_WS_URL` / transport watch endpoint that looks Helius-hosted.
pub fn resolved_helius_transaction_subscribe_ws_url(base_watch_endpoint: Option<&str>) -> Option<String> {
    if let Some(url) = configured_helius_ws_url_trimmed() {
        return Some(url);
    }
    base_watch_endpoint
        .filter(|endpoint| endpoint.trim().to_ascii_lowercase().contains("helius"))
        .map(|endpoint| endpoint.trim().to_string())
}

pub fn configured_enable_helius_transaction_subscribe() -> bool {
    match env::var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "" => true,
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => true,
    }
}

/// When `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE` is true, returns whether a Helius WS URL is available.
pub fn prefers_helius_transaction_subscribe_path(
    helius_subscribe_enabled: bool,
    base_watch_endpoint: Option<&str>,
) -> bool {
    helius_subscribe_enabled
        && resolved_helius_transaction_subscribe_ws_url(base_watch_endpoint).is_some()
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
    if let Some(url) = configured_helius_ws_url_trimmed() {
        return vec![url];
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
    if provider == "jito-bundle"
        || (provider == "hellomoon" && execution.mevMode.trim().eq_ignore_ascii_case("secure"))
    {
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
        "hellomoon" => {
            if execution.mevMode.trim().eq_ignore_ascii_case("secure") {
                "hellomoon-bundle".to_string()
            } else {
                "hellomoon-quic".to_string()
            }
        }
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

fn jito_name_matches_metro_token(name: &str, token: &str) -> bool {
    match token.trim().to_lowercase().as_str() {
        "slc" => name.contains("slc."),
        "ewr" | "ny" => name.contains("ny."),
        "fra" => name.contains("frankfurt."),
        "ams" => name.contains("amsterdam."),
        "lon" => name.contains("london."),
        "sg" => name.contains("singapore."),
        "tyo" => name.contains("tokyo."),
        _ => false,
    }
}

fn jito_endpoint_matches_profile(endpoint: &JitoBundleEndpoint, endpoint_profile: &str) -> bool {
    let name = endpoint.name.to_lowercase();
    if endpoint_profile.contains(',') {
        return endpoint_profile
            .split(',')
            .any(|t| jito_name_matches_metro_token(&name, t.trim()));
    }
    match endpoint_profile {
        "us" => name.contains("ny.") || name.contains("slc."),
        "eu" => {
            name.contains("frankfurt.")
                || name.contains("amsterdam.")
                || name.contains("london.")
                || name.contains("dublin.")
        }
        "asia" => name.contains("singapore.") || name.contains("tokyo."),
        "global" => true,
        other => jito_name_matches_metro_token(&name, other),
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
    let hello_moon_quic_endpoints = if transport == "hellomoon-quic" {
        configured_hellomoon_quic_endpoints_for_profile(&resolved_endpoint_profile)
    } else {
        vec![]
    };
    let hello_moon_bundle_endpoints = if transport == "hellomoon-bundle" {
        configured_hellomoon_bundle_endpoints_for_profile(&resolved_endpoint_profile)
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
    if transport == "jito-bundle" && jito_bundle_endpoints.is_empty() {
        warnings.push(
            "Bundle execution selected but no Jito bundle endpoints are configured.".to_string(),
        );
    }
    if resolved == "jito-bundle" && transaction_count > 5 {
        warnings.push("Jito bundle transport supports at most 5 transactions.".to_string());
    }
    if transport == "hellomoon-bundle" && transaction_count > 4 {
        warnings.push("Hello Moon bundle transport supports at most 4 transactions.".to_string());
    }
    if resolved == "helius-sender" && !execution.skipPreflight {
        warnings.push(
            "Helius Sender requires skipPreflight=true and will hard-fail if it is disabled."
                .to_string(),
        );
    }
    if resolved == "hellomoon" && !execution.skipPreflight {
        warnings.push(
            "Hello Moon QUIC runs as a low-latency fire-and-forget path and expects skipPreflight=true."
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
    if resolved == "hellomoon" {
        if !hellomoon_api_key_configured() {
            warnings.push(
                "Hello Moon QUIC requires HELLOMOON_API_KEY.".to_string(),
            );
        }
        if let Some(override_endpoint) = configured_hellomoon_quic_override() {
            warnings.push(format!(
                "HELLOMOON_QUIC_ENDPOINT override is active ({override_endpoint}); endpoint profile fanout is bypassed."
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
        requiresInlineTip: matches!(
            transport.as_str(),
            "helius-sender" | "hellomoon-quic" | "hellomoon-bundle"
        ),
        requiresPriorityFee: matches!(
            transport.as_str(),
            "helius-sender" | "hellomoon-quic" | "hellomoon-bundle"
        ),
        separateTipTransaction: transport == "jito-bundle",
        skipPreflight: matches!(
            transport.as_str(),
            "helius-sender"
                | "hellomoon-quic"
                | "hellomoon-bundle"
                | "standard-rpc-fanout"
                | "jito-bundle"
        ) || execution.skipPreflight,
        maxRetries: if matches!(
            transport.as_str(),
            "helius-sender" | "hellomoon-quic" | "hellomoon-bundle" | "standard-rpc-fanout"
        ) {
            0
        } else {
            3
        },
        standardRpcSubmitEndpoints: standard_rpc_submit_endpoints,
        helloMoonApiKeyConfigured: hellomoon_api_key_configured(),
        helloMoonMevProtect: transport == "hellomoon-quic" && execution.mevProtect,
        helloMoonQuicEndpoint: if transport == "hellomoon-quic" {
            hello_moon_quic_endpoints.first().cloned()
        } else {
            None
        },
        helloMoonQuicEndpoints: hello_moon_quic_endpoints,
        helloMoonBundleEndpoint: if transport == "hellomoon-bundle" {
            hello_moon_bundle_endpoints.first().cloned()
        } else {
            None
        },
        helloMoonBundleEndpoints: hello_moon_bundle_endpoints,
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
        let tip_lamports = if provider == "hellomoon" {
            1_000_000
        } else {
            200_000
        };
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
                "jitoTipLamports": tip_lamports,
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
    fn hellomoon_resolves_to_quic_transport() {
        let config = sample_config("hellomoon");
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.transportType, "hellomoon-quic");
        assert_eq!(plan.executionClass, "sequential");
        assert!(plan.requiresInlineTip);
        assert!(plan.requiresPriorityFee);
        assert_eq!(plan.maxRetries, 0);
    }

    #[test]
    fn hellomoon_reduced_keeps_quic_transport() {
        let mut config = sample_config("hellomoon");
        config.execution.mevMode = "reduced".to_string();
        config.execution.mevProtect = true;
        config.execution.jitodontfront = true;

        let plan = build_transport_plan(&config.execution, 2);

        assert_eq!(plan.transportType, "hellomoon-quic");
        assert_eq!(plan.executionClass, "sequential");
        assert!(plan.helloMoonMevProtect);
        assert!(!plan.helloMoonQuicEndpoints.is_empty());
        assert!(plan.helloMoonBundleEndpoints.is_empty());
    }

    #[test]
    fn hellomoon_secure_resolves_to_bundle_transport() {
        let mut config = sample_config("hellomoon");
        config.execution.mevMode = "secure".to_string();
        config.execution.mevProtect = true;
        config.execution.jitodontfront = true;

        let plan = build_transport_plan(&config.execution, 3);

        assert_eq!(plan.transportType, "hellomoon-bundle");
        assert_eq!(plan.executionClass, "bundle");
        assert!(plan.requiresInlineTip);
        assert!(plan.requiresPriorityFee);
        assert!(plan.skipPreflight);
        assert_eq!(plan.maxRetries, 0);
        assert!(plan.helloMoonQuicEndpoints.is_empty());
        assert!(!plan.helloMoonBundleEndpoints.is_empty());
        assert!(
            plan.helloMoonBundleEndpoints
                .iter()
                .all(|endpoint| endpoint.contains("/sendBundle"))
        );
    }

    #[test]
    fn helius_sender_is_unchanged_when_mev_modes_exist() {
        let mut config = sample_config("helius-sender");
        config.execution.mevMode = "secure".to_string();
        config.execution.mevProtect = true;
        config.execution.jitodontfront = true;

        let plan = build_transport_plan(&config.execution, 2);

        assert_eq!(plan.transportType, "helius-sender");
        assert_eq!(plan.executionClass, "sequential");
        assert!(plan.helloMoonQuicEndpoints.is_empty());
        assert!(plan.helloMoonBundleEndpoints.is_empty());
        assert!(!plan.heliusSenderEndpoints.is_empty());
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
    fn helius_sender_multi_metro_profile_filters_endpoints() {
        let mut config = sample_config("helius-sender");
        config.execution.endpointProfile = "fra,lon".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "fra,lon");
        assert_eq!(plan.heliusSenderEndpoints.len(), 2);
        assert!(
            plan.heliusSenderEndpoints
                .iter()
                .all(|entry| { entry.contains("fra-") || entry.contains("lon-") })
        );
    }

    #[test]
    fn helius_sender_eu_profile_filters_endpoints() {
        let mut config = sample_config("helius-sender");
        config.execution.endpointProfile = "eu".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "eu");
        assert!(!plan.heliusSenderEndpoints.is_empty());
        assert!(
            plan.heliusSenderEndpoints
                .iter()
                .all(|entry| { entry.contains("fra-") || entry.contains("ams-") })
        );
        assert!(
            plan.heliusSenderEndpoints
                .iter()
                .all(|entry| !entry.contains("lon-"))
        );
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
    fn helius_sender_asia_profile_filters_to_singapore_and_tokyo() {
        let mut config = sample_config("helius-sender");
        config.execution.endpointProfile = "asia".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "asia");
        assert!(!plan.heliusSenderEndpoints.is_empty());
        assert!(
            plan.heliusSenderEndpoints
                .iter()
                .all(|entry| entry.contains("sg-") || entry.contains("tyo-"))
        );
    }

    #[test]
    fn helius_sender_singapore_profile_stays_singapore_only() {
        let mut config = sample_config("helius-sender");
        config.execution.endpointProfile = "sg".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "sg");
        assert!(!plan.heliusSenderEndpoints.is_empty());
        assert!(
            plan.heliusSenderEndpoints
                .iter()
                .all(|entry| entry.contains("sg-"))
        );
    }

    #[test]
    fn helius_sender_tokyo_profile_stays_tokyo_only() {
        let mut config = sample_config("helius-sender");
        config.execution.endpointProfile = "tyo".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "tyo");
        assert!(!plan.heliusSenderEndpoints.is_empty());
        assert!(
            plan.heliusSenderEndpoints
                .iter()
                .all(|entry| entry.contains("tyo-"))
        );
    }

    #[test]
    fn standard_rpc_ignores_endpoint_profile() {
        let mut config = sample_config("standard-rpc");
        config.execution.endpointProfile = "asia".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "");
        assert!(plan.heliusSenderEndpoints.is_empty());
        assert!(plan.helloMoonQuicEndpoints.is_empty());
        assert!(plan.jitoBundleEndpoints.is_empty());
    }

    #[test]
    fn hellomoon_us_profile_maps_to_dual_us_quic_endpoints() {
        let mut config = sample_config("hellomoon");
        config.execution.endpointProfile = "us".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "us");
        assert_eq!(plan.helloMoonQuicEndpoints.len(), 2);
        assert!(
            plan.helloMoonQuicEndpoints
                .iter()
                .any(|entry| entry.contains("nyc.lunar-lander"))
        );
        assert!(
            plan.helloMoonQuicEndpoints
                .iter()
                .any(|entry| entry.contains("ash.lunar-lander"))
        );
    }

    #[test]
    fn hellomoon_ewr_profile_still_maps_to_dual_us_quic_endpoints() {
        let mut config = sample_config("hellomoon");
        config.execution.endpointProfile = "ewr".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "ewr");
        assert_eq!(plan.helloMoonQuicEndpoints.len(), 2);
        assert!(
            plan.helloMoonQuicEndpoints
                .iter()
                .any(|entry| entry.contains("nyc.lunar-lander"))
        );
        assert!(
            plan.helloMoonQuicEndpoints
                .iter()
                .any(|entry| entry.contains("ash.lunar-lander"))
        );
    }

    #[test]
    fn hellomoon_slc_profile_still_maps_to_dual_us_quic_endpoints() {
        let mut config = sample_config("hellomoon");
        config.execution.endpointProfile = "slc".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "slc");
        assert_eq!(plan.helloMoonQuicEndpoints.len(), 2);
        assert!(
            plan.helloMoonQuicEndpoints
                .iter()
                .any(|entry| entry.contains("nyc.lunar-lander"))
        );
        assert!(
            plan.helloMoonQuicEndpoints
                .iter()
                .any(|entry| entry.contains("ash.lunar-lander"))
        );
    }

    #[test]
    fn hellomoon_lon_profile_falls_back_to_eu_endpoints() {
        let mut config = sample_config("hellomoon");
        config.execution.endpointProfile = "lon".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "lon");
        assert_eq!(plan.helloMoonQuicEndpoints.len(), 2);
        assert!(
            plan.helloMoonQuicEndpoints
                .iter()
                .all(|entry| entry.contains("fra.") || entry.contains("ams."))
        );
    }

    #[test]
    fn hellomoon_asia_profile_maps_to_tokyo_endpoint() {
        let mut config = sample_config("hellomoon");
        config.execution.endpointProfile = "asia".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "asia");
        assert_eq!(plan.helloMoonQuicEndpoints, vec!["tyo.lunar-lander.hellomoon.io:16888"]);
    }

    #[test]
    fn hellomoon_singapore_profile_falls_back_to_tokyo_endpoint() {
        let mut config = sample_config("hellomoon");
        config.execution.endpointProfile = "sg".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "sg");
        assert_eq!(plan.helloMoonQuicEndpoints, vec!["tyo.lunar-lander.hellomoon.io:16888"]);
    }

    #[test]
    fn hellomoon_tokyo_profile_stays_tokyo_only() {
        let mut config = sample_config("hellomoon");
        config.execution.endpointProfile = "tyo".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert_eq!(plan.resolvedEndpointProfile, "tyo");
        assert_eq!(plan.helloMoonQuicEndpoints, vec!["tyo.lunar-lander.hellomoon.io:16888"]);
    }

    #[test]
    fn user_region_defaults_endpoint_profile() {
        assert_eq!(default_endpoint_profile_from_user_region("EU"), "eu");
        assert_eq!(default_endpoint_profile_from_user_region("asia"), "asia");
        assert_eq!(default_endpoint_profile_from_user_region("fra"), "fra");
        assert_eq!(
            default_endpoint_profile_from_user_region("fra, ams"),
            "fra,ams"
        );
        assert_eq!(default_endpoint_profile_from_user_region("NY"), "ewr");
        assert_eq!(default_endpoint_profile_from_user_region(""), "global");
        assert_eq!(default_endpoint_profile_from_user_region("nope"), "global");
        assert_eq!(default_endpoint_profile_from_user_region("west"), "global");
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

    #[test]
    fn resolved_helius_transaction_subscribe_ws_from_helius_hosted_watch() {
        assert_eq!(
            resolved_helius_transaction_subscribe_ws_url(Some("wss://mainnet.helius-rpc.com/?k=1"))
                .as_deref(),
            Some("wss://mainnet.helius-rpc.com/?k=1")
        );
        assert!(resolved_helius_transaction_subscribe_ws_url(Some("wss://rpc.shyft.to/ws")).is_none());
        assert!(resolved_helius_transaction_subscribe_ws_url(None).is_none());
    }

    #[test]
    fn prefers_helius_subscribe_path_respects_enable_flag() {
        assert!(!prefers_helius_transaction_subscribe_path(
            false,
            Some("wss://mainnet.helius-rpc.com/?k=1")
        ));
        assert!(prefers_helius_transaction_subscribe_path(
            true,
            Some("wss://mainnet.helius-rpc.com/?k=1")
        ));
        assert!(!prefers_helius_transaction_subscribe_path(
            true,
            Some("wss://rpc.shyft.to/ws")
        ));
    }

    #[test]
    fn helius_transaction_subscribe_defaults_to_enabled() {
        unsafe {
            env::remove_var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE");
        }
        assert!(configured_enable_helius_transaction_subscribe());
        unsafe {
            env::set_var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE", "false");
        }
        assert!(!configured_enable_helius_transaction_subscribe());
        unsafe {
            env::remove_var("LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE");
        }
    }

    #[test]
    fn jito_bundle_fra_profile_filters_to_frankfurt_only() {
        let mut config = sample_config("jito-bundle");
        config.execution.endpointProfile = "fra".to_string();
        let plan = build_transport_plan(&config.execution, 2);
        assert!(!plan.jitoBundleEndpoints.is_empty());
        assert!(
            plan.jitoBundleEndpoints
                .iter()
                .all(|e| e.name.to_lowercase().contains("frankfurt"))
        );
    }
}
