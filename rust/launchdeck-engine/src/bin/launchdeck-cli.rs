#[path = "../app_logs.rs"]
mod app_logs;
#[path = "../bags_native.rs"]
mod bags_native;
#[path = "../bonk_native.rs"]
mod bonk_native;
#[path = "../config.rs"]
mod config;
#[path = "../endpoint_profile.rs"]
mod endpoint_profile;
#[path = "../fs_utils.rs"]
mod fs_utils;
#[path = "../helper_worker.rs"]
mod helper_worker;
#[path = "../launchpad_dispatch.rs"]
mod launchpad_dispatch;
#[path = "../observability.rs"]
mod observability;
#[path = "../paths.rs"]
mod paths;
#[path = "../providers.rs"]
mod providers;
#[path = "../provider_tip.rs"]
mod provider_tip;
#[path = "../pump_native.rs"]
mod pump_native;
#[path = "../report.rs"]
mod report;
#[path = "../reports_browser.rs"]
mod reports_browser;
#[path = "../rpc.rs"]
mod rpc;
#[path = "../transport.rs"]
mod transport;
#[path = "../wallet.rs"]
mod wallet;

use clap::{Parser, Subcommand};
use config::{RawConfig, normalize_raw_config};
use launchpad_dispatch::try_compile_native_launchpad;
use observability::{new_trace_context, persist_launch_report};
use rpc::{send_transactions_for_transport, simulate_transactions};
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use transport::{build_transport_plan, estimate_transaction_count};
use wallet::{
    load_solana_wallet_by_env_key, public_key_from_secret, selected_wallet_key_or_default,
};

#[derive(Parser, Debug)]
#[command(name = "launchdeck-cli")]
#[command(about = "Rust-native LaunchDeck CLI for build, simulate, and send")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Build(CommonArgs),
    Simulate(CommonArgs),
    Send(CommonArgs),
}

#[derive(clap::Args, Debug, Clone)]
struct CommonArgs {
    #[arg(long)]
    config: PathBuf,
    #[arg(long)]
    wallet: Option<String>,
    #[arg(long)]
    json: bool,
}

fn configured_rpc_url() -> String {
    if let Ok(explicit) = env::var("SOLANA_RPC_URL") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "http://127.0.0.1:8899".to_string()
}

fn now_timestamp_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    millis.to_string()
}

fn set_report_timing(report: &mut Value, key: &str, value_ms: u128) {
    if let Some(execution) = report.get_mut("execution") {
        if execution.get("timings").is_none()
            || execution.get("timings").is_some_and(Value::is_null)
        {
            execution["timings"] = json!({});
        }
        execution["timings"][key] = Value::from(value_ms as u64);
    }
}

fn set_optional_report_timing(report: &mut Value, key: &str, value_ms: Option<u128>) {
    if let Some(value_ms) = value_ms {
        set_report_timing(report, key, value_ms);
    }
}

fn read_raw_config(path: &Path) -> Result<RawConfig, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read config {}: {error}", path.display()))?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if extension == "json" {
        return serde_json::from_str(&raw)
            .map_err(|error| format!("Invalid JSON config {}: {error}", path.display()));
    }

    serde_yaml::from_str(&raw)
        .or_else(|_| serde_json::from_str(&raw))
        .map_err(|error| format!("Invalid config {}: {error}", path.display()))
}

