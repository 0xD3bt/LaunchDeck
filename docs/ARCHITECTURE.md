# Architecture

This document explains how LaunchDeck is organized today from an operator perspective: what runs locally, what each process is responsible for, how a launch request moves through the system, and where local state is stored.

## High-Level Shape

LaunchDeck is a local multi-process application with three main layers:

- `ui/launchdeck/`: the current browser application shell for form entry, presets, settings, image management, dashboard/reporting, runtime status, and follow-action configuration
- `rust/launchdeck-engine`: the main Rust host that serves the UI and owns config normalization, launch execution, reporting, and local persistence
- `rust/launchdeck-engine/src/bin/launchdeck-follow-daemon.rs`: a separate Rust daemon for follow actions that need to keep running after the main launch request has been submitted

Archived pre-shell UI and legacy helpers live under `Legacy/`. They are not the current frontend/runtime path.

There is also an operator CLI:

- `rust/launchdeck-engine/src/bin/launchdeck-cli.rs`

That CLI uses the same normalization, launchpad dispatch, transport, and reporting stack as the UI host.

## Process Model

LaunchDeck normally runs as two local Rust processes:

- the main host on `LAUNCHDECK_PORT`, default `8789`
- the follow daemon on `LAUNCHDECK_FOLLOW_DAEMON_PORT`, default `8790`

### Main Host Responsibilities

The main host is responsible for:

- serving `GET /` and the static browser assets
- serving browser-facing `/api/*` routes
- serving internal `/engine/*` routes
- serving uploaded files under `/uploads/*`
- reading and writing persisted app settings
- loading wallet inventory from `SOLANA_PRIVATE_KEY*`
- normalizing launch configs from the UI
- choosing provider-aware transport behavior
- building, simulating, and sending launch transactions
- reserving and arming follow jobs with the daemon
- writing durable reports
- exposing runtime warm telemetry and operator activity endpoints
- exposing the image library, report browser, and Bags identity control plane

### Follow Daemon Responsibilities

The follow daemon is responsible for:

- accepting reserved and armed follow jobs
- maintaining websocket-backed watchers for slots, signatures, and market conditions
- executing delayed sniper buys
- executing automatic dev sell actions
- executing snipe sells
- tracking job state outside the main request lifecycle
- persisting follow telemetry, watcher health, and timing profiles

Same-time sniper buys are compiled and submitted by the main host, not the daemon, because they must land alongside the launch path itself.

## Request Flow

A normal UI-driven launch follows this path:

1. the browser loads bootstrap and settings data from the Rust host
2. the operator fills token metadata, launch settings, provider settings, and optional follow actions
3. the browser pre-uploads metadata when possible and sends the request to the main host
4. the host converts UI state into the raw config shape and normalizes it
5. the host validates launchpad rules, provider rules, and follow-action rules
6. the host builds launchpad-specific transactions and a provider-specific transport plan
7. the host simulates and/or sends the launch flow
8. if follow behavior is enabled, the host reserves and then arms a follow job with launch-specific context
9. the follow daemon takes over delayed and watcher-driven actions
10. reports are persisted for later review in Dashboard

## Launchpad Boundaries

### Pump

Pump is the most native path in the current runtime.

- launch assembly is native Rust
- transaction shaping is Rust-owned
- reporting is Rust-owned
- follow integration is Rust-owned

Verified Pump modes:

- `regular`
- `cashback`
- `agent-custom`
- `agent-unlocked`
- `agent-locked`

Verified Pump support also includes:

- immediate dev buy
- same-time sniper buys
- delayed sniper buys
- snipe sells
- automatic dev sell
- compile-time and daemon-side follow handling for agent-mode creator-vault transitions

### Bonk

Bonk is Rust-orchestrated but not fully Rust-assembled.

- validation is Rust-owned
- transport planning is Rust-owned
- reporting is Rust-owned
- follow integration is Rust-owned
- launch assembly uses the Raydium LaunchLab-backed helper bridge

Verified Bonk support includes:

- `regular`
- `bonkers`
- `sol`
- `usd1`
- immediate dev buy
- same-time sniper buys
- delayed sniper buys
- snipe sells
- automatic dev sell

