use clap::{Args, Parser, Subcommand};
use serde_json::{Value, json};
use std::{collections::HashMap, env, time::Duration};

const DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const PROGRAM_PUMP: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const PROGRAM_AGENT: &str = "AgenTMiC2hvxGebTsgmsD4HHBa8WEcqGFf87iwRRxLo7";
const PROGRAM_ASSOCIATED_TOKEN: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const PROGRAM_SYSTEM: &str = "11111111111111111111111111111111";

#[derive(Parser, Debug)]
#[command(name = "launchdeck-debug-cli")]
#[command(about = "Rust-native LaunchDeck diagnostics")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(name = "analyze-launch")]
    AnalyzeLaunch(AnalyzeArgs),
    #[command(name = "trace-agent-account")]
    TraceAgentAccount(TraceArgs),
}

#[derive(Args, Debug)]
struct AnalyzeArgs {
    #[arg(long)]
    tx: String,
    #[arg(long = "rpc-url")]
    rpc_url: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct TraceArgs {
    #[arg(long)]
    address: String,
    #[arg(long, default_value_t = 10)]
    limit: usize,
    #[arg(long = "rpc-url")]
    rpc_url: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Default)]
struct InstructionIndexes {
    create: isize,
    extend: isize,
    buy: isize,
    buy_exact_sol_in: isize,
    agent_initialize: isize,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    if let Err(error) = run().await {
        eprintln!("\nError: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Command::AnalyzeLaunch(args) => analyze_launch(args).await,
        Command::TraceAgentAccount(args) => trace_agent_account(args).await,
    }
}

fn configured_rpc_url(override_url: Option<&str>) -> String {
    if let Some(value) = override_url {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Ok(explicit) = env::var("SOLANA_RPC_URL") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    DEFAULT_RPC_URL.to_string()
}

async fn rpc_json(url: &str, method: &str, params: Value) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .post(url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let payload: Value = response.json().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, payload));
    }
    if let Some(error) = payload.get("error") {
        return Err(error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("RPC request failed.")
            .to_string());
    }
    Ok(payload.get("result").cloned().unwrap_or(Value::Null))
}

async fn rpc_json_with_retry(url: &str, method: &str, params: Value) -> Result<Value, String> {
    let mut last_error = String::new();
    for attempt in 0..5u64 {
        match rpc_json(url, method, params.clone()).await {
            Ok(value) => return Ok(value),
            Err(error) => {
                let lower = error.to_lowercase();
                let rate_limited = lower.contains("429")
                    || lower.contains("-32429")
                    || lower.contains("rate limit")
                    || lower.contains("too many requests");
                last_error = error;
                if !rate_limited || attempt == 4 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis((500 * (1u64 << attempt)).min(5_000)))
                    .await;
            }
        }
    }
    Err(last_error)
}

async fn get_transaction(rpc_url: &str, signature: &str) -> Result<Value, String> {
    rpc_json_with_retry(
        rpc_url,
        "getTransaction",
        json!([
            signature,
            {
                "encoding": "jsonParsed",
                "maxSupportedTransactionVersion": 0,
                "commitment": "confirmed",
            }
        ]),
    )
    .await
}

async fn get_account_info(rpc_url: &str, address: &str) -> Result<Value, String> {
    rpc_json_with_retry(
        rpc_url,
        "getAccountInfo",
        json!([
            address,
            {
                "encoding": "jsonParsed",
                "commitment": "confirmed",
            }
        ]),
    )
    .await
}

