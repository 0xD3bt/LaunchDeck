# Environment Variable Reference

This page is the exhaustive env reference for LaunchDeck.

Use `.env.example` for normal setup.

Use this page when you want to know:

- every supported env var
- what it does
- what happens when it is left blank
- whether it belongs in normal setup or advanced tuning

`docs/CONFIG.md` is the human-readable setup guide. This page is the reference sheet.

## Easy-setup variables

These are the only variables most operators need on day one.

| Variable | Effective default when blank | What it does | Normal setup? |
| --- | --- | --- | --- |
| `SOLANA_PRIVATE_KEY` | unset | First wallet slot loaded by the UI and engine | Yes |
| `SOLANA_PRIVATE_KEY2` ... `SOLANA_PRIVATE_KEY10` | unset | Additional wallet slots; optional `<privatekey>,<label>` format is supported | Yes |
| `SOLANA_RPC_URL` | unset | Main Solana HTTP RPC for reads, confirmations, and general runtime RPC behavior | Yes |
| `SOLANA_WS_URL` | unset | Main watcher websocket for realtime follow behavior | Yes |
| `USER_REGION` | provider fallback | Shared default routing profile for region-aware providers | Yes |
| `LAUNCHDECK_WARM_RPC_URL` | reuse `SOLANA_RPC_URL` | Startup warm, continuous warm probes, and block-height reads | Yes |
| `HELLOMOON_API_KEY` | unset | Enables Hello Moon execution paths | Optional |
| `BAGS_API_KEY` | unset | Enables Bags identity and Bags launchpad usage | Optional |
| `LAUNCHDECK_METADATA_UPLOAD_PROVIDER` | `pump-fun` | Metadata upload provider | Optional |
| `PINATA_JWT` | unset | Required only when metadata provider is `pinata` | Optional |
| `LAUNCHDECK_BENCHMARK_MODE` | `full` | Report timing detail level | Optional |

## Wallet loading

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `SOLANA_PRIVATE_KEY` | unset | Primary wallet slot | Label format: `<privatekey>,<label>` |
| `SOLANA_PRIVATE_KEY2` ... `SOLANA_PRIVATE_KEY10` | unset | Additional wallet slots | Untagged wallets appear as numbered slots |
| `SOLANA_KEYPAIR_PATH` | unset | Optional filesystem keypair path | Advanced override only |

## Core RPC, websocket, and routing

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `SOLANA_RPC_URL` | unset | Main HTTP RPC | Recommended: Helius Gatekeeper HTTP |
| `SOLANA_WS_URL` | unset | Main watcher websocket | Recommended: Helius standard websocket |
| `USER_REGION` | provider fallback | Shared region or metro selection | Supports `global`, `us`, `eu`, `asia`, `slc`, `ewr`, `lon`, `fra`, `ams`, `sg`, `tyo`, comma lists |
| `LAUNCHDECK_WARM_RPC_URL` | reuse `SOLANA_RPC_URL` | Warm RPC for startup warm, continuous warm, and block-height reads | Recommended: separate Shyft RPC |
| `LAUNCHDECK_EXTRA_STANDARD_RPC_SEND_URLS` | none | Extra submit-only RPC fanout endpoints for `standard-rpc` | Comma-separated; `SOLANA_RPC_URL` stays primary read/confirm RPC |
| `LAUNCHDECK_STANDARD_RPC_SEND_URLS` | legacy fallback only | Old name for the extra standard-RPC send list | Supported for backward compatibility; prefer `LAUNCHDECK_EXTRA_STANDARD_RPC_SEND_URLS` |

### USER_REGION behavior

Current practical routing rules:

- `eu` fans out across Amsterdam + Frankfurt
- `us` fans out across Salt Lake City + Newark on Helius Sender
- `asia` fans out across Singapore + Tokyo on Helius Sender
- Hello Moon `us`, `slc`, and `ewr` map to New York + Ashburn
- Hello Moon `lon` maps to Frankfurt + Amsterdam
- Hello Moon `asia` and `sg` map to Tokyo

