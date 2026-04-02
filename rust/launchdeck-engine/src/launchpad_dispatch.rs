#![allow(non_snake_case, dead_code)]

use serde_json::Value;

use crate::{
    bags_native::{
        NativeBagsArtifacts,
        compile_atomic_follow_buy_transaction as compile_atomic_bags_follow_buy,
        quote_launch as quote_bags_launch, try_compile_native_bags,
    },
    bonk_native::{
        NativeBonkArtifacts,
        compile_atomic_follow_buy_transaction as compile_atomic_bonk_follow_buy,
        quote_launch as quote_bonk_launch, try_compile_native_bonk,
    },
    config::{NormalizedConfig, NormalizedExecution},
    pump_native::{
        LaunchQuote, NativeCompileTimings, NativePumpArtifacts,
        compile_atomic_follow_buy_transaction as compile_atomic_pump_follow_buy,
        quote_launch as quote_pump_launch, try_compile_native_pump,
    },
    rpc::CompiledTransaction,
    transport::TransportPlan,
};

#[derive(Debug, Clone)]
pub struct NativeLaunchArtifacts {
    pub compiled_transactions: Vec<CompiledTransaction>,
    pub creation_transactions: Vec<CompiledTransaction>,
    pub deferred_setup_transactions: Vec<CompiledTransaction>,
    pub report: Value,
    pub text: String,
    pub compile_timings: NativeCompileTimings,
    pub mint: String,
    pub launch_creator: String,
}

impl From<NativePumpArtifacts> for NativeLaunchArtifacts {
    fn from(value: NativePumpArtifacts) -> Self {
        Self {
            compiled_transactions: value.compiled_transactions,
            creation_transactions: value.creation_transactions,
            deferred_setup_transactions: value.deferred_setup_transactions,
            report: value.report,
            text: value.text,
            compile_timings: value.compile_timings,
            mint: value.mint,
            launch_creator: value.launch_creator,
        }
    }
}

impl From<NativeBonkArtifacts> for NativeLaunchArtifacts {
    fn from(value: NativeBonkArtifacts) -> Self {
        Self {
            compiled_transactions: value.compiled_transactions,
            creation_transactions: value.creation_transactions,
            deferred_setup_transactions: value.deferred_setup_transactions,
            report: value.report,
            text: value.text,
            compile_timings: value.compile_timings,
            mint: value.mint,
            launch_creator: value.launch_creator,
        }
    }
}

impl From<NativeBagsArtifacts> for NativeLaunchArtifacts {
    fn from(value: NativeBagsArtifacts) -> Self {
        let compiled_transactions = value.compiled_transactions;
        Self {
            creation_transactions: compiled_transactions.clone(),
            compiled_transactions,
            deferred_setup_transactions: vec![],
            report: value.report,
            text: value.text,
            compile_timings: value.compile_timings,
            mint: value.mint,
            launch_creator: value.launch_creator,
        }
    }
}

pub async fn try_compile_native_launchpad(
    rpc_url: &str,
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    wallet_secret: &[u8],
    built_at: String,
    creator_public_key: String,
    config_path: Option<String>,
    allow_ata_creation: bool,
) -> Result<Option<NativeLaunchArtifacts>, String> {
    match config.launchpad.as_str() {
        "pump" => try_compile_native_pump(
            rpc_url,
            config,
            transport_plan,
            wallet_secret,
            built_at,
            creator_public_key,
            config_path,
        )
        .await
        .map(|result| result.map(Into::into)),
        "bonk" => try_compile_native_bonk(
            rpc_url,
            config,
            transport_plan,
            wallet_secret,
            built_at,
            creator_public_key,
            config_path,
            allow_ata_creation,
        )
        .await
        .map(|result| result.map(Into::into)),
        "bagsapp" => try_compile_native_bags(
            rpc_url,
            config,
            transport_plan,
            wallet_secret,
            built_at,
            creator_public_key,
            config_path,
        )
        .await
        .map(|result| result.map(Into::into)),
        _ => Ok(None),
    }
}

pub async fn quote_launch_for_launchpad(
    rpc_url: &str,
    launchpad: &str,
    quote_asset: &str,
    launch_mode: &str,
    mode: &str,
    amount: &str,
) -> Result<Option<LaunchQuote>, String> {
    match launchpad {
        "pump" => quote_pump_launch(rpc_url, mode, amount).await,
        "bonk" => quote_bonk_launch(rpc_url, quote_asset, launch_mode, mode, amount).await,
        "bagsapp" => quote_bags_launch(rpc_url, launch_mode, mode, amount).await,
        _ => Ok(None),
    }
}

pub async fn compile_atomic_follow_buy_for_launchpad(
    launchpad: &str,
    launch_mode: &str,
    quote_asset: &str,
    rpc_url: &str,
    execution: &NormalizedExecution,
    token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    buy_amount_sol: &str,
    allow_ata_creation: bool,
) -> Result<CompiledTransaction, String> {
    match launchpad {
        "pump" => {
            compile_atomic_pump_follow_buy(
                rpc_url,
                execution,
                token_mayhem_mode,
                jito_tip_account,
                wallet_secret,
                mint,
                launch_creator,
                buy_amount_sol,
            )
            .await
        }
        "bonk" => {
            compile_atomic_bonk_follow_buy(
                rpc_url,
                launch_mode,
                quote_asset,
                execution,
                token_mayhem_mode,
                jito_tip_account,
                wallet_secret,
                mint,
                launch_creator,
                buy_amount_sol,
                allow_ata_creation,
            )
            .await
        }
        "bagsapp" => {
            compile_atomic_bags_follow_buy(
                rpc_url,
                launch_mode,
                quote_asset,
                execution,
                token_mayhem_mode,
                jito_tip_account,
                wallet_secret,
                mint,
                launch_creator,
                buy_amount_sol,
            )
            .await
        }
        other => Err(format!(
            "Unsupported launchpad for same-time sniper buys: {other}"
        )),
    }
}
