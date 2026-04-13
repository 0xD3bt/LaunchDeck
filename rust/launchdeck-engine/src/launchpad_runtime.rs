#![allow(non_snake_case, dead_code)]

use serde_json::{Value, json};

use crate::{
    bags_native::BagsFeeRecipientLookupResponse,
    config::NormalizedConfig,
    launchpad_dispatch::{
        LaunchpadStartupWarmResult, NativeLaunchArtifacts, launchpad_action_backend,
        launchpad_action_rollout_state, lookup_fee_recipient_for_launchpad,
        quote_launch_for_launchpad, try_compile_native_launchpad, warm_launchpad_for_startup,
    },
    pump_native::LaunchQuote,
    transport::TransportPlan,
};

#[derive(Debug, Clone)]
pub struct NativeLaunchCompileRequest<'a> {
    pub rpc_url: &'a str,
    pub config: &'a NormalizedConfig,
    pub transport_plan: &'a TransportPlan,
    pub wallet_secret: &'a [u8],
    pub built_at: String,
    pub creator_public_key: String,
    pub config_path: Option<String>,
    pub allow_ata_creation: bool,
    pub launch_blockhash_prime: Option<(String, u64)>,
}

#[derive(Debug, Clone)]
pub struct LaunchQuoteRequest<'a> {
    pub rpc_url: &'a str,
    pub launchpad: &'a str,
    pub quote_asset: &'a str,
    pub launch_mode: &'a str,
    pub mode: &'a str,
    pub amount: &'a str,
}

#[derive(Debug, Clone)]
pub struct FeeRecipientLookupRequest<'a> {
    pub launchpad: &'a str,
    pub rpc_url: &'a str,
    pub provider: &'a str,
    pub username: &'a str,
    pub github_user_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct StartupWarmLaunchpadPayloads {
    pub lookup_tables: Value,
    pub pump_global: Value,
    pub bonk_state: Value,
    pub bags_helper: Value,
}

fn startup_warm_error_payload(error: String) -> Value {
    json!({
        "ok": false,
        "error": error,
    })
}

fn pump_startup_warm_payload(
    result: Result<Option<LaunchpadStartupWarmResult>, String>,
) -> (Value, Value) {
    match result {
        Ok(Some(LaunchpadStartupWarmResult::Pump {
            lookupTablesLoaded,
            previewBasis,
        })) => (
            json!({
                "ok": true,
                "loaded": lookupTablesLoaded.unwrap_or_default(),
            }),
            json!({
                "ok": true,
                "previewBasis": previewBasis,
            }),
        ),
        Ok(Some(other)) => {
            let error = startup_warm_error_payload(format!(
                "Unexpected startup warm payload for pump launchpad: {other:?}"
            ));
            (error.clone(), error)
        }
        Ok(None) => {
            let error = startup_warm_error_payload("Pump startup warm payload was unavailable.".to_string());
            (error.clone(), error)
        }
        Err(error) => {
            let error = startup_warm_error_payload(error);
            (error.clone(), error)
        }
    }
}

fn bonk_startup_warm_payload(result: Result<Option<LaunchpadStartupWarmResult>, String>) -> Value {
    match result {
        Ok(Some(LaunchpadStartupWarmResult::Bonk { payload })) => payload,
        Ok(Some(other)) => startup_warm_error_payload(format!(
            "Unexpected startup warm payload for bonk launchpad: {other:?}"
        )),
        Ok(None) => startup_warm_error_payload("Bonk startup warm payload was unavailable.".to_string()),
        Err(error) => startup_warm_error_payload(error),
    }
}

fn bags_startup_warm_payload(result: Result<Option<LaunchpadStartupWarmResult>, String>) -> Value {
    match result {
        Ok(Some(LaunchpadStartupWarmResult::Bagsapp { payload })) => json!({
            "ok": payload.get("ok").and_then(Value::as_bool).unwrap_or(false),
            "backend": launchpad_action_backend("bagsapp", "startup-warm"),
            "rolloutState": launchpad_action_rollout_state("bagsapp", "startup-warm"),
            "state": payload,
        }),
        Ok(Some(other)) => startup_warm_error_payload(format!(
            "Unexpected startup warm payload for bagsapp launchpad: {other:?}"
        )),
        Ok(None) => startup_warm_error_payload("Bags startup warm payload was unavailable.".to_string()),
        Err(error) => startup_warm_error_payload(error),
    }
}

pub async fn warm_launchpads_for_startup(
    rpc_url: &str,
) -> Result<StartupWarmLaunchpadPayloads, String> {
    let (pump, bonk, bags) = tokio::join!(
        warm_launchpad_for_startup("pump", rpc_url),
        warm_launchpad_for_startup("bonk", rpc_url),
        warm_launchpad_for_startup("bagsapp", rpc_url),
    );
    let (lookup_tables, pump_global) = pump_startup_warm_payload(pump);
    let bonk_state = bonk_startup_warm_payload(bonk);
    let bags_helper = bags_startup_warm_payload(bags);

    Ok(StartupWarmLaunchpadPayloads {
        lookup_tables,
        pump_global,
        bonk_state,
        bags_helper,
    })
}

pub async fn compile_native_launch(
    request: NativeLaunchCompileRequest<'_>,
) -> Result<Option<NativeLaunchArtifacts>, String> {
    try_compile_native_launchpad(
        request.rpc_url,
        request.config,
        request.transport_plan,
        request.wallet_secret,
        request.built_at,
        request.creator_public_key,
        request.config_path,
        request.allow_ata_creation,
        request.launch_blockhash_prime,
    )
    .await
}

pub async fn quote_launch(
    request: LaunchQuoteRequest<'_>,
) -> Result<Option<LaunchQuote>, String> {
    quote_launch_for_launchpad(
        request.rpc_url,
        request.launchpad,
        request.quote_asset,
        request.launch_mode,
        request.mode,
        request.amount,
    )
    .await
}

pub async fn lookup_fee_recipient(
    request: FeeRecipientLookupRequest<'_>,
) -> Result<Option<BagsFeeRecipientLookupResponse>, String> {
    lookup_fee_recipient_for_launchpad(
        request.launchpad,
        request.rpc_url,
        request.provider,
        request.username,
        request.github_user_id,
    )
    .await
}
