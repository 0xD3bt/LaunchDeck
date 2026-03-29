#![allow(non_snake_case, dead_code)]

use std::{collections::BTreeMap, env};

use serde::Serialize;

use crate::config::NormalizedExecution;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub verified: bool,
    pub supportsSingle: bool,
    pub supportsSequential: bool,
    pub supportsBundle: bool,
    pub supportsEndpointProfiles: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderAvailability {
    pub provider: String,
    pub available: bool,
    pub verified: bool,
    pub supportState: String,
    pub supportsSingle: bool,
    pub supportsSequential: bool,
    pub supportsBundle: bool,
    pub supportsEndpointProfiles: bool,
    pub reason: String,
}

pub fn provider_registry() -> Vec<ProviderMeta> {
    vec![
        ProviderMeta {
            id: "standard-rpc",
            label: "Standard RPC",
            verified: true,
            supportsSingle: true,
            supportsSequential: true,
            supportsBundle: false,
            supportsEndpointProfiles: false,
        },
        ProviderMeta {
            id: "helius-sender",
            label: "Helius Sender",
            verified: true,
            supportsSingle: true,
            supportsSequential: true,
            supportsBundle: false,
            supportsEndpointProfiles: true,
        },
        ProviderMeta {
            id: "jito-bundle",
            label: "Jito Bundle",
            verified: true,
            supportsSingle: true,
            supportsSequential: false,
            supportsBundle: true,
            supportsEndpointProfiles: true,
        },
    ]
}

pub fn provider_availability_registry() -> BTreeMap<String, ProviderAvailability> {
    let solana_rpc_configured = env::var("SOLANA_RPC_URL")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    provider_registry()
        .into_iter()
        .map(|provider| {
            let reason = if provider.id == "helius-sender" && !solana_rpc_configured {
                "Using the default Helius Sender endpoint with the localhost RPC fallback; set SOLANA_RPC_URL for a dedicated confirmation RPC.".to_string()
            } else {
                String::new()
            };
            (
                provider.id.to_string(),
                ProviderAvailability {
                    provider: provider.id.to_string(),
                    available: true,
                    verified: provider.verified,
                    supportState: if provider.verified {
                        "verified".to_string()
                    } else {
                        "unverified".to_string()
                    },
                    supportsSingle: provider.supportsSingle,
                    supportsSequential: provider.supportsSequential,
                    supportsBundle: provider.supportsBundle,
                    supportsEndpointProfiles: provider.supportsEndpointProfiles,
                    reason,
                },
            )
        })
        .collect()
}

pub fn get_provider_meta(provider: &str) -> ProviderMeta {
    let normalized = if provider.trim().is_empty() {
        "helius-sender"
    } else {
        provider.trim()
    }
    .to_lowercase();
    provider_registry()
        .into_iter()
        .find(|entry| entry.id == normalized)
        .unwrap_or_else(|| {
            provider_registry()
                .into_iter()
                .find(|entry| entry.id == "helius-sender")
                .expect("helius-sender provider must exist")
        })
}

pub fn get_resolved_provider(execution: &NormalizedExecution, transaction_count: usize) -> String {
    let _ = transaction_count;
    if execution.provider.trim().is_empty() {
        "helius-sender".to_string()
    } else {
        execution.provider.trim().to_lowercase()
    }
}

pub fn get_execution_class(execution: &NormalizedExecution, transaction_count: usize) -> String {
    let resolved_provider = get_resolved_provider(execution, transaction_count);
    if resolved_provider == "jito-bundle" {
        return "bundle".to_string();
    }
    if transaction_count <= 1 {
        return "single".to_string();
    }
    "sequential".to_string()
}
