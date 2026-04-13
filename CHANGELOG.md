# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fixed migrated Bonk Raydium pool decoding so market snapshots accept both the older flat `data: [...]` payload and the newer nested `data: { data: [...] }` response shape returned by Raydium.
- Fixed migrated Bonk market-cap math to avoid `u128` overflow when converting Raydium pool prices into SOL-denominated market caps, restoring correct large-supply migrated-token results.
- Changed SOL/USD normalization for market-cap tracking to prefer Helius asset pricing and fall back to the configured HTTP source, removing the older on-chain fallback path from the active pricing flow.
- Fixed `pump` and `bagsapp` market-cap websocket watching so migration-capable launches keep recomputing from the live post-migration market source instead of staying pinned to only the pre-migration watcher target.
- Fixed Pump sell confirmation parsing on the Helius `transactionSubscribe` path so follow sells request full transaction details and treat both structured websocket errors and log-derived `creator_vault` / `ConstraintSeeds` / `Custom 2006` failures as terminal websocket failures.
- Fixed Pump `creator_vault` mismatch recovery so the special `Custom 2006` rebuild/retry paths now cover both pre-signed sniper buys and Pump follow sells with default-on env toggles to disable either retry path when desired.

## [0.1.0] - 2026-04-13

### Added

- New LaunchDeck browser shell under `ui/launchdeck/` with image assets under `ui/images/`.
- Image-library workflow for uploading, reusing, and organizing images with persisted local metadata.
- `Vamp` import workflow for seeding token metadata and images from an existing mint.
- `Dashboard`-based reporting flow covering persisted `Transactions`, `Launches`, `Jobs`, and `Logs`.
- Runtime status indicators in the UI for warm health, follow-daemon health, and active operator state.
- Rust runtime modules for launchpad execution and warm-state handling: `launchpad_runtime`, `launchpad_warm`, and `warm_manager`.
- Expanded native Rust Bags runtime coverage for launch compilation, quoting, follow actions, market snapshots, import-context detection, and reporting.
- Canonical Bags market/import recovery paths that can detect local Meteora Dynamic Bonding Curve and post-migration Meteora DAMM v2 markets from RPC state.
- Startup launchpad warm flows, warm-target telemetry, and active/idle warm lifecycle handling surfaced back to the UI/runtime layer.
- Warmed lookup-table and launchpad-state handling with local persistence and cached blockhash priming across the host/runtime path.
- Market-cap-based follow actions and the related Helius-first SOL/USD price lookup with HTTP fallback configuration path.
- Expanded operator documentation for VPS provisioning, dependency installation, bootstrap flow, and first-run validation.

### Changed

- Reworked onboarding docs so the default recommendation is a VPS-first setup, including Windows, Linux, and fresh-VPS setup paths.
- Updated setup guidance with region placement advice, explicit dependency baselines, SSH-tunnel usage, and practical VPS notes for testing as well as production.
- Updated README and setup docs to recommend placing the VPS near the provider endpoints and RPCs you actually plan to use, with explicit EU, US, and Asia guidance.
- Updated README and VPS docs to call out the worked Vultr example, referral link, and the practical note that other providers are also fine.
- Updated documentation to reflect the current UI shell, `Dashboard` terminology, runtime warm behavior, and the current launchpad support model.
- Updated follow-daemon docs to explain delayed, confirmed-slot, and market-cap trigger modes plus watcher ownership boundaries.
- Updated environment docs for launchpad warm settings, Bags setup settings, warm probe controls, and market-cap price-source variables.
- Updated wallet configuration docs to clarify that `SOLANA_PRIVATE_KEY<number>` supports open-ended numeric suffixes rather than a fixed wallet cap.
- Updated Bagsapp messaging across runtime and docs to reflect that it is supported when configured.
- Updated the Bags path so the shipped operator flow is explicitly wallet-only in the UI while preserving native follow, fee-split, and automation support.
- Updated Bags setup handling so prelaunch setup, fee-share preparation, and related transport-aware setup orchestration are owned more directly by the Rust runtime.
- Updated the runtime support-state payload so configured Bagsapp now reports as supported instead of unverified.
- Expanded Rust launchpad dispatch and orchestration so Pump, Bonk, and Bags now expose clearer runtime capabilities for compile, warm, quote, follow, and prelaunch-setup behavior.
- Expanded Bonk support to better document and surface the `USD1` quote-asset path, including top-up handling and atomic or split launch/buy behavior where required.
- Moved the live Bonk JS helper path to `scripts/bonk-launchpad.js` so `Legacy/` remains archive/reference code rather than a live runtime dependency.
- Cleaned `.gitignore` to remove stale project-specific entries while keeping standard local, build, and editor ignores.

### Fixed

- Added direct `bn.js` dependency coverage for the live Bonk helper instead of relying on transitive installation behavior.
- Restored the live UI image asset set under `ui/images/` so the new shell ships with its referenced marks and branding files.

### Removed

- Previous flat `ui/` app files in favor of the new `ui/launchdeck/` shell layout.
- Old `runtime-bench` package script.
- Old browser-matrix package scripts from the root package manifest.
- Unused `@pump-fun/pump-sdk` dependency from `package.json`.

### Notes

- `Legacy/` is kept in the repo as reference material only, and the live runtime no longer depends on it.
- This entry captures the current repository refresh and documentation pass; future work should be added under `Unreleased` until the next tagged version is cut.
