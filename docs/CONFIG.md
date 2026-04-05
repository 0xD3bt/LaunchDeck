# Configuration

This page explains the configuration surface that operators interact with most often: environment variables, persisted UI settings, provider defaults, metadata upload behavior, and the rules the engine enforces regardless of what the UI stores.

`.env.example` is the short easy-start template. `.env.advanced` contains the full variable list. This document explains what the settings actually do.

## Recommended Minimum Setup

Most operators can get started with just these values:

- `SOLANA_RPC_URL`
- `SOLANA_WS_URL`
- `LAUNCHDECK_WARM_RPC_URL` if you want startup warm and block-height observation off your main RPC
- `SOLANA_PRIVATE_KEY` or additional `SOLANA_PRIVATE_KEY*`
- `USER_REGION` if you want a default provider profile or metro preference

Optional but common:

- `LAUNCHDECK_METADATA_UPLOAD_PROVIDER=pinata` ([Pinata](https://pinata.cloud/))
- `PINATA_JWT`
- `BAGS_API_KEY`

Current recommended operator stack:

- use Helius Gatekeeper HTTP for `SOLANA_RPC_URL`
- use Helius standard websocket for `SOLANA_WS_URL`
- use a [Shyft](https://shyft.to/) RPC with a free API key for `LAUNCHDECK_WARM_RPC_URL`
- use `helius-sender` or `hellomoon` as the creation, buy, and sell provider
- if you have Helius dev tier and websocket support for it, enable `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true`

LaunchDeck can run on a lower-cost setup, but Helius dev tier is strongly recommended if you want materially better watcher behavior, lower-latency execution, and a better overall operator experience. If you plan to run multiple snipes, delayed follow actions, or watcher-heavy automation, lower Helius tiers are more likely to rate-limit you.

Hello Moon note:

- `hellomoon` is a recommended alternate low-latency provider path
- it requires a Lunar Lander API key from Hello Moon
- request access through the [Lunar Lander docs](https://docs.hellomoon.io/reference/lunar-lander) or the [Hello Moon Discord](https://discord.com/invite/HelloMoon)

Recommended `.env` shape:

```bash
SOLANA_RPC_URL=https://beta.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
SOLANA_WS_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
LAUNCHDECK_WARM_RPC_URL=https://rpc.fra.shyft.to?api_key=YOUR_SHYFT_API_KEY
LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true
```

Put your Helius key immediately after `api-key=`. Put your Shyft key immediately after `api_key=`.

## Environment Variable Categories

### Core Solana Connectivity

- `SOLANA_RPC_URL`
  Main RPC used for reads, confirmations, and general runtime behavior.
- `SOLANA_WS_URL`
  Websocket endpoint used by realtime watchers. This matters for follow actions, sniper timing, and daemon health.
- recommended default pairing:
  - `SOLANA_RPC_URL=https://beta.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY`
  - `SOLANA_WS_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY`
- put your Helius key immediately after `api-key=`
- `LAUNCHDECK_STANDARD_RPC_SEND_URLS`
  Optional comma-separated extra submit endpoints used only for `standard-rpc` send fanout. `SOLANA_RPC_URL` remains the primary read/confirm RPC; these extra endpoints are used only to fan out the same signed payload in parallel on the optimized standard-RPC transport path. We suggest setting your shyft rpc in this slot.
- `LAUNCHDECK_WARM_RPC_URL`
  Optional alternate RPC used for startup warmup and block-height observation. Leave it blank to reuse `SOLANA_RPC_URL`.
- recommended default:
  - `LAUNCHDECK_WARM_RPC_URL=https://rpc.fra.shyft.to?api_key=YOUR_SHYFT_API_KEY`
- put your Shyft key immediately after `api_key=`
- `USER_REGION`
  Default region for providers that support endpoint profiles. Use a regional group (`global`, `us`, `eu`, `asia`) or pin Helius Sender metros: `slc`, `ewr`, `lon`, `fra`, `ams`, `sg`, `tyo`. You can comma-separate metros (e.g. `fra,ams`). `ny` is accepted and normalized to `ewr` (Newark / Jito `ny.`). The old `west` aggregate is removed; use `us`, `eu`, or explicit metros instead.

For a consolidated table of vendor base URLs (**Helius Sender**, **Helius RPC/WS**, **Jito**, **Hello Moon Lunar Lander**, **Shyft**), see [Full endpoint catalog (reference)](PROVIDERS.md#full-endpoint-catalog-reference) in `PROVIDERS.md`.

Recommended practice:

- set `USER_REGION` to your nearest regional group or explicit metro list instead of pinning one sender or bundle endpoint
- profile and metro fanout are usually faster and more reliable because LaunchDeck can send across the selected endpoint set instead of depending on a single host
- use provider-specific region overrides only when one provider needs a different region than your shared default
- for most operators, use Helius Gatekeeper HTTP (`https://beta.helius-rpc.com/?api-key=...`) for `SOLANA_RPC_URL`
- use Helius standard websocket (`wss://mainnet.helius-rpc.com/?api-key=...`) for `SOLANA_WS_URL` so LaunchDeck can use Helius `transactionSubscribe` watchers when enabled
- for `LAUNCHDECK_WARM_RPC_URL`, a separate [Shyft](https://shyft.to/) RPC with a free API key is a good default so startup warm and block-height reads do not consume your main execution RPC budget
- if you care about maximum performance, or plan to run multiple snipes / follow actions, Helius dev tier is strongly recommended rather than treating the free tier as your long-term production setup

If you omit `SOLANA_WS_URL`, LaunchDeck cannot do its best realtime follow behavior.

Standard RPC transport note:

- `standard-rpc` currently resolves to the optimized `standard-rpc-fanout` transport
- the engine always includes `SOLANA_RPC_URL` as the primary read/confirm RPC on that path
- `LAUNCHDECK_STANDARD_RPC_SEND_URLS` adds extra submit-only endpoints for the same signed payload
- this path currently forces `skipPreflight=true` and `maxRetries=0`

- `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE`
  Enables the enhanced Helius `transactionSubscribe` path for slot, signature, and market watchers whenever the current follow job is using a Helius websocket watch endpoint. Recommended when `SOLANA_WS_URL` points at Helius standard websocket and your Helius plan supports `transactionSubscribe`; otherwise leave it `false` and LaunchDeck will stay on the standard websocket watcher path.

Watch endpoint vs execution provider:

- `execution.provider`, `execution.buyProvider`, and `execution.sellProvider` decide how transactions are sent
- `SOLANA_WS_URL` decides which websocket watch endpoint the follow daemon uses for realtime watchers
- those are related but not identical decisions
- a launch can send with `standard-rpc` or `jito-bundle` and still use Helius realtime watchers if `SOLANA_WS_URL` is Helius
- `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true` means "if the watch endpoint is Helius, try the enhanced Helius watcher path"

### Warmup, Block-Height, And Report Timing

- `LAUNCHDECK_ENABLE_STARTUP_WARM`
  One-shot startup warm toggle. `true|false`, with blank defaulting to `true`.
- `LAUNCHDECK_ENABLE_CONTINUOUS_WARM`
  Shared active keep-warm toggle. `true|false`, with blank defaulting to `true` while the browser/app is being used. When enabled, LaunchDeck keeps the currently active provider routes warm; this can become request-heavy if creation, buy, and sell use different providers or wider endpoint sets.
- `LAUNCHDECK_ENABLE_IDLE_WARM_SUSPEND`
  Idle suspend toggle for active keep-warm. Blank defaults to `true`. When enabled, LaunchDeck also pauses the activity-driven background blockhash refresh, fee-market refresh, Jito tip stream, and scheduled wallet balance refresh while the app is idle, then resumes them on the next operator activity.
- `LAUNCHDECK_IDLE_WARM_TIMEOUT_MS`
  Idle timeout before active keep-warm suspends. Blank defaults to `75000`.
- `LAUNCHDECK_CONTINUOUS_WARM_INTERVAL_MS`
  Active keep-warm cadence. Blank defaults to `50000`.
- `LAUNCHDECK_DISABLE_STARTUP_WARM`
  Legacy negative startup-warm fallback. This is only consulted when `LAUNCHDECK_ENABLE_STARTUP_WARM` is unset.
- `LAUNCHDECK_BLOCK_HEIGHT_CACHE_TTL_MS`
  Shared cache TTL for block-height reads.
- `LAUNCHDECK_BLOCK_HEIGHT_SAMPLE_MAX_AGE_MS`
  Maximum age for sampled block-height snapshots used by reporting and diagnostics before a fresh read is forced.
- `LAUNCHDECK_FOLLOW_OFFSET_POLL_INTERVAL_MS`
  Post-confirmation follow offset worker cadence for actions with `targetBlockOffset > 0`. Blank defaults to `400ms`.
- `LAUNCHDECK_ENABLE_APPROXIMATE_FOLLOW_OFFSET_TIMER`
  Optional env-only low-request mode for follow offset timing. When enabled, the follow daemon uses an approximate local timer after confirmation instead of real `getBlockHeight` polling for offset waits. Disabled by default.
- `LAUNCHDECK_FOLLOW_BLOCK_HEIGHT_REFRESH_MS`
  Legacy follow-daemon block-height refresh knob. It no longer drives the main offset worker after the offset-worker switch and should generally stay unset unless a later non-offset block-height refresher is introduced.
- `LAUNCHDECK_BENCHMARK_MODE`
  Current runtime report timing mode. Supported values: `off`, `light`, `full`. Blank defaults to `full`. Legacy `basic` is still accepted and maps to `light`.
- `LAUNCHDECK_TRACK_SEND_BLOCK_HEIGHT`
  Default for `execution.trackSendBlockHeight`. When enabled, reports also capture observed block height at send time and confirmation time. This env default is only applied when benchmark mode is `full`; `off` and `light` keep it off by default unless a request or preset explicitly sets `execution.trackSendBlockHeight`.

Startup warm API and telemetry:

- On engine startup, the one-shot warm path checks lookup tables, Pump/Bonk warm reads, fee-market estimates, and each resolved Helius Sender host.
- **`feeMarket`** in the startup response includes `heliusPriorityLamports`, `heliusLaunchPriorityLamports`, `heliusTradePriorityLamports` (launch/trade fall back to the generic Helius estimate when a template-specific value is missing), and `jitoTipP99Lamports` when the snapshot succeeds.
- **`startupWarm`** in that response includes aggregate **`stateTargets`** / **`endpointTargets`** counts, optional failure lists (**`stateFailures`**, **`endpointFailures`**), and human-readable labels for endpoint rows (Sender prewarm).
- Helius Sender rows are warmed with an HTTP **GET** to the Sender **`/ping`** URL derived from each **`/fast`** submit URL, not with JSON-RPC on your main `SOLANA_RPC_URL`.

Continuous warm (runtime status):

- While active keep-warm runs, **`warm.stateTargets`** and **`warm.endpointTargets`** list each probe (category, label, provider, target host/URL, whether it was part of the latest pass, last attempt/success timestamps, last error, consecutive failures). The UI uses this for the platform runtime indicator.
- Rows from older configs are dropped after about **one hour** without appearing in a new pass, so the map does not grow forever when providers or endpoints change.

- `LAUNCHDECK_RPC_TRAFFIC_METER`
  When `0`, `false`, `no`, or `off`, the engine **stops counting** metered outbound RPC-credit requests for the UI pill (each `record` becomes a cheap flag check only). When enabled (default if unset), the counter includes **Solana JSON-RPC** (reads, sends, sims, confirmations, block height, warm `getVersion`), **Helius priority-fee RPC**, and **wallet balance** RPC. The UI merges **`rpcTraffic`** from both **`/api/runtime-status`** and **`/api/warm/activity`** so the value updates during keep-warm, not only on the runtime poll interval.

How these settings fit together:

- `LAUNCHDECK_ENABLE_STARTUP_WARM` is the primary startup-warm flag now.
- `LAUNCHDECK_DISABLE_STARTUP_WARM` remains only as a legacy fallback for older env setups.
- Continuous warm runs as the active-browser keep-warm layer, and idle suspend can automatically pause it when no meaningful operator interaction happens for the configured timeout.
- Meaningful interaction includes form edits, provider/mode changes, recipient edits, toggles, and build / simulate / deploy style actions.
- If continuous warm is enabled while multiple different providers are actively configured for live use, LaunchDeck should keep all of those active routes warm, which can materially increase request volume and connection usage.
- `LAUNCHDECK_BENCHMARK_MODE` uses `off`, `light`, `full`.
- Meaning:
  - `off` = benchmarking disabled
  - `light` = useful low-overhead live-safe measurement with no benchmark-only RPC calls
  - `full` = full diagnostics
- Legacy `basic` is treated as `light` during migration so older envs do not break.
- `LAUNCHDECK_TRACK_SEND_BLOCK_HEIGHT` controls whether send/confirm block-height snapshots are captured by default
- `LAUNCHDECK_TRACK_SEND_BLOCK_HEIGHT` should effectively stay off in `off`, stay off by default in `light`, and may be enabled in `full` when deeper diagnostics are wanted
- `LAUNCHDECK_WARM_RPC_URL` is used for startup warm, continuous keep-warm probes, and block-height observation so those reads can be separated from your main execution RPC
- for most operators, the best practical split is Helius Gatekeeper HTTP for `SOLANA_RPC_URL`, Helius standard websocket for `SOLANA_WS_URL`, plus Shyft for `LAUNCHDECK_WARM_RPC_URL`
- follow `targetBlockOffset > 0` actions now use a post-confirmation shared offset worker rather than the old always-active shared follow block-height polling path
- `LAUNCHDECK_FOLLOW_OFFSET_POLL_INTERVAL_MS` controls the real `getBlockHeight` polling cadence for that offset worker
- `LAUNCHDECK_ENABLE_APPROXIMATE_FOLLOW_OFFSET_TIMER` is an env-only low-request alternative that trades accuracy for fewer RPC reads
- `LAUNCHDECK_FOLLOW_BLOCK_HEIGHT_REFRESH_MS` should not be treated as the main follow offset timing knob anymore
- the Settings UI also surfaces a Warm status card based on runtime state and recent operator activity pings

### Auto-Fee And Helper Runtime Tuning

- `LAUNCHDECK_AUTO_FEE_HELIUS_PRIORITY_LEVEL`
  Helius priority-fee selector for auto-fee mode. Supported values: `recommended`, `none`, `low`, `medium`, `high`, `veryHigh`, `unsafeMax`.
- `LAUNCHDECK_HELIUS_PRIORITY_REFRESH_INTERVAL_MS`
  Background refresh cadence for the Helius `getPriorityFeeEstimate` snapshot used by auto-fee mode. Blank defaults to `6000ms`.
- `LAUNCHDECK_AUTO_FEE_JITO_TIP_PERCENTILE`
  Jito tip-floor percentile selector for auto-fee mode. Supported values: `p25`, `p50`, `p75`, `p95`, `p99`. Bags setup bundle tip selection follows this shared setting too; there is no separate Bags percentile override.
- `LAUNCHDECK_WALLET_STATUS_REFRESH_INTERVAL_MS`
  Frontend wallet balance/status refresh cadence. Blank defaults to `30000ms`. The UI still does an immediate refresh after launches and persists the last refresh timestamp in `localStorage`, but the steady-state refresh loop auto-pauses during idle suspend and resumes on activity.
- `LAUNCHDECK_LAUNCH_COMPUTE_UNIT_LIMIT`
- `LAUNCHDECK_AGENT_SETUP_COMPUTE_UNIT_LIMIT`
- `LAUNCHDECK_FOLLOW_UP_COMPUTE_UNIT_LIMIT`
- `LAUNCHDECK_SNIPER_BUY_COMPUTE_UNIT_LIMIT`
- `LAUNCHDECK_DEV_AUTO_SELL_COMPUTE_UNIT_LIMIT`
- `LAUNCHDECK_LAUNCH_USD1_TOPUP_COMPUTE_UNIT_LIMIT`
  Optional per-action default compute unit limits used when no explicit `tx.computeUnitLimit` override is set. The shipped defaults are based on observed successful transactions, with a 10% safety buffer and rounded-up values.
- `LAUNCHDECK_LAUNCHPAD_HELPER_TIMEOUT_MS`
  Shared timeout for helper-backed launchpad scripts such as Bonk and Bags.
- `LAUNCHDECK_LAUNCHPAD_HELPER_MAX_CONCURRENCY`
  Shared concurrency cap for helper-backed launchpad scripts.
- `LAUNCHDECK_ENABLE_BAGS_HELPER_WORKER`
  Optional toggle for a persistent Bags helper worker process. Disabled by default.
- `LAUNCHDECK_ENABLE_BONK_HELPER_WORKER`
  Optional toggle for a persistent Bonk helper worker process. Disabled by default.

Practical note:

- the current recommended auto-fee default is `high` plus `p99`, then cap cost in the UI with a max auto-fee value if needed
- current shipped compute-unit defaults are `340000` launch, `180000` agent setup, `175000` follow-up, `120000` sniper buy, `145000` automatic dev sell, and `90000` Bonk `usd1` top-up
- helper worker mode keeps one Node helper process alive for repeated Bonk/Bags requests
- if a helper worker transport call fails or times out, LaunchDeck falls back to restarting the helper and retrying instead of permanently requiring worker mode

### Wallet Import

- `SOLANA_PRIVATE_KEY`
- `SOLANA_PRIVATE_KEY2`
- `SOLANA_PRIVATE_KEY3`
- `SOLANA_PRIVATE_KEY4`
- `SOLANA_KEYPAIR_PATH`

Wallet import behavior:

- the UI discovers wallets from `SOLANA_PRIVATE_KEY*`
- each wallet may optionally include a label using `<privatekey>,<label>`
- unlabeled wallets appear as numbered entries
- the selected wallet is persisted in UI state, but the secret stays env-only

### Runtime And Host Control

- `LAUNCHDECK_PORT`
  Main host port. Default `8789`.
- `LAUNCHDECK_ENGINE_AUTH_TOKEN`
  Local engine control token.
- `LAUNCHDECK_FOLLOW_DAEMON_TRANSPORT`
  Follow daemon transport. Default `local-http`.
- `LAUNCHDECK_FOLLOW_DAEMON_URL`
  Explicit daemon base URL.
- `LAUNCHDECK_FOLLOW_DAEMON_PORT`
  Follow daemon port. Default `8790`.
- `LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN`
  Local follow daemon control token.

Follow concurrency and capacity:

- `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS`
- `LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS`

These are advanced follow-daemon tuning knobs.

- Blank or `0` for `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS` means uncapped active jobs.
- Blank or `0` for `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES` means uncapped follow compile concurrency.
- Blank or `0` for `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS` means uncapped follow send concurrency.
- `LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS` only matters when at least one of those caps is set.
- Invalid non-numeric values are treated as uncapped and produce a startup warning.

### Local Persistence Paths

- `LAUNCHDECK_LOCAL_DATA_DIR`
  Overrides the default `.local/launchdeck` root.
- `LAUNCHDECK_SEND_LOG_DIR`
  Overrides the report directory.
- `LAUNCHDECK_ENGINE_RUNTIME_PATH`
  Overrides the main host runtime state file path.
- `LAUNCHDECK_FOLLOW_DAEMON_STATE_PATH`
  Overrides the follow daemon state file path.

Default paths:

- `.local/launchdeck/app-config.json`
- `.local/launchdeck/image-library.json`
- `.local/launchdeck/lookup-tables.json`
- `.local/launchdeck/follow-daemon-state.json`
- `.local/launchdeck/uploads/`
- `.local/launchdeck/send-reports/`
- `.local/engine-runtime.json`

### Provider Routing And Endpoint Overrides

- `USER_REGION_HELIUS_SENDER`
  Provider-specific override for Helius Sender region.
- `USER_REGION_HELLOMOON`
  Provider-specific override for Hello Moon QUIC region.
- `USER_REGION_JITO_BUNDLE`
  Provider-specific override for Jito Bundle region.
- `HELIUS_SENDER_ENDPOINT`
  Explicit Sender endpoint override.
- `HELIUS_SENDER_BASE_URL`
  Alternate Sender base URL.
- `HELLOMOON_QUIC_ENDPOINT`
  Explicit Hello Moon QUIC endpoint override (`host:port`).
- `HELLOMOON_MEV_PROTECT`
  Enables Hello Moon QUIC MEV protection on the connection.
- `JITO_BUNDLE_BASE_URLS`
  Comma-separated Jito bundle base URLs.
- `JITO_SEND_BUNDLE_ENDPOINT`
  Explicit Jito bundle submission endpoint.
- `JITO_BUNDLE_STATUS_ENDPOINT`
  Explicit Jito bundle status endpoint.

Important behavior:

- if you set explicit endpoint overrides, LaunchDeck bypasses normal regional fanout
- if you use profiles instead, LaunchDeck fans out across the selected profile group rather than pinning a single endpoint
- for most operators, `USER_REGION` plus normal profile fanout is the recommended setup

### Metadata Upload

- `LAUNCHDECK_METADATA_UPLOAD_PROVIDER`
  Supported values: `pump-fun`, `pinata`
- `PINATA_JWT`
  Required when `pinata` is selected

Behavior:

- blank provider defaults to `pump-fun`
- `pinata` uploads the image and metadata separately
- when using `pinata`, the app can reuse the image CID across metadata-only edits
- if Pinata upload fails, LaunchDeck falls back to `pump-fun`

### Integration Credentials

- `BAGS_API_KEY`
  Required for Bagsapp usage
- `ASTRALANE_API_KEY`
- `ASTRALANE_REGION`
- `ASTRALANE_ENDPOINT`
- `BLOXROUTE_AUTH_HEADER`
- `HELLOMOON_API_KEY`
- `HELLOMOON_RPC_URL`

`HELLOMOON_API_KEY` enables the shipped Hello Moon QUIC execution provider. `HELLOMOON_RPC_URL` remains a compatibility variable for generic RPC usage, but the recommended Hello Moon setup in LaunchDeck is:

- Hello Moon QUIC for execution
- Shyft for `SOLANA_RPC_URL` or `LAUNCHDECK_WARM_RPC_URL` if you want a low-friction confirmation / warm split
- optional `HELLOMOON_MEV_PROTECT=true` when you want Hello Moon's connection-level MEV filtering

## Persisted UI Configuration

The operator-facing app persists non-secret state in `.local/launchdeck/app-config.json`.

That includes:

- selected launchpad and mode defaults
- active preset and preset editing state
- creation settings
- buy settings
- sell settings
- post-launch strategy defaults
- default automatic dev-sell state

LaunchDeck currently uses three named presets:

- `preset1`
- `preset2`
- `preset3`

Default preset behavior:

- creation provider defaults to `helius-sender`
- buy provider defaults to `helius-sender`
- sell provider defaults to `helius-sender`
- post-launch strategy defaults to `none`
- automatic dev sell defaults to disabled

Legacy provider values in old saved configs are migrated forward so stale IDs like `auto`, `helius`, or `jito` do not remain live.

## Launch Config Shape

The normalized launch config model is centered around these categories:

- `launchpad`
- `mode`
- `quoteAsset`
- `token`
- `agent`
- `tx`
- `feeSharing`
- `creatorFee`
- `bags`
- `execution`
- `devBuy`
- `postLaunch`
- `followLaunch`
- `presets`

Important operator-facing fields:

- `launchpad`
  Current values: `pump`, `bonk`, `bagsapp`
- `mode`
  Must match the chosen launchpad
- `quoteAsset`
  `bonk` supports `sol` and `usd1`; current other launchpads are `sol` only
- `selectedWalletKey`
  The env key of the wallet selected in the UI
- `token.name` and `token.symbol`
  Required and length-limited
- `token.uri`
  Required by normalization before launch can proceed
- `execution.provider`, `execution.buyProvider`, `execution.sellProvider`
  Separate provider controls for creation, buy, and sell flows
- `execution.priorityFeeSol`, `execution.buyPriorityFeeSol`, `execution.sellPriorityFeeSol`
  UI-facing priority-fee inputs. These are stored as SOL-equivalent values on a fixed `1,000,000` compute-unit basis; the engine converts them to `computeUnitPriceMicroLamports` when compiling transactions.
- `execution.tipSol`, `execution.buyTipSol`, `execution.sellTipSol`
  UI-facing tip inputs for creation, buy, and sell flows.
- `tx.computeUnitPriceMicroLamports`
- `tx.jitoTipLamports`
- `followLaunch`
  Explicit follow-action configuration

## Engine-Enforced Rules

The engine is stricter than the UI and will reject incompatible combinations.

### Launchpad Rules

- `bonk` accepts only `regular` and `bonkers`
- `bonk` rejects fee-sharing setup
- `bonk` rejects `cashback`
- `bonk` rejects `mayhem`
- `bagsapp` accepts only `bags-2-2`, `bags-025-1`, and `bags-1-025`
- `bagsapp` currently supports only `quoteAsset=sol`
- `bagsapp` rejects Pump agent modes
- `bagsapp` requires creator fee to remain the deployer wallet

### Provider Rules

For `helius-sender`:

- `execution.skipPreflight` must be `true`
- `tx.computeUnitPriceMicroLamports` must be greater than `0`
- `tx.jitoTipLamports` must be at least `200000`

For `jito-bundle`:

- `tx.computeUnitPriceMicroLamports` must be greater than `0`
- `tx.jitoTipLamports` must be at least `200000`

For all providers:

- removed provider values such as `auto` are not valid live config values anymore
- the shipped engine is `rust-native-only`, so unsupported launchpad / mode combinations hard-fail instead of falling back to a generic JS compiler path

### Fee-Sharing And Mode Rules

- `feeSharing.generateLaterSetup` is supported only in Pump `regular`
- if later fee-sharing setup is enabled, fee recipients must be present
- fee-sharing recipients must total `10000` bps
- mode-specific creator-fee behavior is enforced by normalization

### Follow Rules

- `followLaunch.snipes[].postBuySell` is not supported yet and is rejected
- `submitWithLaunch` cannot be combined with `submitDelayMs` or `targetBlockOffset`
- follow constraints and retry budgets are validated
- if any follow sniper buy is enabled and `execution.buyAutoGas=false`, then `helius-sender` and `jito-bundle` buy routes require `execution.buyPriorityFeeSol > 0` and `execution.buyTipSol >= 0.0002`
- if automatic dev sell is enabled and `execution.sellAutoGas=false`, then `helius-sender` and `jito-bundle` sell routes require `execution.sellPriorityFeeSol > 0` and `execution.sellTipSol >= 0.0002`
- automatic dev sell supports an exclusive `time` or `market-cap` trigger family
- market-cap timeout is stored in seconds and supports `timeoutAction=stop|sell`
- Pump `agent-custom` and `agent-locked` use an explicit creator-vault split for follow actions:
  - same-window `+0` buys and sells stay on the original launch-creator / deployer vault path
  - delayed buys and delayed sells with `targetBlockOffset > 0` prefer the post-setup fee-sharing config vault path
  - delayed Pump buys in those modes are compiled live in the daemon instead of being pre-signed too early, so they can pick up the current on-chain vault state
  - if a pre-signed Pump follow action still lands against stale state, the daemon can clear it and rebuild a fresh payload before retrying

## Provider Defaults And Preset Defaults

The app tries to give operators a sensible baseline without manual tuning.

Current defaults include:

- default provider: `helius-sender`
- default creation priority fee: `0.000001`
- default creation tip: `0.0002`
- default trade priority fee: `0.000001`
- default trade tip: `0.0002`
- default trade slippage: `20`
- default quick dev-buy presets: `0.5`, `1`, and `2`

Defaults are only a starting point. The engine may still override behavior depending on provider or launch shape, and any settings-modal saves still persist only to the local ignored `.local/launchdeck/app-config.json`.

## Endpoint Profiles

Endpoint profiles are available only for providers that support them:

- `Helius Sender`
- `Jito Bundle`

Supported profile values:

- `global`
- `us`
- `eu`
- `asia`
- Helius Sender metros (and Jito filtering by matching block-engine host): `slc`, `ewr`, `lon`, `fra`, `ams`, `sg`, `tyo` (optional comma list, e.g. `fra,ams`)
- `ny` (normalized to `ewr` for Sender; matches Newark Jito hosts)

The `west` profile is no longer supported.

Resolution order:

1. provider-specific region override such as `USER_REGION_HELIUS_SENDER`
2. shared `USER_REGION`
3. provider default fallback

## Metadata Upload Providers

### `pump-fun`

Use this when you want the default LaunchDeck metadata flow.

- default when no provider is specified
- uploads image and metadata together
- supports URI reuse when the metadata fingerprint is unchanged

### `pinata`

Use this when you want [Pinata](https://pinata.cloud/)-backed uploads.

- requires `PINATA_JWT`
- uploads the image to Pinata
- pins metadata JSON separately
- reuses the image CID across metadata-only edits during the current session
- falls back to `pump-fun` if the Pinata path fails

## Runtime Reports And Storage

Reports are written to `.local/launchdeck/send-reports` by default unless `LAUNCHDECK_SEND_LOG_DIR` overrides the path.

Reports can include:

- requested provider
- resolved provider
- transport type
- endpoint information
- winning endpoint and attempted endpoint list for multi-endpoint transports
- send order
- signature and confirmation state
- benchmark timing data
- auto-fee source summaries when auto-fee resolved live estimates
- follow-job snapshot
- follow-action outcomes
- watcher health
- follow timing profiles

Timing breakdowns separate:

- `total`
- `backendTotal`
- `preRequest`
- compile sub-timings such as `altLoad`, `blockhash`, `global`, `followUpPrep`, and `serialize`
- send sub-timings such as `submit` and `confirm`
