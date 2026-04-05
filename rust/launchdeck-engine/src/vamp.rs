#![allow(non_snake_case, dead_code)]

use crate::{
    bags_native::{BagsImportContext, BagsImportRecipient, detect_bags_import_context},
    bonk_native::{BonkImportContext, detect_bonk_import_context},
    image_library::{SerializedImageRecord, save_image_bytes},
    rpc::{fetch_account_data, fetch_multiple_account_data},
};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::{path::Path, str::FromStr};

const PUMP_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const PUMP_FEE_PROGRAM_ID: &str = "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ";
const PUMP_AGENT_PAYMENTS_PROGRAM_ID: &str = "AgenTMiC2hvxGebTsgmsD4HHBa8WEcqGFf87iwRRxLo7";
const PLATFORM_GITHUB: u8 = 2;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ImportedRouteRecipient {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub githubUsername: String,
    #[serde(default)]
    pub githubUserId: String,
    #[serde(default)]
    pub shareBps: i64,
    #[serde(default)]
    pub sourceProvider: String,
    #[serde(default)]
    pub sourceUsername: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ImportedCreatorFeeRoute {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub githubUsername: String,
    #[serde(default)]
    pub githubUserId: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ImportedRouteData {
    #[serde(default)]
    pub feeSharingRecipients: Vec<ImportedRouteRecipient>,
    #[serde(default)]
    pub agentFeeRecipients: Vec<ImportedRouteRecipient>,
    #[serde(default)]
    pub creatorFee: Option<ImportedCreatorFeeRoute>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ImportedDetectionSummary {
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ImportedTokenData {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub website: String,
    pub twitter: String,
    pub telegram: String,
    pub imageUrl: String,
    pub metadataUri: String,
    #[serde(default)]
    pub launchpad: String,
    #[serde(default)]
    pub quoteAsset: String,
    pub mode: String,
    pub source: String,
    #[serde(default)]
    pub routes: ImportedRouteData,
    #[serde(default)]
    pub detection: ImportedDetectionSummary,
}

fn normalize_http_url(raw_value: &str) -> String {
    let raw = raw_value.trim();
    if raw.is_empty() {
        return String::new();
    }
    let with_protocol = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else {
        format!("https://{}", raw.trim_start_matches("//"))
    };
    reqwest::Url::parse(&with_protocol)
        .ok()
        .filter(|url| url.scheme() == "http" || url.scheme() == "https")
        .map(|url| url.to_string())
        .unwrap_or_default()
}

fn normalize_social_url(raw_value: &str, kind: &str) -> String {
    let raw = raw_value.trim();
    if raw.is_empty() {
        return String::new();
    }
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("//") {
        return normalize_http_url(raw);
    }
    let normalized = raw.trim_start_matches('@').trim_start_matches('/').trim();
    if normalized.is_empty() {
        return String::new();
    }
    match kind {
        "twitter" => format!("https://x.com/{normalized}"),
        "telegram" => format!("https://t.me/{normalized}"),
        _ => normalize_http_url(raw),
    }
}

fn is_ephemeral_launchblitz_image(url: &str) -> bool {
    let normalized = url.trim().to_ascii_lowercase();
    normalized.contains("ipfs.launchblitz.ai/async/")
}

fn choose_merged_image_url(base: &ImportedTokenData, overlay: &ImportedTokenData) -> String {
    let base_image = base.imageUrl.trim();
    let overlay_image = overlay.imageUrl.trim();
    if base_image.is_empty() {
        return overlay_image.to_string();
    }
    if overlay_image.is_empty() {
        return base_image.to_string();
    }
    if is_ephemeral_launchblitz_image(base_image)
        && overlay.source.eq_ignore_ascii_case("DexScreener")
    {
        return overlay_image.to_string();
    }
    base_image.to_string()
}

fn merge_imported(base: ImportedTokenData, overlay: ImportedTokenData) -> ImportedTokenData {
    let merged_image_url = choose_merged_image_url(&base, &overlay);
    ImportedTokenData {
        name: if !base.name.is_empty() {
            base.name
        } else {
            overlay.name
        },
        symbol: if !base.symbol.is_empty() {
            base.symbol
        } else {
            overlay.symbol
        },
        description: if !base.description.is_empty() {
            base.description
        } else {
            overlay.description
        },
        website: if !base.website.is_empty() {
            base.website
        } else {
            overlay.website
        },
        twitter: if !base.twitter.is_empty() {
            base.twitter
        } else {
            overlay.twitter
        },
        telegram: if !base.telegram.is_empty() {
            base.telegram
        } else {
            overlay.telegram
        },
        imageUrl: merged_image_url,
        metadataUri: if !base.metadataUri.is_empty() {
            base.metadataUri
        } else {
            overlay.metadataUri
        },
        launchpad: if !base.launchpad.is_empty() {
            base.launchpad
        } else {
            overlay.launchpad
        },
        quoteAsset: if !base.quoteAsset.is_empty() {
            base.quoteAsset
        } else {
            overlay.quoteAsset
        },
        mode: if !base.mode.is_empty() {
            base.mode
        } else {
            overlay.mode
        },
        source: if !base.source.is_empty() {
            base.source
        } else {
            overlay.source
        },
        routes: if base.routes.feeSharingRecipients.is_empty()
            && base.routes.agentFeeRecipients.is_empty()
            && base.routes.creatorFee.is_none()
            && base.routes.notes.is_empty()
        {
            overlay.routes
        } else {
            base.routes
        },
        detection: if base.detection.sources.is_empty() && base.detection.notes.is_empty() {
            overlay.detection
        } else {
            base.detection
        },
    }
}

fn push_unique(values: &mut Vec<String>, next: String) {
    let normalized = next.trim();
    if normalized.is_empty() || values.iter().any(|entry| entry == normalized) {
        return;
    }
    values.push(normalized.to_string());
}

fn extend_unique(values: &mut Vec<String>, next: Vec<String>) {
    for entry in next {
        push_unique(values, entry);
    }
}

fn apply_import_context(
    mut base: ImportedTokenData,
    overlay: ImportedTokenData,
) -> ImportedTokenData {
    if !overlay.launchpad.is_empty() {
        base.launchpad = overlay.launchpad;
    }
    if !overlay.mode.is_empty() {
        base.mode = overlay.mode;
    }
    if !overlay.quoteAsset.is_empty() {
        base.quoteAsset = overlay.quoteAsset;
    }
    if !overlay.source.is_empty() {
        base.source = overlay.source.clone();
        push_unique(&mut base.detection.sources, overlay.source);
    }
    extend_unique(&mut base.detection.sources, overlay.detection.sources);
    extend_unique(&mut base.detection.notes, overlay.detection.notes);
    extend_unique(&mut base.routes.notes, overlay.routes.notes);
    if !overlay.routes.feeSharingRecipients.is_empty() {
        base.routes.feeSharingRecipients = overlay.routes.feeSharingRecipients;
    }
    if !overlay.routes.agentFeeRecipients.is_empty() {
        base.routes.agentFeeRecipients = overlay.routes.agentFeeRecipients;
    }
    if overlay.routes.creatorFee.is_some() {
        base.routes.creatorFee = overlay.routes.creatorFee;
    }
    base
}

fn infer_imported_mode(payload: &Value) -> String {
    if payload
        .get("is_cashback_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return "cashback".to_string();
    }
    if payload
        .get("tokenized_agent")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        // Pump's public coin payload exposes agent presence but not enough detail
        // to distinguish custom vs locked vs unlocked reward routing.
        return "agent-locked".to_string();
    }
    "regular".to_string()
}

fn normalize_imported_metadata_payload(payload: &Value, source: &str) -> ImportedTokenData {
    let websites = payload
        .get("websites")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let socials = payload
        .get("socials")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let first_website = websites
        .iter()
        .find_map(|entry| entry.get("url").and_then(Value::as_str))
        .unwrap_or_default();
    let twitter_social = socials
        .iter()
        .find(|entry| {
            entry
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .eq_ignore_ascii_case("twitter")
        })
        .and_then(|entry| entry.get("url"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let telegram_social = socials
        .iter()
        .find(|entry| {
            entry
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .eq_ignore_ascii_case("telegram")
        })
        .and_then(|entry| entry.get("url"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    ImportedTokenData {
        name: payload
            .get("name")
            .or_else(|| payload.get("token_name"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string(),
        symbol: payload
            .get("symbol")
            .or_else(|| payload.get("ticker"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string(),
        description: payload
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string(),
        website: normalize_http_url(
            payload
                .get("website")
                .or_else(|| payload.get("external_url"))
                .and_then(Value::as_str)
                .unwrap_or(first_website),
        ),
        twitter: normalize_social_url(
            payload
                .get("twitter")
                .and_then(Value::as_str)
                .unwrap_or(twitter_social),
            "twitter",
        ),
        telegram: normalize_social_url(
            payload
                .get("telegram")
                .and_then(Value::as_str)
                .unwrap_or(telegram_social),
            "telegram",
        ),
        imageUrl: normalize_http_url(
            payload
                .get("image_uri")
                .or_else(|| payload.get("image"))
                .or_else(|| payload.get("imageUrl"))
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ),
        metadataUri: normalize_http_url(
            payload
                .get("metadataUri")
                .or_else(|| payload.get("metadata_uri"))
                .or_else(|| payload.get("uri"))
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ),
        mode: infer_imported_mode(payload),
        source: source.to_string(),
        launchpad: String::new(),
        quoteAsset: String::new(),
        routes: ImportedRouteData::default(),
        detection: ImportedDetectionSummary::default(),
    }
}

async fn fetch_json_or_null(client: &Client, url: &str) -> Option<Value> {
    let response = client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json::<Value>().await.ok()
}

fn parse_pubkey(value: &str, label: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(value).map_err(|error| format!("Invalid {label}: {error}"))
}

fn pump_program_id() -> Result<Pubkey, String> {
    parse_pubkey(PUMP_PROGRAM_ID, "Pump program id")
}

fn pump_fee_program_id() -> Result<Pubkey, String> {
    parse_pubkey(PUMP_FEE_PROGRAM_ID, "Pump fee program id")
}

fn pump_agent_payments_program_id() -> Result<Pubkey, String> {
    parse_pubkey(
        PUMP_AGENT_PAYMENTS_PROGRAM_ID,
        "Pump agent payments program id",
    )
}

fn pump_bonding_curve_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(&[b"bonding-curve", mint.as_ref()], &pump_program_id()?).0)
}

fn pump_fee_sharing_config_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(
        Pubkey::find_program_address(&[b"sharing-config", mint.as_ref()], &pump_fee_program_id()?)
            .0,
    )
}

fn pump_token_agent_payments_pda(mint: &Pubkey) -> Result<Pubkey, String> {
    Ok(Pubkey::find_program_address(
        &[b"token-agent-payments", mint.as_ref()],
        &pump_agent_payments_program_id()?,
    )
    .0)
}

fn read_bool(data: &[u8], offset: &mut usize) -> Result<bool, String> {
    let Some(byte) = data.get(*offset) else {
        return Err("Unexpected end of Pump bonding curve account.".to_string());
    };
    *offset += 1;
    Ok(*byte != 0)
}

fn read_u64(data: &[u8], offset: &mut usize) -> Result<u64, String> {
    let end = offset.saturating_add(8);
    let bytes: [u8; 8] = data
        .get(*offset..end)
        .ok_or_else(|| "Unexpected end of Pump bonding curve account.".to_string())?
        .try_into()
        .map_err(|_| "Failed to decode Pump u64 field.".to_string())?;
    *offset = end;
    Ok(u64::from_le_bytes(bytes))
}

fn read_u32(data: &[u8], offset: &mut usize) -> Result<u32, String> {
    let end = offset.saturating_add(4);
    let bytes: [u8; 4] = data
        .get(*offset..end)
        .ok_or_else(|| "Unexpected end of Pump account while reading u32.".to_string())?
        .try_into()
        .map_err(|_| "Failed to decode Pump u32 field.".to_string())?;
    *offset = end;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u16(data: &[u8], offset: &mut usize) -> Result<u16, String> {
    let end = offset.saturating_add(2);
    let bytes: [u8; 2] = data
        .get(*offset..end)
        .ok_or_else(|| "Unexpected end of Pump account while reading u16.".to_string())?
        .try_into()
        .map_err(|_| "Failed to decode Pump u16 field.".to_string())?;
    *offset = end;
    Ok(u16::from_le_bytes(bytes))
}

fn read_string(data: &[u8], offset: &mut usize) -> Result<String, String> {
    let length = read_u32(data, offset)? as usize;
    let end = offset.saturating_add(length);
    let bytes = data
        .get(*offset..end)
        .ok_or_else(|| "Unexpected end of Pump account while reading string.".to_string())?;
    *offset = end;
    String::from_utf8(bytes.to_vec())
        .map_err(|error| format!("Failed to decode Pump string: {error}"))
}

fn read_pubkey(data: &[u8], offset: &mut usize) -> Result<Pubkey, String> {
    let end = offset.saturating_add(32);
    let bytes: [u8; 32] = data
        .get(*offset..end)
        .ok_or_else(|| "Unexpected end of Pump bonding curve account.".to_string())?
        .try_into()
        .map_err(|_| "Failed to decode Pump pubkey field.".to_string())?;
    *offset = end;
    Ok(Pubkey::new_from_array(bytes))
}

fn parse_pump_creator_and_cashback(data: &[u8]) -> Result<(Pubkey, bool), String> {
    let mut offset = 8usize;
    let _virtual_token_reserves = read_u64(data, &mut offset)?;
    let _virtual_sol_reserves = read_u64(data, &mut offset)?;
    let _real_token_reserves = read_u64(data, &mut offset)?;
    let _real_sol_reserves = read_u64(data, &mut offset)?;
    let _token_total_supply = read_u64(data, &mut offset)?;
    let _complete = read_bool(data, &mut offset)?;
    let creator = read_pubkey(data, &mut offset)?;
    let cashback_enabled = data.len() > 82 && data[82] != 0;
    Ok((creator, cashback_enabled))
}

fn parse_pump_social_fee_pda(data: &[u8]) -> Result<(String, u8), String> {
    if data.len() < 8 + 1 + 1 + 4 + 1 + 8 + 8 {
        return Err("Pump social fee PDA account was too short.".to_string());
    }
    let mut offset = 8usize;
    let _bump = data[offset];
    offset += 1;
    let _version = data[offset];
    offset += 1;
    let user_id = read_string(data, &mut offset)?;
    let Some(platform) = data.get(offset) else {
        return Err("Unexpected end of Pump social fee PDA while reading platform.".to_string());
    };
    Ok((user_id, *platform))
}

async fn resolve_github_username_from_id(user_id: &str) -> Option<String> {
    let trimmed = user_id.trim();
    if trimmed.is_empty() {
        return None;
    }
    let response = Client::new()
        .get(format!("https://api.github.com/user/{trimmed}"))
        .header("accept", "application/vnd.github+json")
        .header("user-agent", "launchdeck-rust-engine")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let payload = response.json::<Value>().await.ok()?;
    payload
        .get("login")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

async fn resolve_pump_shareholder_recipient(
    rpc_url: &str,
    address: Pubkey,
    share_bps: i64,
) -> ImportedRouteRecipient {
    if let Ok(account_data) = fetch_account_data(rpc_url, &address.to_string(), "confirmed").await
        && let Ok((user_id, platform)) = parse_pump_social_fee_pda(&account_data)
        && platform == PLATFORM_GITHUB
    {
        let github_username = resolve_github_username_from_id(&user_id)
            .await
            .unwrap_or_default();
        return ImportedRouteRecipient {
            r#type: "github".to_string(),
            address: String::new(),
            githubUsername: github_username.clone(),
            githubUserId: user_id,
            shareBps: share_bps,
            sourceProvider: "github".to_string(),
            sourceUsername: github_username,
        };
    }
    ImportedRouteRecipient {
        r#type: "wallet".to_string(),
        address: address.to_string(),
        githubUsername: String::new(),
        githubUserId: String::new(),
        shareBps: share_bps,
        sourceProvider: String::new(),
        sourceUsername: String::new(),
    }
}

async fn parse_pump_sharing_config_recipients(
    rpc_url: &str,
    data: &[u8],
) -> Result<Vec<ImportedRouteRecipient>, String> {
    if data.len() < 8 + 1 + 1 + 1 + 32 + 32 + 1 + 4 {
        return Err("Pump sharing config account was too short.".to_string());
    }
    let mut offset = 8usize;
    let _bump = data[offset];
    offset += 1;
    let _version = data[offset];
    offset += 1;
    let _status = data[offset];
    offset += 1;
    let _mint = read_pubkey(data, &mut offset)?;
    let _admin = read_pubkey(data, &mut offset)?;
    let _admin_revoked = read_bool(data, &mut offset)?;
    let count = read_u32(data, &mut offset)? as usize;
    let mut entries = Vec::with_capacity(count);
    let mut account_addresses = Vec::with_capacity(count);
    for _ in 0..count {
        let address = read_pubkey(data, &mut offset)?;
        let share_bps = i64::from(read_u16(data, &mut offset)?);
        account_addresses.push(address.to_string());
        entries.push((address, share_bps));
    }
    let account_data =
        fetch_multiple_account_data(rpc_url, &account_addresses, "confirmed").await?;
    let mut recipients = Vec::with_capacity(count);
    for ((address, share_bps), maybe_account_data) in
        entries.into_iter().zip(account_data.into_iter())
    {
        if let Some(account_data) = maybe_account_data
            && let Ok((user_id, platform)) = parse_pump_social_fee_pda(&account_data)
            && platform == PLATFORM_GITHUB
        {
            let github_username = resolve_github_username_from_id(&user_id)
                .await
                .unwrap_or_default();
            recipients.push(ImportedRouteRecipient {
                r#type: "github".to_string(),
                address: String::new(),
                githubUsername: github_username.clone(),
                githubUserId: user_id,
                shareBps: share_bps,
                sourceProvider: "github".to_string(),
                sourceUsername: github_username,
            });
            continue;
        }
        recipients.push(ImportedRouteRecipient {
            r#type: "wallet".to_string(),
            address: address.to_string(),
            githubUsername: String::new(),
            githubUserId: String::new(),
            shareBps: share_bps,
            sourceProvider: String::new(),
            sourceUsername: String::new(),
        });
    }
    Ok(recipients)
}

async fn detect_pump_import_context(
    rpc_url: &str,
    mint: &Pubkey,
    pump_payload: Option<&Value>,
) -> Result<Option<ImportedTokenData>, String> {
    let bonding_curve_address = pump_bonding_curve_pda(mint)?.to_string();
    let agent_payments_address = pump_token_agent_payments_pda(mint)?.to_string();
    let fee_sharing_address = pump_fee_sharing_config_pda(mint)?.to_string();
    let batch_accounts = fetch_multiple_account_data(
        rpc_url,
        &[
            bonding_curve_address,
            agent_payments_address,
            fee_sharing_address.clone(),
        ],
        "confirmed",
    )
    .await?;
    if let Some(data) = batch_accounts.first().cloned().flatten() {
        let (creator, cashback_enabled) = parse_pump_creator_and_cashback(&data)?;
        let agent_enabled = batch_accounts.get(1).is_some_and(|value| value.is_some());
        let fee_sharing_data = batch_accounts.get(2).cloned().flatten();
        let fee_sharing_enabled = fee_sharing_data.is_some();
        let mut imported = ImportedTokenData {
            launchpad: "pump".to_string(),
            mode: if agent_enabled {
                "agent-locked".to_string()
            } else if cashback_enabled {
                "cashback".to_string()
            } else {
                infer_imported_mode(pump_payload.unwrap_or(&Value::Null))
            },
            source: "pump-state".to_string(),
            routes: if agent_enabled {
                ImportedRouteData::default()
            } else {
                ImportedRouteData {
                    creatorFee: Some(ImportedCreatorFeeRoute {
                        mode: "wallet".to_string(),
                        address: creator.to_string(),
                        githubUsername: String::new(),
                        githubUserId: String::new(),
                    }),
                    ..ImportedRouteData::default()
                }
            },
            detection: ImportedDetectionSummary {
                sources: vec!["pump-state".to_string()],
                notes: Vec::new(),
            },
            ..ImportedTokenData::default()
        };
        if fee_sharing_enabled && !agent_enabled {
            if let Some(config_data) = fee_sharing_data {
                if let Ok(recipients) =
                    parse_pump_sharing_config_recipients(rpc_url, &config_data).await
                {
                    imported.routes.feeSharingRecipients = recipients;
                } else {
                    imported.detection.notes.push(
                        "Pump fee-sharing config exists, but recipient decoding failed."
                            .to_string(),
                    );
                }
            }
            imported
                .detection
                .notes
                .push("Pump fee-sharing config detected from current state.".to_string());
        } else if agent_enabled && fee_sharing_enabled {
            imported
                .detection
                .notes
                .push("Agent-style Pump import detected; normal Pump fee-sharing state was ignored and mode was normalized to agent-locked.".to_string());
        }
        return Ok(Some(imported));
    }
    if let Some(payload) = pump_payload {
        return Ok(Some(ImportedTokenData {
            launchpad: "pump".to_string(),
            mode: infer_imported_mode(payload),
            source: "pump.fun".to_string(),
            detection: ImportedDetectionSummary {
                sources: vec!["pump.fun".to_string()],
                notes: Vec::new(),
            },
            ..ImportedTokenData::default()
        }));
    }
    Ok(None)
}

fn map_bonk_import_context(context: BonkImportContext) -> ImportedTokenData {
    ImportedTokenData {
        launchpad: context.launchpad,
        mode: context.mode,
        quoteAsset: context.quoteAsset,
        source: if context.detectionSource.trim().is_empty() {
            "bonk-state".to_string()
        } else {
            context.detectionSource
        },
        detection: ImportedDetectionSummary {
            sources: vec!["bonk-state".to_string()],
            notes: Vec::new(),
        },
        ..ImportedTokenData::default()
    }
}

fn map_bags_import_recipient(entry: BagsImportRecipient) -> ImportedRouteRecipient {
    ImportedRouteRecipient {
        r#type: entry.r#type,
        address: entry.address,
        githubUsername: entry.githubUsername,
        githubUserId: String::new(),
        shareBps: entry.shareBps,
        sourceProvider: entry.sourceProvider,
        sourceUsername: entry.sourceUsername,
    }
}

fn map_bags_import_context(context: BagsImportContext) -> ImportedTokenData {
    ImportedTokenData {
        launchpad: context.launchpad,
        mode: context.mode,
        quoteAsset: if context.quoteAsset.trim().is_empty() {
            "sol".to_string()
        } else {
            context.quoteAsset
        },
        source: if context.detectionSource.trim().is_empty() {
            "bags-state".to_string()
        } else {
            context.detectionSource
        },
        routes: ImportedRouteData {
            feeSharingRecipients: context
                .feeRecipients
                .into_iter()
                .map(map_bags_import_recipient)
                .collect(),
            notes: context.notes.clone(),
            ..ImportedRouteData::default()
        },
        detection: ImportedDetectionSummary {
            sources: vec!["bags-state".to_string()],
            notes: context.notes,
        },
        ..ImportedTokenData::default()
    }
}

fn infer_launchpad_hint(contract_address: &str) -> Option<&'static str> {
    let normalized = contract_address.trim();
    if normalized.ends_with("pump") {
        Some("pump")
    } else if normalized.ends_with("bonk") {
        Some("bonk")
    } else if normalized.to_ascii_lowercase().ends_with("bags") {
        Some("bagsapp")
    } else {
        None
    }
}

fn infer_image_extension_from_bytes(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 8
        && bytes[0] == 0x89
        && bytes[1] == b'P'
        && bytes[2] == b'N'
        && bytes[3] == b'G'
        && bytes[4] == 0x0D
        && bytes[5] == 0x0A
        && bytes[6] == 0x1A
        && bytes[7] == 0x0A
    {
        return Some(".png");
    }
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some(".jpg");
    }
    if bytes.len() >= 6 && (&bytes[..6] == b"GIF87a" || &bytes[..6] == b"GIF89a") {
        return Some(".gif");
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(".webp");
    }
    None
}

fn imported_has_any_content(imported: &ImportedTokenData) -> bool {
    !imported.name.is_empty()
        || !imported.symbol.is_empty()
        || !imported.imageUrl.is_empty()
        || !imported.website.is_empty()
        || !imported.twitter.is_empty()
        || !imported.telegram.is_empty()
        || !imported.launchpad.is_empty()
}

async fn enrich_from_pump_frontend(
    client: &Client,
    contract_address: &str,
    mut imported: ImportedTokenData,
) -> (ImportedTokenData, Option<Value>) {
    let pump_payload = fetch_json_or_null(
        client,
        &format!("https://frontend-api-v3.pump.fun/coins/{contract_address}"),
    )
    .await;
    if let Some(pump_payload) = pump_payload.as_ref() {
        imported = merge_imported(
            imported,
            normalize_imported_metadata_payload(pump_payload, "pump.fun"),
        );
        if !imported.metadataUri.is_empty()
            && let Some(metadata_payload) = fetch_json_or_null(client, &imported.metadataUri).await
        {
            imported = merge_imported(
                imported,
                normalize_imported_metadata_payload(&metadata_payload, "metadata"),
            );
        }
    }
    (imported, pump_payload)
}

async fn enrich_from_dexscreener(
    client: &Client,
    contract_address: &str,
    mut imported: ImportedTokenData,
) -> ImportedTokenData {
    if let Some(dex_payload) = fetch_json_or_null(
        client,
        &format!("https://api.dexscreener.com/latest/dex/tokens/{contract_address}"),
    )
    .await
        && let Some(dex_pair) = dex_payload
            .get("pairs")
            .and_then(Value::as_array)
            .and_then(|entries| entries.iter().find(|entry| !entry.is_null()))
    {
        imported = merge_imported(
            imported,
            normalize_imported_metadata_payload(
                &serde_json::json!({
                    "name": dex_pair.get("baseToken").and_then(|value| value.get("name")).and_then(Value::as_str).unwrap_or_default(),
                    "symbol": dex_pair.get("baseToken").and_then(|value| value.get("symbol")).and_then(Value::as_str).unwrap_or_default(),
                    "imageUrl": dex_pair.get("info").and_then(|value| value.get("imageUrl")).and_then(Value::as_str).unwrap_or_default(),
                    "websites": dex_pair.get("info").and_then(|value| value.get("websites")).cloned().unwrap_or(Value::Array(vec![])),
                    "socials": dex_pair.get("info").and_then(|value| value.get("socials")).cloned().unwrap_or(Value::Array(vec![])),
                }),
                "DexScreener",
            ),
        );
    }
    imported
}

pub async fn fetch_imported_token_metadata(
    contract_address: &str,
    rpc_url: &str,
) -> Result<ImportedTokenData, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|error| error.to_string())?;
    let mint = parse_pubkey(contract_address, "mint")?;
    if let Some(hint) = infer_launchpad_hint(contract_address) {
        let mut hinted = ImportedTokenData::default();
        let mut hint_succeeded = false;
        match hint {
            "pump" => {
                let (next_imported, pump_payload) =
                    enrich_from_pump_frontend(&client, contract_address, hinted).await;
                hinted = next_imported;
                if let Some(context) =
                    detect_pump_import_context(rpc_url, &mint, pump_payload.as_ref()).await?
                {
                    hinted = apply_import_context(hinted, context);
                    hint_succeeded = hinted.launchpad == "pump";
                }
                hinted = enrich_from_dexscreener(&client, contract_address, hinted).await;
            }
            "bonk" => {
                if let Some(context) = detect_bonk_import_context(rpc_url, contract_address).await?
                {
                    hinted = apply_import_context(hinted, map_bonk_import_context(context));
                    hinted = enrich_from_dexscreener(&client, contract_address, hinted).await;
                    hint_succeeded = hinted.launchpad == "bonk";
                }
            }
            "bagsapp" => {
                if let Some(context) = detect_bags_import_context(rpc_url, contract_address).await?
                {
                    hinted = apply_import_context(hinted, map_bags_import_context(context));
                    hinted = enrich_from_dexscreener(&client, contract_address, hinted).await;
                    hint_succeeded = hinted.launchpad == "bagsapp";
                }
            }
            _ => {}
        }
        if hint_succeeded && imported_has_any_content(&hinted) {
            return Ok(hinted);
        }
    }
    let mut imported = ImportedTokenData::default();
    let (next_imported, pump_payload) =
        enrich_from_pump_frontend(&client, contract_address, imported).await;
    imported = next_imported;
    imported = enrich_from_dexscreener(&client, contract_address, imported).await;
    if let Some(context) = detect_pump_import_context(rpc_url, &mint, pump_payload.as_ref()).await?
    {
        imported = apply_import_context(imported, context);
    }
    if let Some(context) = detect_bonk_import_context(rpc_url, contract_address).await? {
        imported = apply_import_context(imported, map_bonk_import_context(context));
    }
    if let Some(context) = detect_bags_import_context(rpc_url, contract_address).await? {
        imported = apply_import_context(imported, map_bags_import_context(context));
    }
    if !imported_has_any_content(&imported) {
        return Err("No token metadata was found for that contract address.".to_string());
    }
    Ok(imported)
}

pub async fn import_remote_image_to_library(
    image_url: &str,
    original_name: &str,
    record_name: &str,
) -> Result<Option<SerializedImageRecord>, String> {
    let safe_url = normalize_http_url(image_url);
    if safe_url.is_empty() {
        return Ok(None);
    }
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .get(&safe_url)
        .send()
        .await
        .map_err(|_| "Unable to download token image.".to_string())?;
    if !response.status().is_success() {
        return Err("Unable to download token image.".to_string());
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let mut extension = match content_type.as_str() {
        "image/png" => Some(".png"),
        "image/jpeg" => Some(".jpg"),
        "image/webp" => Some(".webp"),
        "image/gif" => Some(".gif"),
        _ => None,
    };
    if extension.is_none() {
        extension = Path::new(
            reqwest::Url::parse(&safe_url)
                .ok()
                .and_then(|url| {
                    url.path_segments()
                        .and_then(|mut segments| segments.next_back())
                        .map(|v| v.to_string())
                })
                .unwrap_or_default()
                .as_str(),
        )
        .extension()
        .and_then(|value| value.to_str())
        .map(|ext| match ext.to_ascii_lowercase().as_str() {
            "png" => ".png",
            "jpg" | "jpeg" => ".jpg",
            "webp" => ".webp",
            "gif" => ".gif",
            _ => "",
        })
        .filter(|value| !value.is_empty());
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|_| "Imported token image was empty.".to_string())?;
    if bytes.is_empty() {
        return Err("Imported token image was empty.".to_string());
    }
    if bytes.len() > 8_000_000 {
        return Err("Imported token image is too large.".to_string());
    }
    if extension.is_none() {
        extension = infer_image_extension_from_bytes(bytes.as_ref());
    }
    let Some(extension) = extension else {
        return Err("Imported token image format is not supported.".to_string());
    };
    save_image_bytes(
        &bytes,
        extension,
        original_name,
        if record_name.trim().is_empty() {
            None
        } else {
            Some(record_name)
        },
    )
    .map(Some)
}

#[cfg(test)]
mod tests {
    use super::{
        ImportedTokenData, choose_merged_image_url, infer_image_extension_from_bytes,
        infer_imported_mode,
    };
    use serde_json::json;

    #[test]
    fn infers_cashback_mode_from_pump_payload() {
        assert_eq!(
            infer_imported_mode(&json!({
                "is_cashback_enabled": true,
                "tokenized_agent": false
            })),
            "cashback"
        );
    }

    #[test]
    fn infers_agent_mode_from_pump_payload() {
        assert_eq!(
            infer_imported_mode(&json!({
                "is_cashback_enabled": false,
                "tokenized_agent": true
            })),
            "agent-locked"
        );
    }

    #[test]
    fn defaults_to_regular_mode_when_flags_are_missing() {
        assert_eq!(infer_imported_mode(&json!({})), "regular");
    }

    #[test]
    fn infers_png_extension_from_bytes() {
        assert_eq!(
            infer_image_extension_from_bytes(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
            Some(".png")
        );
    }

    #[test]
    fn infers_webp_extension_from_bytes() {
        assert_eq!(
            infer_image_extension_from_bytes(b"RIFFxxxxWEBP"),
            Some(".webp")
        );
    }

    #[test]
    fn prefers_dexscreener_image_over_dead_launchblitz_image() {
        let base = ImportedTokenData {
            imageUrl: "https://ipfs.launchblitz.ai/async/example.jpg".to_string(),
            ..ImportedTokenData::default()
        };
        let overlay = ImportedTokenData {
            imageUrl: "https://cdn.dexscreener.com/cms/images/example".to_string(),
            source: "DexScreener".to_string(),
            ..ImportedTokenData::default()
        };
        assert_eq!(
            choose_merged_image_url(&base, &overlay),
            "https://cdn.dexscreener.com/cms/images/example"
        );
    }
}
