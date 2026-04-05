//! Regional aggregates and Helius Sender metro codes for execution routing.

use std::collections::HashSet;

const AGGREGATES: &[&str] = &["global", "us", "eu", "asia"];

/// Canonical metro tokens for Helius regional senders. `ny` aliases to `ewr` (Newark, Jito `ny.`).
pub fn metro_token_canonical(t: &str) -> Option<&'static str> {
    match t.trim().to_lowercase().as_str() {
        "slc" => Some("slc"),
        "ewr" | "ny" => Some("ewr"),
        "lon" => Some("lon"),
        "fra" => Some("fra"),
        "ams" => Some("ams"),
        "sg" => Some("sg"),
        "tyo" => Some("tyo"),
        _ => None,
    }
}

fn is_aggregate(s: &str) -> bool {
    AGGREGATES.iter().any(|a| *a == s)
}

fn canonicalize_comma_metro_list(s: &str) -> Result<String, String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for part in s.split(',') {
        let part = part.trim().to_lowercase();
        if part.is_empty() {
            continue;
        }
        if is_aggregate(&part) {
            return Err(
                "comma-separated profiles must list metro codes only (slc, ewr, lon, fra, ams, sg, tyo), not region groups"
                    .to_string(),
            );
        }
        let metro =
            metro_token_canonical(&part).ok_or_else(|| format!("unknown metro token '{part}'"))?;
        if seen.insert(metro) {
            out.push(metro.to_string());
        }
    }
    if out.is_empty() {
        return Err("empty metro list".to_string());
    }
    Ok(out.join(","))
}

/// Strict parse for persisted config: rejects `west`, canonicalizes `ny` → `ewr`.
pub fn parse_config_endpoint_profile(value: &str) -> Result<String, String> {
    let s = value.trim().to_lowercase();
    if s.is_empty() {
        return Err("endpoint profile is empty".to_string());
    }
    if s == "west" {
        return Err(
            "'west' is no longer supported. Use global, us, eu, asia, or Helius sender metros (slc, ewr, lon, fra, ams, sg, tyo); combine metros with commas (e.g. fra,ams). For Newark / NY Jito hosts, use ny (normalized to ewr for Sender) or us for US fanout."
                .to_string(),
        );
    }
    if s.contains(',') {
        return canonicalize_comma_metro_list(&s);
    }
    if is_aggregate(&s) {
        return Ok(s);
    }
    metro_token_canonical(&s)
        .map(|m| m.to_string())
        .ok_or_else(|| {
            format!(
                "unknown endpoint profile '{s}'. Use global, us, eu, asia, or metros: slc, ewr, lon, fra, ams, sg, tyo"
            )
        })
}

/// `USER_REGION` and provider-specific region env: aggregates, one metro, or comma-separated metros.
/// Unknown values (including `west`) return `None` so callers fall back to `global`.
pub fn normalize_user_region(region: &str) -> Option<String> {
    let r = region.trim().to_lowercase();
    if r.is_empty() {
        return None;
    }
    if r == "west" {
        return None;
    }
    if r.contains(',') {
        return canonicalize_comma_metro_list(&r).ok();
    }
    if is_aggregate(&r) {
        return Some(r);
    }
    metro_token_canonical(&r).map(|m| m.to_string())
}
