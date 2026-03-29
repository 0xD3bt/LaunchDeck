# LaunchDeck by Trench.tools

LaunchDeck is a self-hosted Solana launch and snipe tool built under the broader `Trench.tools` project.

Instead of paying fees to third-party launch platforms, LaunchDeck lets you run the launcher locally, use your own wallets and provider keys, and customize how launches are built, simulated, and sent. The basic version can be run with a free-tier Helius key, so getting started does not require paid infrastructure.

This is a fresh and actively worked product. As features become functional, tested, and ready, they will be listed here more clearly over time.

LaunchDeck is open-source tooling provided as-is. Running it, configuring it, modifying it, deploying it, or using it in any way is entirely the user's own responsibility. By using this software, you accept full responsibility for your environment, infrastructure, wallets, keys, dependencies, third-party packages, and any outcomes that result from its use. Trench.tools is not responsible for losses, damages, exploits, malicious code, compromised packages, misconfiguration, misuse, downtime, failed transactions, or any other direct or indirect consequences related to the software or its dependencies.

## What LaunchDeck Is

LaunchDeck is built for anyone who wants to:

- run token launches locally
- run launch/snipe workflows without platform fees
- use their own infrastructure and API keys
- avoid relying on third-party launch services
- customize execution settings, wallets, and launch behavior

## Current Runtime Model

LaunchDeck now runs as two local Rust processes:

- Rust host on the UI/API port
- follow daemon on the follow-daemon port

The Rust host serves:

- the browser UI static files
- browser-facing `/api/*` routes
- engine `/engine/*` routes
- local uploads under `/uploads/*`

The follow daemon manages launch-follow actions, realtime watchers, follow telemetry, and persisted timing profiles.

The current initial active launch runtime is `pump` only, but the UI host, settings persistence, image library, reports browser, vamp import flow, and follow-daemon control plane now also live in Rust.

Default local entrypoints:

- UI host: `http://127.0.0.1:8789`
- follow daemon: `http://127.0.0.1:8790`

## Current Pump Coverage

The Rust-native Pump path currently covers:

- `regular`
- `cashback`
- `agent-custom`
- `agent-unlocked`
- `agent-locked`

Pump launch assembly, transaction shaping, reporting, simulation, and send execution now run through the Rust engine rather than the legacy JS compile bridge for these verified launch shapes.

Bonk and Bagsapp should not currently be treated as active launch flows in the initial version.

## Run Locally

Primary local entrypoints:

- `npm start`
- `npm stop`
- `npm restart`
- `npm run ui`

`npm start` uses `start.ps1` to stop any existing LaunchDeck engine or follow-daemon processes, start both, wait for health, and open the local UI.

`npm stop` uses `stop.ps1` to stop any running LaunchDeck engine or follow-daemon processes, including stale listeners on the configured local ports.

`npm restart` performs the same clean recycle explicitly.

`npm run ui` starts the Rust host directly without the helper script.

The host uses `LAUNCHDECK_PORT` as the local UI/API port.

The send layer now exposes explicit provider choices:

- `Helius Sender`
- `Standard RPC`
- `Jito Bundle`

There is no `auto` provider fallback anymore. The selected provider determines transport shape, send requirements, and reporting.

## Provider Rules

- `Helius Sender` is the recommended default.
- `Helius Sender` requires inline tip, inline priority fee, `skipPreflight=true`, and `maxRetries=0`.
- `Standard RPC` uses standard sequential Solana sending for dependent flows and does not use tip.
- `Jito Bundle` keeps bundle-specific tip behavior.

Providers that expose multiple documented endpoint groups can also use an `Endpoint Profile`.

Current profiles:

- `Global`
- `US`
- `EU`
- `West`
- `Asia`

This is currently relevant to:

- `Helius Sender`
- `Jito Bundle`

When an endpoint profile is selected for a supported provider, LaunchDeck now fans out across the endpoints in that profile group rather than single-picking one endpoint.

The UI collects user intent, but the engine is the source of truth for what actually gets applied to each transaction.

