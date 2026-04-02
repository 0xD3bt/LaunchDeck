# Follow Daemon

This page explains the dedicated follow daemon that LaunchDeck uses for delayed and watcher-driven post-launch actions.

## What The Daemon Is For

The follow daemon exists so LaunchDeck can keep working on follow actions after the main launch request has already returned.

Current follow action families:

- `SniperBuy`
- `DevAutoSell`
- `SniperSell`

The daemon is meant to:

- keep running between launches
- accept follow jobs quickly
- watch launch progress using realtime watchers
- compile and send follow actions without blocking the main host
- persist follow-job state, watcher health, telemetry, and timing profiles

Default local URL:

- `http://127.0.0.1:8790`

## Host Vs Daemon

### Main Host

The main host is responsible for:

- serving the UI
- handling `/api/*`
- normalizing and validating launch requests
- compiling and sending the launch
- compiling and submitting same-time sniper buys
- reserving and arming follow jobs

### Follow Daemon

The daemon is responsible for:

- accepting reserved jobs
- arming jobs once launch-specific context is known
- running slot, signature, and market watchers
- executing delayed sniper buys
- executing dev auto-sells
- executing snipe sells
- persisting independent follow-job state

## Job Lifecycle

The current lifecycle is:

1. the main host reserves a follow job before send when follow behavior is enabled
2. the launch is sent and launch-specific context is captured
3. the host arms the reserved job with mint, signature, send block, and related context
4. the daemon marks actions as armed
5. each action waits for its trigger
6. the daemon compiles, sends, confirms, and records each action independently

This keeps delayed follow behavior off the main request path while still letting the UI work with a single launch action.

## Trigger Modes

### `Same Time`

Same-time sniper buys are not primarily daemon-triggered. They are submitted alongside launch creation.

Use this mainly when your latency is high enough that waiting for observed submit timing may leave you behind. In normal low-latency setups, it is usually better to use a daemon-triggered mode.

How it works:

- selected same-time buys compile alongside the launch
- Bonk uses launch-first submission on non-bundle transports so the buy path does not outrun creation
- Bonk `usd1` same-time sniper buys compile as atomic swap-and-buy transactions
- if a same-time buy lands before creation, it fails
- a same-time fee safeguard warns when buy-side fees are higher than launch fees
- eligible same-time buys can arm a one-time daemon retry if the first landing fails

Retry behavior:

- retry is only available for same-time sniper buys
- the retry is a new deferred buy, not reuse of the original same-time transaction
- the retry is skipped if the wallet already holds the token

### `On Submit + Delay`

Use this for sniper buys or auto-sell actions scheduled from observed launch submission.

How it works:

- `0ms` means send on observed submit
- non-zero values wait from observed submit plus the configured delay
- execution happens in the daemon, not inline with the launch flow
- this mode is faster than `On Confirmed Block`, but it can still fail if the buy reaches chain execution before creation is live

### `On Confirmed Block`

Use this when you want the safest currently shipped buy trigger. This is the default recommendation for most users.

How it works:

- the daemon watches launch-relative block progress
- the action fires when the configured confirmed-block target is observed
- because it waits for observed launch state, it is more conservative than `Same Time`
- use the current UI-configured range rather than older stale docs that referenced a smaller range

### Sell Triggers

Sell-side follow actions can also wait on:

- delay-based timing
- market-cap triggers
- confirmation requirements

Current dev auto-sell behavior:

- the UI exposes mutually exclusive `Time` and `Market Cap` trigger families
- market-cap mode is exclusive and does not silently carry a hidden time delay
- market-cap scan timeout is configured in seconds
- timeout behavior can either `Stop` or proceed to `Sell`

## Watchers

The daemon uses dedicated watchers for realtime trading behavior.

Current watcher types:

- slot watcher
- signature watcher
- market watcher

Operational notes:

- watchers rely on websocket connectivity for best realtime behavior
- watcher health is persisted
- reconnect and backoff behavior are explicit
- if websocket connectivity is poor, follow timing quality can degrade