## Helius-specific overrides

These are override-only in most setups.

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `HELIUS_RPC_URL` | derive from `SOLANA_RPC_URL` when it is already Helius; otherwise unused | Override-only Helius HTTP RPC for priority-fee estimation | Set this only when your main RPC is non-Helius but you still want Helius fee estimates |
| `HELIUS_WS_URL` | derive from `SOLANA_WS_URL` when it is already Helius; otherwise unused | Override-only Helius websocket for enhanced watchers | Set this only when your main watcher websocket is non-Helius or intentionally separate |
| `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE` | `true` | Enables Helius `transactionSubscribe` probing and fallback logic | If the probe fails, LaunchDeck falls back automatically to standard websocket watchers |

## Warmup, keep-alive, and block-height

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `LAUNCHDECK_ENABLE_STARTUP_WARM` | `true` | One-shot startup warm pass | Recommended: leave on |
| `LAUNCHDECK_ENABLE_CONTINUOUS_WARM` | `true` | Active keep-warm loop while the app is in use | Recommended: leave on |
| `LAUNCHDECK_ENABLE_IDLE_WARM_SUSPEND` | `true` | Suspends warm traffic while idle | Recommended: leave on |
| `LAUNCHDECK_IDLE_WARM_TIMEOUT_MS` | `75000` | Idle time before warm suspend kicks in | Advanced tuning |
| `LAUNCHDECK_CONTINUOUS_WARM_INTERVAL_MS` | `50000` | Continuous warm cadence | Advanced tuning |
| `LAUNCHDECK_DISABLE_STARTUP_WARM` | disabled unless explicitly true and the positive flag is unset | Legacy negative startup-warm flag | Backward-compat only |
| `LAUNCHDECK_BLOCK_HEIGHT_CACHE_TTL_MS` | `200` | Shared block-height cache TTL | Advanced tuning |
| `LAUNCHDECK_BLOCK_HEIGHT_SAMPLE_MAX_AGE_MS` | `1000` | Max age of sampled block-height data before forcing refresh | Advanced tuning |
| `LAUNCHDECK_FOLLOW_OFFSET_POLL_INTERVAL_MS` | `400` | Confirmed-block offset worker cadence | Advanced follow tuning |
| `LAUNCHDECK_ENABLE_APPROXIMATE_FOLLOW_OFFSET_TIMER` | `false` | Approximate low-request follow offset timing mode | Trades accuracy for fewer RPC reads |
| `LAUNCHDECK_FOLLOW_BLOCK_HEIGHT_REFRESH_MS` | legacy / not the main offset timing path | Old follow block-height refresh knob | Leave unset unless you explicitly need it |
| `LAUNCHDECK_RPC_TRAFFIC_METER` | enabled | Counts metered outbound RPC/provider traffic for the UI | Disable with `0`, `false`, `no`, or `off` |

## Benchmarking and auto-fee

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `LAUNCHDECK_BENCHMARK_MODE` | `full` | Report timing detail level | Supported: `off`, `light`, `full`; legacy `basic` maps to `light` |
| `LAUNCHDECK_TRACK_SEND_BLOCK_HEIGHT` | off by default | Default for `execution.trackSendBlockHeight` | Only relevant when benchmark mode is `full` |
| `LAUNCHDECK_AUTO_FEE_HELIUS_PRIORITY_LEVEL` | `high` | Helius auto-fee priority level | Supported: `recommended`, `none`, `low`, `medium`, `high`, `veryHigh`, `unsafeMax` |
| `LAUNCHDECK_HELIUS_PRIORITY_REFRESH_INTERVAL_MS` | `6000` | Refresh cadence for Helius fee estimates | Advanced tuning |
| `LAUNCHDECK_AUTO_FEE_JITO_TIP_PERCENTILE` | `p99` | Jito tip-floor percentile selector | Supported: `p25`, `p50`, `p75`, `p95`, `p99` |
| `LAUNCHDECK_WALLET_STATUS_REFRESH_INTERVAL_MS` | `30000` | Wallet balance/status refresh cadence in the UI | Auto-pauses during idle suspend |

