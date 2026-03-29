#![allow(non_snake_case)]

use crate::{
    config::{
        RawAgent, RawAutomaticDevSell, RawConfig, RawCreatorFee, RawExecution, RawFeeSharing,
        RawFollowLaunch, RawFollowLaunchConstraints, RawFollowLaunchMarketCapTrigger,
        RawFollowLaunchSell, RawFollowLaunchSnipe, RawPostLaunch, RawPresets, RawRecipient,
        RawSnipeOwnLaunch, RawToken, RawTx,
    },
    paths,
    pump_native::{LaunchQuote, quote_launch},
    wallet::selected_wallet_key_or_default,
};
use reqwest::{Client, multipart};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    env, fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

const FIXED_COMPUTE_UNIT_LIMIT: u64 = 1_000_000;
const MAX_FEE_SPLIT_RECIPIENTS: usize = 10;
const HELIUS_SENDER_TIP_ACCOUNTS: [&str; 3] = [
    "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE",
    "D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ",
    "9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta",
];
const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetadataUploadProvider {
    PumpFun,
    Pinata,
}

fn parse_metadata_upload_provider(value: &str) -> Result<MetadataUploadProvider, String> {
    match value.trim().to_lowercase().as_str() {
        "" | "pump-fun" | "pumpfun" => Ok(MetadataUploadProvider::PumpFun),
        "pinata" | "custom" => Ok(MetadataUploadProvider::Pinata),
        other => Err(format!(
            "Unsupported metadata upload provider: {other}. Expected pump-fun or pinata."
        )),
    }
}

fn configured_metadata_upload_provider() -> Result<MetadataUploadProvider, String> {
    let configured = env::var("LAUNCHDECK_METADATA_UPLOAD_PROVIDER").unwrap_or_default();
    parse_metadata_upload_provider(&configured)
}

