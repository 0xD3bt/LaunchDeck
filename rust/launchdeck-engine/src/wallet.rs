#![allow(non_snake_case, dead_code)]

use futures_util::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::{
    collections::BTreeMap,
    env, fs,
    str::FromStr,
    sync::{Mutex, OnceLock},
    time::Duration,
};

#[derive(Debug, Clone, Serialize)]
pub struct WalletSummary {
    pub envKey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customName: Option<String>,
    pub publicKey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletStatusSummary {
    pub envKey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customName: Option<String>,
    pub publicKey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balanceLamports: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balanceSol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usd1Balance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balanceError: Option<String>,
}

const TOKEN_ACCOUNT_AMOUNT_OFFSET: usize = 64;
const TOKEN_ACCOUNT_AMOUNT_LEN: usize = 8;
const USD1_DECIMALS_FACTOR: f64 = 1_000_000.0;
const MAX_MULTIPLE_ACCOUNTS_BATCH_SIZE: usize = 100;
const WALLET_ATA_CACHE_SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WalletAtaCachePayload {
    #[serde(default)]
    schemaVersion: u8,
    #[serde(default)]
    entries: BTreeMap<String, String>,
}

pub fn is_solana_wallet_env_key(key: &str) -> bool {
    let key = key.trim();
    key == "SOLANA_PRIVATE_KEY"
        || (key.starts_with("SOLANA_PRIVATE_KEY")
            && key["SOLANA_PRIVATE_KEY".len()..]
                .chars()
                .all(|c| c.is_ascii_digit()))
}

pub fn read_keypair_bytes(raw: &str) -> Result<Vec<u8>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Keypair value was empty.".to_string());
    }
    if trimmed.starts_with('[') {
        let parsed: Value = serde_json::from_str(trimmed).map_err(|error| error.to_string())?;
        let array = parsed
            .as_array()
            .ok_or_else(|| "Keypair JSON must be an array of bytes.".to_string())?;
        let mut bytes = Vec::with_capacity(array.len());
        for item in array {
            let byte = item
                .as_u64()
                .ok_or_else(|| "Keypair byte array contained a non-integer value.".to_string())?;
            if byte > 255 {
                return Err("Keypair byte array contained a value above 255.".to_string());
            }
            bytes.push(byte as u8);
        }
        return Ok(bytes);
    }
    bs58::decode(trimmed)
        .into_vec()
        .map_err(|error| error.to_string())
}

fn public_key_from_secret_bytes(bytes: &[u8]) -> Result<String, String> {
    match bytes.len() {
        64 => Ok(bs58::encode(&bytes[32..64]).into_string()),
        32 => {
            Err("32-byte private keys are not yet supported by the Rust wallet parser.".to_string())
        }
        other => Err(format!("Unsupported keypair length: {other} bytes.")),
    }
}

pub fn public_key_from_secret(bytes: &[u8]) -> Result<String, String> {
    public_key_from_secret_bytes(bytes)
}

fn split_wallet_secret_and_name(raw: &str) -> (String, Option<String>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }
    if trimmed.starts_with('[') {
        if let Some(end_index) = trimmed.rfind(']') {
            let secret = trimmed[..=end_index].trim().to_string();
            let remainder = trimmed[end_index + 1..].trim();
            if let Some(name) = remainder.strip_prefix(',').map(str::trim) {
                return (
                    secret,
                    if name.is_empty() {
                        None
                    } else {
                        Some(name.to_string())
                    },
                );
            }
            return (secret, None);
        }
    }
    if let Some((secret, name)) = trimmed.split_once(',') {
        let secret = secret.trim().to_string();
        let name = name.trim();
        return (
            secret,
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            },
        );
    }
    (trimmed.to_string(), None)
}

pub fn list_solana_env_wallets() -> Vec<WalletSummary> {
    let mut keys: Vec<String> = env::vars()
        .map(|(key, _)| key)
        .filter(|key| is_solana_wallet_env_key(key))
        .collect();
    keys.sort_by_key(|key| {
        key.strip_prefix("SOLANA_PRIVATE_KEY")
            .and_then(|suffix| {
                if suffix.is_empty() {
                    Some(1)
                } else {
                    suffix.parse::<usize>().ok()
                }
            })
            .unwrap_or(usize::MAX)
    });
    keys.into_iter()
        .map(|env_key| {
            let raw_value = env::var(&env_key).unwrap_or_default();
            let (secret, custom_name) = split_wallet_secret_and_name(&raw_value);
            match read_keypair_bytes(&secret).and_then(|bytes| public_key_from_secret_bytes(&bytes))
            {
                Ok(public_key) => WalletSummary {
                    envKey: env_key,
                    customName: custom_name,
                    publicKey: Some(public_key),
                    error: None,
                },
                Err(error) => WalletSummary {
                    envKey: env_key,
                    customName: custom_name,
                    publicKey: None,
                    error: Some(error),
                },
            }
        })
        .collect()
}