async fn get_signatures_for_address(
    rpc_url: &str,
    address: &str,
    limit: usize,
) -> Result<Vec<Value>, String> {
    Ok(rpc_json_with_retry(
        rpc_url,
        "getSignaturesForAddress",
        json!([
            address,
            {
                "limit": limit,
                "commitment": "confirmed",
            }
        ]),
    )
    .await?
    .as_array()
    .cloned()
    .unwrap_or_default())
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

fn instruction_array(tx: &Value) -> Vec<Value> {
    tx.get("transaction")
        .and_then(|value| value.get("message"))
        .and_then(|value| value.get("instructions"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn account_keys_array(tx: &Value) -> Vec<Value> {
    tx.get("transaction")
        .and_then(|value| value.get("message"))
        .and_then(|value| value.get("accountKeys"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn log_array(tx: &Value) -> Vec<Value> {
    tx.get("meta")
        .and_then(|value| value.get("logMessages"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn account_strings(ix: &Value) -> Vec<String> {
    ix.get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect()
}

fn instruction_label(ix: &Value) -> String {
    let program_id = ix
        .get("programId")
        .and_then(Value::as_str)
        .unwrap_or("(unknown)");
    let parsed_type = ix
        .get("parsed")
        .and_then(|parsed| parsed.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if program_id == PROGRAM_ASSOCIATED_TOKEN && !parsed_type.is_empty() {
        return format!("ATA {parsed_type}");
    }
    if program_id == PROGRAM_SYSTEM && !parsed_type.is_empty() {
        return format!("System {parsed_type}");
    }
    program_id.to_string()
}

fn collect_interesting_logs(logs: &[Value]) -> Vec<String> {
    logs.iter()
        .filter_map(Value::as_str)
        .filter(|entry| entry.contains("Instruction:"))
        .map(|entry| {
            entry
                .strip_prefix("Program log: Instruction: ")
                .unwrap_or(entry)
                .trim()
                .to_string()
        })
        .collect()
}

fn build_top_level_log_map(tx: &Value) -> Vec<Vec<String>> {
    let instructions = instruction_array(tx);
    let mut by_program: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, ix) in instructions.iter().enumerate() {
        if let Some(program_id) = ix.get("programId").and_then(Value::as_str) {
            by_program
                .entry(program_id.to_string())
                .or_default()
                .push(index);
        }
    }

    let mut used_counts: HashMap<String, usize> = HashMap::new();
    let mut mapped_logs = vec![Vec::new(); instructions.len()];
    let mut current_instruction_index: Option<usize> = None;

    for entry in log_array(tx).iter().filter_map(Value::as_str) {
        if entry.starts_with("Program ") && entry.ends_with(" invoke [1]") {
            let program_id = entry
                .strip_prefix("Program ")
                .and_then(|value| value.strip_suffix(" invoke [1]"))
                .unwrap_or_default();
            let matches = by_program.get(program_id).cloned().unwrap_or_default();
            let offset = used_counts.get(program_id).copied().unwrap_or(0);
            current_instruction_index = matches.get(offset).copied();
            used_counts.insert(program_id.to_string(), offset + 1);
            continue;
        }

        if let Some(index) = current_instruction_index {
            if let Some(bucket) = mapped_logs.get_mut(index) {
                bucket.push(entry.to_string());
            }
        }
    }

    mapped_logs
}

fn find_instruction_indexes(tx: &Value) -> InstructionIndexes {
    let instructions = instruction_array(tx);
    let logs_by_instruction = build_top_level_log_map(tx);
    let mut indexes = InstructionIndexes {
        create: -1,
        extend: -1,
        buy: -1,
        buy_exact_sol_in: -1,
        agent_initialize: -1,
    };

    for (index, ix) in instructions.iter().enumerate() {
        let program_id = ix
            .get("programId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let logs = logs_by_instruction.get(index).cloned().unwrap_or_default();
        if program_id == PROGRAM_PUMP && logs.iter().any(|entry| entry.contains("CreateV2")) {
            indexes.create = index as isize;
        }
        if program_id == PROGRAM_PUMP && logs.iter().any(|entry| entry.contains("ExtendAccount")) {
            indexes.extend = index as isize;
        }
        if program_id == PROGRAM_PUMP && logs.iter().any(|entry| entry.contains("BuyExactSolIn")) {
            indexes.buy_exact_sol_in = index as isize;
        }
        if program_id == PROGRAM_PUMP
            && indexes.buy == -1
            && logs.iter().any(|entry| entry.contains("Instruction: Buy"))
        {
            indexes.buy = index as isize;
        }
        if program_id == PROGRAM_AGENT
            && logs
                .iter()
                .any(|entry| entry.contains("Instruction: AgentInitialize"))
        {
            indexes.agent_initialize = index as isize;
        }
    }

    indexes
}

fn detect_launch_flavor(indexes: &InstructionIndexes) -> &'static str {
    if indexes.agent_initialize == -1 {
        "pump-normal"
    } else {
        "pump-agent"
    }
}

fn summarize_instruction(ix: &Value, index: usize) -> Value {
    let accounts = account_strings(ix);
    json!({
        "index": index,
        "programId": ix.get("programId").cloned().unwrap_or(Value::Null),
        "label": instruction_label(ix),
        "accountCount": accounts.len(),
        "data": ix.get("data").cloned().unwrap_or_else(|| ix.get("parsed").and_then(|parsed| parsed.get("type")).cloned().unwrap_or(Value::Null)),
    })
}

fn build_analyze_result(signature: &str, tx: &Value, fee_account_info: Option<Value>) -> Value {
    let instructions = instruction_array(tx);
    let logs = collect_interesting_logs(&log_array(tx));
    let indexes = find_instruction_indexes(tx);
    let flavor = detect_launch_flavor(&indexes);

    let get_ix = |index: isize| -> Option<&Value> {
        if index < 0 {
            None
        } else {
            instructions.get(index as usize)
        }
    };

    let agent_instruction = get_ix(indexes.agent_initialize);
    let create_instruction = get_ix(indexes.create);
    let buy_instruction_index = if indexes.buy_exact_sol_in >= 0 {
        indexes.buy_exact_sol_in
    } else {
        indexes.buy
    };
    let buy_instruction = get_ix(buy_instruction_index);

    let mint = agent_instruction
        .and_then(|ix| account_strings(ix).get(3).cloned())
        .or_else(|| create_instruction.and_then(|ix| account_strings(ix).first().cloned()));
    let agent_fee_receiver = agent_instruction.and_then(|ix| account_strings(ix).get(4).cloned());
    let creator_wallet = agent_instruction
        .and_then(|ix| account_strings(ix).first().cloned())
        .or_else(|| create_instruction.and_then(|ix| account_strings(ix).get(5).cloned()));

    json!({
        "signature": signature,
        "slot": tx.get("slot").cloned().unwrap_or(Value::Null),
        "blockTime": tx.get("blockTime").cloned().unwrap_or(Value::Null),
        "flavor": flavor,
        "logs": logs,
        "instructionIndexes": {
            "create": indexes.create,
            "extend": indexes.extend,
            "buy": indexes.buy,
            "buyExactSolIn": indexes.buy_exact_sol_in,
            "agentInitialize": indexes.agent_initialize,
        },
        "instructionSummary": instructions.iter().enumerate().map(|(index, ix)| summarize_instruction(ix, index)).collect::<Vec<_>>(),
        "createInstruction": create_instruction.map(|ix| json!({
            "index": indexes.create,
            "accounts": account_strings(ix),
            "data": ix.get("data").cloned().unwrap_or(Value::Null),
        })).unwrap_or(Value::Null),
        "buyInstruction": buy_instruction.map(|ix| json!({
            "index": buy_instruction_index,
            "programId": ix.get("programId").cloned().unwrap_or(Value::Null),
            "accounts": account_strings(ix),
            "data": ix.get("data").cloned().unwrap_or(Value::Null),
        })).unwrap_or(Value::Null),
        "agentInitialize": agent_instruction.map(|ix| json!({
            "index": indexes.agent_initialize,
            "programId": ix.get("programId").cloned().unwrap_or(Value::Null),
            "accounts": account_strings(ix),
            "data": ix.get("data").cloned().unwrap_or(Value::Null),
        })).unwrap_or(Value::Null),
        "extracted": {
            "mint": mint,
            "creatorWallet": creator_wallet,
            "agentFeeReceiver": agent_fee_receiver,
            "feeReceiverAccountInfo": fee_account_info.unwrap_or(Value::Null),
        }
    })
}

fn render_analyze_result(result: &Value) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Signature: {}",
        result
            .get("signature")
            .and_then(Value::as_str)
            .unwrap_or("(unknown)")
    ));
    lines.push(format!(
        "Flavor: {}",
        result
            .get("flavor")
            .and_then(Value::as_str)
            .unwrap_or("(unknown)")
    ));
    lines.push(format!(
        "Mint: {}",
        result
            .get("extracted")
            .and_then(|value| value.get("mint"))
            .and_then(Value::as_str)
            .unwrap_or("(unknown)")
    ));
    lines.push(format!(
        "Creator wallet: {}",
        result
            .get("extracted")
            .and_then(|value| value.get("creatorWallet"))
            .and_then(Value::as_str)
            .unwrap_or("(unknown)")
    ));
    lines.push(format!(
        "Agent fee receiver: {}",
        result
            .get("extracted")
            .and_then(|value| value.get("agentFeeReceiver"))
            .and_then(Value::as_str)
            .unwrap_or("(none)")
    ));

    if let Some(info) = result
        .get("extracted")
        .and_then(|value| value.get("feeReceiverAccountInfo"))
        .filter(|value| !value.is_null())
    {
        lines.push(format!(
            "Agent fee receiver owner: {} | space={} | lamports={}",
            info.get("owner")
                .and_then(Value::as_str)
                .unwrap_or("(unknown)"),
            info.get("space").cloned().unwrap_or(Value::Null),
            info.get("lamports").cloned().unwrap_or(Value::Null)
        ));
    }

    lines.push(String::new());
    lines.push("Instruction order:".to_string());
    if let Some(items) = result.get("instructionSummary").and_then(Value::as_array) {
        for item in items {
            lines.push(format!(
                "- [{}] {} | accounts={} | data={}",
                item.get("index").cloned().unwrap_or(Value::Null),
                item.get("label")
                    .and_then(Value::as_str)
                    .unwrap_or("(unknown)"),
                item.get("accountCount").cloned().unwrap_or(Value::Null),
                item.get("data")
                    .and_then(Value::as_str)
                    .unwrap_or("(parsed only)")
            ));
        }
    }

    if let Some(agent_initialize) = result
        .get("agentInitialize")
        .filter(|value| !value.is_null())
    {
        lines.push(String::new());
        lines.push("AgentInitialize accounts:".to_string());
        if let Some(accounts) = agent_initialize.get("accounts").and_then(Value::as_array) {
            for (index, account) in accounts.iter().enumerate() {
                let suffix = if index == 4 {
                    "  <- agent fee receiver / buyback escrow"
                } else {
                    ""
                };
                lines.push(format!(
                    "- [{}] {}{}",
                    index,
                    account.as_str().unwrap_or("(unknown)"),
                    suffix
                ));
            }
        }
    }

    lines.push(String::new());
    lines.push("Interesting logs:".to_string());
    if let Some(items) = result.get("logs").and_then(Value::as_array) {
        for entry in items.iter().filter_map(Value::as_str) {
            lines.push(format!("- {entry}"));
        }
    }

    lines.push(String::new());
    lines.push("Interpretation:".to_string());
    match result
        .get("flavor")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "pump-normal" => {
            lines.push("- Normal Pump launch: create + buy, no agent initialization.".to_string())
        }
        "pump-agent" => lines.push(
            "- Native Pump agent launch: create + buy + AgentInitialize in the same transaction."
                .to_string(),
        ),
        _ => lines.push("- Launch flavor could not be classified confidently.".to_string()),
    }
    if result
        .get("extracted")
        .and_then(|value| value.get("agentFeeReceiver"))
        .and_then(Value::as_str)
        .is_some()
    {
        lines.push(
            "- The unique agent-owned fee receiver is the escrow that later accumulates creator fees for buybacks."
                .to_string(),
        );
    }

    lines.join("\n")
}

fn summarize_tx(signature: &str, tx: &Value, address: &str) -> Value {
    let account_keys = account_keys_array(tx);
    let target_index = account_keys.iter().position(|item| {
        item.get("pubkey")
            .and_then(Value::as_str)
            .is_some_and(|entry| entry == address)
    });
    let logs = log_array(tx);
    let instruction_logs = logs
        .iter()
        .filter_map(Value::as_str)
        .filter(|entry| entry.contains("Instruction:"))
        .map(|entry| {
            entry
                .strip_prefix("Program log: Instruction: ")
                .unwrap_or(entry)
                .trim()
                .to_string()
        })
        .collect::<Vec<_>>();

    json!({
        "signature": signature,
        "slot": tx.get("slot").cloned().unwrap_or(Value::Null),
        "blockTime": tx.get("blockTime").cloned().unwrap_or(Value::Null),
        "targetAccountIndex": target_index.map(|index| index as i64),
        "targetWritable": target_index.and_then(|index| account_keys.get(index).and_then(|entry| entry.get("writable")).and_then(Value::as_bool)).unwrap_or(false),
        "targetSigner": target_index.and_then(|index| account_keys.get(index).and_then(|entry| entry.get("signer")).and_then(Value::as_bool)).unwrap_or(false),
        "topLevelPrograms": instruction_array(tx).iter().filter_map(|ix| ix.get("programId").and_then(Value::as_str).map(str::to_string)).collect::<Vec<_>>(),
        "instructionLogs": instruction_logs,
        "hasBuybackTrigger": instruction_logs.iter().any(|entry| entry == "AgentBuybackTrigger"),
        "hasPaymentDistribution": instruction_logs.iter().any(|entry| entry == "AgentDistributePayments"),
        "hasExtraLamportsSweep": instruction_logs.iter().any(|entry| entry == "AgentTransferExtraLamports"),
        "hasBurn": logs.iter().filter_map(Value::as_str).any(|entry| entry.contains("BurnChecked")),
    })
}

fn render_trace_report(report: &Value) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Address: {}",
        report
            .get("address")
            .and_then(Value::as_str)
            .unwrap_or("(unknown)")
    ));
    let account_info = report.get("accountInfo").filter(|value| !value.is_null());
    lines.push(format!(
        "Owner: {} | space={} | lamports={}",
        account_info
            .and_then(|value| value.get("owner"))
            .and_then(Value::as_str)
            .unwrap_or("(unknown)"),
        account_info
            .and_then(|value| value.get("space"))
            .cloned()
            .unwrap_or(Value::Null),
        account_info
            .and_then(|value| value.get("lamports"))
            .cloned()
            .unwrap_or(Value::Null)
    ));
    let count = report
        .get("transactions")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    lines.push(format!("Recent transactions inspected: {count}"));
    lines.push(String::new());

    if account_info
        .and_then(|value| value.get("owner"))
        .and_then(Value::as_str)
        .is_some_and(|owner| owner == PROGRAM_AGENT)
    {
        lines.push("Interpretation:".to_string());
        lines.push(
            "- This is an agent-program-owned escrow/state account, not a normal wallet."
                .to_string(),
        );
        lines.push("- It can receive creator-fee revenue and later be referenced by the agent program during buybacks.".to_string());
        lines.push(String::new());
    }

    lines.push("Recent activity:".to_string());
    if let Some(items) = report.get("transactions").and_then(Value::as_array) {
        for tx in items {
            let mut tags = Vec::new();
            if tx
                .get("hasExtraLamportsSweep")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                tags.push("extra-lamports-sweep");
            }
            if tx
                .get("hasPaymentDistribution")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                tags.push("payment-distribution");
            }
            if tx
                .get("hasBuybackTrigger")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                tags.push("buyback-trigger");
            }
            if tx.get("hasBurn").and_then(Value::as_bool).unwrap_or(false) {
                tags.push("burn");
            }
            let signature = tx
                .get("signature")
                .and_then(Value::as_str)
                .unwrap_or_default();
            lines.push(format!(
                "- {} | writable={} | logs={}",
                short_address(signature, 6, 6),
                tx.get("targetWritable")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                if tags.is_empty() {
                    "none".to_string()
                } else {
                    tags.join(", ")
                }
            ));
        }
    }

    lines.join("\n")
}