Current watcher modes:

- slot watcher: standard websocket by default, or Helius `transactionSubscribe` when enabled and supported
- signature watcher: standard websocket by default, or Helius `transactionSubscribe` when enabled and supported
- market watcher: standard websocket by default, or Helius `transactionSubscribe` when enabled and supported

For the best current setup:

- use Helius for `SOLANA_RPC_URL`
- use Helius for `SOLANA_WS_URL`
- use a [Shyft](https://shyft.to/) RPC with a free API key for `LAUNCHDECK_WARM_RPC_URL`
- use `helius-sender` as the provider
- enable `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true` if you are on Helius dev tier and your websocket supports it

Helius dev tier is strongly recommended here because it gives a major improvement in realtime watcher quality and follow execution behavior compared with a bare-minimum setup.

The daemon now persists both watcher health and watcher mode so reports and follow-job state show whether a market-cap action used the enhanced Helius path or the standard websocket fallback.

## Delayed-Buy Hot Path

Delayed buys use a hotter path than a cold rebuild-from-scratch model.

How it works:

- launch-specific follow state is pre-resolved when the job is armed
- static buy preparation is cached per action at arm time
- hot runtime follow state is refreshed while the job is alive
- delayed buys use a `prepare -> finalize` split model

In practice, the trigger-time work focuses on:

- fresh blockhash attachment
- fresh quote or finalize work
- signing and serialization

That keeps delayed triggers lighter than a full cold compile.

## Shared Cache Behavior

The runtime uses warmed blockhash caches in both the host and daemon.

How it works:

- blockhashes are warmed for `processed`, `confirmed`, and `finalized`
- block-height observation can be offloaded to `LAUNCHDECK_WARM_RPC_URL` instead of your main execution RPC
- cache hits can make `compileBlockhashFetchMs` look like `0ms` in reports
- lookup tables and follow-runtime state are also warmed where relevant

Useful tuning env vars for this path:

- `LAUNCHDECK_FOLLOW_BLOCK_HEIGHT_REFRESH_MS`
- `LAUNCHDECK_BLOCK_HEIGHT_CACHE_TTL_MS`
- `LAUNCHDECK_BLOCK_HEIGHT_SAMPLE_MAX_AGE_MS`

## Same-Time Fee Safeguard

The same-time safeguard exists to reduce the chance that a sniper buy lands before the creation transaction.

How it works:

- it applies only when same-time fees are strictly higher than creation fees
- the UI shows the additional creator fee impact inline
- the safeguard is a warning and shaping aid, not a guarantee

## Agent-Mode Sell Hardening

Agent launch modes receive extra handling on the sell side.

How it works:

- `agent-custom` and `agent-locked` prefer the post-setup creator-vault authority path
- creator-vault seed mismatch can trigger targeted sell retry behavior
- daemon-side snipe sells track attempt counters in reports

## Telemetry And Timing Profiles

The daemon persists telemetry that later appears in reports.

Current telemetry includes:

- provider
- endpoint profile
- transport type
- trigger type
- delay and jitter settings
- submit latency
- confirm latency
- launch-to-action latency
- launch-to-action block distance
- schedule slip
- action outcome and quality labels

Timing profiles include historical percentiles such as:

- `P50 Submit`
- `P75 Submit`
- `P90 Submit`

These are visibility aids. They do not automatically slow or retime current actions.

## Current Limitation

The daemon does not currently support per-sniper `postBuySell` chaining.

This config is explicitly rejected:

- `followLaunch.snipes[].postBuySell`

## Relevant Configuration

Key daemon env vars:

- `LAUNCHDECK_FOLLOW_DAEMON_TRANSPORT`
- `LAUNCHDECK_FOLLOW_DAEMON_URL`
- `LAUNCHDECK_FOLLOW_DAEMON_PORT`
- `LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN`
- `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS`
- `LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS`
- `LAUNCHDECK_FOLLOW_DAEMON_STATE_PATH`
