#![allow(non_snake_case, dead_code)]

use futures_util::future::join_all;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::{env, time::Duration};

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

async fn rpc_request(
    client: &Client,
    rpc_url: &str,
    method: &str,
    params: Value,
) -> Result<Value, String> {
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
    let tasks = wallets.iter().cloned().map(|wallet| {
        let client = client.clone();
        let rpc_url = rpc_url.to_string();
        let usd1_mint = usd1_mint.to_string();
        async move {
            if wallet.error.is_some() || wallet.publicKey.is_none() {
                return WalletStatusSummary {
                    envKey: wallet.envKey.clone(),
                    customName: wallet.customName.clone(),
                    publicKey: wallet.publicKey.clone(),
                    error: wallet.error.clone(),
                    balanceLamports: None,
                    balanceSol: None,
                    usd1Balance: None,
                    balanceError: None,
                };
            }
            let public_key = wallet.publicKey.clone().unwrap_or_default();
            let (balance_result, token_result) = tokio::join!(
                fetch_balance_lamports_with_client(&client, &rpc_url, &public_key),
                fetch_token_balance_with_client(&client, &rpc_url, &public_key, &usd1_mint, "confirmed"),
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
                    WalletStatusSummary {
                        envKey: wallet.envKey.clone(),
                        customName: wallet.customName.clone(),
                        publicKey: wallet.publicKey.clone(),
                        error: wallet.error.clone(),
                        balanceLamports: None,
                        balanceSol: None,
                        usd1Balance: None,
                        balanceError: Some(balance_error),
                    }
                }
            }
        }
    });
    join_all(tasks).await
}
