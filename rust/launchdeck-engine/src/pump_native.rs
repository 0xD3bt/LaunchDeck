#![allow(dead_code, non_snake_case)]

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::{AddressLookupTableAccount, VersionedMessage, v0},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use solana_system_interface::{instruction::transfer, program as system_program};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use std::{
    collections::HashMap,
    fs,
    str::FromStr,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
use tokio::{join, task::JoinSet, time::sleep};

use crate::{
    config::{
        NormalizedConfig, NormalizedExecution, NormalizedRecipient,
        configured_default_agent_setup_compute_unit_limit,
        configured_default_dev_auto_sell_compute_unit_limit,
        configured_default_follow_up_compute_unit_limit,
        configured_default_launch_compute_unit_limit,
        configured_default_sniper_buy_compute_unit_limit, has_launch_follow_up,
        launch_follow_up_label,
    },
    paths,
    report::{LaunchReport, build_report, render_report},
    rpc::{
        CompiledTransaction, fetch_account_data, fetch_account_exists,
        fetch_latest_blockhash_cached, fetch_multiple_account_exists,
    },
    transport::TransportPlan,
    wallet::read_keypair_bytes,
};

const JITODONTFRONT_ACCOUNT: &str = "jitodontfront111111111111111111111111111111";
const PUMP_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const PUMP_AMM_PROGRAM_ID: &str = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA";
const MAYHEM_PROGRAM_ID: &str = "MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e";
const PUMP_FEE_PROGRAM_ID: &str = "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ";
const PUMP_AGENT_PAYMENTS_PROGRAM_ID: &str = "AgenTMiC2hvxGebTsgmsD4HHBa8WEcqGFf87iwRRxLo7";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const COMPUTE_BUDGET_PROGRAM_ID: &str = "ComputeBudget111111111111111111111111111111";
const PRIORITY_FEE_PRICE_BASE_COMPUTE_UNIT_LIMIT: u64 = 1_000_000;
const TOKEN_DECIMALS: u32 = 6;
const GLOBAL_ACCOUNT_DISCRIMINATOR_BYTES: usize = 8;
const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
const PLATFORM_GITHUB: u8 = 2;
const DEFAULT_LOOKUP_TABLES: [&str; 2] = [
    "AXVvmhWaaPtV52jqYuTNqp1xRrkbxhfJfeHQKxq5cbvZ",
    "BckPpoRV4h329qAuhTCNoWdWAy2pZSJ89Qu3nuCU1zsj",
];
const DEFAULT_LAUNCH_LOOKUP_TABLE_PROFILES: [[&str; 1]; 2] = [
    ["AXVvmhWaaPtV52jqYuTNqp1xRrkbxhfJfeHQKxq5cbvZ"],
    ["BckPpoRV4h329qAuhTCNoWdWAy2pZSJ89Qu3nuCU1zsj"],
];
const DEFAULT_FOLLOW_UP_LOOKUP_TABLE_PROFILES: [[&str; 1]; 2] = [
    ["BckPpoRV4h329qAuhTCNoWdWAy2pZSJ89Qu3nuCU1zsj"],
    ["AXVvmhWaaPtV52jqYuTNqp1xRrkbxhfJfeHQKxq5cbvZ"],
];
const LOOKUP_TABLE_CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub struct NativePumpArtifacts {
    pub compiled_transactions: Vec<CompiledTransaction>,
    pub creation_transactions: Vec<CompiledTransaction>,
    pub deferred_setup_transactions: Vec<CompiledTransaction>,
    pub report: Value,
    pub text: String,
    pub compile_timings: NativeCompileTimings,
    pub mint: String,
    pub launch_creator: String,
}

#[derive(Debug, Clone, Default)]
pub struct NativeCompileTimings {
    pub launch_creator_prep_ms: u128,
    pub alt_load_ms: u128,
    pub blockhash_fetch_ms: u128,
    pub global_fetch_ms: Option<u128>,
    pub follow_up_prep_ms: Option<u128>,
    pub tx_serialize_ms: u128,
    pub launch_serialize_ms: Option<u128>,
    pub follow_up_serialize_ms: Option<u128>,
    pub tip_serialize_ms: Option<u128>,
}

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchQuote {
    pub mode: String,
    pub input: String,
    pub estimatedTokens: String,
    pub estimatedSol: String,
    #[serde(default)]
    pub estimatedQuoteAmount: String,
    #[serde(default)]
    pub quoteAsset: String,
    #[serde(default)]
    pub quoteAssetLabel: String,
    pub estimatedSupplyPercent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpMarketSnapshot {
    pub mint: String,
    pub creator: String,
    pub virtualTokenReserves: u64,
    pub virtualSolReserves: u64,
    pub realTokenReserves: u64,
    pub realSolReserves: u64,
    pub tokenTotalSupply: u64,
    pub complete: bool,
    pub marketCapLamports: u64,
    pub marketCapSol: String,
}

pub fn supports_native_pump_compile(config: &NormalizedConfig) -> bool {
    if config.launchpad != "pump" {
        return false;
    }
    if config.mode != "regular"
        && config.mode != "cashback"
        && config.mode != "agent-unlocked"
        && config.mode != "agent-custom"
        && config.mode != "agent-locked"
    {
        return false;
    }
    if config.feeSharing.generateLaterSetup
        && config.feeSharing.recipients.iter().any(|entry| {
            (!entry.githubUserId.is_empty() || !entry.githubUsername.is_empty())
                && entry.githubUserId.is_empty()
        })
    {
        return false;
    }
    if config.creatorFee.mode == "github" && config.creatorFee.githubUserId.is_empty() {
        return false;
    }
    if config.mode == "agent-custom"
        && config.agent.feeRecipients.iter().any(|entry| {
            (!entry.githubUserId.is_empty() || !entry.githubUsername.is_empty())
                && entry.githubUserId.is_empty()
        })
    {
        return false;
    }
    true
}

pub async fn try_compile_native_pump(
    rpc_url: &str,
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    wallet_secret: &[u8],
    built_at: String,
    creator_public_key: String,
    config_path: Option<String>,
) -> Result<Option<NativePumpArtifacts>, String> {
    if !supports_native_pump_compile(config) {
        return Ok(None);
    }

    let creator_keypair = keypair_from_secret_bytes(wallet_secret)?;
    let creator = creator_keypair.pubkey();
    let agent_authority = resolve_agent_authority(config, &creator)?;
    let mint_keypair = resolve_mint_keypair(config)?;
    let mint = mint_keypair.pubkey();
    let separate_tip_transaction =
        transport_plan.separateTipTransaction && config.tx.jitoTipLamports > 0;
    let has_follow_up_transaction = has_launch_follow_up(config);
    let launch_creator_future = async {
        let started = Instant::now();
        let resolved =
            resolve_launch_creator_and_pre_instructions(rpc_url, config, &creator).await?;
        Ok::<_, String>((started.elapsed().as_millis(), resolved))
    };
    let lookup_tables_future = async {
        let started = Instant::now();
        let tables = load_lookup_table_accounts(rpc_url, config).await?;
        Ok::<_, String>((started.elapsed().as_millis(), tables))
    };
    let blockhash_future = async {
        let started = Instant::now();
        let blockhash = fetch_latest_blockhash_cached(rpc_url, "confirmed").await?;
        Ok::<_, String>((started.elapsed().as_millis(), blockhash))
    };
    let global_future = async {
        if config.devBuy.is_some() {
            let started = Instant::now();
            let global = fetch_global_state_cached(rpc_url).await?;
            Ok::<_, String>(Some((started.elapsed().as_millis(), global)))
        } else {
            Ok::<_, String>(None)
        }
    };
    let (launch_creator_result, lookup_tables_result, blockhash_result, global_result) = join!(
        launch_creator_future,
        lookup_tables_future,
        blockhash_future,
        global_future
    );
    let (launch_creator_prep_ms, (launch_creator, launch_pre_instructions)) =
        launch_creator_result?;
    let (alt_load_ms, lookup_tables) = lookup_tables_result?;
    let (blockhash_fetch_ms, (blockhash, last_valid_block_height)) = blockhash_result?;
    let global_result = global_result?;
    let global_fetch_ms = global_result.as_ref().map(|(elapsed_ms, _)| *elapsed_ms);
    let global = global_result.map(|(_, global)| global);
    let mut compile_timings = NativeCompileTimings {
        launch_creator_prep_ms,
        alt_load_ms,
        blockhash_fetch_ms,
        global_fetch_ms,
        follow_up_prep_ms: None,
        tx_serialize_ms: 0,
        launch_serialize_ms: None,
        follow_up_serialize_ms: None,
        tip_serialize_ms: None,
    };
    let tx_format = select_native_format(&config.execution.txFormat, !lookup_tables.is_empty())?;
    let creation_compute_unit_price_micro_lamports =
        effective_creation_compute_unit_price_micro_lamports(
            config,
            transport_plan,
            has_follow_up_transaction,
        );
    let single_bundle_tip_last_tx = transport_plan.transportType == "hellomoon-bundle";
    let launch_tip_lamports = if transport_plan.requiresInlineTip || !separate_tip_transaction {
        if single_bundle_tip_last_tx && has_follow_up_transaction {
            0
        } else {
            config.tx.jitoTipLamports
        }
    } else {
        0
    };
    let launch_tip_account = if launch_tip_lamports > 0 && !config.tx.jitoTipAccount.is_empty() {
        config.tx.jitoTipAccount.clone()
    } else {
        String::new()
    };
    let follow_up_tip_lamports = if transport_plan.requiresInlineTip {
        config.tx.jitoTipLamports
    } else {
        0
    };
    let follow_up_tip_account =
        if follow_up_tip_lamports > 0 && !config.tx.jitoTipAccount.is_empty() {
            config.tx.jitoTipAccount.clone()
        } else {
            String::new()
        };

    let launch_tx_config = NativeTxConfig {
        compute_unit_limit: configured_launch_compute_unit_limit(config)?,
        compute_unit_price_micro_lamports: creation_compute_unit_price_micro_lamports,
        jito_tip_lamports: launch_tip_lamports,
        jito_tip_account: launch_tip_account,
    };

    let launch_lookup_table_variants =
        lookup_table_variants_for_transaction("launch", config, &lookup_tables);
    let mut launch_instructions = launch_pre_instructions;
    launch_instructions.extend(build_launch_instructions(
        config,
        mint,
        creator,
        launch_creator,
        agent_authority.as_ref(),
        global.as_ref(),
    )?);
    let launch_tx_instructions = with_tx_settings(
        launch_instructions,
        &launch_tx_config,
        &creator,
        config.execution.jitodontfront,
    )?;
    let launch_serialize_started = Instant::now();
    let (launch_compiled, launch_metrics) = compile_transaction_with_metrics(
        "launch",
        tx_format,
        &blockhash,
        last_valid_block_height,
        &creator_keypair,
        Some(&mint_keypair),
        launch_tx_instructions.clone(),
        &launch_tx_config,
        &launch_lookup_table_variants,
    )?;
    compile_timings.tx_serialize_ms += launch_serialize_started.elapsed().as_millis();
    compile_timings.launch_serialize_ms = Some(launch_serialize_started.elapsed().as_millis());
    let mut launch_metrics = launch_metrics;
    launch_metrics.warnings.extend(transaction_size_diagnostics(
        &launch_tx_instructions,
        &launch_tx_config,
    ));
    let mut compiled_transactions = vec![launch_compiled.clone()];
    let mut creation_transactions = vec![launch_compiled];
    let mut deferred_setup_transactions = vec![];
    let mut compile_metrics = vec![launch_metrics];
    let mut instruction_summaries = vec![summarize_instructions(&launch_tx_instructions)];

    if let Some(follow_up_label) = native_follow_up_label(config) {
        let follow_up_lookup_table_variants =
            lookup_table_variants_for_transaction(follow_up_label, config, &lookup_tables);
        let follow_up_prep_started = Instant::now();
        let follow_up_instructions = build_native_follow_up_instructions(
            rpc_url,
            config,
            mint,
            creator,
            agent_authority.as_ref(),
        )
        .await?;
        compile_timings.follow_up_prep_ms = Some(follow_up_prep_started.elapsed().as_millis());
        let follow_up_tx_instructions = with_tx_settings(
            follow_up_instructions,
            &NativeTxConfig {
                compute_unit_limit: configured_follow_up_compute_unit_limit(
                    config,
                    follow_up_label,
                )?,
                compute_unit_price_micro_lamports:
                    effective_follow_up_compute_unit_price_micro_lamports(config, transport_plan),
                jito_tip_lamports: follow_up_tip_lamports,
                jito_tip_account: follow_up_tip_account.clone(),
            },
            &creator,
            config.execution.jitodontfront,
        )?;
        let follow_up_serialize_started = Instant::now();
        let (follow_up_compiled, follow_up_metrics) = compile_transaction_with_metrics(
            follow_up_label,
            tx_format,
            &blockhash,
            last_valid_block_height,
            &creator_keypair,
            None,
            follow_up_tx_instructions.clone(),
            &NativeTxConfig {
                compute_unit_limit: configured_follow_up_compute_unit_limit(
                    config,
                    follow_up_label,
                )?,
                compute_unit_price_micro_lamports:
                    effective_follow_up_compute_unit_price_micro_lamports(config, transport_plan),
                jito_tip_lamports: follow_up_tip_lamports,
                jito_tip_account: follow_up_tip_account.clone(),
            },
            &follow_up_lookup_table_variants,
        )?;
        compile_timings.tx_serialize_ms += follow_up_serialize_started.elapsed().as_millis();
        compile_timings.follow_up_serialize_ms =
            Some(follow_up_serialize_started.elapsed().as_millis());
        let mut follow_up_metrics = follow_up_metrics;
        follow_up_metrics
            .warnings
            .extend(transaction_size_diagnostics(
                &follow_up_tx_instructions,
                &NativeTxConfig {
                    compute_unit_limit: configured_follow_up_compute_unit_limit(
                        config,
                        follow_up_label,
                    )?,
                    compute_unit_price_micro_lamports:
                        effective_follow_up_compute_unit_price_micro_lamports(
                            config,
                            transport_plan,
                        ),
                    jito_tip_lamports: follow_up_tip_lamports,
                    jito_tip_account: follow_up_tip_account.clone(),
                },
            ));
        deferred_setup_transactions.push(follow_up_compiled.clone());
        compiled_transactions.push(follow_up_compiled);
        compile_metrics.push(follow_up_metrics);
        instruction_summaries.push(summarize_instructions(&follow_up_tx_instructions));
    }

    if separate_tip_transaction {
        let tip_instruction = build_jito_tip_instruction(config, creator)?;
        let tip_tx_instructions = vec![tip_instruction];
        let tip_lookup_table_variants =
            lookup_table_variants_for_transaction("jito-tip", config, &lookup_tables);
        let tip_serialize_started = Instant::now();
        let (tip_compiled, tip_metrics) = compile_transaction_with_metrics(
            "jito-tip",
            tx_format,
            &blockhash,
            last_valid_block_height,
            &creator_keypair,
            None,
            tip_tx_instructions.clone(),
            &NativeTxConfig {
                compute_unit_limit: configured_launch_compute_unit_limit(config)?,
                compute_unit_price_micro_lamports: 0,
                jito_tip_lamports: config.tx.jitoTipLamports,
                jito_tip_account: config.tx.jitoTipAccount.clone(),
            },
            &tip_lookup_table_variants,
        )?;
        compile_timings.tx_serialize_ms += tip_serialize_started.elapsed().as_millis();
        compile_timings.tip_serialize_ms = Some(tip_serialize_started.elapsed().as_millis());
        let mut tip_metrics = tip_metrics;
        tip_metrics.warnings.extend(transaction_size_diagnostics(
            &tip_tx_instructions,
            &NativeTxConfig {
                compute_unit_limit: configured_launch_compute_unit_limit(config)?,
                compute_unit_price_micro_lamports: 0,
                jito_tip_lamports: config.tx.jitoTipLamports,
                jito_tip_account: config.tx.jitoTipAccount.clone(),
            },
        ));
        creation_transactions.push(tip_compiled.clone());
        compiled_transactions.push(tip_compiled);
        compile_metrics.push(tip_metrics);
        instruction_summaries.push(summarize_instructions(&tip_tx_instructions));
    }

    let mut report = build_report(
        config,
        transport_plan,
        built_at,
        rpc_url.to_string(),
        creator_public_key,
        mint.to_string(),
        agent_authority.map(|authority| authority.to_string()),
        config_path,
        lookup_tables
            .iter()
            .map(|table| table.key.to_string())
            .collect(),
    );
    if let Some(first_note) = report.execution.notes.first_mut() {
        *first_note =
            "Rust engine owns validation, runtime state, and API contracts. Native Pump assembly now covers LaunchDeck's Pump launch modes end-to-end; non-Pump flows still fall back to the JS compile bridge."
                .to_string();
    }
    report.execution.notes.push(
        "Native Pump assembly now includes compute-budget and priority-fee instructions for supported launch shapes."
            .to_string(),
    );
    apply_transaction_details(
        &mut report,
        &compiled_transactions,
        &instruction_summaries,
        &compile_metrics,
    )?;
    let text = render_report(&report);
    let report = serde_json::to_value(report).map_err(|error| error.to_string())?;

    Ok(Some(NativePumpArtifacts {
        compiled_transactions,
        creation_transactions,
        deferred_setup_transactions,
        report,
        text,
        compile_timings,
        mint: mint.to_string(),
        launch_creator: launch_creator.to_string(),
    }))
}

#[derive(Debug, Clone)]
struct NativeTxConfig {
    compute_unit_limit: u32,
    compute_unit_price_micro_lamports: i64,
    jito_tip_lamports: i64,
    jito_tip_account: String,
}

#[derive(Debug, Clone, Default)]
struct TransactionCompileMetrics {
    legacy_length: Option<usize>,
    v0_length: Option<usize>,
    v0_alt_length: Option<usize>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct CompiledTxCandidate {
    compiled: CompiledTransaction,
    serialized_len: usize,
}

fn create_v2_metadata_payload_bytes(instructions: &[Instruction]) -> Option<usize> {
    instructions.iter().find_map(|instruction| {
        if instruction.program_id.to_string() != PUMP_PROGRAM_ID {
            return None;
        }
        if instruction.data.len() < 8
            || instruction.data[..8] != [214, 144, 76, 236, 95, 139, 49, 180]
        {
            return None;
        }
        Some(instruction.data.len().saturating_sub(42))
    })
}

fn transaction_size_diagnostics(
    instructions: &[Instruction],
    tx_config: &NativeTxConfig,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(metadata_payload_bytes) = create_v2_metadata_payload_bytes(instructions) {
        warnings.push(format!(
            "CreateV2 metadata payload contributes {metadata_payload_bytes} bytes from name/symbol/uri."
        ));
    }
    if tx_config.compute_unit_price_micro_lamports > 0 {
        warnings.push(
            "Priority fee adds a ComputeBudget price instruction to this transaction.".to_string(),
        );
    }
    if tx_config.jito_tip_lamports > 0 {
        warnings.push("Inline tip adds a SystemProgram transfer instruction.".to_string());
    }
    warnings
}

fn effective_creation_compute_unit_price_micro_lamports(
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
    has_follow_up_transaction: bool,
) -> i64 {
    if transport_plan.transportType == "jito-bundle" && has_follow_up_transaction {
        return 0;
    }
    config.tx.computeUnitPriceMicroLamports.unwrap_or(0)
}

fn effective_follow_up_compute_unit_price_micro_lamports(
    config: &NormalizedConfig,
    transport_plan: &TransportPlan,
) -> i64 {
    if transport_plan.transportType == "jito-bundle" {
        return 0;
    }
    config.tx.computeUnitPriceMicroLamports.unwrap_or(0)
}

fn u64_to_u32_limit(value: u64, label: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("{label} is too large for a u32 compute unit limit."))
}

fn configured_launch_compute_unit_limit(config: &NormalizedConfig) -> Result<u32, String> {
    let limit = config
        .tx
        .computeUnitLimit
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or_else(configured_default_launch_compute_unit_limit);
    u64_to_u32_limit(limit, "launch compute unit limit")
}

fn configured_follow_up_compute_unit_limit(
    config: &NormalizedConfig,
    follow_up_label: &str,
) -> Result<u32, String> {
    let limit = config
        .tx
        .computeUnitLimit
        .and_then(|value| u64::try_from(value).ok())
        .unwrap_or_else(|| match follow_up_label {
            "agent-setup" => configured_default_agent_setup_compute_unit_limit(),
            _ => configured_default_follow_up_compute_unit_limit(),
        });
    u64_to_u32_limit(limit, "follow-up compute unit limit")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeTxFormat {
    Legacy,
    Auto,
    V0,
    V0Alt,
}

#[derive(Debug, Clone)]
struct PumpGlobalState {
    fee_recipient: Pubkey,
    initial_virtual_token_reserves: u64,
    initial_virtual_sol_reserves: u64,
    initial_real_token_reserves: u64,
    fee_basis_points: u64,
    creator_fee_basis_points: u64,
    fee_recipients: [Pubkey; 7],
    reserved_fee_recipient: Pubkey,
    reserved_fee_recipients: [Pubkey; 7],
}

#[derive(Debug, Clone)]
struct PumpBondingCurveState {
    virtual_token_reserves: u64,
    virtual_sol_reserves: u64,
    real_token_reserves: u64,
    real_sol_reserves: u64,
    token_total_supply: u64,
    complete: bool,
    creator: Pubkey,
    cashback_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PreparedFollowBuyStatic {
    user: Pubkey,
    mint: Pubkey,
    launch_creator: Pubkey,
    sol_amount: u64,
    tx_config: NativeTxConfig,
    tx_format: NativeTxFormat,
}

#[derive(Debug, Clone)]
pub struct PreparedFollowBuyRuntime {
    creator_vault_authority: Pubkey,
    global: PumpGlobalState,
    curve: PumpBondingCurveState,
}

fn global_state_cache() -> &'static Mutex<Option<PumpGlobalState>> {
    static CACHE: OnceLock<Mutex<Option<PumpGlobalState>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

fn get_cached_global_state() -> Option<PumpGlobalState> {
    global_state_cache().lock().ok()?.clone()
}

fn cache_global_state(state: &PumpGlobalState) {
    if let Ok(mut cache) = global_state_cache().lock() {
        *cache = Some(state.clone());
    }
}

fn keypair_from_secret_bytes(bytes: &[u8]) -> Result<Keypair, String> {
    Keypair::try_from(bytes).map_err(|error| error.to_string())
}

fn resolve_mint_keypair(config: &NormalizedConfig) -> Result<Keypair, String> {
    let vanity_private_key = config.vanityPrivateKey.trim();
    if vanity_private_key.is_empty() {
        return Ok(Keypair::new());
    }
    let bytes = read_keypair_bytes(vanity_private_key)
        .map_err(|error| format!("Invalid vanity private key: {error}"))?;
    keypair_from_secret_bytes(&bytes)
        .map_err(|error| format!("Invalid vanity private key: {error}"))
}

fn parse_pubkey(value: &str, label: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(value).map_err(|error| format!("Invalid {label}: {error}"))
}

fn pump_program_id() -> Result<Pubkey, String> {
    parse_pubkey(PUMP_PROGRAM_ID, "PUMP program id")
}

fn mayhem_program_id() -> Result<Pubkey, String> {
    parse_pubkey(MAYHEM_PROGRAM_ID, "Mayhem program id")
}

fn pump_amm_program_id() -> Result<Pubkey, String> {
    parse_pubkey(PUMP_AMM_PROGRAM_ID, "PUMP AMM program id")
}

fn token_2022_program_id() -> Result<Pubkey, String> {
    parse_pubkey(TOKEN_2022_PROGRAM_ID, "Token 2022 program id")
}

fn token_program_id() -> Result<Pubkey, String> {
    parse_pubkey(TOKEN_PROGRAM_ID, "Token program id")
}

fn pump_fee_program_id() -> Result<Pubkey, String> {
    parse_pubkey(PUMP_FEE_PROGRAM_ID, "PUMP fee program id")
}

fn compute_budget_program_id() -> Result<Pubkey, String> {
    parse_pubkey(COMPUTE_BUDGET_PROGRAM_ID, "Compute Budget program id")
}

fn pump_agent_payments_program_id() -> Result<Pubkey, String> {
    parse_pubkey(
        PUMP_AGENT_PAYMENTS_PROGRAM_ID,
        "Pump Agent Payments program id",
    )
}

fn wsol_mint() -> Result<Pubkey, String> {
    parse_pubkey(WSOL_MINT, "WSOL mint")
}

fn event_authority_pda(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], program_id).0
}

fn bonding_curve_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"bonding-curve", mint.as_ref()], &pump_program_id()?).0)
}