async fn analyze_launch(args: AnalyzeArgs) -> Result<(), String> {
    let rpc_url = configured_rpc_url(args.rpc_url.as_deref());
    let tx = get_transaction(&rpc_url, &args.tx).await?;
    if tx.is_null() {
        return Err(format!("Transaction not found: {}", args.tx));
    }

    let tentative_agent_ix = instruction_array(&tx)
        .into_iter()
        .find(|ix| ix.get("programId").and_then(Value::as_str) == Some(PROGRAM_AGENT));
    let fee_account_info = if let Some(ix) = tentative_agent_ix {
        let accounts = account_strings(&ix);
        if let Some(address) = accounts.get(4) {
            get_account_info(&rpc_url, address)
                .await?
                .get("value")
                .cloned()
        } else {
            None
        }
    } else {
        None
    };

    let result = build_analyze_result(&args.tx, &tx, fee_account_info);
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|error| error.to_string())?
        );
    } else {
        println!("{}", render_analyze_result(&result));
    }
    Ok(())
}

async fn trace_agent_account(args: TraceArgs) -> Result<(), String> {
    let rpc_url = configured_rpc_url(args.rpc_url.as_deref());
    let limit = args.limit.clamp(1, 25);
    let account_info = get_account_info(&rpc_url, &args.address)
        .await?
        .get("value")
        .cloned()
        .unwrap_or(Value::Null);
    let signatures = get_signatures_for_address(&rpc_url, &args.address, limit).await?;

    let mut transactions = Vec::new();
    for item in signatures {
        let Some(signature) = item.get("signature").and_then(Value::as_str) else {
            continue;
        };
        let tx = get_transaction(&rpc_url, signature).await?;
        if tx.is_null() {
            continue;
        }
        transactions.push(summarize_tx(signature, &tx, &args.address));
    }

    let report = json!({
        "address": args.address,
        "accountInfo": account_info,
        "transactions": transactions,
    });
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
    } else {
        println!("{}", render_trace_report(&report));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_launch_is_classified_as_pump_agent() {
        let indexes = InstructionIndexes {
            create: 2,
            extend: 3,
            buy: 5,
            buy_exact_sol_in: -1,
            agent_initialize: 6,
        };

        assert_eq!(detect_launch_flavor(&indexes), "pump-agent");
    }

    #[test]
    fn non_agent_launch_is_classified_as_pump_normal() {
        let indexes = InstructionIndexes {
            create: 2,
            extend: 3,
            buy: 5,
            buy_exact_sol_in: -1,
            agent_initialize: -1,
        };

        assert_eq!(detect_launch_flavor(&indexes), "pump-normal");
    }
}