pub fn selected_wallet_key_or_default(requested_key: &str) -> Option<String> {
    selected_wallet_key_or_default_from_wallets(requested_key, &list_solana_env_wallets())
}

pub fn selected_wallet_key_or_default_from_wallets(
    requested_key: &str,
    wallets: &[WalletSummary],
) -> Option<String> {
    if !requested_key.trim().is_empty() {
        return Some(requested_key.trim().to_string());
    }
    wallets
        .iter()
        .into_iter()
        .find(|wallet| wallet.error.is_none())
        .map(|wallet| wallet.envKey.clone())
}

pub fn load_solana_wallet_by_env_key(env_key: &str) -> Result<Vec<u8>, String> {
    if !is_solana_wallet_env_key(env_key) {
        return Err(format!("Invalid Solana wallet env key: {env_key}"));
    }
    let raw_value = env::var(env_key).map_err(|_| format!("Missing env value for {env_key}"))?;
    let (secret, _) = split_wallet_secret_and_name(&raw_value);
    read_keypair_bytes(&secret)
}

fn wallet_rpc_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|error| format!("Failed to build wallet RPC client: {error}"))
}

fn wallet_ata_cache_path() -> std::path::PathBuf {
    crate::paths::local_root_dir().join("wallet-ata-cache.json")
}

fn load_wallet_ata_cache_from_disk() -> BTreeMap<String, String> {
    let path = wallet_ata_cache_path();
    let Ok(raw) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    let Ok(payload) = serde_json::from_str::<WalletAtaCachePayload>(&raw) else {
        return BTreeMap::new();
    };
    if payload.schemaVersion != WALLET_ATA_CACHE_SCHEMA_VERSION {
        return BTreeMap::new();
    }
    payload.entries
}

fn wallet_ata_cache_store() -> &'static Mutex<BTreeMap<String, String>> {
    static STORE: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(load_wallet_ata_cache_from_disk()))
}

fn wallet_ata_cache_key(owner: &str, mint: &str) -> String {
    format!("{owner}:{mint}")
}

fn persist_wallet_ata_cache(entries: &BTreeMap<String, String>) {
    let payload = WalletAtaCachePayload {
        schemaVersion: WALLET_ATA_CACHE_SCHEMA_VERSION,
        entries: entries.clone(),
    };
    let Ok(serialized) = serde_json::to_vec_pretty(&payload) else {
        return;
    };
    let _ = crate::fs_utils::atomic_write(&wallet_ata_cache_path(), &serialized);
}

fn resolve_cached_associated_token_accounts(
    owners: &[String],
    mint: &Pubkey,
) -> Result<Vec<String>, String> {
    let mint_string = mint.to_string();
    let cache = wallet_ata_cache_store();
    let mut guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut changed = false;
    let mut addresses = Vec::with_capacity(owners.len());
    for owner_string in owners {
        let cache_key = wallet_ata_cache_key(owner_string, &mint_string);
        if let Some(cached) = guard.get(&cache_key) {
            addresses.push(cached.clone());
            continue;
        }
        let owner = Pubkey::from_str(owner_string)
            .map_err(|error| format!("Invalid wallet public key {owner_string}: {error}"))?;
        let ata = get_associated_token_address(&owner, mint).to_string();
        guard.insert(cache_key, ata.clone());
        addresses.push(ata);
        changed = true;
    }
    if changed {
        persist_wallet_ata_cache(&guard);
    }
    Ok(addresses)
}

async fn rpc_request(
    client: &Client,
    rpc_url: &str,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    crate::observability::record_outbound_provider_http_request();
    let response = client
        .post(rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        }))
        .send()
        .await
        .map_err(|error| format!("RPC {method} request failed: {error}"))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let payload: Value = serde_json::from_str(&body)
        .map_err(|error| format!("RPC {method} returned invalid JSON: {error}"))?;
    if !status.is_success() {
        return Err(format!("RPC {method} failed with status {status}: {body}"));
    }
    if let Some(message) = payload
        .get("error")
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str)
    {
        return Err(format!("RPC {method} failed: {message}"));
    }
    payload
        .get("result")
        .cloned()
        .ok_or_else(|| format!("RPC {method} did not return a result."))
}

