# Follow Daemon

## Overview

`LaunchDeck` now uses a dedicated local Rust follow daemon for launch-follow actions that need to continue after the creation request has already been submitted.

Current follow action types:

- `SniperBuy`
- `DevAutoSell`
- `SniperSell`

The daemon is designed to:

- stay running between launches
- accept new follow jobs immediately
- watch launch progress over websocket-backed watchers
- compile and send follow actions without blocking the main UI host
- persist follow-job state, telemetry, timing profiles, and watcher health

Default local follow-daemon URL:

- `http://127.0.0.1:8790`

## Runtime Shape

LaunchDeck now has two local Rust processes:

- Rust host on the UI/API port
- follow daemon on the follow-daemon port

The Rust host is responsible for:

- serving the browser UI
- handling `/api/*` requests
- compiling launch transactions
- submitting launch creation and any immediate same-time follow buys
- reserving and arming follow jobs with the daemon

The follow daemon is responsible for:

- receiving reserved follow jobs
- arming them once mint/signature/send context is known
- running websocket-backed slot, signature, and market watchers
- executing delayed sniper buys, dev auto-sells, and sniper sells
- persisting job state and telemetry during and after execution

## Job Lifecycle

The current lifecycle is:

1. The Rust host reserves a follow job before send when follow behavior is enabled.
2. The host sends the creation flow and captures launch metadata such as mint, launch creator, signature, and observed send block.
3. The host arms the reserved follow job with that launch-specific information.
4. The daemon marks actions as armed and starts the job runner if needed.
5. Each action waits for its trigger, compiles, sends, confirms, and reports independently.

This keeps follow behavior off the main request path while still allowing immediate post-send actions.

## Trigger Modes

### Same Time

`Same Time` sniper buys are submitted alongside launch creation rather than waiting for the daemon trigger path.

Current behavior:

- all selected same-time buys compile concurrently
- non-bundle transport can submit launch and same-time sniper transactions concurrently
- same-time buys are protected by a creation-fee safeguard when buy fees exceed launch fees
- same-time buys can optionally arm a one-time daemon retry if the first landing fails

Current same-time retry behavior:

- retry is only available for same-time sniper buys
- retry is cloned into a deferred daemon buy rather than reusing the same transaction
- the fallback retry uses a small submit delay
- the retry is skipped if the wallet already holds the launch token

### On Submit + Delay

For sniper buys and dev auto-sell, `On Submit + Delay` means the action schedules from observed launch submission time.

Current behavior:

- `0ms` is effectively immediate after submit observation
- non-zero values add an explicit delay from launch submit observation
- this path is daemon-executed rather than same-time compiled into the launch path

### Block Offset

`Block Offset` means the action is sent when the watcher observes the configured launch-relative block.

Current supported range:

- `0-5`

This is useful when you want block-based timing rather than millisecond timing.

### Confirmation / Delayed Follow Sells

Sell-side follow actions can also wait on:

- delay after launch
- market-cap triggers
- confirmation requirements

Those actions remain daemon-executed and watcher-driven.

## Watchers

The daemon uses dedicated realtime watchers for trading behavior.

Current watcher types:

- slot watcher
- signature watcher
- market watcher

Current behavior:

- realtime trading watchers rely on websocket endpoints
- watcher health is tracked and persisted
- reconnect/backoff rules are explicit
- diagnostics and status tooling can still use non-realtime reads, but the trading watcher path is websocket-first

## Current Delayed-Buy Hot Path

Delayed sniper buys now use a hotter daemon-side compile path than before.

Current improvements:

- launch-specific follow-buy state is pre-resolved when the job is armed
- per-action static buy preparation is cached at arm time
- hot runtime follow-buy state is refreshed in the daemon while the job is alive
- delayed buys now use a split `prepare -> finalize` path instead of paying the full cold compile path every time
- warm-up across sniper wallets is prepared concurrently at arm time

In practice this means the delayed-buy trigger path now mainly pays for:

- fresh blockhash attach
- fresh quote/finalize step
- signing and serialization

rather than redoing the whole setup path from scratch on every trigger.

## Blockhash and Runtime Caching

The shared RPC cache still stores blockhashes on demand and also supports background warming.

Current behavior:

- blockhash refresh interval: `30s`
- blockhash max age before forced miss: `45s`
- the Rust host warms `processed`, `confirmed`, and `finalized`
- the follow daemon also warms `processed`, `confirmed`, and `finalized`

Important note:

- `compileBlockhashFetchMs = 0` in reports usually means a hot cache hit rounded down to `0ms`, not that no blockhash was used

Additional daemon-side warm state now includes:

- prepared delayed-buy static state per action
- hot launch-specific runtime state per job

## Same-Time Safeguard

When same-time sniper buys are enabled, LaunchDeck can automatically raise launch fees above same-time buy fees.

Current behavior:

- safeguard only applies when same-time buy fee settings are strictly higher than creation fees
- the UI shows the extra creator fee cost
- the notice is shown inline under the affected same-time trigger control

This is intended to reduce the chance that same-time buys land before the creation transaction.

## Agent Launch Hardening

Agent launch modes now have additional follow-action handling.

Current hardening:

- `agent-custom` and `agent-locked` sells prefer the post-setup creator vault authority
- creator-vault seed mismatch can trigger a targeted sell retry
- daemon-side follow sells keep explicit state and attempt counters in reports

## Telemetry and Timing Profiles

The daemon writes follow telemetry samples and timing profiles that are later surfaced in persisted reports.

Current telemetry captures:

- provider and endpoint profile
- transport type
- trigger type
- delay and jitter settings
- submit latency
- confirm latency
- launch-to-action latency
- launch-to-action blocks
- schedule slip
- outcome and quality labels

Current timing profiles summarize historical submit performance, including:

- `P50 Submit`
- `P75 Submit`
- `P90 Submit`

These values are historical percentiles used for visibility and timing suggestions. They do not automatically slow current execution.

## Configuration

Key follow-daemon env variables:

- `LAUNCHDECK_FOLLOW_DAEMON_TRANSPORT`
- `LAUNCHDECK_FOLLOW_DAEMON_URL`
- `LAUNCHDECK_FOLLOW_DAEMON_PORT`
- `LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN`
- `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS`
- `LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS`
- `LAUNCHDECK_FOLLOW_DAEMON_STATE_PATH`

Current local helper scripts:

- `npm start`: cleanly stops old Rust host/follow-daemon processes, starts both, and opens the UI
- `npm stop`: stops both local processes
- `npm restart`: runs the same clean recycle behavior explicitly

## Suggested Reading

- `docs/ARCHITECTURE.md`
- `docs/CONFIG.md`
- `docs/STRATEGIES.md`
- `docs/FRONTEND_REGRESSION_CHECKLIST.md`
