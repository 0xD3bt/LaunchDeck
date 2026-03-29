# Execution Provider Plan

This is the merged in-repo execution design for `LaunchDeck`.

## Product Model

The standalone product supports:

- launchpads: `pump`, `bonk`, `bagsapp`
- providers: `helius-sender`, `standard-rpc`, `jito-bundle`
- policies: `fast`, `safe`
- strategies: `none`, `dev-buy`, `snipe-own-launch`, `automatic-dev-sell`

## Architecture

The code is split into:

- `rust/launchdeck-engine`: native UI/API host, config normalization, transaction assembly, simulation, send path, image/report/settings persistence, vamp import, and runtime workers
- `rust/launchdeck-engine/src/bin/launchdeck-cli.rs`: native CLI for build/simulate/send from config files
- `providers/provider-adapters.js`: provider routing and execution-class decisions
- `launchpads/`: launchpad registry and launchpad capability metadata
- `strategies/`: post-launch strategy metadata

The browser UI now talks directly to the Rust host on the local UI port.

The browser also pre-uploads launch metadata when possible so deploy-time latency is not dominated by metadata upload on every click.

## Provider Intent

- `helius-sender`: recommended low-latency send path with strict Sender requirements
- `standard-rpc`: explicit standard Solana RPC send path
- `jito-bundle`: explicit bundle-oriented Jito send path

Providers with multiple documented endpoint groups can also expose endpoint profiles such as `Global`, `US`, `EU`, `West`, and `Asia`.

## Engine-Owned Shaping

The UI captures user intent, but the engine owns final transport shaping.

Examples:

- `Helius Sender` hard-requires inline tip, inline priority fee, `skipPreflight=true`, and `maxRetries=0`.
- `Standard RPC` does not use tip.
- `Jito Bundle` creation can accept both tip and priority in the UI, but the engine may intentionally drop creation priority for multi-transaction launch flows.

## Launchpad Rules

### Pump

- current verified native execution path
- LaunchDeck launch modes are built by the Rust engine rather than the legacy JS planner
- verified native coverage currently includes `regular`, `cashback`, `agent-custom`, `agent-unlocked`, and `agent-locked`
- default lookup tables are warmed on app load, cached locally, and reused in the compile path
- blockhash and Pump global state are cached in the Rust runtime to reduce compile latency
- benchmark reports expose `total`, `backendTotal`, `preRequest`, compile sub-timings, and send sub-timings

### Bonk

- not active in the current initial version
- should not be treated as a supported launch flow yet
- future implementation should use official Raydium SDK surfaces
- community repos are reference-only, not implementation sources

### Bagsapp

- not active in the current initial version
- should not be treated as a supported launch flow yet
- future implementation should use official Bags SDK/docs
- creator BPS must be explicit
- total fee-claimer BPS must equal `10000`
- LUT-aware config creation may be required for larger fee-claimer sets

## Provider Availability

The UI learns provider and launchpad support state from backend status before deploy.

The backend reports:

- `available`
- `verified`
- `supportState`
- `reason`
- bundle/sequential/single support flags

## Current Runtime Notes

- Pump is the only active launch flow in the code today.
- `Helius Sender`, `Standard RPC`, and `Jito Bundle` are the current explicit provider choices.
- `Standard RPC` and `Helius Sender` keep dependent launch/follow-up flows sequential.
- `Jito Bundle` owns the current bundle transport path.
- metadata upload provider is configurable:
  - default: `pump-fun`
  - optional custom provider: `pinata`
  - Pinata reuses uploaded image CIDs across metadata-only edits within the current app session
- Bonk and Bags may appear in model/planning surfaces, but they are not active launch builders in the current initial version.

## Documentation Pointers

- [`README.md`](../README.md)
- [`ARCHITECTURE.md`](ARCHITECTURE.md)
- [`CONFIG.md`](CONFIG.md)
- [`PROVIDERS.md`](PROVIDERS.md)
- [`LAUNCHPADS.md`](LAUNCHPADS.md)
- [`STRATEGIES.md`](STRATEGIES.md)
