#![allow(non_snake_case, dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WarmLifecycleMode {
    Active,
    Maintenance,
    Idle,
}

#[derive(Debug, Clone, Default)]
pub struct WarmControlState {
    pub last_activity_at_ms: u128,
    pub last_resume_at_ms: Option<u128>,
    pub last_suspend_at_ms: Option<u128>,
    pub last_warm_attempt_at_ms: Option<u128>,
    pub last_warm_success_at_ms: Option<u128>,
    pub current_reason: String,
    pub last_error: Option<String>,
    pub selected_routes: Vec<WarmRouteSelection>,
    pub follow_job_routes: Vec<WarmRouteSelection>,
    pub browser_active: bool,
    pub continuous_active: bool,
    pub follow_jobs_active: bool,
    pub in_flight_requests: usize,
    pub warm_pass_in_flight: bool,
    pub warm_targets: HashMap<String, WarmTargetStatus>,
}

impl WarmControlState {
    pub fn lifecycle_mode(&self) -> WarmLifecycleMode {
        if self.continuous_active {
            WarmLifecycleMode::Active
        } else if self.browser_active || self.follow_jobs_active || self.warm_pass_in_flight {
            WarmLifecycleMode::Maintenance
        } else {
            WarmLifecycleMode::Idle
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WarmRouteSelection {
    pub provider: String,
    pub endpoint_profile: String,
    pub hellomoon_mev_mode: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WarmTargetHealth {
    Healthy,
    RateLimited,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WarmTargetStatus {
    pub id: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub label: String,
    pub target: String,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_at_ms: Option<u64>,
    pub status: WarmTargetHealth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_success_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_rate_limited_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_rate_limit_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_recovered_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_recovered_error: Option<String>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone)]
pub struct WatchWarmTarget {
    pub label: String,
    pub target: String,
    pub transport: WatchWarmTransport,
    pub fallback_target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchWarmTransport {
    StandardWs,
    HeliusTransactionSubscribe,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WarmActivityRequest {
    #[serde(default)]
    #[serde(rename = "creationProvider")]
    pub creation_provider: Option<String>,
    #[serde(default)]
    #[serde(rename = "creationEndpointProfile")]
    pub creation_endpoint_profile: Option<String>,
    #[serde(default)]
    #[serde(rename = "creationMevMode")]
    pub creation_mev_mode: Option<String>,
    #[serde(default)]
    #[serde(rename = "buyProvider")]
    pub buy_provider: Option<String>,
    #[serde(default)]
    #[serde(rename = "buyEndpointProfile")]
    pub buy_endpoint_profile: Option<String>,
    #[serde(default)]
    #[serde(rename = "buyMevMode")]
    pub buy_mev_mode: Option<String>,
    #[serde(default)]
    #[serde(rename = "sellProvider")]
    pub sell_provider: Option<String>,
    #[serde(default)]
    #[serde(rename = "sellEndpointProfile")]
    pub sell_endpoint_profile: Option<String>,
    #[serde(default)]
    #[serde(rename = "sellMevMode")]
    pub sell_mev_mode: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_mode_reports_active_when_continuous_warm_is_running() {
        let state = WarmControlState {
            continuous_active: true,
            ..WarmControlState::default()
        };
        assert!(matches!(state.lifecycle_mode(), WarmLifecycleMode::Active));
    }

    #[test]
    fn lifecycle_mode_reports_maintenance_for_browser_or_follow_activity() {
        let browser_active = WarmControlState {
            browser_active: true,
            ..WarmControlState::default()
        };
        assert!(matches!(
            browser_active.lifecycle_mode(),
            WarmLifecycleMode::Maintenance
        ));

        let follow_active = WarmControlState {
            follow_jobs_active: true,
            ..WarmControlState::default()
        };
        assert!(matches!(
            follow_active.lifecycle_mode(),
            WarmLifecycleMode::Maintenance
        ));
    }

    #[test]
    fn lifecycle_mode_reports_idle_without_activity_or_warm_work() {
        assert!(matches!(
            WarmControlState::default().lifecycle_mode(),
            WarmLifecycleMode::Idle
        ));
    }
}
