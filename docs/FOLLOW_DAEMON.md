# Follow Daemon

This page explains the dedicated follow daemon that LaunchDeck uses for delayed and watcher-driven post-launch actions.

Default local URL:

- `http://127.0.0.1:8790`

## What the daemon does

The follow daemon exists so follow actions do not have to stay on the main launch request path.

Current follow action families:

- `SniperBuy`
- `DevAutoSell`
- `SniperSell`

The daemon handles:

- delayed follow execution
- confirmation-driven follow execution
- realtime slot, signature, and market watchers
- follow timing and watcher telemetry
- persisted follow-job state

## Host vs daemon

### Main host

The main host is responsible for:

- serving the UI
- accepting the launch request
- normalizing and validating config
- compiling and sending the launch
- compiling same-time sniper buys
- reserving and arming follow jobs

### Follow daemon

The follow daemon is responsible for:

- accepting armed follow jobs
- waiting for follow triggers
- watching chain activity
- compiling delayed follow actions
- sending and confirming delayed follow actions
- persisting follow state and outcomes

## Job lifecycle

The normal lifecycle is:

1. the main host reserves the follow job before launch send
2. the launch is sent
3. the main host arms the reserved job with launch context such as mint and signature
4. the daemon waits for each trigger
5. the daemon compiles, sends, confirms, and records each action independently

## Trigger modes

| UI trigger | Primary owner | What it waits for |
| --- | --- | --- |
| `Same Time` | main host | launch submission path itself |
| `On Submit + Delay` | follow daemon | observed submit time plus delay |
| `On Confirmed Block +0` | follow daemon watcher | launch confirmation |
| `On Confirmed Block +N` | follow daemon watcher + offset worker | launch confirmation, then extra confirmed blocks |
| `Market Cap` | follow daemon market watcher | USD market-cap threshold or timeout |

Practical guidance:

- `Same Time` is the aggressive path
- `On Submit + Delay` is fast but still earlier than confirmation-driven modes
- `On Confirmed Block` is the safest normal default for buys

## Watchers

The daemon currently uses:

- slot watchers
- signature watchers
- market watchers

Watcher quality depends heavily on the websocket path.

Recommended setup:

- `SOLANA_RPC_URL`: Helius Gatekeeper HTTP
- `SOLANA_WS_URL`: Helius standard websocket
- `LAUNCHDECK_WARM_RPC_URL`: Shyft
- provider: `helius-sender` or `hellomoon`

If you are watcher-heavy or running multiple snipes, Helius dev tier is strongly recommended.

## Standard websocket vs Helius transactionSubscribe

Watcher mode is selected from the websocket path, not from the send provider.

Current selection logic:

| Condition | Watcher mode |
| --- | --- |
| no usable websocket endpoint | websocket watchers cannot be used normally |
| websocket is non-Helius | standard websocket watchers |
| websocket is Helius and `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=false` | standard websocket watchers |
| websocket is Helius and `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true` | probe Helius `transactionSubscribe` first |
| Helius probe fails | fall back to standard websocket watchers |

Important note:

- this is probe-and-fallback behavior, not a hard dependency

## Watcher warm behavior

Watcher websocket warm runs on the same general warm system as the rest of the app.

That means:

- startup warm probes the watcher path once at startup
- continuous warm keeps the active watcher path hot while the app is in use
- idle warm suspend pauses watcher warm traffic when the app is idle
- if the effective watcher path changes, LaunchDeck rewarms the new path
- if the effective path did not change, LaunchDeck does not restart that warm loop just because settings were saved again

## Delayed-buy hot path

Delayed follow actions are not rebuilt completely from scratch at trigger time.

LaunchDeck pre-resolves and caches as much state as it can when the job is armed, then finishes the hot work at trigger time.

In practice that means the trigger-time path focuses on:

- fresh blockhash
- current quote/finalization work where needed
- signing and serialization

## Pump agent-mode notes

Pump `agent-custom` and `agent-locked` follow behavior has extra vault handling.

Current behavior:

- non-secure `+0` buys and sells stay on the original launch-creator / deployer vault path
- secure `+0` buys and sells can switch immediately to the post-setup config-vault path
- delayed buys and sells that need the post-setup vault path prefer the config vault path
- if stale pre-signed state is detected, the daemon can rebuild and retry instead of blindly resubmitting the same stale payload

## Same-time retry

Eligible same-time sniper buys can arm a one-time daemon retry if the first landing fails.

That retry is:

- a new deferred buy
- not reuse of the exact original same-time payload
- skipped if the wallet already holds the token

## Relevant env vars

Most operators do not need to change these, but these are the daemon-related knobs:

- `LAUNCHDECK_FOLLOW_DAEMON_TRANSPORT`
- `LAUNCHDECK_FOLLOW_DAEMON_URL`
- `LAUNCHDECK_FOLLOW_DAEMON_PORT`
- `LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN`
- `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS`
- `LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS`
- `LAUNCHDECK_FOLLOW_DAEMON_STATE_PATH`
- `LAUNCHDECK_FOLLOW_OFFSET_POLL_INTERVAL_MS`
- `LAUNCHDECK_ENABLE_APPROXIMATE_FOLLOW_OFFSET_TIMER`
- `LAUNCHDECK_BLOCK_HEIGHT_CACHE_TTL_MS`
- `LAUNCHDECK_BLOCK_HEIGHT_SAMPLE_MAX_AGE_MS`

Full env details live in `docs/ENV_REFERENCE.md`.