async fn fetch_balance_lamports_with_client(
    client: &Client,
    rpc_url: &str,
    public_key: &str,
) -> Result<u64, String> {
    rpc_request(
        client,
        rpc_url,
        "getBalance",
        serde_json::json!([public_key, "confirmed"]),
    )
    .await?
    .get("value")
    .and_then(Value::as_u64)
    .ok_or_else(|| format!("RPC getBalance did not return a numeric balance for {public_key}."))
}

pub async fn fetch_balance_lamports(rpc_url: &str, public_key: &str) -> Result<u64, String> {
    let client = wallet_rpc_client()?;
    fetch_balance_lamports_with_client(&client, rpc_url, public_key).await
}

async fn fetch_token_balance_with_client(
    client: &Client,
    rpc_url: &str,
    public_key: &str,
    mint: &str,
    commitment: &str,
) -> Result<f64, String> {
    let result = rpc_request(
        client,
        rpc_url,
        "getTokenAccountsByOwner",
        serde_json::json!([
            public_key,
            { "mint": mint },
            { "encoding": "jsonParsed", "commitment": commitment }
        ]),
    )
    .await?;
    let accounts = result
        .get("value")
        .and_then(Value::as_array)
        .ok_or_else(|| "RPC getTokenAccountsByOwner returned invalid account data.".to_string())?;
    Ok(accounts.iter().fold(0.0, |sum, entry| {
        let token_amount = entry
            .get("account")
            .and_then(|value| value.get("data"))
            .and_then(|value| value.get("parsed"))
            .and_then(|value| value.get("info"))
            .and_then(|value| value.get("tokenAmount"));
        let ui_amount_string = token_amount
            .and_then(|value| value.get("uiAmountString"))
            .and_then(Value::as_str)
            .and_then(|value| value.parse::<f64>().ok());
        let ui_amount = token_amount
            .and_then(|value| value.get("uiAmount"))
            .and_then(Value::as_f64);
        sum + ui_amount_string.or(ui_amount).unwrap_or(0.0)
    }))
}

pub async fn fetch_token_balance(
    rpc_url: &str,
    public_key: &str,
    mint: &str,
    commitment: &str,
) -> Result<f64, String> {
    let client = wallet_rpc_client()?;
    fetch_token_balance_with_client(&client, rpc_url, public_key, mint, commitment).await
}

fn wallet_status_without_balance(
    wallet: &WalletSummary,
    balance_error: Option<String>,
) -> WalletStatusSummary {
    WalletStatusSummary {
        envKey: wallet.envKey.clone(),
        customName: wallet.customName.clone(),
        publicKey: wallet.publicKey.clone(),
        error: wallet.error.clone(),
        balanceLamports: None,
        balanceSol: None,
        usd1Balance: None,
        balanceError: balance_error,
    }
}