Examples:

- `Standard RPC` ignores tip even if a preset still has an old tip value.
- `Helius Sender` hard-forces Sender-compatible send flags.
- `Jito Bundle` creation can accept both tip and priority in the UI, but the engine may intentionally drop creation priority for multi-transaction launch flows where it would only waste money.

## Follow Launch System

LaunchDeck now supports a dedicated follow-launch system for launch-adjacent actions.

Current follow behavior includes:

- same-time sniper buys compiled alongside launch creation
- daemon-executed sniper buys using `On Submit + Delay`
- daemon-executed sniper buys using `Block Offset`
- automatic dev sell execution
- sniper sell follow actions
- inline same-time fee safeguard warnings
- optional one-time same-time retry through the daemon

Current timing modes:

- `Same Time`: submit alongside launch creation
- `On Submit + Delay`: schedule from observed launch submit time
- `Block Offset`: send when the configured launch-relative block is observed

## Reporting

LaunchDeck now writes richer execution reports that capture:

- requested provider
- resolved provider
- transport type
- endpoint used
- send order
- signature and confirmation status
- tip and compute-unit settings actually included

Durable send reports are persisted under the local runtime area so launches can be audited after the fact.

The benchmark output now includes both end-to-end and backend-only timings:

- `total`: full click-to-finish time seen by the user
- `backendTotal`: Rust-side execution time after `/api/run` is received
- `preRequest`: browser-side wait before the request is dispatched
- compile breakdowns such as `altLoad`, `blockhash`, `global`, `followUpPrep`, and `serialize`
- send breakdowns such as `submit` and `confirm`

This makes metadata wait, compile latency, and chain confirmation time visible separately in both the main output and persisted reports.

Follow reports now also include:

- persisted follow-job state
- action-level outcomes for sniper buys and follow sells
- watcher health
- follow telemetry samples
- timing profiles such as `P50 Submit`, `P75 Submit`, and `P90 Submit`

## Launch Optimizations

Recent Pump-focused optimizations in the Rust runtime include:

- measured versioned transaction selection with lookup-table-aware sizing diagnostics
- curated default lookup table coverage for launch and follow-up flows
- local lookup-table persistence and warm-up on page load
- background blockhash refresh with cache age limits
- cached Pump global state for dev-buy quoting and compile-time launch assembly
- arm-time warm-up for delayed follow-buy state
- split delayed-buy prepare/finalize flow in the follow daemon
- concurrent same-time sniper compile and non-bundle submit paths
- immediate metadata pre-upload from the UI once image, name, and ticker are present
- configurable metadata upload provider:
  - `pump-fun` remains the default
  - `pinata` is optional through env config
  - Pinata uploads reuse the uploaded image CID across metadata-only edits so name/symbol/description changes only need metadata JSON repinning
  - when `pinata` is selected and its upload fails, LaunchDeck automatically falls back to `pump-fun`

Pinata can also be tested on its free tier, which is enough for basic LaunchDeck experimentation:

- storage: `1 GB`
- pinned files: `500`
- API rate limit: `60 requests/minute`
- dedicated gateway: `1`
- gateway bandwidth: `10 GB/month`
- gateway requests: `10,000/month`

## Local Data

The Rust host preserves the existing local storage layout under `.local/launchdeck`:

- `app-config.json`
- `image-library.json`
- `lookup-tables.json`
- `uploads/`
- `send-reports/`

This keeps existing UI settings, uploaded images, and persisted reports compatible through the Rust-only cutover.

Follow-daemon state and telemetry are also persisted under the same local runtime area.

## Docs

- `docs/ARCHITECTURE.md`
- `docs/PROVIDERS.md`
- `docs/CONFIG.md`
- `docs/LAUNCHPADS.md`
- `docs/STRATEGIES.md`
- `docs/FOLLOW_DAEMON.md`
- `docs/FRONTEND_REGRESSION_CHECKLIST.md`
- `docs/EXECUTION_PROVIDER_PLAN.md`