fn print_human_output(action: &str, text: &str, extra: Option<&Value>) -> Result<(), String> {
    println!("{text}");
    if let Some(payload) = extra {
        println!();
        match action {
            "simulate" => println!("Simulation:"),
            "send" => println!("Send:"),
            _ => println!("Result:"),
        }
        println!(
            "{}",
            serde_json::to_string_pretty(payload).map_err(|error| error.to_string())?
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    if let Err(error) = run_cli().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run_cli() -> Result<(), String> {
    let cli = Cli::parse();
    let trace = new_trace_context();
    let (action, args) = match cli.command {
        Command::Build(args) => ("build", args),
        Command::Simulate(args) => ("simulate", args),
        Command::Send(args) => ("send", args),
    };

    let mut raw = read_raw_config(&args.config)?;
    raw.execution.simulate = Some(Value::Bool(action == "simulate"));
    raw.execution.send = Some(Value::Bool(action == "send"));
    if let Some(wallet_key) = &args.wallet {
        raw.selectedWalletKey = wallet_key.clone();
    }

    let selected_wallet_key =
        selected_wallet_key_or_default(&raw.selectedWalletKey).ok_or_else(|| {
            "Creator keypair is required via --wallet, selectedWalletKey, or SOLANA_PRIVATE_KEY."
                .to_string()
        })?;
    let normalized = normalize_raw_config(raw).map_err(|error| error.to_string())?;
    let wallet_secret = load_solana_wallet_by_env_key(&selected_wallet_key)?;
    let creator_public_key = public_key_from_secret(&wallet_secret)?;
    let rpc_url = configured_rpc_url();
    let transport_plan = build_transport_plan(
        &normalized.execution,
        estimate_transaction_count(&normalized),
    );

    let native = try_compile_native_launchpad(
        &rpc_url,
        &normalized,
        &transport_plan,
        &wallet_secret,
        now_timestamp_string(),
        creator_public_key,
        Some(format!("Rust CLI {}", args.config.display())),
        false,
    )
    .await?
    .ok_or_else(|| {
        format!(
            "Native Rust engine does not support launchpad={} mode={} yet.",
            normalized.launchpad, normalized.mode
        )
    })?;

    let compiled_transactions = native.compiled_transactions;
    let mut report = native.report;
    let text = native.text;
    let compile_timings = native.compile_timings;
    set_report_timing(&mut report, "compileAltLoadMs", compile_timings.alt_load_ms);
    set_report_timing(
        &mut report,
        "compileBlockhashFetchMs",
        compile_timings.blockhash_fetch_ms,
    );
    set_optional_report_timing(
        &mut report,
        "compileGlobalFetchMs",
        compile_timings.global_fetch_ms,
    );
    set_optional_report_timing(
        &mut report,
        "compileFollowUpPrepMs",
        compile_timings.follow_up_prep_ms,
    );
    set_report_timing(
        &mut report,
        "compileTxSerializeMs",
        compile_timings.tx_serialize_ms,
    );
    let mut extra = None;
    let should_persist_report = normalized.tx.writeReport || action == "send";

    if action == "simulate" {
        let (simulation, warnings) = simulate_transactions(
            &rpc_url,
            &compiled_transactions,
            &normalized.execution.commitment,
        )
        .await?;
        if let Some(execution) = report.get_mut("execution") {
            execution["simulation"] =
                serde_json::to_value(&simulation).map_err(|error| error.to_string())?;
            let mut existing_warnings = execution
                .get("warnings")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            existing_warnings.extend(warnings.into_iter().map(Value::String));
            execution["warnings"] = Value::Array(existing_warnings);
        }
        extra = Some(json!(simulation));
    } else if action == "send" {
        let (sent, warnings, send_timing) = send_transactions_for_transport(
            &rpc_url,
            &transport_plan,
            &compiled_transactions,
            &normalized.execution.commitment,
            normalized.execution.skipPreflight,
            normalized.execution.trackSendBlockHeight,
        )
        .await?;
        set_report_timing(
            &mut report,
            "sendMs",
            send_timing.submit_ms.saturating_add(send_timing.confirm_ms),
        );
        set_report_timing(&mut report, "sendSubmitMs", send_timing.submit_ms);
        set_report_timing(&mut report, "sendConfirmMs", send_timing.confirm_ms);
        if let Some(execution) = report.get_mut("execution") {
            execution["sent"] = serde_json::to_value(&sent).map_err(|error| error.to_string())?;
            let mut existing_warnings = execution
                .get("warnings")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            existing_warnings.extend(warnings.into_iter().map(Value::String));
            execution["warnings"] = Value::Array(existing_warnings);
        }
        extra = Some(json!(sent));
    }

    let send_log_path = if should_persist_report {
        let path = persist_launch_report(&trace.traceId, action, &transport_plan, &report)?;
        report["outPath"] = Value::String(path.clone());
        Some(path)
    } else {
        None
    };

    let payload = json!({
        "ok": true,
        "service": "launchdeck-cli",
        "action": action,
        "executor": "rust-native",
        "assemblyExecutor": "rust-native",
        "walletKey": selected_wallet_key,
        "normalizedConfig": normalized,
        "transportPlan": transport_plan,
        "report": report,
        "sendLogPath": send_log_path,
        "text": text,
        "compiledTransactions": compiled_transactions,
    });

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?
        );
        return Ok(());
    }

    print_human_output(
        action,
        payload["text"].as_str().unwrap_or_default(),
        extra.as_ref(),
    )
}