pub fn pump_bonding_curve_address(mint: &str) -> Result<String, String> {
    let mint = parse_pubkey(mint, "mint")?;
    Ok(bonding_curve_pda(&mint)?.to_string())
}

fn mint_authority_pda() -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"mint-authority"], &pump_program_id()?).0)
}

fn global_pda() -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"global"], &pump_program_id()?).0)
}

fn global_params_pda() -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"global-params"], &mayhem_program_id()?).0)
}

fn global_volume_accumulator_pda() -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"global_volume_accumulator"], &pump_program_id()?).0)
}

fn user_volume_accumulator_pda(user: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(
        &[b"user_volume_accumulator", user.as_ref()],
        &pump_program_id()?,
    )
    .0)
}

fn creator_vault_pda(creator: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"creator-vault", creator.as_ref()], &pump_program_id()?).0)
}

fn bonding_curve_v2_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"bonding-curve-v2", mint.as_ref()], &pump_program_id()?).0)
}

fn fee_config_pda() -> Result<Pubkey, String> {
    let pump_program = pump_program_id()?;
    let fee_program = pump_fee_program_id()?;
    Ok(Pubkey::find_program_address(&[b"fee_config", pump_program.as_ref()], &fee_program).0)
}

fn fee_sharing_config_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(
        Pubkey::find_program_address(&[b"sharing-config", mint.as_ref()], &pump_fee_program_id()?)
            .0,
    )
}

async fn resolve_follow_creator_vault_authority(
    rpc_url: &str,
    mint: &Pubkey,
    launch_creator: &Pubkey,
    prefer_post_setup_creator_vault: bool,
) -> Result<Pubkey, String> {
    let sharing_config = fee_sharing_config_pda(mint)?;
    Ok(select_follow_creator_vault_authority(
        launch_creator,
        &sharing_config,
        prefer_post_setup_creator_vault,
        fetch_account_exists(rpc_url, &sharing_config.to_string(), "confirmed").await?,
    ))
}

fn select_follow_creator_vault_authority(
    launch_creator: &Pubkey,
    sharing_config: &Pubkey,
    prefer_post_setup_creator_vault: bool,
    sharing_config_exists: bool,
) -> Pubkey {
    if sharing_config_exists {
        return *sharing_config;
    }
    let _ = prefer_post_setup_creator_vault;
    // Same-window sells can execute before deferred fee-sharing setup, so the live account state
    // decides which creator-vault seed path is valid.
    *launch_creator
}

fn fee_program_global_pda() -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"fee-program-global"], &pump_fee_program_id()?).0)
}

fn agent_global_config_pda() -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"global-config"], &pump_agent_payments_program_id()?).0)
}

fn token_agent_payments_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(
        &[b"token-agent-payments", mint.as_ref()],
        &pump_agent_payments_program_id()?,
    )
    .0)
}

fn coin_creator_vault_authority_pda(coin_creator: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(
        &[b"creator_vault", coin_creator.as_ref()],
        &pump_amm_program_id()?,
    )
    .0)
}

fn coin_creator_vault_ata_pda(coin_creator_vault_authority: &Pubkey) -> Result<Pubkey, String> {
    Ok(get_associated_token_address_with_program_id(
        coin_creator_vault_authority,
        &wsol_mint()?,
        &token_program_id()?,
    ))
}

fn social_fee_pda(user_id: &str, platform: u8) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(
        &[b"social-fee-pda", user_id.as_bytes(), &[platform]],
        &pump_fee_program_id()?,
    )
    .0)
}

async fn fetch_global_state(rpc_url: &str) -> Result<PumpGlobalState, String> {
    let account_data = fetch_account_data(rpc_url, &global_pda()?.to_string(), "confirmed").await?;
    decode_global_state(&account_data)
}

async fn fetch_global_state_cached(rpc_url: &str) -> Result<PumpGlobalState, String> {
    if let Some(global) = get_cached_global_state() {
        return Ok(global);
    }
    let global = fetch_global_state(rpc_url).await?;
    cache_global_state(&global);
    Ok(global)
}

#[allow(dead_code)]
pub async fn warm_pump_global_state(rpc_url: &str) -> Result<(), String> {
    let global = fetch_global_state(rpc_url).await?;
    cache_global_state(&global);
    Ok(())
}

fn read_bool(data: &[u8], offset: &mut usize) -> Result<bool, String> {
    let Some(byte) = data.get(*offset) else {
        return Err("Unexpected end of global account while reading bool.".to_string());
    };
    *offset += 1;
    Ok(*byte != 0)
}

fn read_u64(data: &[u8], offset: &mut usize) -> Result<u64, String> {
    let end = offset.saturating_add(8);
    let bytes: [u8; 8] = data
        .get(*offset..end)
        .ok_or_else(|| "Unexpected end of global account while reading u64.".to_string())?
        .try_into()
        .map_err(|_| "Failed to read u64 bytes from global account.".to_string())?;
    *offset = end;
    Ok(u64::from_le_bytes(bytes))
}

fn read_pubkey(data: &[u8], offset: &mut usize) -> Result<Pubkey, String> {
    let end = offset.saturating_add(32);
    let bytes: [u8; 32] = data
        .get(*offset..end)
        .ok_or_else(|| "Unexpected end of global account while reading pubkey.".to_string())?
        .try_into()
        .map_err(|_| "Failed to read pubkey bytes from global account.".to_string())?;
    *offset = end;
    Ok(Pubkey::new_from_array(bytes))
}

fn read_pubkey_array<const N: usize>(
    data: &[u8],
    offset: &mut usize,
) -> Result<[Pubkey; N], String> {
    let mut values = Vec::with_capacity(N);
    for _ in 0..N {
        values.push(read_pubkey(data, offset)?);
    }
    values
        .try_into()
        .map_err(|_| "Failed to decode pubkey array from global account.".to_string())
}

fn decode_global_state(data: &[u8]) -> Result<PumpGlobalState, String> {
    if data.len() < GLOBAL_ACCOUNT_DISCRIMINATOR_BYTES {
        return Err("Global account data was too short.".to_string());
    }
    let mut offset = GLOBAL_ACCOUNT_DISCRIMINATOR_BYTES;
    let _initialized = read_bool(data, &mut offset)?;
    let _authority = read_pubkey(data, &mut offset)?;
    let fee_recipient = read_pubkey(data, &mut offset)?;
    let initial_virtual_token_reserves = read_u64(data, &mut offset)?;
    let initial_virtual_sol_reserves = read_u64(data, &mut offset)?;
    let initial_real_token_reserves = read_u64(data, &mut offset)?;
    let _token_total_supply = read_u64(data, &mut offset)?;
    let fee_basis_points = read_u64(data, &mut offset)?;
    let _withdraw_authority = read_pubkey(data, &mut offset)?;
    let _enable_migrate = read_bool(data, &mut offset)?;
    let _pool_migration_fee = read_u64(data, &mut offset)?;
    let creator_fee_basis_points = read_u64(data, &mut offset)?;
    let fee_recipients = read_pubkey_array::<7>(data, &mut offset)?;
    let _set_creator_authority = read_pubkey(data, &mut offset)?;
    let _admin_set_creator_authority = read_pubkey(data, &mut offset)?;
    let _create_v2_enabled = read_bool(data, &mut offset)?;
    let _whitelist_pda = read_pubkey(data, &mut offset)?;
    let reserved_fee_recipient = read_pubkey(data, &mut offset)?;
    let _mayhem_mode_enabled = read_bool(data, &mut offset)?;
    let reserved_fee_recipients = read_pubkey_array::<7>(data, &mut offset)?;
    let _is_cashback_enabled = read_bool(data, &mut offset)?;

    Ok(PumpGlobalState {
        fee_recipient,
        initial_virtual_token_reserves,
        initial_virtual_sol_reserves,
        initial_real_token_reserves,
        fee_basis_points,
        creator_fee_basis_points,
        fee_recipients,
        reserved_fee_recipient,
        reserved_fee_recipients,
    })
}