### Bagsapp

Bagsapp is a supported launchpad path when configured.

- availability depends on Bags credentials
- launch/trade assembly uses the hosted Bags API or SDK bridge
- Rust still owns normalization, transport planning, reporting, and the UI integration layer
- current operator support includes `bags-2-2`, `bags-025-1`, `bags-1-025`, `sol`, immediate dev buy, same-time sniper buys, delayed sniper buys, snipe sells, and automatic dev sell

## Provider And Transport Layer

LaunchDeck uses explicit provider choices rather than hidden provider fallback.

Current providers:

- `helius-sender`
- `hellomoon`
- `standard-rpc`
- `jito-bundle`

From those providers, the engine resolves a transport class:

- `single`
- `sequential`
- `bundle`

The selected provider controls:

- whether tip is used
- whether priority fee is required or optional
- whether `skipPreflight` must be forced
- whether sends are standard sequential sends or bundle sends
- whether endpoint profiles are available
- which endpoint group is used

The browser expresses intent, but the engine owns final transaction shaping.

Examples:

- `standard-rpc` ignores tip
- `helius-sender` hard-fails if Sender requirements are not satisfied
- `hellomoon` hard-fails if QUIC requirements are not satisfied
- `jito-bundle` may drop creation priority in some multi-transaction launch flows

## Engine-Owned Decisions

The engine, not the UI, decides:

- validation of supported launchpad and mode combinations
- provider compatibility checks
- transaction format choice such as `legacy`, `v0`, or `v0-alt`
- address lookup table usage
- metadata upload fallback behavior
- same-time fee safeguards
- delayed snipe-buy prepare versus finalize timing

This is why the same saved preset can produce different final wire behavior depending on provider, transaction count, and launch shape.

## Persistence Model

By default LaunchDeck stores operator data under `.local/launchdeck`:

- `app-config.json`
- `image-library.json`
- `lookup-tables.json`
- `follow-daemon-state.json`
- `uploads/`
- `send-reports/`

Other runtime state lives here:

- `.local/engine-runtime.json` for host runtime worker state by default

The important point for operators is that settings, images, uploads, and historical reports survive restarts unless you explicitly remove the local data directory.

## Performance-Oriented Runtime Features

The current runtime uses several caching and warm-up paths to reduce launch latency:

- startup launchpad warm for lookup tables, Pump global state, Bonk state, and Bags helper state
- background metadata pre-upload from the browser
- warmed lookup tables cached in memory and persisted locally
- cached blockhash refresh in the host and daemon
- cached Pump global state for compile-time launch assembly and dev-buy quoting
- arm-time preparation of delayed sniper buys
- daemon-side hot-state refresh for delayed follow jobs
- runtime-status and warm-target telemetry surfaced back to the UI
- concurrency for same-time compile and non-bundle submit paths that preserve launch ordering

Reports separate user-visible wait from backend execution so operators can tell whether latency came from metadata, compile, or chain confirmation.

## Frontend Module Layout

The UI is still one browser app, but it is now organized as a shell plus feature/domain modules:

- `ui/launchdeck/index.html`: current shell and modal/layout markup
- `ui/launchdeck/app.js`: main app composition and shared state
- `ui/launchdeck/sniper-feature.js`: sniper editor behavior
- `ui/launchdeck/auto-sell-feature.js`: dev auto-sell and sniper autosell behavior
- `ui/launchdeck/images-feature.js`: image-library rendering and actions
- `ui/launchdeck/reports-feature.js`: dashboard/report terminal behavior
- `ui/launchdeck/app/settings-domain.js`: settings modal and preset routing defaults
- `ui/launchdeck/app/wallet-runtime-domain.js`: wallet refresh, runtime status, warm indicators, and follow status chrome
- `ui/launchdeck/app/reports-history.js` and `ui/launchdeck/app/reports-presenters.js`: launch-history assembly plus dashboard presentation

This is mainly useful to know when you are trying to understand which part of the app owns a feature such as snipers, auto-sell, image library behavior, dashboard rendering, or runtime indicators.