async fn fetch_multiple_balance_lamports_with_client(
    client: &Client,
    rpc_url: &str,
    accounts: &[String],
    commitment: &str,
) -> Result<Vec<Option<u64>>, String> {
    if accounts.is_empty() {
        return Ok(vec![]);
    }
    let mut combined = Vec::with_capacity(accounts.len());
    for account_chunk in accounts.chunks(MAX_MULTIPLE_ACCOUNTS_BATCH_SIZE) {
        let result = rpc_request(
            client,
            rpc_url,
            "getMultipleAccounts",
            serde_json::json!([
                account_chunk,
                {
                    "encoding": "base64",
                    "commitment": commitment,
                    "dataSlice": {
                        "offset": 0,
                        "length": 0
                    }
                }
            ]),
        )
        .await?;
        let values = result
            .get("value")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| "RPC getMultipleAccounts did not return a value array.".to_string())?;
        if values.len() != account_chunk.len() {
            return Err(format!(
                "RPC getMultipleAccounts returned {} entries for {} requested accounts.",
                values.len(),
                account_chunk.len()
            ));
        }
        let parsed_chunk = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                if value.is_null() {
                    return Ok(None);
                }
                value
                    .get("lamports")
                    .and_then(Value::as_u64)
                    .map(Some)
                    .ok_or_else(|| {
                        format!(
                            "RPC getMultipleAccounts did not return lamports for {}.",
                            account_chunk[index]
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        combined.extend(parsed_chunk);
    }
    Ok(combined)
}

async fn fetch_multiple_account_data_with_client(
    client: &Client,
    rpc_url: &str,
    accounts: &[String],
    commitment: &str,
) -> Result<Vec<Option<Vec<u8>>>, String> {
    if accounts.is_empty() {
        return Ok(vec![]);
    }
    let mut combined = Vec::with_capacity(accounts.len());
    for account_chunk in accounts.chunks(MAX_MULTIPLE_ACCOUNTS_BATCH_SIZE) {
        let result = rpc_request(
            client,
            rpc_url,
            "getMultipleAccounts",
            serde_json::json!([
                account_chunk,
                {
                    "encoding": "base64",
                    "commitment": commitment,
                }
            ]),
        )
        .await?;
        let values = result
            .get("value")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| "RPC getMultipleAccounts did not return a value array.".to_string())?;
        if values.len() != account_chunk.len() {
            return Err(format!(
                "RPC getMultipleAccounts returned {} entries for {} requested accounts.",
                values.len(),
                account_chunk.len()
            ));
        }
        let parsed_chunk = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                if value.is_null() {
                    return Ok(None);
                }
                let data = value
                    .get("data")
                    .and_then(Value::as_array)
                    .and_then(|items| items.first())
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        format!(
                            "RPC getMultipleAccounts returned invalid base64 data for {}.",
                            account_chunk[index]
                        )
                    })?;
                use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
                BASE64
                    .decode(data)
                    .map(Some)
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        combined.extend(parsed_chunk);
    }
    Ok(combined)
}

fn parse_token_account_raw_balance(data: &[u8]) -> Result<u64, String> {
    let end = TOKEN_ACCOUNT_AMOUNT_OFFSET + TOKEN_ACCOUNT_AMOUNT_LEN;
    if data.len() < end {
        return Err("Token account data was too short to contain a token amount.".to_string());
    }
    let amount_bytes: [u8; TOKEN_ACCOUNT_AMOUNT_LEN] = data[TOKEN_ACCOUNT_AMOUNT_OFFSET..end]
        .try_into()
        .map_err(|_| "Token account amount bytes were malformed.".to_string())?;
    Ok(u64::from_le_bytes(amount_bytes))
}

async fn enrich_wallet_statuses_individual(
    client: &Client,
    rpc_url: &str,
    usd1_mint: &str,
    wallets: &[WalletSummary],
) -> Vec<WalletStatusSummary> {
    let tasks = wallets.iter().cloned().map(|wallet| {
        let client = client.clone();
        let rpc_url = rpc_url.to_string();
        let usd1_mint = usd1_mint.to_string();
        async move {
            if wallet.error.is_some() || wallet.publicKey.is_none() {
                return wallet_status_without_balance(&wallet, None);
            }
            let public_key = wallet.publicKey.clone().unwrap_or_default();
            let (balance_result, token_result) = tokio::join!(
                fetch_balance_lamports_with_client(&client, &rpc_url, &public_key),
                fetch_token_balance_with_client(
                    &client,
                    &rpc_url,
                    &public_key,
                    &usd1_mint,
                    "confirmed"
                ),
            );
            match (balance_result, token_result) {
                (Ok(balance_lamports), Ok(usd1_balance)) => WalletStatusSummary {
                    envKey: wallet.envKey.clone(),
                    customName: wallet.customName.clone(),
                    publicKey: wallet.publicKey.clone(),
                    error: wallet.error.clone(),
                    balanceLamports: Some(balance_lamports),
                    balanceSol: Some(balance_lamports as f64 / 1_000_000_000.0),
                    usd1Balance: Some(usd1_balance),
                    balanceError: None,
                },
                (balance_result, token_result) => {
                    let balance_error = balance_result
                        .err()
                        .or_else(|| token_result.err())
                        .unwrap_or_else(|| "Unknown wallet balance error.".to_string());
                    wallet_status_without_balance(&wallet, Some(balance_error))
                }
            }
        }
    });
    join_all(tasks).await
}

