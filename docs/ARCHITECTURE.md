# Architecture

`LaunchDeck` is organized as a standalone publishable product.

## Layers

- `ui/`: browser form, preview, validation, and status rendering
- `rust/launchdeck-engine`: native UI/API host plus launch config validation, transaction assembly, transport planning, simulation, sending, durable send logs, runtime workers, local settings persistence, image library APIs, reports browsing, vamp import, and follow-daemon client orchestration
- `rust/launchdeck-engine/src/bin/launchdeck-follow-daemon.rs`: dedicated realtime follow-action daemon for sniper buys, dev auto-sells, sniper sells, watcher health, and follow telemetry
- `rust/launchdeck-engine/src/bin/launchdeck-cli.rs`: Rust-native CLI for build/simulate/send from config files
- `providers/`: provider adapter layer and execution routing
- `launchpads/`: launchpad registry and launchpad-specific capability metadata

## Execution Flow

1. UI reads backend status and defaults.
2. UI pre-uploads launch metadata when possible and submits normalized form data to `/api/run`.
3. `rust/launchdeck-engine` converts UI form state into raw launch config and normalizes the request.
4. The Rust engine builds a provider-aware transport plan.
5. The Rust engine validates provider requirements, builds native transactions, and applies provider-specific tx shaping.
6. The Rust engine simulates and/or sends transactions through the selected transport.
7. When follow behavior is enabled, the Rust host reserves and later arms a follow job with the local follow daemon.
8. The Rust engine persists execution/send report output for later auditing.

## Transport Layer

The runtime now uses explicit transport types instead of implicit provider fallback:

- `helius-sender`
- `standard-rpc-sequential`
- `jito-bundle`

The selected transport controls:

- whether tip is inline or separate
- whether priority fee is required, allowed, or intentionally dropped
- `skipPreflight`
- `maxRetries`
- endpoint selection
- endpoint profile selection where the provider supports multiple documented endpoint groups
- sequential vs bundle send semantics

## Engine-Owned Decisions

The UI collects requested settings, but the engine owns final transaction shaping.

Important examples:

- `Helius Sender` requires inline tip and compute unit price, and hard-fails if the request is incompatible.
- `Standard RPC` remains sequential for dependent launch/follow-up flows.
- `Jito Bundle` creation may accept both tip and priority in the UI, but the engine can intentionally drop creation priority for multi-transaction launch flows where it is not useful.

The same engine-owned shaping applies to:

- measured versioned transaction selection for Pump launch flows
- curated address lookup table usage per transaction shape
- metadata upload provider selection and send-time metadata fallback
- same-time buy fee safeguards
- delayed follow-buy pre-resolution and finalize behavior

## Current Boundary

Pump is the only active native build/send path in the current initial version.

Verified native Pump coverage currently includes:

- `regular`
- `cashback`
- `agent-custom`
- `agent-unlocked`
- `agent-locked`

Bonk and Bags are not active launch flows in the current initial version and should not be treated as supported runtime launch targets yet.

## Runtime Shape

LaunchDeck now runs as two local Rust processes:

- Rust host on the UI/API port
- follow daemon on the follow-daemon port

Current boundary:

- `GET /`, `/app.js`, `/styles.css`, root image assets, and `/uploads/*` are served directly by the Rust host.
- Browser-facing `/api/*` routes call Rust internals directly rather than proxying through a Node bridge.
- `/engine/*` routes remain available on the same process for engine-oriented tooling and compatibility.
- Follow jobs are reserved and armed by the Rust host, then executed by the follow daemon.
- Existing `.local/launchdeck` files remain the source of persistent UI settings, uploads, send reports, and follow-daemon state.

The runtime also maintains a local lookup table cache under `.local/launchdeck/lookup-tables.json`.

## Follow-Daemon Responsibilities

The follow daemon is responsible for:

- running websocket-backed slot, signature, and market watchers
- executing delayed sniper buys, automatic dev sells, and sniper sells
- maintaining action/job state independently from the main request lifecycle
- persisting watcher health, follow telemetry samples, and timing profiles

Same-time sniper buys are still compiled by the Rust host because they must be submitted alongside the launch path itself.

## Current Performance Model

The hot path now relies on:

- background metadata pre-upload from the browser
- warmed and persisted default lookup tables
- cached blockhash refresh in the backend
- cached Pump global state for dev-buy compile paths
- arm-time pre-resolution of delayed follow-buy state
- daemon-side hot runtime refresh for delayed follow buys
- concurrent same-time compile and non-bundle submit where safe
- benchmark reports that separate `preRequest`, `backendTotal`, compile breakdowns, and send breakdowns

Delayed follow buys now use a split model:

- `prepare`: wallet- and launch-specific static state is cached at job-arm time
- `finalize`: fresh quote/blockhash/signing happens close to action fire time

## Lookup Table Strategy

Pump transaction assembly does not blindly force one static transaction format.

Current runtime behavior:

- the engine loads curated default lookup tables for known Pump launch and follow-up flows
- lookup tables are warmed on page load, cached in memory, and persisted locally under `.local/launchdeck/lookup-tables.json`
- compile measures candidate transaction sizes and prefers the smallest valid versioned shape for the current instruction set
- reports expose the chosen format and size diagnostics so near-limit transactions are visible during testing

In practice this means:

- `v0-alt` is used when lookup tables materially reduce packet size
- plain `v0` remains available when lookup tables are not beneficial or not required for that transaction shape
- different transactions in the same launch flow can use different lookup-table sets depending on whether they are the launch tx, follow-up tx, or tip-only tx

## Frontend Structure

The browser UI has also been split into feature-oriented modules rather than keeping all behavior in one file.

Current extracted UI feature modules include:

- `ui/sniper-feature.js`
- `ui/auto-sell-feature.js`
- `ui/images-feature.js`
- `ui/reports-feature.js`
- `ui/request-utils.js`
- `ui/render-utils.js`

This keeps the Rust runtime architecture unchanged while making the browser-side state and event wiring easier to evolve.