fn parse_bonding_curve_state(data: &[u8]) -> Result<PumpBondingCurveState, String> {
    let mut offset = 8usize;
    let virtual_token_reserves = read_u64(data, &mut offset)?;
    let virtual_sol_reserves = read_u64(data, &mut offset)?;
    let real_token_reserves = read_u64(data, &mut offset)?;
    let real_sol_reserves = read_u64(data, &mut offset)?;
    let token_total_supply = read_u64(data, &mut offset)?;
    let complete = read_bool(data, &mut offset)?;
    let creator = read_pubkey(data, &mut offset)?;
    let cashback_enabled = data.len() > 82 && data[82] != 0;
    Ok(PumpBondingCurveState {
        virtual_token_reserves,
        virtual_sol_reserves,
        real_token_reserves,
        real_sol_reserves,
        token_total_supply,
        complete,
        creator,
        cashback_enabled,
    })
}

async fn fetch_bonding_curve_state(
    rpc_url: &str,
    mint: &Pubkey,
) -> Result<PumpBondingCurveState, String> {
    let data =
        fetch_account_data(rpc_url, &bonding_curve_pda(mint)?.to_string(), "confirmed").await?;
    parse_bonding_curve_state(&data)
}

fn parse_decimal_u64(value: &str, decimals: u32, label: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} cannot be blank."));
    }
    if !trimmed
        .chars()
        .all(|char| char.is_ascii_digit() || char == '.')
    {
        return Err(format!(
            "{label} must be a positive decimal string. Got: {value}"
        ));
    }
    let mut parts = trimmed.split('.');
    let whole = parts.next().unwrap_or_default();
    let fractional = parts.next().unwrap_or_default();
    if parts.next().is_some() {
        return Err(format!(
            "{label} must be a positive decimal string. Got: {value}"
        ));
    }
    if fractional.len() > decimals as usize {
        return Err(format!(
            "{label} supports at most {decimals} decimal places. Got: {value}"
        ));
    }
    let mut normalized = String::new();
    normalized.push_str(whole);
    normalized.push_str(&format!("{fractional:0<width$}", width = decimals as usize));
    let digits = normalized.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(0);
    }
    digits.parse::<u64>().map_err(|error| error.to_string())
}

fn ceil_div(numerator: u128, denominator: u128) -> u128 {
    numerator.div_ceil(denominator)
}

fn compute_total_fee_basis_points(global: &PumpGlobalState, include_creator_fee: bool) -> u128 {
    u128::from(global.fee_basis_points)
        + if include_creator_fee {
            u128::from(global.creator_fee_basis_points)
        } else {
            0
        }
}

#[allow(dead_code)]
fn format_decimal_u128(value: u128, decimals: u32, max_fraction_digits: u32) -> String {
    let base = 10u128.pow(decimals);
    let whole = value / base;
    let fraction = value % base;
    if fraction == 0 {
        return whole.to_string();
    }
    let width = decimals as usize;
    let mut fraction_text = format!("{fraction:0width$}");
    fraction_text.truncate(max_fraction_digits.min(decimals) as usize);
    while fraction_text.ends_with('0') {
        fraction_text.pop();
    }
    if fraction_text.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{fraction_text}")
    }
}

#[allow(dead_code)]
fn format_supply_percent(raw_token_amount: u64) -> String {
    if raw_token_amount == 0 {
        return "0".to_string();
    }
    const TOTAL_SUPPLY_RAW: u128 = 1_000_000_000u128 * 1_000_000u128;
    let scaled = (u128::from(raw_token_amount) * 1_000_000u128) / TOTAL_SUPPLY_RAW;
    format_decimal_u128(scaled, 4, 4)
}

fn quote_buy_tokens_from_sol(global: &PumpGlobalState, spendable_sol: u64) -> u64 {
    if spendable_sol == 0 {
        return 0;
    }
    let total_fee_basis_points = compute_total_fee_basis_points(global, true);
    let input_amount = ((u128::from(spendable_sol).saturating_sub(1)) * 10_000)
        / (10_000 + total_fee_basis_points);
    if input_amount == 0 {
        return 0;
    }
    let virtual_token_reserves = u128::from(global.initial_virtual_token_reserves);
    let virtual_sol_reserves = u128::from(global.initial_virtual_sol_reserves);
    let tokens = (input_amount * virtual_token_reserves) / (virtual_sol_reserves + input_amount);
    tokens
        .min(u128::from(global.initial_real_token_reserves))
        .try_into()
        .unwrap_or(u64::MAX)
}

fn quote_buy_tokens_from_curve(
    curve: &PumpBondingCurveState,
    global: &PumpGlobalState,
    spendable_sol: u64,
) -> u64 {
    if spendable_sol == 0 {
        return 0;
    }
    let total_fee_basis_points = compute_total_fee_basis_points(global, true);
    let input_amount = ((u128::from(spendable_sol).saturating_sub(1)) * 10_000)
        / (10_000 + total_fee_basis_points);
    if input_amount == 0 {
        return 0;
    }
    let virtual_token_reserves = u128::from(curve.virtual_token_reserves);
    let virtual_sol_reserves = u128::from(curve.virtual_sol_reserves);
    let tokens = (input_amount * virtual_token_reserves) / (virtual_sol_reserves + input_amount);
    tokens
        .min(u128::from(curve.real_token_reserves))
        .try_into()
        .unwrap_or(u64::MAX)
}

fn quote_buy_sol_from_tokens(global: &PumpGlobalState, token_amount: u64) -> u64 {
    if token_amount == 0 {
        return 0;
    }
    let amount = u128::from(token_amount).min(u128::from(global.initial_real_token_reserves));
    let virtual_token_reserves = u128::from(global.initial_virtual_token_reserves);
    let virtual_sol_reserves = u128::from(global.initial_virtual_sol_reserves);
    let sol_cost = ((amount * virtual_sol_reserves) / (virtual_token_reserves - amount)) + 1;
    let protocol_fee = ceil_div(sol_cost * u128::from(global.fee_basis_points), 10_000);
    let creator_fee = ceil_div(
        sol_cost * u128::from(global.creator_fee_basis_points),
        10_000,
    );
    (sol_cost + protocol_fee + creator_fee)
        .min(u128::from(u64::MAX))
        .try_into()
        .unwrap_or(u64::MAX)
}

fn quote_sell_sol_from_curve(
    curve: &PumpBondingCurveState,
    global: &PumpGlobalState,
    token_amount: u64,
) -> u64 {
    if token_amount == 0 {
        return 0;
    }
    let gross_output = (u128::from(token_amount) * u128::from(curve.virtual_sol_reserves))
        / (u128::from(curve.virtual_token_reserves) + u128::from(token_amount));
    let protocol_fee = ceil_div(gross_output * u128::from(global.fee_basis_points), 10_000);
    let creator_fee = ceil_div(
        gross_output * u128::from(global.creator_fee_basis_points),
        10_000,
    );
    gross_output
        .saturating_sub(protocol_fee)
        .saturating_sub(creator_fee)
        .min(u128::from(curve.real_sol_reserves))
        .try_into()
        .unwrap_or_default()
}

fn quote_buy_curve_input_from_tokens(global: &PumpGlobalState, token_amount: u64) -> u64 {
    if token_amount == 0 {
        return 0;
    }
    let amount = u128::from(token_amount).min(u128::from(global.initial_real_token_reserves));
    let virtual_token_reserves = u128::from(global.initial_virtual_token_reserves);
    let virtual_sol_reserves = u128::from(global.initial_virtual_sol_reserves);
    (((amount * virtual_sol_reserves) / (virtual_token_reserves.saturating_sub(amount))) + 1)
        .min(u128::from(u64::MAX))
        .try_into()
        .unwrap_or(u64::MAX)
}

fn current_market_cap_lamports(curve: &PumpBondingCurveState) -> u64 {
    if curve.virtual_token_reserves == 0 {
        return 0;
    }
    ((u128::from(curve.token_total_supply) * u128::from(curve.virtual_sol_reserves))
        / u128::from(curve.virtual_token_reserves))
    .min(u128::from(u64::MAX))
    .try_into()
    .unwrap_or(u64::MAX)
}

fn resolve_dev_buy_quote(
    config: &NormalizedConfig,
    global: &PumpGlobalState,
) -> Result<Option<(u64, u64)>, String> {
    let Some(dev_buy) = &config.devBuy else {
        return Ok(None);
    };
    if dev_buy.mode == "sol" {
        let sol_amount = parse_decimal_u64(&dev_buy.amount, 9, "devBuy.amount")?;
        return Ok(Some((
            sol_amount,
            quote_buy_tokens_from_sol(global, sol_amount),
        )));
    }
    if dev_buy.mode == "tokens" {
        let token_amount = parse_decimal_u64(&dev_buy.amount, TOKEN_DECIMALS, "devBuy.amount")?;
        return Ok(Some((
            quote_buy_sol_from_tokens(global, token_amount),
            token_amount,
        )));
    }
    Err(format!(
        "Unsupported devBuy.mode for native Pump compile: {}",
        dev_buy.mode
    ))
}