## Compute-unit defaults

These are only used when a request does not already set its own compute unit limit.

| Variable | Effective default when blank | What it does |
| --- | --- | --- |
| `LAUNCHDECK_LAUNCH_COMPUTE_UNIT_LIMIT` | `340000` | Default launch compute unit limit |
| `LAUNCHDECK_AGENT_SETUP_COMPUTE_UNIT_LIMIT` | `180000` | Default agent setup compute unit limit |
| `LAUNCHDECK_FOLLOW_UP_COMPUTE_UNIT_LIMIT` | `175000` | Default follow-up compute unit limit |
| `LAUNCHDECK_SNIPER_BUY_COMPUTE_UNIT_LIMIT` | `120000` | Default sniper buy compute unit limit |
| `LAUNCHDECK_DEV_AUTO_SELL_COMPUTE_UNIT_LIMIT` | `145000` | Default automatic dev sell compute unit limit |
| `LAUNCHDECK_LAUNCH_USD1_TOPUP_COMPUTE_UNIT_LIMIT` | `90000` | Default Bonk USD1 top-up compute unit limit |

## Host, daemon, and capacity

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `LAUNCHDECK_PORT` | `8789` | Main host port | Normal local UI port |
| `LAUNCHDECK_ENGINE_AUTH_TOKEN` | built-in local token | Main host local control token | Local runtime internal auth |
| `LAUNCHDECK_FOLLOW_DAEMON_TRANSPORT` | `local-http` | Follow daemon transport mode | Current shipped default |
| `LAUNCHDECK_FOLLOW_DAEMON_URL` | `http://127.0.0.1:<follow-daemon-port>` | Explicit follow daemon base URL | Override only |
| `LAUNCHDECK_FOLLOW_DAEMON_PORT` | `8790` | Follow daemon port | Default local daemon port |
| `LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN` | built-in local token | Follow daemon local control token | Local runtime internal auth |
| `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS` | uncapped | Max active follow jobs | Blank or `0` means uncapped |
| `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES` | uncapped | Max concurrent follow compiles | Blank or `0` means uncapped |
| `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS` | uncapped | Max concurrent follow sends | Blank or `0` means uncapped |
| `LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS` | `5000` | Wait time for follow capacity when caps are set | Only matters when a cap is set |

## Helper runtime

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `LAUNCHDECK_LAUNCHPAD_HELPER_TIMEOUT_MS` | `30000` | Shared timeout for helper-backed launchpad scripts | Advanced tuning |
| `LAUNCHDECK_LAUNCHPAD_HELPER_MAX_CONCURRENCY` | `4` | Shared helper concurrency cap | Advanced tuning |
| `LAUNCHDECK_ENABLE_BAGS_HELPER_WORKER` | `true` | Persistent Bags helper worker toggle | Recommended: leave on |
| `LAUNCHDECK_ENABLE_BONK_HELPER_WORKER` | `true` | Persistent Bonk helper worker toggle | Recommended: leave on |

## Local paths

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `LAUNCHDECK_LOCAL_DATA_DIR` | `.local/launchdeck` | Base local-data directory | Root for most persisted state |
| `LAUNCHDECK_SEND_LOG_DIR` | `.local/launchdeck/send-reports` | Launch report directory | Override only when you want a custom path |
| `LAUNCHDECK_ENGINE_RUNTIME_PATH` | `.local/engine-runtime.json` | Engine runtime state path | Separate from `LAUNCHDECK_LOCAL_DATA_DIR` subtree |
| `LAUNCHDECK_FOLLOW_DAEMON_STATE_PATH` | `.local/launchdeck/follow-daemon-state.json` | Follow daemon state path | Override only when needed |

Other default local outputs under `LAUNCHDECK_LOCAL_DATA_DIR`:

- `app-config.json`
- `image-library.json`
- `lookup-tables.json`
- `uploads/`
- `send-reports/`