fn configured_pinata_jwt() -> Result<String, String> {
    let jwt = env::var("PINATA_JWT").unwrap_or_default();
    let trimmed = jwt.trim();
    if trimmed.is_empty() {
        Err("PINATA_JWT is required when metadata upload provider is pinata.".to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn pinata_image_cache() -> &'static Mutex<HashMap<u64, String>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Deserialize, Default)]
pub struct QuoteForm {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub amount: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UiForm {
    #[serde(default)]
    pub selectedWalletKey: String,
    #[serde(default)]
    pub vanityPrivateKey: String,
    #[serde(default)]
    pub activePresetId: String,
    #[serde(default)]
    pub launchpad: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub twitter: String,
    #[serde(default)]
    pub telegram: String,
    #[serde(default)]
    pub metadataUri: String,
    #[serde(default)]
    pub mayhemMode: bool,
    #[serde(default)]
    pub agentAuthority: String,
    #[serde(default)]
    pub buybackPercent: String,
    #[serde(default)]
    pub agentSplitRecipients: Vec<UiRecipientInput>,
    #[serde(default)]
    pub devBuyMode: String,
    #[serde(default)]
    pub devBuyAmount: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub priorityFeeSol: String,
    #[serde(default)]
    pub creationTipSol: String,
    #[serde(default)]
    pub buyProvider: String,
    #[serde(default)]
    pub buyPriorityFeeSol: String,
    #[serde(default)]
    pub buyTipSol: String,
    #[serde(default)]
    pub buySlippagePercent: String,
    #[serde(default)]
    pub sellProvider: String,
    #[serde(default)]
    pub sellPriorityFeeSol: String,
    #[serde(default)]
    pub sellTipSol: String,
    #[serde(default)]
    pub sellSlippagePercent: String,
    #[serde(default)]
    pub skipPreflight: bool,
    #[serde(default)]
    pub trackSendBlockHeight: bool,
    #[serde(default)]
    pub feeSplitEnabled: bool,
    #[serde(default)]
    pub feeSplitRecipients: Vec<UiRecipientInput>,
    #[serde(default)]
    pub postLaunchStrategy: String,
    #[serde(default)]
    pub snipeBuyAmountSol: String,
    #[serde(default)]
    pub sniperEnabled: bool,
    #[serde(default)]
    pub sniperWallets: Vec<UiSniperWalletInput>,
    #[serde(default)]
    pub followLaunch: UiFollowLaunch,
    #[serde(default)]
    pub automaticDevSellEnabled: bool,
    #[serde(default)]
    pub automaticDevSellPercent: String,
    #[serde(default)]
    pub automaticDevSellTriggerMode: String,
    #[serde(default)]
    pub automaticDevSellDelayMs: String,
    #[serde(default)]
    pub automaticDevSellBlockOffset: String,
    #[serde(default)]
    pub imageFileName: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct UiRecipientInput {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub githubUsername: String,
    #[serde(default)]
    pub githubUserId: String,
    #[serde(default)]
    pub shareBps: Option<i64>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct UiSniperWalletInput {
    #[serde(default)]
    pub envKey: String,
    #[serde(default)]
    pub amountSol: String,
    #[serde(default)]
    pub triggerMode: String,
    #[serde(default)]
    pub submitDelayMs: Option<i64>,
    #[serde(default)]
    pub targetBlockOffset: Option<i64>,
    #[serde(default)]
    pub retryOnce: bool,
    #[serde(default)]
    pub jitterMs: Option<i64>,
    #[serde(default)]
    pub feeJitterBps: Option<i64>,
    #[serde(default)]
    pub sellPercent: Option<i64>,
    #[serde(default)]
    pub sellDelayMs: Option<i64>,
    #[serde(default)]
    pub sellMarketCapThreshold: String,
    #[serde(default)]
    pub sellMarketCapDirection: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct UiFollowLaunch {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub snipes: Vec<UiSniperWalletInput>,
    #[serde(default)]
    pub devAutoSell: UiFollowLaunchSell,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct UiFollowLaunchSell {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub percent: String,
    #[serde(default)]
    pub triggerMode: String,
    #[serde(default)]
    pub delayMs: String,
    #[serde(default)]
    pub targetBlockOffset: String,
    #[serde(default)]
    pub marketCapThreshold: String,
    #[serde(default)]
    pub marketCapDirection: String,
}

fn uploads_dir() -> std::path::PathBuf {
    paths::uploads_dir()
}

fn sanitize_provider(value: &str) -> String {
    let trimmed = value.trim().to_lowercase();
    if trimmed.is_empty() {
        "helius-sender".to_string()
    } else {
        trimmed
    }
}

fn tip_supported(provider: &str) -> bool {
    matches!(provider, "helius-sender" | "jito-bundle")
}

fn pick_tip_account(provider: &str) -> String {
    let accounts = if provider == "helius-sender" {
        &HELIUS_SENDER_TIP_ACCOUNTS[..]
    } else {
        &JITO_TIP_ACCOUNTS[..]
    };
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as usize)
        .unwrap_or_default();
    accounts[seed % accounts.len()].to_string()
}

fn parse_decimal_to_u64(value: &str, decimals: u32, label: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(0);
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
    let normalized = format!("{whole}{fractional:0<width$}", width = decimals as usize);
    let digits = normalized.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(0);
    }
    digits.parse::<u64>().map_err(|error| error.to_string())
}

fn lamports_to_priority_fee_micro_lamports(priority_fee_lamports: u64) -> u64 {
    if priority_fee_lamports == 0 {
        0
    } else {
        (priority_fee_lamports.saturating_mul(1_000_000)) / FIXED_COMPUTE_UNIT_LIMIT
    }
}

fn buyback_percent_to_bps(raw_value: &str) -> Result<Option<i64>, String> {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let numeric = trimmed
        .parse::<f64>()
        .map_err(|_| format!("buyback percentage must be between 0 and 100. Got: {raw_value}"))?;
    if !(0.0..=100.0).contains(&numeric) {
        return Err(format!(
            "buyback percentage must be between 0 and 100. Got: {raw_value}"
        ));
    }
    Ok(Some((numeric * 100.0).round() as i64))
}

fn parse_recipients(
    entries: &[UiRecipientInput],
    allow_agent: bool,
) -> Result<Vec<RawRecipient>, String> {
    if entries.len() > MAX_FEE_SPLIT_RECIPIENTS {
        return Err(format!(
            "Fee split supports at most {MAX_FEE_SPLIT_RECIPIENTS} recipients."
        ));
    }
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let entry_type = if entry.r#type.trim().is_empty() {
                "wallet".to_string()
            } else {
                entry.r#type.trim().to_lowercase()
            };
            let share_bps = entry.shareBps.ok_or_else(|| {
                format!(
                    "Fee split recipient {} must have a positive share.",
                    index + 1
                )
            })?;
            if share_bps <= 0 {
                return Err(format!(
                    "Fee split recipient {} must have a positive share.",
                    index + 1
                ));
            }
            if allow_agent && entry_type == "agent" {
                return Ok(RawRecipient {
                    r#type: "agent".to_string(),
                    shareBps: Some(json!(share_bps)),
                    ..RawRecipient::default()
                });
            }
            if entry_type == "wallet" {
                let address = entry.address.trim();
                if address.is_empty() {
                    return Err(format!(
                        "Fee split recipient {} is missing a wallet address.",
                        index + 1
                    ));
                }
                return Ok(RawRecipient {
                    address: address.to_string(),
                    shareBps: Some(json!(share_bps)),
                    ..RawRecipient::default()
                });
            }
            if entry_type == "github" {
                let username = entry.githubUsername.trim();
                if username.is_empty() {
                    return Err(format!(
                        "Fee split recipient {} is missing a GitHub username.",
                        index + 1
                    ));
                }
                return Ok(RawRecipient {
                    r#type: "github".to_string(),
                    githubUsername: username.to_string(),
                    githubUserId: entry.githubUserId.trim().to_string(),
                    shareBps: Some(json!(share_bps)),
                    ..RawRecipient::default()
                });
            }
            Err(format!(
                "Unsupported fee split recipient type: {entry_type}"
            ))
        })
        .collect()
}

async fn resolve_github_user(username: &str) -> Result<(String, String), String> {
    let trimmed = username.trim();
    if trimmed.is_empty() {
        return Err("GitHub username is required.".to_string());
    }
    let response = Client::new()
        .get(format!("https://api.github.com/users/{trimmed}"))
        .header("accept", "application/vnd.github+json")
        .header("user-agent", "launchdeck-rust-engine")
        .send()
        .await
        .map_err(|error| format!("Failed to look up GitHub user @{trimmed}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "GitHub user @{trimmed} was not found (status {}).",
            response.status()
        ));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse GitHub response for @{trimmed}: {error}"))?;
    let login = payload
        .get("login")
        .and_then(Value::as_str)
        .unwrap_or(trimmed)
        .trim()
        .to_string();
    let user_id = payload
        .get("id")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .ok_or_else(|| format!("GitHub user @{trimmed} did not include a numeric id."))?;
    Ok((login, user_id))
}

async fn resolve_recipient_github_ids(recipients: &mut [RawRecipient]) -> Result<(), String> {
    for recipient in recipients {
        if recipient.githubUsername.trim().is_empty() || !recipient.githubUserId.trim().is_empty() {
            continue;
        }
        let (username, user_id) = resolve_github_user(&recipient.githubUsername).await?;
        recipient.githubUsername = username;
        recipient.githubUserId = user_id;
    }
    Ok(())
}

fn image_mime(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

fn normalize_metadata_uri(uri: &str) -> String {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with("ipfs://") {
        return trimmed.to_string();
    }
    let Some(without_scheme) = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
    else {
        return trimmed.to_string();
    };
    let Some((_, path)) = without_scheme.split_once('/') else {
        return trimmed.to_string();
    };
    let Some(ipfs_path) = path.strip_prefix("ipfs/") else {
        return trimmed.to_string();
    };
    let normalized_path = ipfs_path
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .trim_matches('/');
    if normalized_path.is_empty() {
        return trimmed.to_string();
    }
    format!("ipfs://{normalized_path}")
}

fn uploaded_image_details(config: &RawConfig) -> Result<(String, PathBuf), String> {
    let image_file_name = Path::new(&config.imageLocalPath)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    if image_file_name.is_empty() {
        return Err("Image is required before launching.".to_string());
    }
    let image_path = uploads_dir().join(&image_file_name);
    Ok((image_file_name, image_path))
}

fn build_launch_metadata_json(config: &RawConfig, image_uri: &str) -> Value {
    let mut payload = serde_json::Map::new();
    payload.insert("name".to_string(), Value::String(config.token.name.clone()));
    payload.insert(
        "symbol".to_string(),
        Value::String(config.token.symbol.clone()),
    );
    payload.insert(
        "description".to_string(),
        Value::String(config.token.description.clone()),
    );
    payload.insert("image".to_string(), Value::String(image_uri.to_string()));
    payload.insert("showName".to_string(), Value::Bool(true));
    payload.insert(
        "createdOn".to_string(),
        Value::String("https://pump.fun".to_string()),
    );
    if !config.token.website.trim().is_empty() {
        payload.insert(
            "website".to_string(),
            Value::String(config.token.website.clone()),
        );
    }
    if !config.token.twitter.trim().is_empty() {
        payload.insert(
            "twitter".to_string(),
            Value::String(config.token.twitter.clone()),
        );
    }
    if !config.token.telegram.trim().is_empty() {
        payload.insert(
            "telegram".to_string(),
            Value::String(config.token.telegram.clone()),
        );
    }
    Value::Object(payload)
}

fn pinata_cid(payload: &Value, label: &str) -> Result<String, String> {
    payload
        .get("cid")
        .or_else(|| payload.get("IpfsHash"))
        .or_else(|| payload.get("data").and_then(|value| value.get("cid")))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("{label} did not return an IPFS CID."))
}

fn stable_image_cache_key(image_file_name: &str, image_bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    image_file_name.hash(&mut hasher);
    image_bytes.hash(&mut hasher);
    hasher.finish()
}

fn cached_pinata_image_uri(cache_key: u64) -> Option<String> {
    pinata_image_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.get(&cache_key).cloned())
}

fn store_pinata_image_uri(cache_key: u64, image_uri: &str) {
    if let Ok(mut cache) = pinata_image_cache().lock() {
        cache.insert(cache_key, image_uri.to_string());
    }
}

async fn upload_image_to_pinata(
    client: &Client,
    jwt: &str,
    image_file_name: &str,
    image_path: &Path,
    image_bytes: Vec<u8>,
) -> Result<String, String> {
    let image_cache_key = stable_image_cache_key(image_file_name, &image_bytes);
    if let Some(image_uri) = cached_pinata_image_uri(image_cache_key) {
        return Ok(image_uri);
    }
    let image_part = multipart::Part::bytes(image_bytes)
        .file_name(image_file_name.to_string())
        .mime_str(image_mime(image_path))
        .map_err(|error| format!("Failed to prepare uploaded image: {error}"))?;
    let image_upload_response = client
        .post("https://uploads.pinata.cloud/v3/files")
        .bearer_auth(jwt)
        .multipart(
            multipart::Form::new()
                .text("network", "public")
                .text("name", image_file_name.to_string())
                .part("file", image_part),
        )
        .send()
        .await
        .map_err(|error| format!("Pinata image upload failed: {error}"))?;
    if !image_upload_response.status().is_success() {
        let status = image_upload_response.status();
        let body = image_upload_response.text().await.unwrap_or_default();
        return Err(format!(
            "Pinata image upload failed with status {status}: {}",
            if body.trim().is_empty() {
                "empty response".to_string()
            } else {
                body
            }
        ));
    }
    let image_payload: Value = image_upload_response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Pinata image upload response: {error}"))?;
    let image_uri = format!(
        "ipfs://{}",
        pinata_cid(&image_payload, "Pinata image upload")?
    );
    store_pinata_image_uri(image_cache_key, &image_uri);
    Ok(image_uri)
}

async fn upload_metadata_to_pump_fun(config: &RawConfig) -> Result<String, String> {
    let (image_file_name, image_path) = uploaded_image_details(config)?;
    let image_bytes = fs::read(&image_path).map_err(|error| {
        format!(
            "Failed to read uploaded image {}: {error}",
            image_path.display()
        )
    })?;
    let image_part = multipart::Part::bytes(image_bytes)
        .file_name(image_file_name)
        .mime_str(image_mime(&image_path))
        .map_err(|error| format!("Failed to prepare uploaded image: {error}"))?;
    let mut metadata = serde_json::Map::new();
    if !config.token.website.trim().is_empty() {
        metadata.insert(
            "website".to_string(),
            Value::String(config.token.website.clone()),
        );
    }
    if !config.token.twitter.trim().is_empty() {
        metadata.insert(
            "twitter".to_string(),
            Value::String(config.token.twitter.clone()),
        );
    }
    if !config.token.telegram.trim().is_empty() {
        metadata.insert(
            "telegram".to_string(),
            Value::String(config.token.telegram.clone()),
        );
    }
    let form = multipart::Form::new()
        .text("name", config.token.name.clone())
        .text("symbol", config.token.symbol.clone())
        .text("description", config.token.description.clone())
        .text("showName", "true")
        .text("metadata", Value::Object(metadata).to_string())
        .part("file", image_part);
    let response = Client::new()
        .post("https://pump.fun/api/ipfs")
        .multipart(form)
        .send()
        .await
        .map_err(|error| format!("Metadata upload failed: {error}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "Metadata upload failed with status {status}: {}",
            if body.trim().is_empty() {
                "empty response".to_string()
            } else {
                body
            }
        ));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse metadata upload response: {error}"))?;
    payload
        .get("metadataUri")
        .and_then(Value::as_str)
        .map(normalize_metadata_uri)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Metadata upload did not return metadataUri.".to_string())
}

async fn upload_metadata_to_pinata(config: &RawConfig) -> Result<String, String> {
    let jwt = configured_pinata_jwt()?;
    let client = Client::new();
    let (image_file_name, image_path) = uploaded_image_details(config)?;
    let image_bytes = fs::read(&image_path).map_err(|error| {
        format!(
            "Failed to read uploaded image {}: {error}",
            image_path.display()
        )
    })?;
    let image_uri =
        upload_image_to_pinata(&client, &jwt, &image_file_name, &image_path, image_bytes).await?;

    let metadata_name = Path::new(&image_file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| format!("{value}-metadata.json"))
        .unwrap_or_else(|| "metadata.json".to_string());
    let metadata_payload = json!({
        "pinataContent": build_launch_metadata_json(config, &image_uri),
        "pinataMetadata": {
            "name": metadata_name,
        },
        "pinataOptions": {
            "cidVersion": 1,
        }
    });
    let metadata_response = client
        .post("https://api.pinata.cloud/pinning/pinJSONToIPFS")
        .bearer_auth(&jwt)
        .json(&metadata_payload)
        .send()
        .await
        .map_err(|error| format!("Pinata metadata upload failed: {error}"))?;
    if !metadata_response.status().is_success() {
        let status = metadata_response.status();
        let body = metadata_response.text().await.unwrap_or_default();
        return Err(format!(
            "Pinata metadata upload failed with status {status}: {}",
            if body.trim().is_empty() {
                "empty response".to_string()
            } else {
                body
            }
        ));
    }
    let metadata_payload: Value = metadata_response
        .json()
        .await
        .map_err(|error| format!("Failed to parse Pinata metadata upload response: {error}"))?;
    Ok(format!(
        "ipfs://{}",
        pinata_cid(&metadata_payload, "Pinata metadata upload")?
    ))
}

async fn upload_metadata(config: &RawConfig) -> Result<String, String> {
    match configured_metadata_upload_provider()? {
        MetadataUploadProvider::PumpFun => upload_metadata_to_pump_fun(config).await,
        MetadataUploadProvider::Pinata => match upload_metadata_to_pinata(config).await {
            Ok(uri) => Ok(uri),
            Err(pinata_error) => upload_metadata_to_pump_fun(config).await.map_err(|pump_error| {
                format!(
                    "Pinata metadata upload failed and pump-fun fallback also failed. Pinata: {pinata_error} | pump-fun: {pump_error}"
                )
            }),
        },
    }
}

fn provided_metadata_uri(form: &UiForm) -> Option<String> {
    let normalized = normalize_metadata_uri(&form.metadataUri);
    if normalized.trim().is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub async fn quote_from_form(
    rpc_url: &str,
    form_value: Value,
) -> Result<Option<LaunchQuote>, String> {
    let form: QuoteForm = serde_json::from_value(form_value)
        .map_err(|error| format!("Invalid quote form payload: {error}"))?;
    quote_launch(rpc_url, &form.mode, &form.amount).await
}

async fn build_raw_config_from_ui_form(action: &str, form: UiForm) -> Result<RawConfig, String> {
    let mode = if form.mode.trim().is_empty() {
        "regular".to_string()
    } else {
        form.mode.trim().to_lowercase()
    };
    let launchpad = if form.launchpad.trim().is_empty() {
        "pump".to_string()
    } else {
        form.launchpad.trim().to_lowercase()
    };
    let provider = sanitize_provider(&form.provider);
    let endpoint_profile = String::new();
    let buy_provider = sanitize_provider(if form.buyProvider.trim().is_empty() {
        &provider
    } else {
        &form.buyProvider
    });
    let buy_endpoint_profile = String::new();
    let sell_provider = sanitize_provider(if form.sellProvider.trim().is_empty() {
        &provider
    } else {
        &form.sellProvider
    });
    let sell_endpoint_profile = String::new();
    let buy_tip_supported = tip_supported(&buy_provider);
    let sell_tip_supported = tip_supported(&sell_provider);
    let priority_fee_lamports =
        parse_decimal_to_u64(&form.priorityFeeSol, 9, "creation priority fee")?;
    let tip_lamports = if tip_supported(&provider) {
        parse_decimal_to_u64(&form.creationTipSol, 9, "creation tip")?
    } else {
        0
    };
    let is_agent_locked = mode == "agent-locked";
    let is_agent_custom = mode == "agent-custom";
    let is_agent_unlocked = mode == "agent-unlocked";
    let fee_split_enabled = mode == "regular" && form.feeSplitEnabled;
    let mut agent_fee_recipients = if is_agent_custom {
        parse_recipients(&form.agentSplitRecipients, true)?
    } else {
        vec![]
    };
    let agent_recipient_share = agent_fee_recipients
        .iter()
        .find(|entry| entry.r#type == "agent")
        .and_then(|entry| entry.shareBps.as_ref())
        .and_then(Value::as_i64);
    let buyback_bps = if is_agent_locked {
        Some(10_000)
    } else if is_agent_custom {
        agent_recipient_share
            .map(Some)
            .unwrap_or(buyback_percent_to_bps(&form.buybackPercent)?)
    } else {
        buyback_percent_to_bps(&form.buybackPercent)?
    };
    let mut fee_sharing_recipients = if fee_split_enabled {
        parse_recipients(&form.feeSplitRecipients, false)?
    } else {
        vec![]
    };
    resolve_recipient_github_ids(&mut fee_sharing_recipients).await?;
    resolve_recipient_github_ids(&mut agent_fee_recipients).await?;

    let image_file_name = Path::new(form.imageFileName.trim())
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let selected_wallet_key =
        selected_wallet_key_or_default(&form.selectedWalletKey).unwrap_or_default();
    let follow_snipes = if !form.followLaunch.snipes.is_empty() {
        form.followLaunch.snipes.clone()
    } else {
        form.sniperWallets.clone()
    };
    let follow_launch_snipes = follow_snipes
        .iter()
        .enumerate()
        .filter(|(_, entry)| !entry.envKey.trim().is_empty() && !entry.amountSol.trim().is_empty())
        .map(|(index, entry)| RawFollowLaunchSnipe {
            actionId: format!("snipe-{}-buy", index + 1),
            enabled: Some(json!(true)),
            walletEnvKey: entry.envKey.trim().to_string(),
            buyAmountSol: entry.amountSol.trim().to_string(),
            submitWithLaunch: Some(json!(entry.triggerMode.trim().eq_ignore_ascii_case("same-time"))),
            retryOnFailure: Some(json!(entry.retryOnce)),
            submitDelayMs: if entry.triggerMode.trim().eq_ignore_ascii_case("submit-delay") {
                entry.submitDelayMs.map(|value| json!(value.max(0)))
            } else {
                Some(json!(0))
            },
            targetBlockOffset: if entry.triggerMode.trim().eq_ignore_ascii_case("block-offset") {
                entry.targetBlockOffset.map(|value| json!(value.max(0)))
            } else {
                None
            },
            jitterMs: entry.jitterMs.map(|value| json!(value.max(0))),
            feeJitterBps: entry.feeJitterBps.map(|value| json!(value.max(0))),
            postBuySell: if entry.sellPercent.unwrap_or(0) > 0
                || entry.sellDelayMs.unwrap_or(0) > 0
                || !entry.sellMarketCapThreshold.trim().is_empty()
            {
                Some(RawFollowLaunchSell {
                    actionId: format!("snipe-{}-sell", index + 1),
                    enabled: Some(json!(true)),
                    walletEnvKey: entry.envKey.trim().to_string(),
                    percent: entry.sellPercent.map(|value| json!(value.max(0))),
                    delayMs: entry.sellDelayMs.map(|value| json!(value.max(0))),
                    targetBlockOffset: None,
                    marketCap: RawFollowLaunchMarketCapTrigger {
                        enabled: Some(json!(!entry.sellMarketCapThreshold.trim().is_empty())),
                        direction: if entry.sellMarketCapDirection.trim().is_empty() {
                            "gte".to_string()
                        } else {
                            entry.sellMarketCapDirection.trim().to_string()
                        },
                        threshold: entry.sellMarketCapThreshold.trim().to_string(),
                    },
                    precheckRequired: Some(json!(false)),
                    requireConfirmation: Some(json!(true)),
                })
            } else {
                None
            },
        })
        .collect::<Vec<_>>();
    let dev_auto_sell_percent = if !form.followLaunch.devAutoSell.percent.trim().is_empty() {
        form.followLaunch.devAutoSell.percent.trim().to_string()
    } else {
        form.automaticDevSellPercent.trim().to_string()
    };
    let dev_auto_sell_trigger_mode = if !form.followLaunch.devAutoSell.triggerMode.trim().is_empty() {
        form.followLaunch.devAutoSell.triggerMode.trim().to_string()
    } else if !form.automaticDevSellTriggerMode.trim().is_empty() {
        form.automaticDevSellTriggerMode.trim().to_string()
    } else {
        "confirmation".to_string()
    };
    let dev_auto_sell_delay_ms = if !form.followLaunch.devAutoSell.delayMs.trim().is_empty() {
        form.followLaunch.devAutoSell.delayMs.trim().to_string()
    } else {
        form.automaticDevSellDelayMs.trim().to_string()
    };
    let dev_auto_sell_block_offset = if !form.followLaunch.devAutoSell.targetBlockOffset.trim().is_empty() {
        form.followLaunch.devAutoSell.targetBlockOffset.trim().to_string()
    } else {
        form.automaticDevSellBlockOffset.trim().to_string()
    };
    let dev_auto_sell_require_confirmation = dev_auto_sell_trigger_mode == "confirmation";
    let dev_auto_sell_target_block_offset = if dev_auto_sell_trigger_mode == "block-offset" {
        dev_auto_sell_block_offset.clone()
    } else {
        String::new()
    };
    let dev_auto_sell_delay_ms = if dev_auto_sell_trigger_mode == "submit-delay" {
        dev_auto_sell_delay_ms
    } else {
        String::new()
    };
    let follow_launch_enabled = form.followLaunch.enabled
        || form.sniperEnabled
        || !follow_launch_snipes.is_empty()
        || form.automaticDevSellEnabled
        || form.followLaunch.devAutoSell.enabled;
    Ok(RawConfig {
        mode: mode.clone(),
        launchpad,
        token: RawToken {
            name: form.name.trim().to_string(),
            symbol: form.symbol.trim().to_string(),
            uri: form.metadataUri.trim().to_string(),
            description: form.description.trim().to_string(),
            website: form.website.trim().to_string(),
            twitter: form.twitter.trim().to_string(),
            telegram: form.telegram.trim().to_string(),
            mayhemMode: Some(json!(form.mayhemMode)),
            cashback: if mode == "cashback" {
                Some(json!(true))
            } else {
                None
            },
        },
        signer: Default::default(),
        agent: RawAgent {
            authority: if is_agent_locked {
                String::new()
            } else {
                form.agentAuthority.trim().to_string()
            },
            buybackBps: buyback_bps.map(|value| json!(value)),
            splitAgentInit: Some(json!(is_agent_custom || mode == "agent-locked")),
            feeReceiver: String::new(),
            feeRecipients: if is_agent_locked || is_agent_unlocked {
                vec![]
            } else {
                agent_fee_recipients
            },
        },
        tx: RawTx {
            computeUnitLimit: Some(json!(FIXED_COMPUTE_UNIT_LIMIT)),
            computeUnitPriceMicroLamports: Some(json!(lamports_to_priority_fee_micro_lamports(
                priority_fee_lamports
            ))),
            jitoTipLamports: Some(json!(tip_lamports)),
            jitoTipAccount: if tip_lamports > 0 {
                pick_tip_account(&provider)
            } else {
                String::new()
            },
            lookupTables: vec![],
            useDefaultLookupTables: Some(json!(true)),
            dumpBase64: Some(json!(false)),
            writeReport: Some(json!(true)),
        },
        feeSharing: RawFeeSharing {
            generateLaterSetup: Some(json!(fee_split_enabled)),
            recipients: fee_sharing_recipients,
        },
        creatorFee: RawCreatorFee {
            mode: if mode == "cashback" {
                "cashback".to_string()
            } else if is_agent_locked {
                "agent-escrow".to_string()
            } else {
                "deployer".to_string()
            },
            ..RawCreatorFee::default()
        },
        execution: RawExecution {
            simulate: Some(json!(action == "simulate")),
            send: Some(json!(action == "send")),
            txFormat: "auto".to_string(),
            commitment: "confirmed".to_string(),
            skipPreflight: Some(json!(if provider == "helius-sender" {
                true
            } else {
                form.skipPreflight
            })),
            trackSendBlockHeight: Some(json!(form.trackSendBlockHeight)),
            provider: provider.clone(),
            endpointProfile: endpoint_profile.clone(),
            policy: String::new(),
            autoGas: Some(json!(true)),
            autoMode: "launchAuto".to_string(),
            priorityFeeSol: form.priorityFeeSol.trim().to_string(),
            tipSol: if tip_lamports > 0 {
                form.creationTipSol.trim().to_string()
            } else {
                String::new()
            },
            maxPriorityFeeSol: form.priorityFeeSol.trim().to_string(),
            maxTipSol: if tip_lamports > 0 {
                form.creationTipSol.trim().to_string()
            } else {
                String::new()
            },
            buyProvider: buy_provider,
            buyEndpointProfile: buy_endpoint_profile,
            buyPolicy: String::new(),
            buyAutoGas: Some(json!(true)),
            buyAutoMode: "buyAuto".to_string(),
            buyPriorityFeeSol: form.buyPriorityFeeSol.trim().to_string(),
            buyTipSol: if buy_tip_supported {
                form.buyTipSol.trim().to_string()
            } else {
                String::new()
            },
            buySlippagePercent: form.buySlippagePercent.trim().to_string(),
            buyMaxPriorityFeeSol: form.buyPriorityFeeSol.trim().to_string(),
            buyMaxTipSol: if buy_tip_supported {
                form.buyTipSol.trim().to_string()
            } else {
                String::new()
            },
            sellProvider: sell_provider,
            sellEndpointProfile: sell_endpoint_profile,
            sellPolicy: String::new(),
            sellPriorityFeeSol: form.sellPriorityFeeSol.trim().to_string(),
            sellTipSol: if sell_tip_supported {
                form.sellTipSol.trim().to_string()
            } else {
                String::new()
            },
            sellSlippagePercent: form.sellSlippagePercent.trim().to_string(),
        },
        initialBuySol: if form.devBuyMode.trim().eq_ignore_ascii_case("sol") {
            form.devBuyAmount.trim().to_string()
        } else {
            String::new()
        },
        initialBuyTokens: if form.devBuyMode.trim().eq_ignore_ascii_case("tokens") {
            form.devBuyAmount.trim().to_string()
        } else {
            String::new()
        },
        devBuy: None,
        postLaunch: RawPostLaunch {
            strategy: form.postLaunchStrategy.trim().to_string(),
            snipeOwnLaunch: RawSnipeOwnLaunch {
                buyAmountSol: form.snipeBuyAmountSol.trim().to_string(),
            },
            automaticDevSell: RawAutomaticDevSell {
                enabled: Some(json!(form.automaticDevSellEnabled)),
                percent: Some(json!(form.automaticDevSellPercent.trim())),
                delaySeconds: Some(json!("0")),
            },
        },
        followLaunch: RawFollowLaunch {
            enabled: Some(json!(follow_launch_enabled)),
            schemaVersion: Some(json!(1)),
            snipes: follow_launch_snipes,
            devAutoSell: if form.followLaunch.devAutoSell.enabled
                || form.automaticDevSellEnabled
                || !dev_auto_sell_percent.trim().is_empty()
            {
                Some(RawFollowLaunchSell {
                    actionId: "dev-auto-sell".to_string(),
                    enabled: Some(json!(true)),
                    walletEnvKey: selected_wallet_key.clone(),
                    percent: Some(json!(dev_auto_sell_percent)),
                    delayMs: Some(json!(dev_auto_sell_delay_ms)),
                    targetBlockOffset: Some(json!(dev_auto_sell_target_block_offset)),
                    marketCap: RawFollowLaunchMarketCapTrigger {
                        enabled: Some(json!(
                            !form
                                .followLaunch
                                .devAutoSell
                                .marketCapThreshold
                                .trim()
                                .is_empty()
                        )),
                        direction: if form
                            .followLaunch
                            .devAutoSell
                            .marketCapDirection
                            .trim()
                            .is_empty()
                        {
                            "gte".to_string()
                        } else {
                            form.followLaunch
                                .devAutoSell
                                .marketCapDirection
                                .trim()
                                .to_string()
                        },
                        threshold: form
                            .followLaunch
                            .devAutoSell
                            .marketCapThreshold
                            .trim()
                            .to_string(),
                    },
                    precheckRequired: Some(json!(false)),
                    requireConfirmation: Some(json!(dev_auto_sell_require_confirmation)),
                })
            } else {
                None
            },
            constraints: RawFollowLaunchConstraints {
                pumpOnly: Some(json!(true)),
                retryBudget: Some(json!(1)),
                requireDaemonReadiness: Some(json!(true)),
                blockOnRequiredPrechecks: Some(json!(true)),
            },
        },
        presets: RawPresets {
            activePresetId: form.activePresetId.trim().to_string(),
            selectedLaunchPresetId: form.activePresetId.trim().to_string(),
            selectedSniperPresetId: form.activePresetId.trim().to_string(),
        },
        imageLocalPath: if image_file_name.is_empty() {
            String::new()
        } else {
            uploads_dir().join(&image_file_name).display().to_string()
        },
        selectedWalletKey: selected_wallet_key,
        vanityPrivateKey: form.vanityPrivateKey.trim().to_string(),
    })
}

pub async fn upload_metadata_from_form(form_value: Value) -> Result<String, String> {
    let form: UiForm = serde_json::from_value(form_value)
        .map_err(|error| format!("Invalid launch form payload: {error}"))?;
    let raw = build_raw_config_from_ui_form("send", form).await?;
    upload_metadata(&raw).await
}

pub async fn build_raw_config_from_form(
    action: &str,
    form_value: Value,
) -> Result<(RawConfig, Option<String>), String> {
    let form: UiForm = serde_json::from_value(form_value)
        .map_err(|error| format!("Invalid launch form payload: {error}"))?;
    let existing_metadata_uri = provided_metadata_uri(&form);
    let mut raw = build_raw_config_from_ui_form(action, form).await?;
    let metadata_uri = if let Some(metadata_uri) = existing_metadata_uri {
        metadata_uri
    } else if action == "send" {
        upload_metadata(&raw).await?
    } else {
        return Err(format!(
            "Metadata is still uploading. Wait for the metadata pre-upload to finish before {action}."
        ));
    };
    raw.token.uri = metadata_uri.clone();
    Ok((raw, Some(metadata_uri)))
}

#[cfg(test)]
mod tests {
    use super::{
        MetadataUploadProvider, UiFollowLaunch, UiFollowLaunchSell, UiForm, UiSniperWalletInput,
        build_raw_config_from_ui_form, normalize_metadata_uri, parse_metadata_upload_provider,
    };
    use serde_json::json;

    #[test]
    fn keeps_ipfs_uri_unchanged() {
        assert_eq!(
            normalize_metadata_uri("ipfs://QmExampleCid"),
            "ipfs://QmExampleCid"
        );
    }

    #[test]
    fn normalizes_ipfs_gateway_uri() {
        assert_eq!(
            normalize_metadata_uri("https://ipfs.io/ipfs/QmExampleCid"),
            "ipfs://QmExampleCid"
        );
    }

    #[test]
    fn normalizes_gateway_uri_with_nested_path() {
        assert_eq!(
            normalize_metadata_uri("https://example.com/ipfs/QmExampleCid/metadata.json"),
            "ipfs://QmExampleCid/metadata.json"
        );
    }

    #[test]
    fn leaves_non_ipfs_url_unchanged() {
        assert_eq!(
            normalize_metadata_uri("https://example.com/metadata.json"),
            "https://example.com/metadata.json"
        );
    }

    #[test]
    fn metadata_upload_provider_defaults_to_pump_fun() {
        assert_eq!(
            parse_metadata_upload_provider("").expect("provider"),
            MetadataUploadProvider::PumpFun
        );
    }

    #[test]
    fn metadata_upload_provider_accepts_pinata() {
        assert_eq!(
            parse_metadata_upload_provider("pinata").expect("provider"),
            MetadataUploadProvider::Pinata
        );
    }

    #[tokio::test]
    async fn agent_unlocked_preserves_configured_buyback_percent() {
        let raw = build_raw_config_from_ui_form(
            "send",
            UiForm {
                mode: "agent-unlocked".to_string(),
                buybackPercent: "1".to_string(),
                ..UiForm::default()
            },
        )
        .await
        .expect("agent-unlocked config should build");

        assert_eq!(raw.agent.buybackBps, Some(json!(100)));
        assert_eq!(raw.creatorFee.mode, "deployer");
    }

    #[tokio::test]
    async fn agent_unlocked_allows_zero_buyback_percent() {
        let raw = build_raw_config_from_ui_form(
            "send",
            UiForm {
                mode: "agent-unlocked".to_string(),
                buybackPercent: "0".to_string(),
                ..UiForm::default()
            },
        )
        .await
        .expect("zero buyback should be accepted");

        assert_eq!(raw.agent.buybackBps, Some(json!(0)));
    }

    #[tokio::test]
    async fn preserves_vanity_private_key_from_ui_form() {
        let raw = build_raw_config_from_ui_form(
            "send",
            UiForm {
                vanityPrivateKey: " vanity-secret ".to_string(),
                ..UiForm::default()
            },
        )
        .await
        .expect("vanity key should be preserved");

        assert_eq!(raw.vanityPrivateKey, "vanity-secret");
    }

    #[tokio::test]
    async fn builds_structured_follow_launch_from_ui_payload() {
        let raw = build_raw_config_from_ui_form(
            "send",
            UiForm {
                selectedWalletKey: "SOLANA_PRIVATE_KEY".to_string(),
                sniperEnabled: true,
                sniperWallets: vec![UiSniperWalletInput {
                    envKey: "SOLANA_PRIVATE_KEY2".to_string(),
                    amountSol: "0.25".to_string(),
                    triggerMode: "submit-delay".to_string(),
                    submitDelayMs: Some(25),
                    jitterMs: Some(5),
                    feeJitterBps: Some(200),
                    ..UiSniperWalletInput::default()
                }],
                followLaunch: UiFollowLaunch {
                    enabled: true,
                    devAutoSell: UiFollowLaunchSell {
                        enabled: true,
                        percent: "50".to_string(),
                        triggerMode: "submit-delay".to_string(),
                        delayMs: "2000".to_string(),
                        ..UiFollowLaunchSell::default()
                    },
                    ..UiFollowLaunch::default()
                },
                automaticDevSellEnabled: true,
                automaticDevSellPercent: "50".to_string(),
                automaticDevSellTriggerMode: "submit-delay".to_string(),
                automaticDevSellDelayMs: "2000".to_string(),
                ..UiForm::default()
            },
        )
        .await
        .expect("follow launch config should build");

        assert_eq!(raw.followLaunch.snipes.len(), 1);
        assert_eq!(
            raw.followLaunch.snipes[0].walletEnvKey,
            "SOLANA_PRIVATE_KEY2"
        );
        assert_eq!(raw.followLaunch.snipes[0].buyAmountSol, "0.25");
        assert_eq!(raw.followLaunch.snipes[0].submitWithLaunch, Some(json!(false)));
        assert_eq!(raw.followLaunch.snipes[0].submitDelayMs, Some(json!(25)));
        let dev_auto_sell = raw
            .followLaunch
            .devAutoSell
            .expect("dev auto sell should be present");
        assert_eq!(dev_auto_sell.walletEnvKey, "SOLANA_PRIVATE_KEY");
        assert_eq!(dev_auto_sell.percent, Some(json!("50")));
        assert_eq!(dev_auto_sell.delayMs, Some(json!("2000")));
    }
}
