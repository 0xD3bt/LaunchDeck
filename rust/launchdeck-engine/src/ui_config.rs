#![allow(non_snake_case, dead_code)]

use crate::{
    fs_utils::{atomic_write, quarantine_corrupt_file},
    paths,
};
use serde_json::{Value, json};
use std::{env, fs};

const PRESET_IDS: [&str; 3] = ["preset1", "preset2", "preset3"];
const DEFAULT_PROVIDER: &str = "helius-sender";
const DEFAULT_CREATION_PRIORITY_FEE_SOL: &str = "0.000001";
const DEFAULT_CREATION_TIP_SOL: &str = "0.0002";
const DEFAULT_TRADE_PRIORITY_FEE_SOL: &str = "0.000001";
const DEFAULT_TRADE_TIP_SOL: &str = "0.0002";
const DEFAULT_TRADE_SLIPPAGE_PERCENT: &str = "20";
const DEFAULT_DEV_BUY_AMOUNTS: [&str; 3] = ["0.5", "1", "2"];

fn configured_track_send_block_height_env_enabled() -> bool {
    matches!(
        env::var("LAUNCHDECK_TRACK_SEND_BLOCK_HEIGHT")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn benchmark_mode_allows_track_send_block_height_default(mode: &str) -> bool {
    matches!(mode.trim().to_ascii_lowercase().as_str(), "" | "full")
}

fn configured_track_send_block_height_default() -> bool {
    benchmark_mode_allows_track_send_block_height_default(
        &env::var("LAUNCHDECK_BENCHMARK_MODE").unwrap_or_default(),
    ) && configured_track_send_block_height_env_enabled()
}

fn legacy_provider_alias(provider: &str) -> String {
    match provider {
        "auto" | "helius" => "helius-sender".to_string(),
        "jito" => "jito-bundle".to_string(),
        "astralane" | "bloxroute" | "hellomoon" => "standard-rpc".to_string(),
        _ => provider.to_string(),
    }
}

fn string_value(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn bool_value(value: Option<&Value>, fallback: bool) -> bool {
    match value {
        None | Some(Value::Null) => fallback,
        Some(Value::Bool(v)) => *v,
        Some(Value::Number(n)) => n.as_i64().unwrap_or_default() != 0,
        Some(Value::String(v)) => match v.trim().to_lowercase().as_str() {
            "true" => true,
            "false" => false,
            "" => fallback,
            _ => true,
        },
        Some(_) => fallback,
    }
}

fn number_value(value: Option<&Value>, fallback: i64) -> i64 {
    match value {
        Some(Value::Number(n)) => n.as_i64().unwrap_or(fallback),
        Some(Value::String(v)) => v.trim().parse::<i64>().unwrap_or(fallback),
        _ => fallback,
    }
}

fn normalize_provider(provider: &str, fallback: &str) -> String {
    let normalized = provider.trim().to_lowercase();
    if normalized.is_empty() {
        return fallback.to_string();
    }
    let migrated = legacy_provider_alias(&normalized);
    match migrated.as_str() {
        "helius-sender" | "standard-rpc" | "jito-bundle" => migrated,
        _ => fallback.to_string(),
    }
}

fn normalize_decimal_string(value: &str, fallback: &str) -> String {
    let normalized = value.trim();
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized.to_string()
    }
}

fn creation_settings(
    provider: &str,
    tip_sol: &str,
    priority_fee_sol: &str,
    auto_fee: bool,
    max_fee_sol: &str,
    dev_buy_sol: &str,
) -> Value {
    json!({
        "provider": normalize_provider(provider, DEFAULT_PROVIDER),
        "tipSol": normalize_decimal_string(tip_sol, DEFAULT_CREATION_TIP_SOL),
        "priorityFeeSol": normalize_decimal_string(priority_fee_sol, DEFAULT_CREATION_PRIORITY_FEE_SOL),
        "autoFee": auto_fee,
        "maxFeeSol": max_fee_sol.trim(),
        "devBuySol": normalize_decimal_string(dev_buy_sol, ""),
    })
}

fn trade_settings(
    provider: &str,
    priority_fee_sol: &str,
    tip_sol: &str,
    slippage_percent: &str,
    auto_fee: bool,
    max_fee_sol: &str,
) -> Value {
    json!({
        "provider": normalize_provider(provider, DEFAULT_PROVIDER),
        "priorityFeeSol": normalize_decimal_string(priority_fee_sol, DEFAULT_TRADE_PRIORITY_FEE_SOL),
        "tipSol": normalize_decimal_string(tip_sol, DEFAULT_TRADE_TIP_SOL),
        "slippagePercent": normalize_decimal_string(slippage_percent, DEFAULT_TRADE_SLIPPAGE_PERCENT),
        "autoFee": auto_fee,
        "maxFeeSol": max_fee_sol.trim(),
    })
}

fn default_preset(id: &str, label: &str, dev_buy_sol: &str) -> Value {
    let mut buy = trade_settings("", "", "", "", false, "");
    if let Some(object) = buy.as_object_mut() {
        object.insert(
            "snipeBuyAmountSol".to_string(),
            Value::String(String::new()),
        );
    }
    json!({
        "id": id,
        "label": label,
        "creationSettings": creation_settings("", "", "", false, "", dev_buy_sol),
        "buySettings": buy,
        "sellSettings": trade_settings("", "", "", "", false, ""),
        "postLaunchStrategy": "none",
    })
}

pub fn create_default_persistent_config() -> Value {
    json!({
        "defaults": {
            "launchpad": "pump",
            "mode": "regular",
            "activePresetId": "preset1",
            "presetEditing": false,
            "misc": {
                "trackSendBlockHeight": configured_track_send_block_height_default()
            },
            "automaticDevSell": {
                "enabled": false,
                "percent": 100,
                "triggerMode": "block-offset",
                "delayMs": 0,
                "targetBlockOffset": 0
            }
        },
        "presets": {
            "items": PRESET_IDS.iter().enumerate().map(|(index, id)| {
                default_preset(id, &format!("P{}", index + 1), DEFAULT_DEV_BUY_AMOUNTS[index])
            }).collect::<Vec<_>>()
        }
    })
}

fn first_non_empty(values: &[String]) -> String {
    values
        .iter()
        .find(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_default()
}

fn normalize_preset_shape(preset: Option<&Value>, fallback_preset: &Value, index: usize) -> Value {
    let preset_obj = preset.and_then(Value::as_object);
    let fallback_obj = fallback_preset.as_object().expect("fallback preset object");
    let fallback_creation = fallback_obj
        .get("creationSettings")
        .and_then(Value::as_object)
        .expect("creation settings");
    let fallback_buy = fallback_obj
        .get("buySettings")
        .and_then(Value::as_object)
        .expect("buy settings");
    let fallback_sell = fallback_obj
        .get("sellSettings")
        .and_then(Value::as_object)
        .expect("sell settings");
    let creation = preset_obj
        .and_then(|value| value.get("creationSettings"))
        .and_then(Value::as_object);
    let buy = preset_obj
        .and_then(|value| value.get("buySettings"))
        .and_then(Value::as_object);
    let sell = preset_obj
        .and_then(|value| value.get("sellSettings"))
        .and_then(Value::as_object);
    let id = preset_obj
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
        .if_empty_then(format!("preset{}", index + 1));
    let label = preset_obj
        .and_then(|value| value.get("label"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
        .if_empty_then(format!("P{}", index + 1));
    let creation_settings = creation_settings(
        creation
            .and_then(|value| value.get("provider"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_creation
                    .get("provider")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_PROVIDER),
            ),
        creation
            .and_then(|value| value.get("tipSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_creation
                    .get("tipSol")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_CREATION_TIP_SOL),
            ),
        creation
            .and_then(|value| value.get("priorityFeeSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_creation
                    .get("priorityFeeSol")
                    .and_then(Value::as_str)
                    .unwrap_or("0.001"),
            ),
        bool_value(
            creation.and_then(|value| value.get("autoFee")),
            bool_value(fallback_creation.get("autoFee"), false),
        ),
        creation
            .and_then(|value| value.get("maxFeeSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_creation
                    .get("maxFeeSol")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ),
        creation
            .and_then(|value| value.get("devBuySol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_creation
                    .get("devBuySol")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ),
    );
    let mut buy_settings = trade_settings(
        buy.and_then(|value| value.get("provider"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_buy
                    .get("provider")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_PROVIDER),
            ),
        buy.and_then(|value| value.get("priorityFeeSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_buy
                    .get("priorityFeeSol")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_TRADE_PRIORITY_FEE_SOL),
            ),
        buy.and_then(|value| value.get("tipSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_buy
                    .get("tipSol")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_TRADE_TIP_SOL),
            ),
        buy.and_then(|value| value.get("slippagePercent"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_buy
                    .get("slippagePercent")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_TRADE_SLIPPAGE_PERCENT),
            ),
        bool_value(
            buy.and_then(|value| value.get("autoFee")),
            bool_value(fallback_buy.get("autoFee"), false),
        ),
        buy.and_then(|value| value.get("maxFeeSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_buy
                    .get("maxFeeSol")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ),
    );
    buy_settings
        .as_object_mut()
        .expect("buy settings object")
        .insert(
            "snipeBuyAmountSol".to_string(),
            Value::String(normalize_decimal_string(
                buy.and_then(|value| value.get("snipeBuyAmountSol"))
                    .and_then(Value::as_str)
                    .unwrap_or(
                        fallback_buy
                            .get("snipeBuyAmountSol")
                            .and_then(Value::as_str)
                            .unwrap_or(""),
                    ),
                "",
            )),
        );
    let sell_settings = trade_settings(
        sell.and_then(|value| value.get("provider"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_sell
                    .get("provider")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_PROVIDER),
            ),
        sell.and_then(|value| value.get("priorityFeeSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_sell
                    .get("priorityFeeSol")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_TRADE_PRIORITY_FEE_SOL),
            ),
        sell.and_then(|value| value.get("tipSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_sell
                    .get("tipSol")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_TRADE_TIP_SOL),
            ),
        sell.and_then(|value| value.get("slippagePercent"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_sell
                    .get("slippagePercent")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_TRADE_SLIPPAGE_PERCENT),
            ),
        bool_value(
            sell.and_then(|value| value.get("autoFee")),
            bool_value(fallback_sell.get("autoFee"), false),
        ),
        sell.and_then(|value| value.get("maxFeeSol"))
            .and_then(Value::as_str)
            .unwrap_or(
                fallback_sell
                    .get("maxFeeSol")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ),
    );
    let post_launch_strategy = preset_obj
        .and_then(|value| value.get("postLaunchStrategy"))
        .and_then(Value::as_str)
        .unwrap_or("none")
        .trim()
        .to_string()
        .if_empty_then("none".to_string());
    json!({
        "id": id,
        "label": label,
        "creationSettings": creation_settings,
        "buySettings": buy_settings,
        "sellSettings": sell_settings,
        "postLaunchStrategy": post_launch_strategy
    })
}

fn migrate_legacy_config(parsed: &Value) -> Value {
    let base = create_default_persistent_config();
    let defaults = parsed.get("defaults").unwrap_or(&Value::Null);
    let launch_defaults = defaults.get("launchExecution").unwrap_or(&Value::Null);
    let buy_defaults = defaults.get("buyExecution").unwrap_or(&Value::Null);
    let legacy_auto_sell = defaults.get("automaticDevSell").unwrap_or(&Value::Null);
    let launch_presets = parsed
        .get("presets")
        .and_then(|value| value.get("launch"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let sniper_presets = parsed
        .get("presets")
        .and_then(|value| value.get("sniper"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let base_items = base
        .get("presets")
        .and_then(|value| value.get("items"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let items = PRESET_IDS
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let fallback_preset = base_items.get(index).cloned().unwrap_or_else(|| default_preset(id, &format!("P{}", index + 1), DEFAULT_DEV_BUY_AMOUNTS[index]));
            let launch_preset = launch_presets.get(index).cloned().unwrap_or(Value::Null);
            let sniper_preset = sniper_presets.get(index).cloned().unwrap_or(Value::Null);
            let launch_execution = launch_preset.get("execution").unwrap_or(launch_defaults);
            let buy_execution = sniper_preset.get("execution").unwrap_or(buy_defaults);
            let creation_priority = first_non_empty(&[
                string_value(launch_execution.get("priorityFeeSol")),
                string_value(launch_defaults.get("priorityFeeSol")),
            ]);
            let buy_priority = first_non_empty(&[
                string_value(buy_execution.get("priorityFeeSol")),
                string_value(buy_execution.get("maxPriorityFeeSol")),
                string_value(buy_defaults.get("priorityFeeSol")),
                string_value(buy_defaults.get("maxPriorityFeeSol")),
                string_value(fallback_preset.get("buySettings").and_then(|v| v.get("priorityFeeSol"))),
            ]);
            let buy_tip = first_non_empty(&[
                string_value(buy_execution.get("tipSol")),
                string_value(buy_execution.get("maxTipSol")),
                string_value(buy_defaults.get("tipSol")),
                string_value(buy_defaults.get("maxTipSol")),
                string_value(fallback_preset.get("buySettings").and_then(|v| v.get("tipSol"))),
            ]);
            normalize_preset_shape(Some(&json!({
                "id": string_value(launch_preset.get("id")).if_empty_then(id.to_string()),
                "label": first_non_empty(&[string_value(launch_preset.get("label")), string_value(sniper_preset.get("label")), format!("P{}", index + 1)]),
                "creationSettings": {
                    "provider": normalize_provider(&string_value(launch_execution.get("provider")), string_value(fallback_preset.get("creationSettings").and_then(|v| v.get("provider"))).as_str()),
                    "tipSol": first_non_empty(&[
                        string_value(launch_execution.get("tipSol")),
                        string_value(launch_execution.get("maxTipSol")),
                        string_value(launch_defaults.get("tipSol")),
                        string_value(launch_defaults.get("maxTipSol")),
                        string_value(fallback_preset.get("creationSettings").and_then(|v| v.get("tipSol"))),
                    ]),
                    "priorityFeeSol": creation_priority,
                    "autoFee": bool_value(
                        launch_execution.get("autoGas"),
                        bool_value(fallback_preset.get("creationSettings").and_then(|v| v.get("autoFee")), false),
                    ),
                    "maxFeeSol": first_non_empty(&[
                        string_value(launch_execution.get("maxPriorityFeeSol")),
                        string_value(launch_execution.get("maxTipSol")),
                        string_value(fallback_preset.get("creationSettings").and_then(|v| v.get("maxFeeSol"))),
                    ]),
                    "devBuySol": normalize_decimal_string(&string_value(launch_preset.get("buyAmountSol")), &string_value(fallback_preset.get("creationSettings").and_then(|v| v.get("devBuySol")))),
                },
                "buySettings": {
                    "provider": normalize_provider(&string_value(buy_execution.get("provider")), &string_value(fallback_preset.get("buySettings").and_then(|v| v.get("provider")))),
                    "priorityFeeSol": buy_priority,
                    "tipSol": buy_tip,
                    "slippagePercent": string_value(fallback_preset.get("buySettings").and_then(|v| v.get("slippagePercent"))),
                    "autoFee": bool_value(
                        buy_execution.get("buyAutoGas").or_else(|| buy_execution.get("autoGas")),
                        bool_value(fallback_preset.get("buySettings").and_then(|v| v.get("autoFee")), false),
                    ),
                    "maxFeeSol": first_non_empty(&[
                        string_value(buy_execution.get("buyMaxPriorityFeeSol")),
                        string_value(buy_execution.get("buyMaxTipSol")),
                        string_value(buy_execution.get("maxPriorityFeeSol")),
                        string_value(buy_execution.get("maxTipSol")),
                        string_value(fallback_preset.get("buySettings").and_then(|v| v.get("maxFeeSol"))),
                    ]),
                    "snipeBuyAmountSol": normalize_decimal_string(&string_value(sniper_preset.get("buyAmountSol")), &string_value(fallback_preset.get("buySettings").and_then(|v| v.get("snipeBuyAmountSol")))),
                },
                "sellSettings": {
                    "provider": normalize_provider(&string_value(buy_execution.get("provider")), &string_value(fallback_preset.get("sellSettings").and_then(|v| v.get("provider")))),
                    "priorityFeeSol": if buy_priority.is_empty() { string_value(fallback_preset.get("sellSettings").and_then(|v| v.get("priorityFeeSol"))) } else { buy_priority.clone() },
                    "tipSol": if buy_tip.is_empty() { string_value(fallback_preset.get("sellSettings").and_then(|v| v.get("tipSol"))) } else { buy_tip.clone() },
                    "slippagePercent": string_value(fallback_preset.get("sellSettings").and_then(|v| v.get("slippagePercent"))),
                    "autoFee": bool_value(
                        buy_execution.get("sellAutoGas")
                            .or_else(|| buy_execution.get("buyAutoGas"))
                            .or_else(|| buy_execution.get("autoGas")),
                        bool_value(fallback_preset.get("sellSettings").and_then(|v| v.get("autoFee")), false),
                    ),
                    "maxFeeSol": first_non_empty(&[
                        string_value(buy_execution.get("sellMaxPriorityFeeSol")),
                        string_value(buy_execution.get("sellMaxTipSol")),
                        string_value(buy_execution.get("buyMaxPriorityFeeSol")),
                        string_value(buy_execution.get("buyMaxTipSol")),
                        string_value(buy_execution.get("maxPriorityFeeSol")),
                        string_value(buy_execution.get("maxTipSol")),
                        string_value(fallback_preset.get("sellSettings").and_then(|v| v.get("maxFeeSol"))),
                    ]),
                },
                "postLaunchStrategy": first_non_empty(&[string_value(defaults.get("postLaunchStrategy")), "none".to_string()])
            })), &fallback_preset, index)
        })
        .collect::<Vec<_>>();

    let launchpad = string_value(defaults.get("launchpad")).if_empty_then("pump".to_string());
    let mode = string_value(defaults.get("mode")).if_empty_then("regular".to_string());
    let active_preset_id = {
        let value = string_value(defaults.get("activePresetId"));
        if PRESET_IDS.contains(&value.as_str()) {
            value
        } else {
            "preset1".to_string()
        }
    };
    let legacy_auto_sell_trigger_mode = if number_value(legacy_auto_sell.get("delaySeconds"), 0) > 0
    {
        "submit-delay".to_string()
    } else {
        "block-offset".to_string()
    };
    json!({
        "defaults": {
            "launchpad": launchpad,
            "mode": mode,
            "activePresetId": active_preset_id,
            "presetEditing": bool_value(defaults.get("presetEditing"), false),
            "misc": {
                "trackSendBlockHeight": configured_track_send_block_height_default()
            },
            "automaticDevSell": {
                "enabled": bool_value(legacy_auto_sell.get("enabled"), false),
                "percent": number_value(legacy_auto_sell.get("percent"), 100),
                "triggerMode": legacy_auto_sell_trigger_mode,
                "delayMs": number_value(legacy_auto_sell.get("delaySeconds"), 0) * 1000,
                "targetBlockOffset": 0
            }
        },
        "presets": {
            "items": items
        }
    })
}

trait StringExt {
    fn if_empty_then(self, fallback: String) -> String;
}

impl StringExt for String {
    fn if_empty_then(self, fallback: String) -> String {
        if self.trim().is_empty() {
            fallback
        } else {
            self
        }
    }
}

pub fn normalize_persistent_config(parsed: Value) -> Value {
    let base = create_default_persistent_config();
    let has_new_shape = parsed
        .get("presets")
        .and_then(|value| value.get("items"))
        .and_then(Value::as_array)
        .is_some();
    if !has_new_shape {
        return migrate_legacy_config(&parsed);
    }
    let base_items = base
        .get("presets")
        .and_then(|value| value.get("items"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let merged_defaults = parsed.get("defaults").unwrap_or(&Value::Null);
    let merged_items = parsed
        .get("presets")
        .and_then(|value| value.get("items"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let items = PRESET_IDS
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let fallback = base_items.get(index).cloned().unwrap_or_else(|| {
                default_preset(
                    id,
                    &format!("P{}", index + 1),
                    DEFAULT_DEV_BUY_AMOUNTS[index],
                )
            });
            let existing = merged_items
                .iter()
                .find(|entry| string_value(entry.get("id")) == *id)
                .or_else(|| merged_items.get(index));
            normalize_preset_shape(existing, &fallback, index)
        })
        .collect::<Vec<_>>();
    let launchpad =
        string_value(merged_defaults.get("launchpad")).if_empty_then("pump".to_string());
    let mode = string_value(merged_defaults.get("mode")).if_empty_then("regular".to_string());
    let active_preset_id = {
        let value = string_value(merged_defaults.get("activePresetId"));
        if PRESET_IDS.contains(&value.as_str()) {
            value
        } else {
            "preset1".to_string()
        }
    };
    let automatic_dev_sell_trigger_mode = {
        let mode = string_value(
            merged_defaults
                .get("automaticDevSell")
                .and_then(|value| value.get("triggerMode")),
        );
        if mode.is_empty() {
            "block-offset".to_string()
        } else {
            mode
        }
    };
    json!({
        "defaults": {
            "launchpad": launchpad,
            "mode": mode,
            "activePresetId": active_preset_id,
            "presetEditing": bool_value(merged_defaults.get("presetEditing"), false),
            "misc": {
                "trackSendBlockHeight": bool_value(
                    merged_defaults.get("misc").and_then(|value| value.get("trackSendBlockHeight")),
                    configured_track_send_block_height_default()
                )
            },
            "automaticDevSell": {
                "enabled": bool_value(merged_defaults.get("automaticDevSell").and_then(|value| value.get("enabled")), false),
                "percent": number_value(merged_defaults.get("automaticDevSell").and_then(|value| value.get("percent")), 100),
                "triggerMode": automatic_dev_sell_trigger_mode,
                "delayMs": number_value(
                    merged_defaults.get("automaticDevSell").and_then(|value| value.get("delayMs")),
                    number_value(merged_defaults.get("automaticDevSell").and_then(|value| value.get("delaySeconds")), 0) * 1000
                ),
                "targetBlockOffset": number_value(merged_defaults.get("automaticDevSell").and_then(|value| value.get("targetBlockOffset")), 0)
            }
        },
        "presets": {
            "items": items
        }
    })
}

pub fn read_persistent_config() -> Value {
    let path = paths::app_config_path();
    let raw = fs::read_to_string(&path).unwrap_or_default();
    if raw.trim().is_empty() {
        return create_default_persistent_config();
    }
    serde_json::from_str::<Value>(&raw)
        .map(normalize_persistent_config)
        .unwrap_or_else(|_| {
            let _ = quarantine_corrupt_file(&path, "persistent config");
            create_default_persistent_config()
        })
}

pub fn write_persistent_config(next_config: Value) -> Result<String, String> {
    let path = paths::app_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let normalized = normalize_persistent_config(next_config);
    atomic_write(
        &path,
        &serde_json::to_vec_pretty(&normalized).map_err(|error| error.to_string())?,
    )?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_persistent_config_uses_safe_sender_baseline() {
        let config = create_default_persistent_config();
        let presets = config["presets"]["items"]
            .as_array()
            .expect("preset items array");
        assert_eq!(presets.len(), 3);
        for preset in presets {
            assert_eq!(preset["creationSettings"]["provider"], "helius-sender");
            assert_eq!(preset["creationSettings"]["priorityFeeSol"], "0.000001");
            assert_eq!(preset["creationSettings"]["tipSol"], "0.0002");
            assert_eq!(preset["creationSettings"]["autoFee"], false);
            assert_eq!(preset["buySettings"]["provider"], "helius-sender");
            assert_eq!(preset["buySettings"]["priorityFeeSol"], "0.000001");
            assert_eq!(preset["buySettings"]["tipSol"], "0.0002");
            assert_eq!(preset["buySettings"]["slippagePercent"], "20");
            assert_eq!(preset["buySettings"]["autoFee"], false);
            assert_eq!(preset["sellSettings"]["provider"], "helius-sender");
            assert_eq!(preset["sellSettings"]["priorityFeeSol"], "0.000001");
            assert_eq!(preset["sellSettings"]["tipSol"], "0.0002");
            assert_eq!(preset["sellSettings"]["slippagePercent"], "20");
            assert_eq!(preset["sellSettings"]["autoFee"], false);
        }
    }

    #[test]
    fn normalizes_new_shape_and_strips_legacy_policy_and_endpoint_profile_fields() {
        let normalized = normalize_persistent_config(json!({
            "defaults": {
                "launchpad": "pump",
                "mode": "regular",
                "activePresetId": "preset1"
            },
            "presets": {
                "items": [{
                    "id": "preset1",
                    "label": "P1",
                    "creationSettings": {
                        "provider": "helius-sender",
                        "endpointProfile": "eu",
                        "policy": "fast",
                        "tipSol": "0.02",
                        "priorityFeeSol": "0.003",
                        "devBuySol": "1"
                    },
                    "buySettings": {
                        "provider": "jito-bundle",
                        "endpointProfile": "asia",
                        "policy": "safe",
                        "priorityFeeSol": "0.02",
                        "tipSol": "0.01",
                        "slippagePercent": "42",
                        "snipeBuyAmountSol": "0.5"
                    },
                    "sellSettings": {
                        "provider": "helius-sender",
                        "endpointProfile": "fra",
                        "policy": "fast",
                        "priorityFeeSol": "0.01",
                        "tipSol": "0.02",
                        "slippagePercent": "33"
                    }
                }]
            }
        }));

        let preset = &normalized["presets"]["items"][0];
        assert!(preset["creationSettings"].get("endpointProfile").is_none());
        assert!(preset["creationSettings"].get("policy").is_none());
        assert!(preset["buySettings"].get("endpointProfile").is_none());
        assert!(preset["buySettings"].get("policy").is_none());
        assert!(preset["sellSettings"].get("endpointProfile").is_none());
        assert!(preset["sellSettings"].get("policy").is_none());
        assert_eq!(preset["buySettings"]["snipeBuyAmountSol"], "0.5");
    }

    #[test]
    fn migrates_legacy_shape_without_persisting_endpoint_profile_or_policy() {
        let normalized = normalize_persistent_config(json!({
            "defaults": {
                "launchpad": "pump",
                "mode": "regular",
                "launchExecution": {
                    "provider": "helius-sender",
                    "endpointProfile": "eu",
                    "policy": "fast",
                    "tipSol": "0.02"
                },
                "buyExecution": {
                    "provider": "jito-bundle",
                    "endpointProfile": "asia",
                    "policy": "safe",
                    "tipSol": "0.03"
                }
            },
            "presets": {
                "launch": [{
                    "id": "preset1",
                    "label": "P1",
                    "execution": {
                        "provider": "helius-sender",
                        "endpointProfile": "us",
                        "policy": "safe"
                    }
                }],
                "sniper": [{
                    "id": "preset1",
                    "label": "P1",
                    "execution": {
                        "provider": "jito-bundle",
                        "endpointProfile": "ams",
                        "policy": "fast"
                    }
                }]
            }
        }));

        let preset = &normalized["presets"]["items"][0];
        assert_eq!(preset["creationSettings"]["provider"], "helius-sender");
        assert_eq!(preset["buySettings"]["provider"], "jito-bundle");
        assert!(preset["creationSettings"].get("endpointProfile").is_none());
        assert!(preset["buySettings"].get("endpointProfile").is_none());
        assert!(preset["sellSettings"].get("endpointProfile").is_none());
        assert!(preset["creationSettings"].get("policy").is_none());
        assert!(preset["buySettings"].get("policy").is_none());
        assert!(preset["sellSettings"].get("policy").is_none());
    }
}