pub async fn predict_dev_buy_token_amount(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<Option<u64>, String> {
    let global = fetch_global_state_cached(rpc_url).await?;
    Ok(resolve_dev_buy_quote(config, &global)?.map(|(_, token_amount)| token_amount))
}

#[allow(dead_code)]
pub async fn quote_launch(
    rpc_url: &str,
    mode: &str,
    amount: &str,
) -> Result<Option<LaunchQuote>, String> {
    let trimmed_mode = mode.trim().to_lowercase();
    let trimmed_amount = amount.trim();
    if trimmed_mode.is_empty() || trimmed_amount.is_empty() {
        return Ok(None);
    }
    if trimmed_mode != "sol" && trimmed_mode != "tokens" {
        return Err(format!(
            "Unsupported dev buy quote mode: {mode}. Expected sol or tokens."
        ));
    }
    let global = fetch_global_state_cached(rpc_url).await?;
    if trimmed_mode == "sol" {
        let spendable_sol = parse_decimal_u64(trimmed_amount, 9, "buy amount")?;
        if spendable_sol == 0 {
            return Ok(None);
        }
        let tokens_out = quote_buy_tokens_from_sol(&global, spendable_sol);
        return Ok(Some(LaunchQuote {
            mode: trimmed_mode,
            input: trimmed_amount.to_string(),
            estimatedTokens: format_decimal_u128(u128::from(tokens_out), TOKEN_DECIMALS, 6),
            estimatedSol: format_decimal_u128(u128::from(spendable_sol), 9, 6),
            estimatedQuoteAmount: format_decimal_u128(u128::from(spendable_sol), 9, 6),
            quoteAsset: "sol".to_string(),
            quoteAssetLabel: "SOL".to_string(),
            estimatedSupplyPercent: format_supply_percent(tokens_out),
        }));
    }

    let token_amount = parse_decimal_u64(trimmed_amount, TOKEN_DECIMALS, "buy amount")?;
    if token_amount == 0 || token_amount >= global.initial_virtual_token_reserves {
        return Ok(None);
    }
    Ok(Some(LaunchQuote {
        mode: trimmed_mode,
        input: trimmed_amount.to_string(),
        estimatedTokens: format_decimal_u128(u128::from(token_amount), TOKEN_DECIMALS, 6),
        estimatedSol: format_decimal_u128(
            u128::from(quote_buy_sol_from_tokens(&global, token_amount)),
            9,
            6,
        ),
        estimatedQuoteAmount: format_decimal_u128(
            u128::from(quote_buy_sol_from_tokens(&global, token_amount)),
            9,
            6,
        ),
        quoteAsset: "sol".to_string(),
        quoteAssetLabel: "SOL".to_string(),
        estimatedSupplyPercent: format_supply_percent(token_amount),
    }))
}

pub async fn fetch_pump_market_snapshot(
    rpc_url: &str,
    mint: &str,
) -> Result<PumpMarketSnapshot, String> {
    let mint = parse_pubkey(mint, "mint")?;
    let curve = fetch_bonding_curve_state(rpc_url, &mint).await?;
    let market_cap_lamports = current_market_cap_lamports(&curve);
    Ok(PumpMarketSnapshot {
        mint: mint.to_string(),
        creator: curve.creator.to_string(),
        virtualTokenReserves: curve.virtual_token_reserves,
        virtualSolReserves: curve.virtual_sol_reserves,
        realTokenReserves: curve.real_token_reserves,
        realSolReserves: curve.real_sol_reserves,
        tokenTotalSupply: curve.token_total_supply,
        complete: curve.complete,
        marketCapLamports: market_cap_lamports,
        marketCapSol: format_decimal_u128(u128::from(market_cap_lamports), 9, 6),
    })
}

fn priority_fee_sol_to_micro_lamports(priority_fee_sol: &str) -> Result<u64, String> {
    let lamports = parse_decimal_u64(priority_fee_sol, 9, "priority fee")?;
    if lamports == 0 {
        Ok(0)
    } else {
        Ok((lamports.saturating_mul(1_000_000)) / PRIORITY_FEE_PRICE_BASE_COMPUTE_UNIT_LIMIT)
    }
}

fn slippage_bps_from_percent(slippage_percent: &str) -> Result<u64, String> {
    let percent = parse_decimal_u64(slippage_percent, 2, "slippage percent")?;
    Ok(percent.min(10_000))
}

pub async fn compile_follow_buy_transaction(
    rpc_url: &str,
    execution: &NormalizedExecution,
    token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    buy_amount_sol: &str,
    prefer_post_setup_creator_vault: bool,
) -> Result<CompiledTransaction, String> {
    let prepared = prepare_follow_buy_static(
        rpc_url,
        execution,
        jito_tip_account,
        wallet_secret,
        mint,
        launch_creator,
        buy_amount_sol,
    )
    .await?;
    let runtime = prepare_follow_buy_runtime(
        rpc_url,
        mint,
        launch_creator,
        prefer_post_setup_creator_vault,
    )
    .await?;
    finalize_follow_buy_transaction(
        rpc_url,
        execution,
        token_mayhem_mode,
        wallet_secret,
        &prepared,
        &runtime,
    )
    .await
}

fn provider_uses_follow_tip(provider: &str) -> bool {
    matches!(
        provider.trim().to_ascii_lowercase().as_str(),
        "helius-sender" | "hellomoon" | "jito-bundle"
    )
}

const HELLOMOON_MIN_INLINE_TIP_LAMPORTS: i64 = 1_000_000;

fn resolve_follow_tip_config(
    provider: &str,
    tip_sol: &str,
    jito_tip_account: &str,
    label: &str,
) -> Result<(i64, String), String> {
    if !provider_uses_follow_tip(provider) {
        return Ok((0, String::new()));
    }
    let tip_lamports = parse_decimal_u64(tip_sol, 9, label)? as i64;
    if provider.trim().eq_ignore_ascii_case("hellomoon")
        && tip_lamports < HELLOMOON_MIN_INLINE_TIP_LAMPORTS
    {
        return Err(format!(
            "{label} must be at least 0.001 SOL when using Hello Moon for follow / snipe / auto-sell."
        ));
    }
    let tip_account = if tip_sol.trim().is_empty() {
        String::new()
    } else {
        jito_tip_account.to_string()
    };
    Ok((tip_lamports, tip_account))
}

pub async fn prepare_follow_buy_static(
    rpc_url: &str,
    execution: &NormalizedExecution,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    buy_amount_sol: &str,
) -> Result<PreparedFollowBuyStatic, String> {
    let user_keypair = keypair_from_secret_bytes(wallet_secret)?;
    let user = user_keypair.pubkey();
    let mint = parse_pubkey(mint, "mint")?;
    let launch_creator = parse_pubkey(launch_creator, "launch creator")?;
    let sol_amount = parse_decimal_u64(buy_amount_sol, 9, "followLaunch.snipes.buyAmountSol")?;
    let (jito_tip_lamports, jito_tip_account) = resolve_follow_tip_config(
        &execution.buyProvider,
        &execution.buyTipSol,
        jito_tip_account,
        "buy tip",
    )?;
    let tx_config = NativeTxConfig {
        compute_unit_limit: u64_to_u32_limit(
            configured_default_sniper_buy_compute_unit_limit(),
            "sniper buy compute unit limit",
        )?,
        compute_unit_price_micro_lamports: priority_fee_sol_to_micro_lamports(
            &execution.buyPriorityFeeSol,
        )? as i64,
        jito_tip_lamports,
        jito_tip_account,
    };
    let tx_format = select_native_format(&execution.txFormat, false)?;
    let _ = rpc_url;
    Ok(PreparedFollowBuyStatic {
        user,
        mint,
        launch_creator,
        sol_amount,
        tx_config,
        tx_format,
    })
}

pub async fn prepare_follow_buy_runtime(
    rpc_url: &str,
    mint: &str,
    launch_creator: &str,
    prefer_post_setup_creator_vault: bool,
) -> Result<PreparedFollowBuyRuntime, String> {
    let mint = parse_pubkey(mint, "mint")?;
    let launch_creator = parse_pubkey(launch_creator, "launch creator")?;
    let creator_vault_authority = resolve_follow_creator_vault_authority(
        rpc_url,
        &mint,
        &launch_creator,
        prefer_post_setup_creator_vault,
    )
    .await?;
    let global = fetch_global_state_cached(rpc_url).await?;
    let curve = fetch_bonding_curve_state(rpc_url, &mint).await?;
    Ok(PreparedFollowBuyRuntime {
        creator_vault_authority,
        global,
        curve,
    })
}

pub async fn finalize_follow_buy_transaction(
    rpc_url: &str,
    execution: &NormalizedExecution,
    token_mayhem_mode: bool,
    wallet_secret: &[u8],
    prepared: &PreparedFollowBuyStatic,
    runtime: &PreparedFollowBuyRuntime,
) -> Result<CompiledTransaction, String> {
    let user_keypair = keypair_from_secret_bytes(wallet_secret)?;
    let user = user_keypair.pubkey();
    if user != prepared.user {
        return Err("Prepared follow buy no longer matches the active wallet secret.".to_string());
    }
    let token_amount =
        quote_buy_tokens_from_curve(&runtime.curve, &runtime.global, prepared.sol_amount);
    let (blockhash, last_valid_block_height) =
        fetch_latest_blockhash_cached(rpc_url, &execution.commitment).await?;
    let instructions = vec![
        build_create_token_ata_instruction(&user, &prepared.mint)?,
        build_buy_instruction(
            &runtime.global,
            &prepared.mint,
            &runtime.creator_vault_authority,
            &user,
            apply_atomic_buy_buffer(prepared.sol_amount),
            token_amount,
            token_mayhem_mode,
        )?,
    ];
    let tx_instructions = with_tx_settings(
        instructions,
        &prepared.tx_config,
        &user,
        execution.buyJitodontfront,
    )?;
    let (compiled, _) = compile_transaction_with_metrics(
        "follow-buy",
        prepared.tx_format,
        &blockhash,
        last_valid_block_height,
        &user_keypair,
        None,
        tx_instructions,
        &prepared.tx_config,
        &[vec![]],
    )?;
    Ok(compiled)
}

pub async fn compile_atomic_follow_buy_transaction(
    rpc_url: &str,
    execution: &NormalizedExecution,
    token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    buy_amount_sol: &str,
) -> Result<CompiledTransaction, String> {
    let user_keypair = keypair_from_secret_bytes(wallet_secret)?;
    let user = user_keypair.pubkey();
    let mint = parse_pubkey(mint, "mint")?;
    let launch_creator = parse_pubkey(launch_creator, "launch creator")?;
    let global = fetch_global_state_cached(rpc_url).await?;
    let (blockhash, last_valid_block_height) =
        fetch_latest_blockhash_cached(rpc_url, &execution.commitment).await?;
    let sol_amount = parse_decimal_u64(buy_amount_sol, 9, "followLaunch.snipes.buyAmountSol")?;
    let tokens_out = quote_buy_tokens_from_sol(&global, sol_amount);
    let instructions = vec![
        build_create_token_ata_instruction(&user, &mint)?,
        build_buy_instruction(
            &global,
            &mint,
            &launch_creator,
            &user,
            apply_atomic_buy_buffer(sol_amount),
            tokens_out,
            token_mayhem_mode,
        )?,
    ];
    let (jito_tip_lamports, jito_tip_account) = resolve_follow_tip_config(
        &execution.buyProvider,
        &execution.buyTipSol,
        jito_tip_account,
        "buy tip",
    )?;
    let tx_config = NativeTxConfig {
        compute_unit_limit: u64_to_u32_limit(
            configured_default_sniper_buy_compute_unit_limit(),
            "sniper buy compute unit limit",
        )?,
        compute_unit_price_micro_lamports: priority_fee_sol_to_micro_lamports(
            &execution.buyPriorityFeeSol,
        )? as i64,
        jito_tip_lamports,
        jito_tip_account,
    };
    let tx_instructions =
        with_tx_settings(instructions, &tx_config, &user, execution.buyJitodontfront)?;
    let tx_format = select_native_format(&execution.txFormat, false)?;
    let (compiled, _) = compile_transaction_with_metrics(
        "follow-buy-atomic",
        tx_format,
        &blockhash,
        last_valid_block_height,
        &user_keypair,
        None,
        tx_instructions,
        &tx_config,
        &[vec![]],
    )?;
    Ok(compiled)
}

pub async fn compile_follow_sell_transaction(
    rpc_url: &str,
    execution: &NormalizedExecution,
    token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    sell_percent: u8,
    prefer_post_setup_creator_vault: bool,
) -> Result<Option<CompiledTransaction>, String> {
    compile_follow_sell_transaction_with_token_amount(
        rpc_url,
        execution,
        token_mayhem_mode,
        jito_tip_account,
        wallet_secret,
        mint,
        launch_creator,
        sell_percent,
        prefer_post_setup_creator_vault,
        None,
        None,
    )
    .await
}

pub async fn compile_follow_sell_transaction_with_token_amount(
    rpc_url: &str,
    execution: &NormalizedExecution,
    token_mayhem_mode: bool,
    jito_tip_account: &str,
    wallet_secret: &[u8],
    mint: &str,
    launch_creator: &str,
    sell_percent: u8,
    prefer_post_setup_creator_vault: bool,
    token_amount_override: Option<u64>,
    cashback_enabled_override: Option<bool>,
) -> Result<Option<CompiledTransaction>, String> {
    let user_keypair = keypair_from_secret_bytes(wallet_secret)?;
    let user = user_keypair.pubkey();
    let mint = parse_pubkey(mint, "mint")?;
    let launch_creator = parse_pubkey(launch_creator, "launch creator")?;
    let creator_vault_authority = resolve_follow_creator_vault_authority(
        rpc_url,
        &mint,
        &launch_creator,
        prefer_post_setup_creator_vault,
    )
    .await?;
    let global = fetch_global_state_cached(rpc_url).await?;
    let curve = if let Some(token_amount_override) = token_amount_override {
        let curve_input = quote_buy_curve_input_from_tokens(&global, token_amount_override);
        PumpBondingCurveState {
            virtual_token_reserves: global
                .initial_virtual_token_reserves
                .saturating_sub(token_amount_override),
            virtual_sol_reserves: global
                .initial_virtual_sol_reserves
                .saturating_add(curve_input),
            real_token_reserves: global
                .initial_real_token_reserves
                .saturating_sub(token_amount_override),
            real_sol_reserves: curve_input,
            token_total_supply: global.initial_real_token_reserves,
            complete: false,
            creator: launch_creator,
            cashback_enabled: cashback_enabled_override.unwrap_or(false),
        }
    } else {
        fetch_bonding_curve_state(rpc_url, &mint).await?
    };
    let token_amount = if let Some(token_amount_override) = token_amount_override {
        ((u128::from(token_amount_override) * u128::from(sell_percent)) / 100u128)
            .min(u128::from(u64::MAX)) as u64
    } else {
        let associated_user =
            get_associated_token_address_with_program_id(&user, &mint, &token_2022_program_id()?);
        let account_key = associated_user.to_string();
        let mut account_data = None;
        let mut last_error = None;
        for attempt in 0..15 {
            match fetch_account_data(rpc_url, &account_key, &execution.commitment).await {
                Ok(data) => {
                    account_data = Some(data);
                    last_error = None;
                    break;
                }
                Err(error) if error.contains("was not found") && attempt < 14 => {
                    last_error = Some(error);
                    sleep(Duration::from_millis(200)).await;
                }
                Err(error) => return Err(error),
            }
        }
        let account_data = account_data.ok_or_else(|| {
            last_error.unwrap_or_else(|| format!("Account {account_key} was not found."))
        })?;
        let token_balance = read_token_account_amount(&account_data)?;
        ((u128::from(token_balance) * u128::from(sell_percent)) / 100u128).min(u128::from(u64::MAX))
            as u64
    };
    if token_amount == 0 {
        return Ok(None);
    }
    let gross_quote = quote_sell_sol_from_curve(&curve, &global, token_amount);
    let slippage_bps = slippage_bps_from_percent(&execution.sellSlippagePercent)?;
    let min_sol_output =
        gross_quote.saturating_mul(10_000u64.saturating_sub(slippage_bps)) / 10_000;
    let (blockhash, last_valid_block_height) =
        fetch_latest_blockhash_cached(rpc_url, &execution.commitment).await?;
    let instructions = vec![build_sell_instruction(
        &global,
        &mint,
        &creator_vault_authority,
        &user,
        token_amount,
        min_sol_output,
        curve.cashback_enabled,
        token_mayhem_mode,
    )?];
    let (jito_tip_lamports, jito_tip_account) = resolve_follow_tip_config(
        &execution.sellProvider,
        &execution.sellTipSol,
        jito_tip_account,
        "sell tip",
    )?;
    let tx_config = NativeTxConfig {
        compute_unit_limit: u64_to_u32_limit(
            configured_default_dev_auto_sell_compute_unit_limit(),
            "dev auto sell compute unit limit",
        )?,
        compute_unit_price_micro_lamports: priority_fee_sol_to_micro_lamports(
            &execution.sellPriorityFeeSol,
        )? as i64,
        jito_tip_lamports,
        jito_tip_account,
    };
    let tx_instructions =
        with_tx_settings(instructions, &tx_config, &user, execution.sellJitodontfront)?;
    let tx_format = select_native_format(&execution.txFormat, false)?;
    let (compiled, _) = compile_transaction_with_metrics(
        "follow-sell",
        tx_format,
        &blockhash,
        last_valid_block_height,
        &user_keypair,
        None,
        tx_instructions,
        &tx_config,
        &[vec![]],
    )?;
    Ok(Some(compiled))
}

fn apply_atomic_buy_buffer(sol_amount: u64) -> u64 {
    ceil_div(u128::from(sol_amount) * 10_100, 10_000).min(u128::from(u64::MAX)) as u64
}

fn select_buy_fee_recipient(global: &PumpGlobalState, mayhem_mode: bool) -> Pubkey {
    if mayhem_mode {
        if global.reserved_fee_recipient != Pubkey::default() {
            return global.reserved_fee_recipient;
        }
        if let Some(entry) = global
            .reserved_fee_recipients
            .iter()
            .copied()
            .find(|entry| *entry != Pubkey::default())
        {
            return entry;
        }
    } else {
        if global.fee_recipient != Pubkey::default() {
            return global.fee_recipient;
        }
        if let Some(entry) = global
            .fee_recipients
            .iter()
            .copied()
            .find(|entry| *entry != Pubkey::default())
        {
            return entry;
        }
    }
    Pubkey::default()
}

fn sol_vault_pda() -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"sol-vault"], &mayhem_program_id()?).0)
}

fn mayhem_state_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"mayhem-state", mint.as_ref()], &mayhem_program_id()?).0)
}

async fn resolve_launch_creator_and_pre_instructions(
    rpc_url: &str,
    config: &NormalizedConfig,
    creator: &Pubkey,
) -> Result<(Pubkey, Vec<Instruction>), String> {
    if config.creatorFee.mode == "wallet" && !config.creatorFee.address.is_empty() {
        return Ok((
            parse_pubkey(&config.creatorFee.address, "creatorFee.address")?,
            vec![],
        ));
    }
    if config.creatorFee.mode == "github" && !config.creatorFee.githubUserId.is_empty() {
        let social_fee = social_fee_pda(&config.creatorFee.githubUserId, PLATFORM_GITHUB)?;
        let mut pre_instructions = Vec::new();
        if !fetch_account_exists(rpc_url, &social_fee.to_string(), "confirmed").await? {
            pre_instructions.push(build_create_social_fee_pda_instruction(
                creator,
                &config.creatorFee.githubUserId,
                PLATFORM_GITHUB,
            )?);
        }
        return Ok((social_fee, pre_instructions));
    }
    Ok((*creator, vec![]))
}

fn resolve_agent_authority(
    config: &NormalizedConfig,
    creator: &Pubkey,
) -> Result<Option<Pubkey>, String> {
    if config.mode == "regular" || config.mode == "cashback" {
        return Ok(None);
    }
    if config.agent.authority.trim().is_empty() {
        return Ok(Some(*creator));
    }
    Ok(Some(parse_pubkey(
        &config.agent.authority,
        "agent.authority",
    )?))
}

fn build_launch_instructions(
    config: &NormalizedConfig,
    mint: Pubkey,
    creator: Pubkey,
    launch_creator: Pubkey,
    agent_authority: Option<&Pubkey>,
    global: Option<&PumpGlobalState>,
) -> Result<Vec<Instruction>, String> {
    let mut instructions = vec![build_create_v2_instruction(
        &mint,
        &creator,
        &launch_creator,
        &config.token.name,
        &config.token.symbol,
        &config.token.uri,
        config.token.mayhemMode,
        config.mode == "cashback",
    )?];
    if let Some(global) = global {
        if let Some((sol_amount, token_amount)) = resolve_dev_buy_quote(config, global)? {
            instructions.push(build_extend_account_instruction(
                &bonding_curve_pda(&mint)?,
                &creator,
            )?);
            instructions.push(build_create_token_ata_instruction(&creator, &mint)?);
            instructions.push(build_buy_instruction(
                global,
                &mint,
                &launch_creator,
                &creator,
                apply_atomic_buy_buffer(sol_amount),
                token_amount,
                config.token.mayhemMode,
            )?);
        }
    }
    if config.mode == "agent-unlocked"
        || config.mode == "agent-locked"
        || config.mode == "agent-custom"
    {
        let authority = agent_authority
            .ok_or_else(|| format!("agent authority is required for {} mode.", config.mode))?;
        instructions.push(build_agent_initialize_instruction(
            &mint,
            &creator,
            authority,
            config.agent.buybackBps.unwrap_or(0) as u16,
        )?);
    }
    Ok(instructions)
}

fn encode_borsh_string(buffer: &mut Vec<u8>, value: &str) {
    buffer.extend_from_slice(&(value.len() as u32).to_le_bytes());
    buffer.extend_from_slice(value.as_bytes());
}