async fn enrich_wallet_statuses_batched(
    client: &Client,
    rpc_url: &str,
    usd1_mint: &str,
    wallets: &[WalletSummary],
) -> Result<Vec<WalletStatusSummary>, String> {
    let usd1_mint_pubkey = Pubkey::from_str(usd1_mint)
        .map_err(|error| format!("Invalid USD1 mint {usd1_mint}: {error}"))?;
    let mut results = wallets
        .iter()
        .map(|wallet| wallet_status_without_balance(wallet, None))
        .collect::<Vec<_>>();
    let mut valid_indices = Vec::new();
    let mut public_keys = Vec::new();
    for (index, wallet) in wallets.iter().enumerate() {
        if wallet.error.is_some() {
            continue;
        }
        let Some(public_key) = wallet.publicKey.as_ref() else {
            continue;
        };
        valid_indices.push(index);
        public_keys.push(public_key.clone());
    }
    if valid_indices.is_empty() {
        return Ok(results);
    }
    let usd1_ata_accounts =
        resolve_cached_associated_token_accounts(&public_keys, &usd1_mint_pubkey)?;
    let (balance_result, usd1_accounts_result) = tokio::join!(
        fetch_multiple_balance_lamports_with_client(client, rpc_url, &public_keys, "confirmed"),
        fetch_multiple_account_data_with_client(client, rpc_url, &usd1_ata_accounts, "confirmed"),
    );
    let balances = balance_result?;
    let usd1_accounts = usd1_accounts_result?;
    if balances.len() != valid_indices.len() || usd1_accounts.len() != valid_indices.len() {
        return Err("Batched wallet balance results did not match the wallet count.".to_string());
    }
    let mut fallback_wallet_positions = Vec::new();
    for (position, wallet_index) in valid_indices.iter().copied().enumerate() {
        let wallet = &wallets[wallet_index];
        let balance_lamports = balances[position].unwrap_or(0);
        let usd1_balance = match usd1_accounts[position].as_ref() {
            Some(account_data) => match parse_token_account_raw_balance(account_data) {
                Ok(amount) => amount as f64 / USD1_DECIMALS_FACTOR,
                Err(_error) => {
                    fallback_wallet_positions.push(position);
                    0.0
                }
            },
            None => {
                fallback_wallet_positions.push(position);
                0.0
            }
        };
        results[wallet_index] = WalletStatusSummary {
            envKey: wallet.envKey.clone(),
            customName: wallet.customName.clone(),
            publicKey: wallet.publicKey.clone(),
            error: wallet.error.clone(),
            balanceLamports: Some(balance_lamports),
            balanceSol: Some(balance_lamports as f64 / 1_000_000_000.0),
            usd1Balance: Some(usd1_balance),
            balanceError: None,
        };
    }
    if !fallback_wallet_positions.is_empty() {
        let fallback_tasks = fallback_wallet_positions.iter().copied().map(|position| {
            let client = client.clone();
            let rpc_url = rpc_url.to_string();
            let public_key = public_keys[position].clone();
            let usd1_mint = usd1_mint.to_string();
            async move {
                (
                    position,
                    fetch_token_balance_with_client(
                        &client,
                        &rpc_url,
                        &public_key,
                        &usd1_mint,
                        "confirmed",
                    )
                    .await,
                )
            }
        });
        for (position, fallback_result) in join_all(fallback_tasks).await {
            let wallet_index = valid_indices[position];
            match fallback_result {
                Ok(usd1_balance) => {
                    results[wallet_index].usd1Balance = Some(usd1_balance);
                }
                Err(error) => {
                    results[wallet_index].usd1Balance = None;
                    results[wallet_index].balanceError = Some(error);
                }
            }
        }
    }
    Ok(results)
}

pub async fn enrich_wallet_statuses(
    rpc_url: &str,
    usd1_mint: &str,
    wallets: &[WalletSummary],
) -> Vec<WalletStatusSummary> {
    let client = match wallet_rpc_client() {
        Ok(client) => client,
        Err(error) => {
            return wallets
                .iter()
                .map(|wallet| WalletStatusSummary {
                    envKey: wallet.envKey.clone(),
                    customName: wallet.customName.clone(),
                    publicKey: wallet.publicKey.clone(),
                    error: wallet.error.clone(),
                    balanceLamports: None,
                    balanceSol: None,
                    usd1Balance: None,
                    balanceError: if wallet.error.is_some() {
                        None
                    } else {
                        Some(error.clone())
                    },
                })
                .collect();
        }
    };
    match enrich_wallet_statuses_batched(&client, rpc_url, usd1_mint, wallets).await {
        Ok(wallets) => wallets,
        Err(_error) => {
            enrich_wallet_statuses_individual(&client, rpc_url, usd1_mint, wallets).await
        }
    }
}