## Provider routing and endpoint overrides

These bypass or narrow the normal profile-based routing.

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `USER_REGION_HELIUS_SENDER` | inherit `USER_REGION` | Provider-specific Sender routing override | Use only when Sender should differ from the shared region |
| `USER_REGION_HELLOMOON` | inherit `USER_REGION` | Provider-specific Hello Moon routing override | Use only when Hello Moon should differ from the shared region |
| `USER_REGION_JITO_BUNDLE` | inherit `USER_REGION` | Provider-specific Jito routing override | Use only when Jito should differ from the shared region |
| `HELIUS_SENDER_ENDPOINT` | none | Explicit Sender endpoint override | Bypasses normal profile fanout |
| `HELIUS_SENDER_BASE_URL` | none | Alternate Sender base URL | Advanced/private integration override |
| `HELLOMOON_QUIC_ENDPOINT` | none | Explicit Hello Moon QUIC endpoint override | Format: `host:port` |
| `HELLOMOON_MEV_PROTECT` | `false` | Hello Moon connection-level MEV protection toggle | Applies to the QUIC connection |
| `JITO_BUNDLE_BASE_URLS` | none | Explicit Jito base URL set | Comma-separated |
| `JITO_SEND_BUNDLE_ENDPOINT` | none | Explicit Jito bundle send endpoint | Pair with explicit status endpoint |
| `JITO_BUNDLE_STATUS_ENDPOINT` | none | Explicit Jito bundle status endpoint | Pair with explicit send endpoint |

## Metadata upload

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `LAUNCHDECK_METADATA_UPLOAD_PROVIDER` | `pump-fun` | Metadata upload provider | Supported: `pump-fun`, `pinata` |
| `PINATA_JWT` | unset | Pinata auth token | Required only when provider is `pinata` |

Current behavior:

- `pinata` uploads fall back to `pump-fun` if Pinata fails
- the UI surfaces that fallback as a warning

## Launchpad and provider integration variables

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `BAGS_API_KEY` | unset | Bags API key | Needed for Bags usage |
| `BAGS_API_BASE_URL` | vendor default | Bags API base URL override | Advanced override |
| `HELLOMOON_API_KEY` | unset | Hello Moon Lunar Lander API key | Needed for Hello Moon execution |
| `LAUNCHDECK_BAGS_SETUP_JITO_TIP_MIN_LAMPORTS` | `1000` | Minimum Bags setup Jito tip | Advanced tuning |
| `LAUNCHDECK_BAGS_SETUP_JITO_TIP_CAP_LAMPORTS` | `1000000` | Maximum Bags setup Jito tip | Advanced tuning |

## Script and tooling compatibility

| Variable | Effective default when blank | What it does | Notes |
| --- | --- | --- | --- |
| `LAUNCHDECK_MATRIX_BASE_URL` | local runtime base URL | Base URL override for browser matrix scripts | Tooling-only |
| `RPC_URL` | unset | Generic script compatibility alias | Main LaunchDeck runtime does not need this when `SOLANA_RPC_URL` is set |

## Recommended defaults summary

If you want the shortest possible checklist, keep these as your baseline:

- `SOLANA_RPC_URL`: Helius Gatekeeper HTTP
- `SOLANA_WS_URL`: Helius standard websocket
- `LAUNCHDECK_WARM_RPC_URL`: Shyft
- `LAUNCHDECK_ENABLE_STARTUP_WARM=true`
- `LAUNCHDECK_ENABLE_CONTINUOUS_WARM=true`
- `LAUNCHDECK_ENABLE_IDLE_WARM_SUSPEND=true`
- `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true`
- `LAUNCHDECK_ENABLE_BAGS_HELPER_WORKER=true`
- `LAUNCHDECK_ENABLE_BONK_HELPER_WORKER=true`

## Related docs

- `README.md`
- `docs/CONFIG.md`
- `docs/PROVIDERS.md`
- `.env.example`
- `.env.advanced`

