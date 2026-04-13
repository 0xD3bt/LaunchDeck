//! Request-scoped launch warm context and rollout documentation.
//!
//! # Transaction and fetch dependency matrix (summary)
//!
//! | Phase | Parallel-safe | Notes |
//! |-------|---------------|-------|
//! | Fee market / blockhash prime (warm context) | Yes | Deduped via RPC caches; no signing |
//! | Pump/Bonk native compile internals | Partial | ALT load + blockhash + globals: ordered inside each compile |
//! | Bags `prepare-launch` helper | No | Single helper round-trip; emits signed setup + mint metadata |
//! | Bags setup Jito bundles | Sequential bundles | Order preserved between bundles |
//! | Bags setup transactions | Sequential submit+confirm batches | Rebroadcast retries are order-dependent |
//! | Bags `build-launch-transaction` | After setup confirms | Requires on-chain config + mint state |
//! | Creation / launch submit | After setup | Pump/Bonk may parallelize with follow reserve when safe |
//! | Deferred setup / follow | Per follow daemon contract | Opaque or presigned payloads are not rebuildable |
//!
//! # Cache policy (summary)
//!
//! | Datum | Scope | TTL / invalidation |
//! |-------|--------|-------------------|
//! | Blockhash (RPC cache) | Process | Short TTL in `rpc`; refreshed by background task + send-time refresh |
//! | Fee market snapshot | Process + key | Cached in `main` auto-fee path; percentile keys in cache key |
//! | Startup warm targets | Per engine warm call | No cross-request struct; RPC caches benefit later launches |
//! | Launchpad warm context | Single launch HTTP request | Built once per compile/send attempt; not stored |
//!
//! Cross-launchpad ALT merge / shared super-ALT policy is intentionally **out of scope** here; see plan todo `shared-alt-strategy`.
//!
//! ## Blockhash cache key and compile handoff
//!
//! This uses the **same** `rpc_url` string as native compile (`configured_rpc_url()` in the handler),
//! not `LAUNCHDECK_WARM_RPC_URL`, so `fetch_latest_blockhash_cached` shares one process-wide cache entry
//! with Pump, Bonk, and Bags compile paths.
//!
//! On success, the handler passes `(blockhash, last_valid_block_height)` into `try_compile_native_launchpad`
//! as `launch_blockhash_prime` so Pump/Bags can skip a redundant cache lookup/RPC when the value is still valid.

use std::time::Instant;

use crate::{
    config::{
        configured_launchpad_warm_context_enabled, configured_warm_parallel_fetch_enabled,
        launchpad_warm_max_parallel_fetches,
    },
    rpc::{COMPILE_BLOCKHASH_MIN_REMAINING_BLOCKS, fetch_latest_blockhash_fresh_or_recent},
};

/// Per-launch attempt warm data shared across launchpads (blockhash priming only in this rollout).
#[derive(Debug, Clone)]
pub struct LaunchpadWarmContext {
    pub blockhash: String,
    pub last_valid_block_height: u64,
    /// Same URL string as the compile path (`configured_rpc_url()`); useful for logs/tests.
    #[allow(dead_code)]
    pub rpc_url: String,
}

#[derive(Debug, Clone, Default)]
pub struct LaunchpadWarmBuildReport {
    pub build_ms: u128,
    pub blockhash_fetch_ms: u128,
    pub parallel_enabled: bool,
    pub warm_context_enabled: bool,
    /// From `LAUNCHDECK_LAUNCHPAD_WARM_MAX_PARALLEL_FETCH` (budget for future parallel warm steps).
    pub max_parallel_warm_fetches: usize,
}

/// Env-derived telemetry when a full warm build did not run (e.g. RPC prime failed).
pub fn launchpad_warm_env_snapshot() -> LaunchpadWarmBuildReport {
    LaunchpadWarmBuildReport {
        warm_context_enabled: configured_launchpad_warm_context_enabled(),
        parallel_enabled: configured_warm_parallel_fetch_enabled(),
        max_parallel_warm_fetches: launchpad_warm_max_parallel_fetches(),
        ..Default::default()
    }
}

/// Build request-scoped warm context: primes blockhash cache used by native compile paths.
pub async fn build_launchpad_warm_context(
    main_rpc_url: &str,
    commitment: &str,
) -> Result<(LaunchpadWarmContext, LaunchpadWarmBuildReport), String> {
    let mut report = LaunchpadWarmBuildReport {
        warm_context_enabled: configured_launchpad_warm_context_enabled(),
        parallel_enabled: configured_warm_parallel_fetch_enabled(),
        max_parallel_warm_fetches: launchpad_warm_max_parallel_fetches(),
        ..Default::default()
    };
    if !report.warm_context_enabled {
        return Ok((
            LaunchpadWarmContext {
                blockhash: String::new(),
                last_valid_block_height: 0,
                rpc_url: String::new(),
            },
            report,
        ));
    }
    let started = Instant::now();
    let bh_started = Instant::now();
    // Must match compile path URL so blockhash cache hits for try_compile_native_* / Bags helper priming.
    let (blockhash, last_valid_block_height) = fetch_latest_blockhash_fresh_or_recent(
        main_rpc_url,
        commitment,
        COMPILE_BLOCKHASH_MIN_REMAINING_BLOCKS,
    )
    .await?;
    report.blockhash_fetch_ms = bh_started.elapsed().as_millis();
    report.build_ms = started.elapsed().as_millis();
    Ok((
        LaunchpadWarmContext {
            blockhash,
            last_valid_block_height,
            rpc_url: main_rpc_url.to_string(),
        },
        report,
    ))
}