fn build_create_v2_instruction(
    mint: &Pubkey,
    user: &Pubkey,
    creator: &Pubkey,
    name: &str,
    symbol: &str,
    uri: &str,
    mayhem_mode: bool,
    cashback_enabled: bool,
) -> Result<Instruction, String> {
    let pump_program = pump_program_id()?;
    let token_2022 = token_2022_program_id()?;
    let associated_bonding_curve =
        get_associated_token_address_with_program_id(&bonding_curve_pda(mint)?, mint, &token_2022);
    let sol_vault = sol_vault_pda()?;
    let mayhem_state = mayhem_state_pda(mint)?;
    let mayhem_token_vault =
        get_associated_token_address_with_program_id(&sol_vault, mint, &token_2022);

    let mut data = vec![214, 144, 76, 236, 95, 139, 49, 180];
    encode_borsh_string(&mut data, name);
    encode_borsh_string(&mut data, symbol);
    encode_borsh_string(&mut data, uri);
    data.extend_from_slice(creator.as_ref());
    data.push(u8::from(mayhem_mode));
    data.push(u8::from(cashback_enabled));

    let instruction = Instruction {
        program_id: pump_program,
        accounts: vec![
            AccountMeta::new(*mint, true),
            AccountMeta::new_readonly(mint_authority_pda()?, false),
            AccountMeta::new(bonding_curve_pda(mint)?, false),
            AccountMeta::new(associated_bonding_curve, false),
            AccountMeta::new_readonly(global_pda()?, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(token_2022, false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(mayhem_program_id()?, false),
            AccountMeta::new_readonly(global_params_pda()?, false),
            AccountMeta::new(sol_vault, false),
            AccountMeta::new(mayhem_state, false),
            AccountMeta::new(mayhem_token_vault, false),
            AccountMeta::new_readonly(event_authority_pda(&pump_program), false),
            AccountMeta::new_readonly(pump_program, false),
        ],
        data,
    };
    Ok(instruction)
}

fn build_jito_tip_instruction(
    config: &NormalizedConfig,
    payer: Pubkey,
) -> Result<Instruction, String> {
    let tip_account = parse_pubkey(&config.tx.jitoTipAccount, "tx.jitoTipAccount")?;
    Ok(transfer(
        &payer,
        &tip_account,
        config.tx.jitoTipLamports as u64,
    ))
}

fn build_agent_initialize_instruction(
    mint: &Pubkey,
    creator: &Pubkey,
    agent_authority: &Pubkey,
    buyback_bps: u16,
) -> Result<Instruction, String> {
    let program_id = pump_agent_payments_program_id()?;
    let mut data = vec![180, 248, 163, 8, 49, 94, 126, 96];
    data.extend_from_slice(agent_authority.as_ref());
    data.extend_from_slice(&buyback_bps.to_le_bytes());
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(*creator, true),
            AccountMeta::new_readonly(bonding_curve_pda(mint)?, false),
            AccountMeta::new(agent_global_config_pda()?, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(token_agent_payments_pda(mint)?, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(event_authority_pda(&program_id), false),
            AccountMeta::new_readonly(program_id, false),
        ],
        data,
    };
    Ok(instruction)
}

fn build_extend_account_instruction(
    account: &Pubkey,
    user: &Pubkey,
) -> Result<Instruction, String> {
    let program_id = pump_program_id()?;
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(*account, false),
            AccountMeta::new_readonly(*user, true),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(event_authority_pda(&program_id), false),
            AccountMeta::new_readonly(program_id, false),
        ],
        data: vec![234, 102, 194, 203, 150, 72, 62, 229],
    })
}

fn build_create_token_ata_instruction(
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<Instruction, String> {
    Ok(
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            owner,
            owner,
            mint,
            &token_2022_program_id()?,
        ),
    )
}

fn build_buy_instruction(
    global: &PumpGlobalState,
    mint: &Pubkey,
    launch_creator: &Pubkey,
    user: &Pubkey,
    sol_amount: u64,
    token_amount: u64,
    mayhem_mode: bool,
) -> Result<Instruction, String> {
    let pump_program = pump_program_id()?;
    let token_2022 = token_2022_program_id()?;
    let bonding_curve = bonding_curve_pda(mint)?;
    let associated_bonding_curve =
        get_associated_token_address_with_program_id(&bonding_curve, mint, &token_2022);
    let associated_user = get_associated_token_address_with_program_id(user, mint, &token_2022);
    let mut data = vec![102, 6, 61, 18, 1, 218, 235, 234];
    data.extend_from_slice(&token_amount.to_le_bytes());
    data.extend_from_slice(&sol_amount.to_le_bytes());
    data.push(1);
    let instruction = Instruction {
        program_id: pump_program,
        accounts: vec![
            AccountMeta::new_readonly(global_pda()?, false),
            AccountMeta::new(select_buy_fee_recipient(global, mayhem_mode), false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(bonding_curve, false),
            AccountMeta::new(associated_bonding_curve, false),
            AccountMeta::new(associated_user, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(token_2022, false),
            AccountMeta::new(creator_vault_pda(launch_creator)?, false),
            AccountMeta::new_readonly(event_authority_pda(&pump_program), false),
            AccountMeta::new_readonly(pump_program, false),
            AccountMeta::new_readonly(global_volume_accumulator_pda()?, false),
            AccountMeta::new(user_volume_accumulator_pda(user)?, false),
            AccountMeta::new_readonly(fee_config_pda()?, false),
            AccountMeta::new_readonly(pump_fee_program_id()?, false),
            AccountMeta::new_readonly(bonding_curve_v2_pda(mint)?, false),
        ],
        data,
    };
    Ok(instruction)
}

fn build_sell_instruction(
    global: &PumpGlobalState,
    mint: &Pubkey,
    launch_creator: &Pubkey,
    user: &Pubkey,
    token_amount: u64,
    min_sol_output: u64,
    cashback_enabled: bool,
    mayhem_mode: bool,
) -> Result<Instruction, String> {
    let pump_program = pump_program_id()?;
    let token_2022 = token_2022_program_id()?;
    let bonding_curve = bonding_curve_pda(mint)?;
    let associated_bonding_curve =
        get_associated_token_address_with_program_id(&bonding_curve, mint, &token_2022);
    let associated_user = get_associated_token_address_with_program_id(user, mint, &token_2022);
    let mut data = vec![51, 230, 133, 164, 1, 127, 131, 173];
    data.extend_from_slice(&token_amount.to_le_bytes());
    data.extend_from_slice(&min_sol_output.to_le_bytes());
    data.push(u8::from(mayhem_mode));
    let mut instruction = Instruction {
        program_id: pump_program,
        accounts: vec![
            AccountMeta::new_readonly(global_pda()?, false),
            AccountMeta::new(select_buy_fee_recipient(global, mayhem_mode), false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(bonding_curve, false),
            AccountMeta::new(associated_bonding_curve, false),
            AccountMeta::new(associated_user, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new(creator_vault_pda(launch_creator)?, false),
            AccountMeta::new_readonly(token_2022, false),
            AccountMeta::new_readonly(event_authority_pda(&pump_program), false),
            AccountMeta::new_readonly(pump_program, false),
            AccountMeta::new_readonly(fee_config_pda()?, false),
            AccountMeta::new_readonly(pump_fee_program_id()?, false),
        ],
        data,
    };
    if cashback_enabled {
        instruction
            .accounts
            .push(AccountMeta::new(user_volume_accumulator_pda(user)?, false));
    }
    instruction.accounts.push(AccountMeta::new_readonly(
        bonding_curve_v2_pda(mint)?,
        false,
    ));
    Ok(instruction)
}

fn read_token_account_amount(data: &[u8]) -> Result<u64, String> {
    if data.len() < 72 {
        return Err("Token account data was shorter than expected.".to_string());
    }
    let amount_bytes: [u8; 8] = data[64..72]
        .try_into()
        .map_err(|_| "Token account amount bytes were invalid.".to_string())?;
    Ok(u64::from_le_bytes(amount_bytes))
}

fn recipient_pubkey(recipient: &NormalizedRecipient) -> Result<Pubkey, String> {
    if !recipient.address.is_empty() {
        return parse_pubkey(&recipient.address, "feeSharing.recipient.address");
    }
    if !recipient.githubUserId.is_empty() {
        return social_fee_pda(&recipient.githubUserId, PLATFORM_GITHUB);
    }
    Err("Each fee-sharing recipient must provide either an address or githubUserId.".to_string())
}

fn encode_shareholders(recipients: &[NormalizedRecipient]) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    data.extend_from_slice(&(recipients.len() as u32).to_le_bytes());
    for recipient in recipients {
        let address = recipient_pubkey(recipient)?;
        data.extend_from_slice(address.as_ref());
        let share_bps = u16::try_from(recipient.shareBps).map_err(|_| {
            format!(
                "shareBps is out of range for recipient {}",
                recipient.address
            )
        })?;
        data.extend_from_slice(&share_bps.to_le_bytes());
    }
    Ok(data)
}

fn build_create_fee_sharing_config_instruction(
    mint: &Pubkey,
    payer: &Pubkey,
) -> Result<Instruction, String> {
    let program_id = pump_fee_program_id()?;
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(event_authority_pda(&program_id), false),
            AccountMeta::new_readonly(program_id, false),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(global_pda()?, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(fee_sharing_config_pda(mint)?, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new(bonding_curve_pda(mint)?, false),
            AccountMeta::new_readonly(pump_program_id()?, false),
            AccountMeta::new_readonly(event_authority_pda(&pump_program_id()?), false),
            AccountMeta::new_readonly(program_id, false),
            AccountMeta::new_readonly(pump_amm_program_id()?, false),
            AccountMeta::new_readonly(program_id, false),
        ],
        data: vec![195, 78, 86, 76, 111, 52, 251, 213],
    })
}

fn build_update_fee_shares_instruction(
    mint: &Pubkey,
    authority: &Pubkey,
    current_shareholders: &[Pubkey],
    recipients: &[crate::config::NormalizedRecipient],
) -> Result<Instruction, String> {
    let program_id = pump_fee_program_id()?;
    let sharing_config = fee_sharing_config_pda(mint)?;
    let coin_creator_vault_authority = coin_creator_vault_authority_pda(&sharing_config)?;
    let mut data = vec![189, 13, 136, 99, 187, 164, 237, 35];
    data.extend_from_slice(&encode_shareholders(recipients)?);
    let mut accounts = vec![
        AccountMeta::new_readonly(event_authority_pda(&program_id), false),
        AccountMeta::new_readonly(program_id, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(global_pda()?, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(sharing_config, false),
        AccountMeta::new_readonly(bonding_curve_pda(mint)?, false),
        AccountMeta::new(creator_vault_pda(&sharing_config)?, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(pump_program_id()?, false),
        AccountMeta::new_readonly(event_authority_pda(&pump_program_id()?), false),
        AccountMeta::new_readonly(pump_amm_program_id()?, false),
        AccountMeta::new_readonly(event_authority_pda(&pump_amm_program_id()?), false),
        AccountMeta::new_readonly(wsol_mint()?, false),
        AccountMeta::new_readonly(token_program_id()?, false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new(coin_creator_vault_authority, false),
        AccountMeta::new(
            coin_creator_vault_ata_pda(&coin_creator_vault_authority)?,
            false,
        ),
    ];
    accounts.extend(
        current_shareholders
            .iter()
            .map(|shareholder| AccountMeta::new(*shareholder, false)),
    );
    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

fn build_create_social_fee_pda_instruction(
    payer: &Pubkey,
    user_id: &str,
    platform: u8,
) -> Result<Instruction, String> {
    let program_id = pump_fee_program_id()?;
    let mut data = vec![144, 224, 59, 211, 78, 248, 202, 220];
    encode_borsh_string(&mut data, user_id);
    data.push(platform);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(social_fee_pda(user_id, platform)?, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(fee_program_global_pda()?, false),
            AccountMeta::new_readonly(event_authority_pda(&program_id), false),
            AccountMeta::new_readonly(program_id, false),
        ],
        data,
    })
}

async fn build_social_fee_pda_create_instructions(
    rpc_url: &str,
    recipients: &[NormalizedRecipient],
    payer: &Pubkey,
) -> Result<Vec<Instruction>, String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut ordered_user_ids = Vec::new();
    let mut addresses = Vec::new();
    for (index, recipient) in recipients.iter().enumerate() {
        if recipient.githubUserId.is_empty() {
            continue;
        }
        if !seen.insert(recipient.githubUserId.clone()) {
            continue;
        }
        ordered_user_ids.push((index, recipient.githubUserId.clone()));
        addresses.push(social_fee_pda(&recipient.githubUserId, PLATFORM_GITHUB)?.to_string());
    }
    let exists = fetch_multiple_account_exists(rpc_url, &addresses, "confirmed").await?;
    ordered_user_ids.sort_by_key(|(index, _)| *index);
    let mut instructions = Vec::new();
    for ((_, user_id), exists) in ordered_user_ids.into_iter().zip(exists.into_iter()) {
        if !exists {
            instructions.push(build_create_social_fee_pda_instruction(
                payer,
                &user_id,
                PLATFORM_GITHUB,
            )?);
        }
    }
    Ok(instructions)
}

async fn build_fee_sharing_follow_up_instructions(
    rpc_url: &str,
    config: &NormalizedConfig,
    mint: Pubkey,
    creator: Pubkey,
) -> Result<Vec<Instruction>, String> {
    build_fee_sharing_setup_instructions(
        rpc_url,
        mint,
        creator,
        std::slice::from_ref(&creator),
        &config.feeSharing.recipients,
    )
    .await
}

async fn build_fee_sharing_setup_instructions(
    rpc_url: &str,
    mint: Pubkey,
    creator: Pubkey,
    current_shareholders: &[Pubkey],
    recipients: &[NormalizedRecipient],
) -> Result<Vec<Instruction>, String> {
    if recipients.is_empty() {
        return Err("fee sharing recipients are required for native follow-up setup.".to_string());
    }
    let mut instructions = vec![build_create_fee_sharing_config_instruction(
        &mint, &creator,
    )?];
    instructions
        .extend(build_social_fee_pda_create_instructions(rpc_url, recipients, &creator).await?);
    instructions.push(build_update_fee_shares_instruction(
        &mint,
        &creator,
        current_shareholders,
        recipients,
    )?);
    Ok(instructions)
}

fn resolve_agent_fee_recipients(
    config: &NormalizedConfig,
    mint: &Pubkey,
    creator: &Pubkey,
) -> Result<Vec<NormalizedRecipient>, String> {
    let configured = if config.agent.feeRecipients.is_empty() {
        let buyback_bps = config.agent.buybackBps.unwrap_or(0);
        let mut recipients = Vec::new();
        if buyback_bps > 0 {
            recipients.push(NormalizedRecipient {
                r#type: Some("agent".to_string()),
                address: String::new(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: buyback_bps,
            });
        }
        let wallet_share = 10_000 - buyback_bps;
        if wallet_share > 0 {
            recipients.push(NormalizedRecipient {
                r#type: Some("wallet".to_string()),
                address: if config.agent.feeReceiver.is_empty() {
                    creator.to_string()
                } else {
                    config.agent.feeReceiver.clone()
                },
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: wallet_share,
            });
        }
        recipients
    } else {
        config.agent.feeRecipients.clone()
    };

    let agent_receiver = token_agent_payments_pda(mint)?.to_string();
    let mut resolved = Vec::with_capacity(configured.len());
    for entry in configured {
        if entry.r#type.as_deref() == Some("agent") {
            resolved.push(NormalizedRecipient {
                r#type: Some("wallet".to_string()),
                address: agent_receiver.clone(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: entry.shareBps,
            });
        } else {
            resolved.push(entry);
        }
    }
    Ok(resolved)
}

fn native_follow_up_label(config: &NormalizedConfig) -> Option<&'static str> {
    launch_follow_up_label(config)
}

async fn build_native_follow_up_instructions(
    rpc_url: &str,
    config: &NormalizedConfig,
    mint: Pubkey,
    creator: Pubkey,
    _agent_authority: Option<&Pubkey>,
) -> Result<Vec<Instruction>, String> {
    match config.mode.as_str() {
        "regular" | "cashback" => {
            build_fee_sharing_follow_up_instructions(rpc_url, config, mint, creator).await
        }
        "agent-custom" => {
            let recipients = resolve_agent_fee_recipients(config, &mint, &creator)?;
            let mut instructions = vec![];
            instructions.extend(
                build_fee_sharing_setup_instructions(
                    rpc_url,
                    mint,
                    creator,
                    std::slice::from_ref(&creator),
                    &recipients,
                )
                .await?,
            );
            Ok(instructions)
        }
        "agent-locked" => {
            let recipients = vec![NormalizedRecipient {
                r#type: Some("wallet".to_string()),
                address: token_agent_payments_pda(&mint)?.to_string(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 10_000,
            }];
            let mut instructions = vec![];
            instructions.extend(
                build_fee_sharing_setup_instructions(
                    rpc_url,
                    mint,
                    creator,
                    std::slice::from_ref(&creator),
                    &recipients,
                )
                .await?,
            );
            Ok(instructions)
        }
        unsupported => Err(format!(
            "Native Pump follow-up builder does not support mode={unsupported}."
        )),
    }
}

fn build_compute_unit_limit_instruction(compute_unit_limit: u32) -> Result<Instruction, String> {
    let mut data = vec![2];
    data.extend_from_slice(&compute_unit_limit.to_le_bytes());
    Ok(Instruction {
        program_id: compute_budget_program_id()?,
        accounts: vec![],
        data,
    })
}

fn build_compute_unit_price_instruction(micro_lamports: u64) -> Result<Instruction, String> {
    let mut data = vec![3];
    data.extend_from_slice(&micro_lamports.to_le_bytes());
    Ok(Instruction {
        program_id: compute_budget_program_id()?,
        accounts: vec![],
        data,
    })
}

fn with_tx_settings(
    core_instructions: Vec<Instruction>,
    tx_config: &NativeTxConfig,
    payer: &Pubkey,
    jitodontfront_enabled: bool,
) -> Result<Vec<Instruction>, String> {
    let mut instructions = vec![build_compute_unit_limit_instruction(
        tx_config.compute_unit_limit,
    )?];
    if tx_config.compute_unit_price_micro_lamports > 0 {
        instructions.push(build_compute_unit_price_instruction(
            tx_config.compute_unit_price_micro_lamports as u64,
        )?);
    }
    instructions.extend(apply_jitodontfront(
        core_instructions,
        jitodontfront_enabled,
    )?);
    if tx_config.jito_tip_lamports > 0 {
        let tip_account = parse_pubkey(&tx_config.jito_tip_account, "tx.jitoTipAccount")?;
        instructions.push(transfer(
            payer,
            &tip_account,
            tx_config.jito_tip_lamports as u64,
        ));
    }
    Ok(instructions)
}

fn apply_jitodontfront(
    mut instructions: Vec<Instruction>,
    enabled: bool,
) -> Result<Vec<Instruction>, String> {
    if !enabled {
        return Ok(instructions);
    }
    let dontfront = parse_pubkey(JITODONTFRONT_ACCOUNT, "jitodontfront")?;
    for instruction in &mut instructions {
        if instruction
            .accounts
            .iter()
            .any(|meta| meta.pubkey == dontfront)
        {
            continue;
        }
        instruction
            .accounts
            .push(AccountMeta::new_readonly(dontfront, false));
    }
    Ok(instructions)
}

fn parse_lookup_table_addresses(config: &NormalizedConfig) -> Vec<String> {
    let mut values = Vec::new();
    if config.tx.useDefaultLookupTables {
        values.extend(DEFAULT_LOOKUP_TABLES.iter().map(|entry| entry.to_string()));
    }
    values.extend(config.tx.lookupTables.clone());
    dedupe_lookup_table_addresses(values)
}

fn dedupe_lookup_table_addresses(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.iter().any(|entry| entry == &value) {
            deduped.push(value);
        }
    }
    deduped
}

fn default_lookup_table_profiles_for_label(label: &str) -> Vec<Vec<String>> {
    match label {
        "launch" => DEFAULT_LAUNCH_LOOKUP_TABLE_PROFILES
            .iter()
            .map(|profile| profile.iter().map(|entry| entry.to_string()).collect())
            .collect(),
        "follow-up" | "agent-setup" => DEFAULT_FOLLOW_UP_LOOKUP_TABLE_PROFILES
            .iter()
            .map(|profile| profile.iter().map(|entry| entry.to_string()).collect())
            .collect(),
        _ => DEFAULT_LOOKUP_TABLES
            .iter()
            .map(|entry| vec![entry.to_string()])
            .collect(),
    }
}

fn lookup_table_variants_for_transaction(
    label: &str,
    config: &NormalizedConfig,
    loaded_lookup_tables: &[AddressLookupTableAccount],
) -> Vec<Vec<AddressLookupTableAccount>> {
    let explicit = dedupe_lookup_table_addresses(config.tx.lookupTables.clone());
    let mut requested_variants = Vec::new();
    if config.tx.useDefaultLookupTables {
        for profile in default_lookup_table_profiles_for_label(label) {
            let mut combined = profile;
            combined.extend(explicit.clone());
            requested_variants.push(dedupe_lookup_table_addresses(combined));
        }
        let mut union = DEFAULT_LOOKUP_TABLES
            .iter()
            .map(|entry| entry.to_string())
            .collect::<Vec<_>>();
        union.extend(explicit.clone());
        requested_variants.push(dedupe_lookup_table_addresses(union));
    }
    if !explicit.is_empty() {
        requested_variants.push(explicit);
    }

    let mut unique_keys = Vec::new();
    let mut variants = Vec::new();
    for addresses in requested_variants {
        let mut variant = Vec::new();
        for address in &addresses {
            if let Some(table) = loaded_lookup_tables
                .iter()
                .find(|table| table.key.to_string() == *address)
            {
                variant.push(table.clone());
            }
        }
        if variant.is_empty() {
            continue;
        }
        let key = variant
            .iter()
            .map(|table| table.key.to_string())
            .collect::<Vec<_>>()
            .join(",");
        if unique_keys.iter().any(|entry| entry == &key) {
            continue;
        }
        unique_keys.push(key);
        variants.push(variant);
    }
    variants
}

#[derive(Clone)]
struct CachedLookupTableAccount {
    loaded_at: Instant,
    table: AddressLookupTableAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedLookupTableCache {
    tables: HashMap<String, PersistedLookupTableEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedLookupTableEntry {
    addresses: Vec<String>,
}

fn lookup_table_cache() -> &'static Mutex<HashMap<String, CachedLookupTableAccount>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedLookupTableAccount>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn persisted_lookup_table_cache() -> &'static Mutex<PersistedLookupTableCache> {
    static CACHE: OnceLock<Mutex<PersistedLookupTableCache>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let cache = fs::read_to_string(paths::lookup_table_cache_path())
            .ok()
            .and_then(|raw| serde_json::from_str::<PersistedLookupTableCache>(&raw).ok())
            .unwrap_or_default();
        Mutex::new(cache)
    })
}

fn is_default_lookup_table_address(address: &str) -> bool {
    DEFAULT_LOOKUP_TABLES.contains(&address)
}

fn persist_lookup_table_account(
    address: &str,
    table: &AddressLookupTableAccount,
) -> Result<(), String> {
    if !is_default_lookup_table_address(address) {
        return Ok(());
    }
    let mut cache = persisted_lookup_table_cache()
        .lock()
        .map_err(|error| error.to_string())?;
    cache.tables.insert(
        address.to_string(),
        PersistedLookupTableEntry {
            addresses: table
                .addresses
                .iter()
                .map(|entry| entry.to_string())
                .collect(),
        },
    );
    let serialized = serde_json::to_string_pretty(&*cache).map_err(|error| error.to_string())?;
    let path = paths::lookup_table_cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, serialized).map_err(|error| error.to_string())?;
    Ok(())
}

fn load_persisted_lookup_table_account(address: &str) -> Option<AddressLookupTableAccount> {
    if !is_default_lookup_table_address(address) {
        return None;
    }
    let cache = persisted_lookup_table_cache().lock().ok()?;
    let entry = cache.tables.get(address)?;
    let key = parse_pubkey(address, "lookup table address").ok()?;
    let addresses = entry
        .addresses
        .iter()
        .map(|entry| parse_pubkey(entry, "lookup table entry"))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    Some(AddressLookupTableAccount { key, addresses })
}

fn get_cached_lookup_table_account(address: &str) -> Option<AddressLookupTableAccount> {
    let mut cache = lookup_table_cache().lock().ok()?;
    match cache.get(address) {
        Some(entry) if entry.loaded_at.elapsed() <= LOOKUP_TABLE_CACHE_TTL => {
            Some(entry.table.clone())
        }
        Some(_) => {
            cache.remove(address);
            None
        }
        None => None,
    }
}

fn cache_lookup_table_account(address: &str, table: &AddressLookupTableAccount) {
    if let Ok(mut cache) = lookup_table_cache().lock() {
        cache.insert(
            address.to_string(),
            CachedLookupTableAccount {
                loaded_at: Instant::now(),
                table: table.clone(),
            },
        );
    }
}

async fn load_lookup_table_account(
    rpc_url: String,
    address: String,
) -> Result<Option<AddressLookupTableAccount>, String> {
    if let Some(table) = get_cached_lookup_table_account(&address) {
        return Ok(Some(table));
    }
    if let Some(table) = load_persisted_lookup_table_account(&address) {
        cache_lookup_table_account(&address, &table);
        return Ok(Some(table));
    }
    let data = match fetch_account_data(&rpc_url, &address, "confirmed").await {
        Ok(data) => data,
        Err(error) if error.contains("was not found") => return Ok(None),
        Err(error) => return Err(error),
    };
    let table = AddressLookupTable::deserialize(&data)
        .map_err(|error| format!("Failed to decode address lookup table {address}: {error}"))?;
    let account = AddressLookupTableAccount {
        key: parse_pubkey(&address, "lookup table address")?,
        addresses: table.addresses.to_vec(),
    };
    cache_lookup_table_account(&address, &account);
    persist_lookup_table_account(&address, &account)?;
    Ok(Some(account))
}

async fn load_lookup_table_accounts_for_addresses(
    rpc_url: &str,
    requested: &[String],
) -> Result<Vec<AddressLookupTableAccount>, String> {
    if requested.is_empty() {
        return Ok(vec![]);
    }
    let mut tasks = JoinSet::new();
    for (index, address) in requested.iter().cloned().enumerate() {
        let rpc_url = rpc_url.to_string();
        tasks.spawn(async move { (index, load_lookup_table_account(rpc_url, address).await) });
    }
    let mut resolved = vec![None; requested.len()];
    while let Some(joined) = tasks.join_next().await {
        let (index, table_result) = joined.map_err(|error| error.to_string())?;
        resolved[index] = Some(table_result?);
    }
    Ok(resolved.into_iter().flatten().flatten().collect::<Vec<_>>())
}

async fn load_lookup_table_accounts(
    rpc_url: &str,
    config: &NormalizedConfig,
) -> Result<Vec<AddressLookupTableAccount>, String> {
    let requested = parse_lookup_table_addresses(config);
    let should_load = matches!(config.execution.txFormat.as_str(), "auto" | "v0-alt");
    if !should_load || requested.is_empty() {
        return Ok(vec![]);
    }
    load_lookup_table_accounts_for_addresses(rpc_url, &requested).await
}

#[allow(dead_code)]
pub async fn warm_default_lookup_tables(rpc_url: &str) -> Result<usize, String> {
    let requested = DEFAULT_LOOKUP_TABLES
        .iter()
        .map(|entry| entry.to_string())
        .collect::<Vec<_>>();
    let loaded = load_lookup_table_accounts_for_addresses(rpc_url, &requested).await?;
    Ok(loaded.len())
}

fn select_native_format(
    requested: &str,
    has_lookup_tables: bool,
) -> Result<NativeTxFormat, String> {
    match requested {
        "legacy" => Ok(NativeTxFormat::Legacy),
        "v0" => Ok(NativeTxFormat::V0),
        "v0-alt" => {
            if has_lookup_tables {
                Ok(NativeTxFormat::V0Alt)
            } else {
                Err("Native Pump compile requires at least one loaded lookup table for txFormat=v0-alt.".to_string())
            }
        }
        "auto" => Ok(NativeTxFormat::Auto),
        unsupported => Err(format!(
            "Native Pump compile does not yet support txFormat={unsupported}."
        )),
    }
}

fn metrics_length_mut<'a>(
    metrics: &'a mut TransactionCompileMetrics,
    format: &str,
) -> Result<&'a mut Option<usize>, String> {
    match format {
        "legacy" => Ok(&mut metrics.legacy_length),
        "v0" => Ok(&mut metrics.v0_length),
        "v0-alt" => Ok(&mut metrics.v0_alt_length),
        unsupported => Err(format!(
            "Unsupported compiled transaction format in metrics: {unsupported}"
        )),
    }
}

fn choose_best_auto_candidate(candidates: &[CompiledTxCandidate]) -> usize {
    let packet_limit = 1232usize;
    candidates
        .iter()
        .enumerate()
        .min_by_key(|(_, candidate)| {
            (
                candidate.serialized_len > packet_limit,
                candidate.serialized_len,
                candidate.compiled.format == "v0-alt",
            )
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn auto_selection_warning(
    selected: &CompiledTxCandidate,
    v0_length: usize,
    v0_alt_length: Option<usize>,
) -> String {
    let selected_suffix = if selected.serialized_len > 1232 {
        " but it still exceeds the 1232-byte packet limit"
    } else {
        ""
    };
    match v0_alt_length {
        Some(alt_length) if selected.compiled.format == "v0-alt" => format!(
            "Auto selected v0-alt at {} bytes over v0 at {} bytes{}.",
            alt_length, v0_length, selected_suffix
        ),
        Some(alt_length) => format!(
            "Auto kept v0 at {} bytes because v0-alt measured {} bytes{}.",
            v0_length, alt_length, selected_suffix
        ),
        None => format!(
            "Auto used v0 at {} bytes because no lookup-table candidate was available{}.",
            v0_length, selected_suffix
        ),
    }
}

fn compile_transaction_with_metrics(
    label: &str,
    tx_format: NativeTxFormat,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    mint_signer: Option<&Keypair>,
    instructions: Vec<Instruction>,
    tx_config: &NativeTxConfig,
    lookup_table_variants: &[Vec<AddressLookupTableAccount>],
) -> Result<(CompiledTransaction, TransactionCompileMetrics), String> {
    let preferred_lookup_tables = lookup_table_variants
        .first()
        .map(|variant| variant.as_slice())
        .unwrap_or(&[]);
    if tx_format != NativeTxFormat::Auto {
        let candidate = compile_transaction_candidate(
            label,
            tx_format,
            blockhash,
            last_valid_block_height,
            payer,
            mint_signer,
            instructions,
            tx_config,
            preferred_lookup_tables,
        )?;
        let mut metrics = TransactionCompileMetrics::default();
        *metrics_length_mut(&mut metrics, &candidate.compiled.format)? =
            Some(candidate.serialized_len);
        return Ok((candidate.compiled, metrics));
    }

    let v0_candidate = compile_transaction_candidate(
        label,
        NativeTxFormat::V0,
        blockhash,
        last_valid_block_height,
        payer,
        mint_signer,
        instructions.clone(),
        tx_config,
        &[],
    )?;

    if lookup_table_variants.is_empty() {
        let candidate = compile_transaction_candidate(
            label,
            NativeTxFormat::V0,
            blockhash,
            last_valid_block_height,
            payer,
            mint_signer,
            instructions,
            tx_config,
            &[],
        )?;
        let mut metrics = TransactionCompileMetrics::default();
        *metrics_length_mut(&mut metrics, &candidate.compiled.format)? =
            Some(candidate.serialized_len);
        metrics.warnings.push(auto_selection_warning(
            &candidate,
            candidate.serialized_len,
            None,
        ));
        return Ok((candidate.compiled, metrics));
    }

    let mut metrics = TransactionCompileMetrics::default();
    *metrics_length_mut(&mut metrics, &v0_candidate.compiled.format)? =
        Some(v0_candidate.serialized_len);
    let mut candidates = vec![v0_candidate];
    let mut best_alt_length: Option<usize> = None;
    for variant in lookup_table_variants {
        if variant.is_empty() {
            continue;
        }
        let alt_candidate = compile_transaction_candidate(
            label,
            NativeTxFormat::V0Alt,
            blockhash,
            last_valid_block_height,
            payer,
            mint_signer,
            instructions.clone(),
            tx_config,
            variant,
        )?;
        if alt_candidate.compiled.format == "v0-alt" {
            best_alt_length = Some(match best_alt_length {
                Some(current_best) => current_best.min(alt_candidate.serialized_len),
                None => alt_candidate.serialized_len,
            });
        }
        candidates.push(alt_candidate);
    }
    *metrics_length_mut(&mut metrics, "v0-alt")? = best_alt_length;
    let selected_index = choose_best_auto_candidate(&candidates);
    let selected = candidates
        .get(selected_index)
        .cloned()
        .ok_or_else(|| "Auto transaction candidate selection failed.".to_string())?;
    metrics.warnings.push(auto_selection_warning(
        &selected,
        candidates[0].serialized_len,
        best_alt_length,
    ));
    Ok((selected.compiled, metrics))
}

fn compile_transaction_candidate(
    label: &str,
    tx_format: NativeTxFormat,
    blockhash: &str,
    last_valid_block_height: u64,
    payer: &Keypair,
    mint_signer: Option<&Keypair>,
    instructions: Vec<Instruction>,
    tx_config: &NativeTxConfig,
    lookup_tables: &[AddressLookupTableAccount],
) -> Result<CompiledTxCandidate, String> {
    if tx_format == NativeTxFormat::Auto {
        return Err("Auto transaction format must be resolved before compile.".to_string());
    }
    let hash = Hash::from_str(blockhash).map_err(|error| error.to_string())?;
    let (serialized, format, lookup_tables_used) = if tx_format == NativeTxFormat::Legacy {
        let signers: Vec<&Keypair> = match mint_signer {
            Some(mint) => vec![payer, mint],
            None => vec![payer],
        };
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &signers,
            hash,
        );
        (
            bincode::serialize(&transaction).map_err(|error| error.to_string())?,
            "legacy".to_string(),
            vec![],
        )
    } else {
        let dynamic_lookups = if tx_format == NativeTxFormat::V0Alt {
            lookup_tables
        } else {
            &[]
        };
        let message =
            v0::Message::try_compile(&payer.pubkey(), &instructions, dynamic_lookups, hash)
                .map_err(|error| error.to_string())?;
        let lookup_tables_used = message
            .address_table_lookups
            .iter()
            .map(|lookup| lookup.account_key.to_string())
            .collect::<Vec<_>>();
        let signers: Vec<&Keypair> = match mint_signer {
            Some(mint) => vec![payer, mint],
            None => vec![payer],
        };
        let transaction = VersionedTransaction::try_new(VersionedMessage::V0(message), &signers)
            .map_err(|error| error.to_string())?;
        (
            bincode::serialize(&transaction).map_err(|error| error.to_string())?,
            if lookup_tables_used.is_empty() {
                "v0".to_string()
            } else {
                "v0-alt".to_string()
            },
            lookup_tables_used,
        )
    };
    let serialized_len = serialized.len();
    let serialized_base64 = BASE64.encode(serialized);
    let signature = crate::rpc::precompute_transaction_signature(&serialized_base64);
    Ok(CompiledTxCandidate {
        serialized_len,
        compiled: CompiledTransaction {
            label: label.to_string(),
            format,
            blockhash: blockhash.to_string(),
            lastValidBlockHeight: last_valid_block_height,
            serializedBase64: serialized_base64,
            signature,
            lookupTablesUsed: lookup_tables_used,
            computeUnitLimit: Some(u64::from(tx_config.compute_unit_limit)),
            computeUnitPriceMicroLamports: if tx_config.compute_unit_price_micro_lamports > 0 {
                Some(tx_config.compute_unit_price_micro_lamports as u64)
            } else {
                None
            },
            inlineTipLamports: if tx_config.jito_tip_lamports > 0 {
                Some(tx_config.jito_tip_lamports as u64)
            } else {
                None
            },
            inlineTipAccount: if tx_config.jito_tip_lamports > 0
                && !tx_config.jito_tip_account.is_empty()
            {
                Some(tx_config.jito_tip_account.clone())
            } else {
                None
            },
        },
    })
}

fn summarize_instructions(instructions: &[Instruction]) -> Vec<crate::report::InstructionSummary> {
    instructions
        .iter()
        .enumerate()
        .map(|(index, instruction)| crate::report::InstructionSummary {
            index,
            programId: instruction.program_id.to_string(),
            keyCount: instruction.accounts.len(),
            writableKeys: instruction
                .accounts
                .iter()
                .filter(|meta| meta.is_writable)
                .count(),
            signerKeys: instruction
                .accounts
                .iter()
                .filter(|meta| meta.is_signer)
                .count(),
        })
        .collect()
}

fn apply_transaction_details(
    report: &mut LaunchReport,
    compiled_transactions: &[CompiledTransaction],
    instruction_summaries: &[Vec<crate::report::InstructionSummary>],
    compile_metrics: &[TransactionCompileMetrics],
) -> Result<(), String> {
    for (index, summary) in report.transactions.iter_mut().enumerate() {
        let Some(compiled) = compiled_transactions.get(index) else {
            break;
        };
        let metrics = compile_metrics.get(index).cloned().unwrap_or_default();
        let raw = BASE64
            .decode(&compiled.serializedBase64)
            .map_err(|error| error.to_string())?;
        let encoded_len = compiled.serializedBase64.len();
        summary.legacyLength = metrics.legacy_length;
        summary.legacyBase64Length = metrics.legacy_length.map(|_| encoded_len);
        summary.v0Length = metrics.v0_length;
        summary.v0Base64Length = metrics.v0_length.map(|_| encoded_len);
        summary.v0AltLength = metrics.v0_alt_length;
        summary.v0AltBase64Length = metrics.v0_alt_length.map(|_| encoded_len);
        if compiled.format == "legacy" {
            if summary.legacyLength.is_none() {
                summary.legacyLength = Some(raw.len());
                summary.legacyBase64Length = Some(encoded_len);
            }
            summary.v0Error = Some("Native path compiled as legacy only.".to_string());
            summary.v0AltError = Some("Native path compiled as legacy only.".to_string());
        } else {
            if compiled.format == "v0-alt" {
                if summary.v0AltLength.is_none() {
                    summary.v0AltLength = Some(raw.len());
                    summary.v0AltBase64Length = Some(encoded_len);
                }
                summary.v0Error = Some("Native path compiled with lookup tables.".to_string());
            } else {
                if summary.v0Length.is_none() {
                    summary.v0Length = Some(raw.len());
                    summary.v0Base64Length = Some(encoded_len);
                }
                summary.v0AltError =
                    Some("Native path compiled as versioned without lookup tables.".to_string());
            }
            summary.legacyError = Some("Native path compiled as versioned only.".to_string());
        }
        summary.fitsWithAlts = compiled.format == "v0-alt";
        summary.lookupTablesUsed = if compiled.format == "v0-alt" {
            compiled.lookupTablesUsed.clone()
        } else {
            vec![]
        };
        summary.exceedsPacketLimit = raw.len() > 1232;
        summary.instructionSummary = instruction_summaries
            .get(index)
            .cloned()
            .unwrap_or_default();
        summary.warnings.extend(metrics.warnings);
        if raw.len() > 1232 {
            summary.warnings.push(format!(
                "Compiled packet exceeds the 1232-byte Solana limit by {} bytes.",
                raw.len() - 1232
            ));
        } else if raw.len() >= 1150 {
            summary.warnings.push(format!(
                "Compiled packet is within {} bytes of the 1232-byte Solana limit.",
                1232 - raw.len()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{RawConfig, normalize_raw_config};
    use crate::transport::{build_transport_plan, estimate_transaction_count};

    fn regular_config() -> crate::config::NormalizedConfig {
        let mut raw = RawConfig {
            mode: "regular".to_string(),
            launchpad: "pump".to_string(),
            ..RawConfig::default()
        };
        raw.token.name = "LaunchDeck".to_string();
        raw.token.symbol = "LDECK".to_string();
        raw.token.uri = "ipfs://fixture".to_string();
        raw.tx.computeUnitPriceMicroLamports = Some(Value::from(1));
        raw.tx.jitoTipLamports = Some(Value::from(200_000));
        raw.tx.jitoTipAccount = "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE".to_string();
        raw.execution.skipPreflight = Some(Value::Bool(true));
        raw.execution.provider = "helius-sender".to_string();
        raw.execution.buyProvider = "helius-sender".to_string();
        raw.execution.sellProvider = "helius-sender".to_string();
        normalize_raw_config(raw).expect("normalized config")
    }

    #[test]
    fn jito_single_creation_keeps_priority_fee() {
        let mut config = regular_config();
        config.execution.provider = "jito-bundle".to_string();
        let transport_plan =
            build_transport_plan(&config.execution, estimate_transaction_count(&config));
        assert_eq!(
            effective_creation_compute_unit_price_micro_lamports(&config, &transport_plan, false),
            config.tx.computeUnitPriceMicroLamports.unwrap_or(0)
        );
    }

    #[test]
    fn slippage_percent_maps_to_expected_bps() {
        assert_eq!(slippage_bps_from_percent("20").expect("20%"), 2_000);
        assert_eq!(slippage_bps_from_percent("0.5").expect("0.5%"), 50);
        assert_eq!(slippage_bps_from_percent("100").expect("100%"), 10_000);
    }

    #[test]
    fn jito_multi_tx_creation_drops_priority_fee() {
        let mut config = regular_config();
        config.execution.provider = "jito-bundle".to_string();
        config.mode = "agent-custom".to_string();
        config.agent.buybackBps = Some(2_500);
        config.agent.feeRecipients = vec![
            crate::config::NormalizedRecipient {
                r#type: Some("agent".to_string()),
                address: String::new(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 2_500,
            },
            crate::config::NormalizedRecipient {
                r#type: Some("wallet".to_string()),
                address: Pubkey::new_unique().to_string(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 7_500,
            },
        ];
        let transport_plan =
            build_transport_plan(&config.execution, estimate_transaction_count(&config));
        assert_eq!(
            effective_creation_compute_unit_price_micro_lamports(&config, &transport_plan, true),
            0
        );
        assert_eq!(
            effective_follow_up_compute_unit_price_micro_lamports(&config, &transport_plan),
            0
        );
    }

    #[test]
    fn supports_plain_regular_pump_launches() {
        let config = regular_config();
        assert!(supports_native_pump_compile(&config));
    }

    #[test]
    fn resolve_mint_keypair_uses_vanity_private_key_when_present() {
        let vanity_keypair = Keypair::new();
        let mut config = regular_config();
        config.vanityPrivateKey = bs58::encode(vanity_keypair.to_bytes()).into_string();

        let resolved = resolve_mint_keypair(&config).expect("vanity mint keypair");

        assert_eq!(resolved.pubkey(), vanity_keypair.pubkey());
    }

    #[test]
    fn resolve_mint_keypair_rejects_invalid_vanity_private_key() {
        let mut config = regular_config();
        config.vanityPrivateKey = "not-a-keypair".to_string();

        let error = resolve_mint_keypair(&config).expect_err("invalid vanity key should fail");

        assert!(error.contains("Invalid vanity private key"));
    }

    #[test]
    fn supports_sol_dev_buy_for_native_pump_path() {
        let mut config = regular_config();
        config.devBuy = Some(crate::config::NormalizedDevBuy {
            mode: "sol".to_string(),
            amount: "0.1".to_string(),
            source: "test".to_string(),
        });
        assert!(supports_native_pump_compile(&config));
    }

    #[test]
    fn supports_agent_unlocked_for_native_pump_path() {
        let mut config = regular_config();
        config.mode = "agent-unlocked".to_string();
        config.agent.buybackBps = Some(2_500);
        assert!(supports_native_pump_compile(&config));
    }

    #[test]
    fn supports_agent_custom_for_native_pump_path() {
        let mut config = regular_config();
        config.mode = "agent-custom".to_string();
        config.agent.buybackBps = Some(2_500);
        config.agent.feeRecipients = vec![
            crate::config::NormalizedRecipient {
                r#type: Some("agent".to_string()),
                address: String::new(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 2_500,
            },
            crate::config::NormalizedRecipient {
                r#type: Some("wallet".to_string()),
                address: Pubkey::new_unique().to_string(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 7_500,
            },
        ];
        assert!(supports_native_pump_compile(&config));
    }

    #[test]
    fn supports_agent_locked_for_native_pump_path() {
        let mut config = regular_config();
        config.mode = "agent-locked".to_string();
        config.agent.buybackBps = Some(2_500);
        config.creatorFee.mode = "agent-escrow".to_string();
        assert!(supports_native_pump_compile(&config));
    }

    #[test]
    fn auto_keeps_auto_format_until_compile_time() {
        assert_eq!(
            select_native_format("auto", true).expect("format"),
            NativeTxFormat::Auto
        );
        assert_eq!(
            select_native_format("auto", false).expect("format"),
            NativeTxFormat::Auto
        );
    }

    #[test]
    fn v0_alt_requires_loaded_lookup_tables() {
        assert_eq!(
            select_native_format("v0-alt", true).expect("format"),
            NativeTxFormat::V0Alt
        );
        assert!(select_native_format("v0-alt", false).is_err());
    }

    fn test_compiled_candidate(format: &str, serialized_len: usize) -> CompiledTxCandidate {
        CompiledTxCandidate {
            serialized_len,
            compiled: CompiledTransaction {
                label: "launch".to_string(),
                format: format.to_string(),
                blockhash: "blockhash".to_string(),
                lastValidBlockHeight: 123,
                serializedBase64: String::new(),
                signature: None,
                lookupTablesUsed: vec![],
                computeUnitLimit: Some(0),
                computeUnitPriceMicroLamports: None,
                inlineTipLamports: None,
                inlineTipAccount: None,
            },
        }
    }

    #[test]
    fn auto_selection_prefers_smaller_versioned_candidate() {
        let candidates = vec![
            test_compiled_candidate("v0", 1210),
            test_compiled_candidate("v0-alt", 1178),
        ];
        let selected = choose_best_auto_candidate(&candidates);
        assert_eq!(selected, 1);
    }

    #[test]
    fn auto_selection_prefers_fitting_candidate_when_only_one_fits() {
        let candidates = vec![
            test_compiled_candidate("v0", 1228),
            test_compiled_candidate("v0-alt", 1240),
        ];
        let selected = choose_best_auto_candidate(&candidates);
        assert_eq!(selected, 0);
    }

    #[test]
    fn launch_lookup_table_profiles_prioritize_launchdeck_table() {
        let profiles = default_lookup_table_profiles_for_label("launch");
        assert_eq!(
            profiles[0],
            vec!["AXVvmhWaaPtV52jqYuTNqp1xRrkbxhfJfeHQKxq5cbvZ".to_string()]
        );
        assert_eq!(
            profiles[1],
            vec!["BckPpoRV4h329qAuhTCNoWdWAy2pZSJ89Qu3nuCU1zsj".to_string()]
        );
    }

    #[test]
    fn follow_up_lookup_table_profiles_prioritize_agent_table() {
        let profiles = default_lookup_table_profiles_for_label("agent-setup");
        assert_eq!(
            profiles[0],
            vec!["BckPpoRV4h329qAuhTCNoWdWAy2pZSJ89Qu3nuCU1zsj".to_string()]
        );
        assert_eq!(
            profiles[1],
            vec!["AXVvmhWaaPtV52jqYuTNqp1xRrkbxhfJfeHQKxq5cbvZ".to_string()]
        );
    }

    #[test]
    fn create_v2_metadata_payload_bytes_tracks_name_symbol_and_uri() {
        let mint = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let instruction = build_create_v2_instruction(
            &mint,
            &user,
            &creator,
            "LaunchDeck",
            "LDECK",
            "ipfs://fixture",
            false,
            false,
        )
        .expect("instruction");
        let metadata_payload_bytes =
            create_v2_metadata_payload_bytes(&[instruction]).expect("metadata payload");

        assert_eq!(
            metadata_payload_bytes,
            12 + "LaunchDeck".len() + "LDECK".len() + "ipfs://fixture".len()
        );
    }

    #[test]
    fn supports_wallet_only_deferred_fee_sharing() {
        let mut config = regular_config();
        config.feeSharing.generateLaterSetup = true;
        config.feeSharing.recipients = vec![crate::config::NormalizedRecipient {
            r#type: Some("wallet".to_string()),
            address: Pubkey::new_unique().to_string(),
            githubUserId: String::new(),
            githubUsername: String::new(),
            shareBps: 10_000,
        }];
        assert!(supports_native_pump_compile(&config));
    }

    #[test]
    fn supports_github_creator_fee_routing_with_user_id() {
        let mut config = regular_config();
        config.creatorFee.mode = "github".to_string();
        config.creatorFee.githubUserId = "12345".to_string();
        config.creatorFee.githubUsername = "launchdeck".to_string();
        assert!(supports_native_pump_compile(&config));
    }

    #[test]
    fn supports_social_fee_sharing_follow_up_when_user_ids_are_present() {
        let mut config = regular_config();
        config.feeSharing.generateLaterSetup = true;
        config.feeSharing.recipients = vec![crate::config::NormalizedRecipient {
            r#type: Some("github".to_string()),
            address: String::new(),
            githubUserId: "12345".to_string(),
            githubUsername: "launchdeck".to_string(),
            shareBps: 10_000,
        }];
        assert!(supports_native_pump_compile(&config));
    }

    #[test]
    fn still_rejects_social_fee_sharing_without_user_id() {
        let mut config = regular_config();
        config.feeSharing.generateLaterSetup = true;
        config.feeSharing.recipients = vec![crate::config::NormalizedRecipient {
            r#type: Some("github".to_string()),
            address: String::new(),
            githubUserId: String::new(),
            githubUsername: "launchdeck".to_string(),
            shareBps: 10_000,
        }];
        assert!(!supports_native_pump_compile(&config));
    }

    #[test]
    fn still_rejects_agent_custom_social_recipient_without_user_id() {
        let mut config = regular_config();
        config.mode = "agent-custom".to_string();
        config.agent.buybackBps = Some(2_500);
        config.agent.feeRecipients = vec![crate::config::NormalizedRecipient {
            r#type: Some("github".to_string()),
            address: String::new(),
            githubUserId: String::new(),
            githubUsername: "launchdeck".to_string(),
            shareBps: 10_000,
        }];
        assert!(!supports_native_pump_compile(&config));
    }

    #[test]
    fn create_v2_instruction_shape_matches_expected_accounts() {
        let mint = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let instruction = build_create_v2_instruction(
            &mint,
            &user,
            &creator,
            "LaunchDeck",
            "LDECK",
            "ipfs://fixture",
            false,
            false,
        )
        .expect("instruction");

        assert_eq!(instruction.program_id.to_string(), PUMP_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 16);
        assert_eq!(
            &instruction.data[..8],
            &[214, 144, 76, 236, 95, 139, 49, 180]
        );
    }

    #[test]
    fn buy_instruction_shape_matches_expected_accounts() {
        let global = PumpGlobalState {
            fee_recipient: Pubkey::new_unique(),
            initial_virtual_token_reserves: 1_073_000_000_000_000,
            initial_virtual_sol_reserves: 30_000_000_000,
            initial_real_token_reserves: 793_100_000_000_000,
            fee_basis_points: 100,
            creator_fee_basis_points: 50,
            fee_recipients: [Pubkey::default(); 7],
            reserved_fee_recipient: Pubkey::default(),
            reserved_fee_recipients: [Pubkey::default(); 7],
        };
        let mint = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let instruction = build_buy_instruction(
            &global,
            &mint,
            &creator,
            &user,
            100_000_000,
            1_000_000,
            false,
        )
        .expect("buy instruction");

        assert_eq!(instruction.program_id.to_string(), PUMP_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 17);
        assert_eq!(&instruction.data[..8], &[102, 6, 61, 18, 1, 218, 235, 234]);
    }

    #[test]
    fn buy_instruction_keeps_fee_accounts_before_bonding_curve_v2() {
        let global = PumpGlobalState {
            fee_recipient: Pubkey::new_unique(),
            initial_virtual_token_reserves: 1_073_000_000_000_000,
            initial_virtual_sol_reserves: 30_000_000_000,
            initial_real_token_reserves: 793_100_000_000_000,
            fee_basis_points: 100,
            creator_fee_basis_points: 50,
            fee_recipients: [Pubkey::default(); 7],
            reserved_fee_recipient: Pubkey::default(),
            reserved_fee_recipients: [Pubkey::default(); 7],
        };
        let mint = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let instruction = build_buy_instruction(
            &global,
            &mint,
            &creator,
            &user,
            101_000_000,
            1_000_000,
            false,
        )
        .expect("buy instruction");

        assert_eq!(
            instruction.accounts[14].pubkey,
            fee_config_pda().expect("fee config pda")
        );
        assert_eq!(
            instruction.accounts[15].pubkey,
            pump_fee_program_id().expect("fee program id")
        );
        assert_eq!(
            instruction.accounts[16].pubkey,
            bonding_curve_v2_pda(&mint).expect("bonding curve v2 pda")
        );
    }

    #[test]
    fn launch_instructions_match_sdk_create_and_buy_shape_for_sol_dev_buy() {
        let mut config = regular_config();
        config.devBuy = Some(crate::config::NormalizedDevBuy {
            mode: "sol".to_string(),
            amount: "0.5".to_string(),
            source: "test".to_string(),
        });
        let global = PumpGlobalState {
            fee_recipient: Pubkey::new_unique(),
            initial_virtual_token_reserves: 1_073_000_000_000_000,
            initial_virtual_sol_reserves: 30_000_000_000,
            initial_real_token_reserves: 793_100_000_000_000,
            fee_basis_points: 100,
            creator_fee_basis_points: 50,
            fee_recipients: [Pubkey::default(); 7],
            reserved_fee_recipient: Pubkey::default(),
            reserved_fee_recipients: [Pubkey::default(); 7],
        };
        let mint = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let launch_creator = Pubkey::new_unique();

        let instructions =
            build_launch_instructions(&config, mint, creator, launch_creator, None, Some(&global))
                .expect("launch instructions");

        assert_eq!(instructions.len(), 4);
        assert_eq!(instructions[0].program_id.to_string(), PUMP_PROGRAM_ID);
        assert_eq!(instructions[1].program_id.to_string(), PUMP_PROGRAM_ID);
        assert_eq!(
            instructions[2].program_id.to_string(),
            spl_associated_token_account::id().to_string()
        );
        assert_eq!(instructions[3].program_id.to_string(), PUMP_PROGRAM_ID);
        assert_eq!(
            &instructions[3].data[..8],
            &[102, 6, 61, 18, 1, 218, 235, 234]
        );

        let quoted_sol_amount =
            quote_buy_sol_from_tokens(&global, quote_buy_tokens_from_sol(&global, 500_000_000));
        let expected_max_sol_cost = apply_atomic_buy_buffer(quoted_sol_amount);
        assert_eq!(
            &instructions[3].data[16..24],
            &expected_max_sol_cost.to_le_bytes()
        );
    }

    #[test]
    fn agent_locked_launch_keeps_agent_initialize_in_creation_tx() {
        let mut config = regular_config();
        config.mode = "agent-locked".to_string();
        config.agent.buybackBps = Some(2_500);
        config.creatorFee.mode = "agent-escrow".to_string();
        let mint = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let launch_creator = Pubkey::new_unique();
        let agent_authority = Pubkey::new_unique();

        let instructions = build_launch_instructions(
            &config,
            mint,
            creator,
            launch_creator,
            Some(&agent_authority),
            None,
        )
        .expect("agent locked launch instructions");

        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0].program_id.to_string(), PUMP_PROGRAM_ID);
        assert_eq!(
            instructions[1].program_id.to_string(),
            PUMP_AGENT_PAYMENTS_PROGRAM_ID
        );
    }

    #[test]
    fn agent_custom_launch_keeps_agent_initialize_in_creation_tx() {
        let mut config = regular_config();
        config.mode = "agent-custom".to_string();
        config.agent.buybackBps = Some(2_500);
        config.agent.feeRecipients = vec![
            crate::config::NormalizedRecipient {
                r#type: Some("agent".to_string()),
                address: String::new(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 2_500,
            },
            crate::config::NormalizedRecipient {
                r#type: Some("wallet".to_string()),
                address: Pubkey::new_unique().to_string(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 7_500,
            },
        ];
        let mint = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let launch_creator = Pubkey::new_unique();
        let agent_authority = Pubkey::new_unique();

        let instructions = build_launch_instructions(
            &config,
            mint,
            creator,
            launch_creator,
            Some(&agent_authority),
            None,
        )
        .expect("agent custom launch instructions");

        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0].program_id.to_string(), PUMP_PROGRAM_ID);
        assert_eq!(
            instructions[1].program_id.to_string(),
            PUMP_AGENT_PAYMENTS_PROGRAM_ID
        );
    }

    #[test]
    fn compute_budget_instruction_layout_matches_web3() {
        let compute_unit_limit = 340_000u32;
        let limit =
            build_compute_unit_limit_instruction(compute_unit_limit).expect("limit instruction");
        let price = build_compute_unit_price_instruction(123_456).expect("price instruction");

        assert_eq!(limit.program_id.to_string(), COMPUTE_BUDGET_PROGRAM_ID);
        assert_eq!(
            limit.data,
            [&[2u8][..], &compute_unit_limit.to_le_bytes()[..]].concat()
        );
        assert_eq!(price.program_id.to_string(), COMPUTE_BUDGET_PROGRAM_ID);
        assert_eq!(price.data, [3, 64, 226, 1, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn social_fee_pda_instruction_shape_matches_expected_accounts() {
        let payer = Pubkey::new_unique();
        let instruction = build_create_social_fee_pda_instruction(&payer, "12345", PLATFORM_GITHUB)
            .expect("social fee instruction");

        assert_eq!(instruction.program_id.to_string(), PUMP_FEE_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 6);
        assert_eq!(
            &instruction.data[..8],
            &[144, 224, 59, 211, 78, 248, 202, 220]
        );
    }

    #[test]
    fn follow_sell_uses_launch_creator_until_sharing_config_exists() {
        let launch_creator = Pubkey::new_unique();
        let sharing_config = Pubkey::new_unique();

        let authority =
            select_follow_creator_vault_authority(&launch_creator, &sharing_config, true, false);

        assert_eq!(authority, launch_creator);
    }

    #[test]
    fn follow_sell_switches_to_sharing_config_once_live() {
        let launch_creator = Pubkey::new_unique();
        let sharing_config = Pubkey::new_unique();

        let authority =
            select_follow_creator_vault_authority(&launch_creator, &sharing_config, true, true);

        assert_eq!(authority, sharing_config);
    }

    #[test]
    fn create_fee_sharing_config_instruction_matches_live_account_shape() {
        let mint = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let instruction = build_create_fee_sharing_config_instruction(&mint, &payer)
            .expect("create fee sharing config instruction");

        assert_eq!(instruction.program_id.to_string(), PUMP_FEE_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 13);
        assert_eq!(
            instruction.accounts[10].pubkey,
            pump_fee_program_id().expect("fee program id")
        );
        assert_eq!(
            instruction.accounts[11].pubkey,
            pump_amm_program_id().expect("pump amm program id")
        );
        assert_eq!(
            instruction.accounts[12].pubkey,
            pump_fee_program_id().expect("fee program id")
        );
        assert_eq!(
            &instruction.data[..8],
            &[195, 78, 86, 76, 111, 52, 251, 213]
        );
    }

    #[test]
    fn agent_initialize_instruction_shape_matches_sdk_layout() {
        let mint = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let agent_authority = Pubkey::new_unique();
        let instruction =
            build_agent_initialize_instruction(&mint, &creator, &agent_authority, 2_500)
                .expect("agent initialize instruction");

        assert_eq!(
            instruction.program_id.to_string(),
            PUMP_AGENT_PAYMENTS_PROGRAM_ID
        );
        assert_eq!(instruction.accounts.len(), 8);
        assert_eq!(&instruction.data[..8], &[180, 248, 163, 8, 49, 94, 126, 96]);
        assert_eq!(&instruction.data[8..40], agent_authority.as_ref(),);
        assert_eq!(&instruction.data[40..42], &2_500u16.to_le_bytes());
    }

    #[tokio::test]
    async fn fee_sharing_follow_up_contains_two_instructions_without_revoke() {
        let mut config = regular_config();
        config.feeSharing.generateLaterSetup = true;
        config.feeSharing.recipients = vec![crate::config::NormalizedRecipient {
            r#type: Some("wallet".to_string()),
            address: Pubkey::new_unique().to_string(),
            githubUserId: String::new(),
            githubUsername: String::new(),
            shareBps: 10_000,
        }];
        let instructions = build_fee_sharing_follow_up_instructions(
            "http://127.0.0.1:8899",
            &config,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        )
        .await
        .expect("follow-up instructions");

        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0].program_id.to_string(), PUMP_FEE_PROGRAM_ID);
        assert_eq!(instructions[1].program_id.to_string(), PUMP_FEE_PROGRAM_ID);
    }

    #[tokio::test]
    async fn agent_locked_follow_up_only_contains_fee_setup() {
        let mut config = regular_config();
        config.mode = "agent-locked".to_string();
        config.agent.buybackBps = Some(2_500);
        config.creatorFee.mode = "agent-escrow".to_string();
        let creator = Pubkey::new_unique();
        let instructions = build_native_follow_up_instructions(
            "http://127.0.0.1:8899",
            &config,
            Pubkey::new_unique(),
            creator,
            None,
        )
        .await
        .expect("agent locked follow-up instructions");

        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0].program_id.to_string(), PUMP_FEE_PROGRAM_ID);
        assert_eq!(instructions[1].program_id.to_string(), PUMP_FEE_PROGRAM_ID);
    }

    #[tokio::test]
    async fn agent_custom_follow_up_only_contains_fee_setup() {
        let mut config = regular_config();
        config.mode = "agent-custom".to_string();
        config.agent.buybackBps = Some(2_500);
        config.agent.feeRecipients = vec![
            crate::config::NormalizedRecipient {
                r#type: Some("agent".to_string()),
                address: String::new(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 2_500,
            },
            crate::config::NormalizedRecipient {
                r#type: Some("wallet".to_string()),
                address: Pubkey::new_unique().to_string(),
                githubUserId: String::new(),
                githubUsername: String::new(),
                shareBps: 7_500,
            },
        ];
        let creator = Pubkey::new_unique();
        let instructions = build_native_follow_up_instructions(
            "http://127.0.0.1:8899",
            &config,
            Pubkey::new_unique(),
            creator,
            None,
        )
        .await
        .expect("agent custom follow-up instructions");

        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0].program_id.to_string(), PUMP_FEE_PROGRAM_ID);
        assert_eq!(instructions[1].program_id.to_string(), PUMP_FEE_PROGRAM_ID);
    }

    #[test]
    fn agent_unlocked_has_no_follow_up_transaction() {
        let mut config = regular_config();
        config.mode = "agent-unlocked".to_string();
        config.agent.buybackBps = Some(2_500);

        assert_eq!(native_follow_up_label(&config), None);
    }

    #[test]
    fn standard_rpc_follow_buy_tip_is_ignored_when_blank() {
        let (tip_lamports, tip_account) =
            resolve_follow_tip_config("standard-rpc", "", "tip-account", "buy tip")
                .expect("standard rpc buy tip");
        assert_eq!(tip_lamports, 0);
        assert!(tip_account.is_empty());
    }

    #[test]
    fn standard_rpc_follow_sell_tip_is_ignored_even_when_present() {
        let (tip_lamports, tip_account) =
            resolve_follow_tip_config("standard-rpc", "0.01", "tip-account", "sell tip")
                .expect("standard rpc sell tip");
        assert_eq!(tip_lamports, 0);
        assert!(tip_account.is_empty());
    }

    #[test]
    fn jito_follow_tip_still_requires_valid_tip_value() {
        let error = resolve_follow_tip_config("jito-bundle", "", "tip-account", "buy tip")
            .expect_err("blank jito tip should fail");
        assert!(error.contains("buy tip"));
    }

    #[test]
    fn hellomoon_follow_tip_requires_at_least_point_zero_zero_one_sol() {
        let (lamports, account) = resolve_follow_tip_config(
            "hellomoon",
            "0.001",
            "tip-account",
            "buy tip",
        )
        .expect("valid hellomoon tip");
        assert_eq!(lamports, 1_000_000);
        assert_eq!(account, "tip-account");
        let error = resolve_follow_tip_config("hellomoon", "0.0001", "tip-account", "buy tip")
            .expect_err("sub-minimum hellomoon tip should fail");
        assert!(error.contains("0.001 SOL"), "unexpected: {error}");
    }

    #[test]
    fn agent_custom_without_init_has_no_follow_up_transaction() {
        let mut config = regular_config();
        config.mode = "agent-custom".to_string();
        config.agent.buybackBps = Some(0);
        config.agent.splitAgentInit = false;
        config.agent.feeRecipients = vec![];

        assert_eq!(native_follow_up_label(&config), None);
    }

    #[test]
    fn update_fee_shares_uses_current_shareholders_as_remaining_accounts() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let current_shareholder = Pubkey::new_unique();
        let next_shareholder = NormalizedRecipient {
            r#type: Some("wallet".to_string()),
            address: Pubkey::new_unique().to_string(),
            githubUserId: String::new(),
            githubUsername: String::new(),
            shareBps: 10_000,
        };

        let instruction = build_update_fee_shares_instruction(
            &mint,
            &authority,
            &[current_shareholder],
            &[next_shareholder],
        )
        .expect("update fee shares instruction");

        assert_eq!(instruction.program_id.to_string(), PUMP_FEE_PROGRAM_ID);
        assert_eq!(
            instruction.accounts.last().map(|account| account.pubkey),
            Some(current_shareholder)
        );
        assert!(
            instruction
                .accounts
                .last()
                .map(|account| account.is_writable)
                .unwrap_or(false)
        );
        assert!(
            !instruction
                .accounts
                .last()
                .map(|account| account.is_signer)
                .unwrap_or(true)
        );
    }
}
