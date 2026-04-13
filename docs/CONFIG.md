# Configuration

This page is the main setup and configuration guide for LaunchDeck.

If you are setting LaunchDeck up for the first time:

1. start with `.env.example`
2. fill only the values already listed there
3. leave the advanced defaults alone unless you have a specific reason to change them

If you want the full variable list and defaults, use `docs/ENV_REFERENCE.md`.

If you are provisioning a brand-new server first, use `docs/VPS_SETUP.md` before this page. That guide covers the current Ubuntu/VPS bootstrap flow, required system dependencies, SSH-tunnel access pattern, and the `scripts/vps-bootstrap.sh` installer.

## Recommended First Setup

For most operators, the current recommended production stack is:

- run LaunchDeck on a VPS rather than on a normal everyday workstation
- place the VPS near the provider endpoints and RPCs you actually plan to use
- EU VPS location: Frankfurt or Amsterdam
- US VPS location: New York / Newark area or Salt Lake City area
- Asia VPS location: Singapore or Tokyo
- Helius dev tier for the main stack
- Helius Gatekeeper HTTP for `SOLANA_RPC_URL`
- Helius standard websocket for `SOLANA_WS_URL`
- Shyft free tier for `LAUNCHDECK_WARM_RPC_URL`
- `helius-sender` or `hellomoon` as the execution provider

Why this is the default recommendation:

- Helius Gatekeeper HTTP benchmarked best for the main HTTP RPC path
- Helius standard websocket benchmarked best for the watcher websocket path
- Helius dev tier gives much better watcher quality and overall runtime behavior than a bare-minimum free setup
- Shyft is fast, cheap, and a good fit for warm/cache/block-height traffic

Current recommended `.env` shape:

```bash
SOLANA_RPC_URL=https://beta.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
SOLANA_WS_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
LAUNCHDECK_WARM_RPC_URL=https://rpc.fra.shyft.to?api_key=YOUR_SHYFT_API_KEY
USER_REGION=eu
```

Put your Helius key immediately after `api-key=`. Put your Shyft key immediately after `api_key=`.

## What To Set First

For most operators, `.env.example` is enough.

Those are the values we suggest setting first:

- `SOLANA_PRIVATE_KEY` or your `SOLANA_PRIVATE_KEY*` wallet set
- `SOLANA_RPC_URL`
- `SOLANA_WS_URL`
- `USER_REGION`
- `LAUNCHDECK_WARM_RPC_URL`

Optional but common:

- `HELLOMOON_API_KEY`
- `BAGS_API_KEY`
- `LAUNCHDECK_METADATA_UPLOAD_PROVIDER=pinata`
- `PINATA_JWT`
- `LAUNCHDECK_BENCHMARK_MODE`

You usually do not need to set these in a first setup:

- `HELIUS_RPC_URL`
- `HELIUS_WS_URL`
- warm toggles
- helper worker toggles
- daemon ports
- capacity limits
- path overrides
- explicit endpoint overrides

Those already have runtime defaults or only matter for special cases.

## What Is Already Enabled

LaunchDeck already defaults to the setup we recommend in most cases.

These are already on or already set to the shipped defaults:

- startup warm
- continuous warm
- idle warm suspend
- Helius `transactionSubscribe` probe/fallback behavior
- Bonk helper worker
- Bags helper worker
- benchmark mode `full`
- Helius priority level `high`
- Jito percentile `p99`
- wallet refresh `30000ms`
- main host port `8789`
- follow daemon port `8790`

In practice, the easiest setup is to leave those alone.

## Runtime Model

LaunchDeck separates:

- execution transport
- read/confirm RPC
- watcher websocket
- warm/block-height RPC

That split is important.

In a normal setup:

- `execution.provider` decides how creation, buy, and sell transactions are sent
- `SOLANA_RPC_URL` is used for reads, confirmations, and general runtime RPC behavior
- `SOLANA_WS_URL` is used for realtime watchers
- `LAUNCHDECK_WARM_RPC_URL` is used for startup warm, continuous warm probes, and block-height observation

Those are related, but they are not the same path.

### Warmup and keep-alive

LaunchDeck currently uses:

- startup warm
- continuous warm
- idle warm suspend

How that works:

- startup warm runs once at startup
- continuous warm keeps active execution paths and watcher paths hot while the app is in use
- idle warm suspend pauses that background warm traffic when the app is idle
- changing routes in settings triggers an immediate rewarm of the new effective paths
- saving unchanged routes does not force a needless rewarm

What gets warmed:

- your warm RPC path
- fee-market snapshots
- Helius Sender endpoints when they are active
- Hello Moon QUIC or Hello Moon bundle endpoints, depending on the active Hello Moon mode
- Jito endpoints when they are active
- the watcher websocket path

Watcher websocket warm is driven by the configured websocket path, not by per-provider endpoint fanout.

## Helius RPC and WS overrides

You only need these when your main `SOLANA_RPC_URL` / `SOLANA_WS_URL` are not already the Helius values you want.

- `HELIUS_RPC_URL`
  Override-only. Use this when your main `SOLANA_RPC_URL` is not Helius but you still want Helius priority-fee estimates.
- `HELIUS_WS_URL`
  Override-only. Use this when your main `SOLANA_WS_URL` is not Helius or when you intentionally want a separate Helius watcher path.

If your normal `SOLANA_RPC_URL` and `SOLANA_WS_URL` already point to Helius, LaunchDeck picks that up automatically.

## Helius transactionSubscribe

`LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE` is default-on when blank.

That means:

- if the watcher websocket is Helius, LaunchDeck probes the Helius `transactionSubscribe` path automatically
- if it works, LaunchDeck uses it
- if it does not work, LaunchDeck falls back to standard websocket watchers automatically

You usually do not need to touch this setting at all.

## USER_REGION and endpoint profiles

`USER_REGION` is the shared default routing profile for region-aware providers.

Supported groups:

- `global`
- `us`
- `eu`
- `asia`

Supported metro tokens:

- `slc`
- `ewr`
- `lon`
- `fra`
- `ams`
- `sg`
- `tyo`

Recommended practical usage:

- `eu` fans out across Amsterdam and Frankfurt
- `asia` fans out across Singapore and Tokyo on Helius Sender
- `us` fans out across Salt Lake City and Newark on Helius Sender
- if those grouped metros are far apart, place your server in one of them and use the exact metro token when you want to stay pinned there

Provider-specific notes:

- Helius Sender supports exact metro routing where those metros exist
- Hello Moon maps unsupported metros onto the closest Hello Moon endpoints it actually exposes
- Hello Moon `asia` and `sg` currently use Tokyo
- Hello Moon `lon` currently uses the EU pair: Frankfurt + Amsterdam
- Hello Moon `us`, `slc`, and `ewr` currently use the US pair: New York + Ashburn

Use provider-specific overrides only when one provider needs a different region than your shared default:

- `USER_REGION_HELIUS_SENDER`
- `USER_REGION_HELLOMOON`
- `USER_REGION_JITO_BUNDLE`

## Provider recommendations

For most operators:

- start with `helius-sender`
- use `hellomoon` when you want a strong alternate low-latency path
- use `standard-rpc` when you want explicit plain-RPC transport behavior
- use `jito-bundle` when you explicitly want bundle semantics

Provider details and endpoint catalogs live in `docs/PROVIDERS.md`.

## Metadata upload

LaunchDeck supports:

- `pump-fun`
- `pinata`

Default behavior:

- blank `LAUNCHDECK_METADATA_UPLOAD_PROVIDER` means `pump-fun`
- if `pinata` is selected, `PINATA_JWT` is required
- if Pinata upload fails, LaunchDeck automatically falls back to `pump-fun`
- the UI now surfaces that fallback to the user instead of silently hiding it

## Helper workers

Bonk and Bags helper workers now default on:

- `LAUNCHDECK_ENABLE_BAGS_HELPER_WORKER`
- `LAUNCHDECK_ENABLE_BONK_HELPER_WORKER`

That is the recommended default.

Use `false` only if you intentionally want one-shot helper execution instead of a long-lived helper process.

## Benchmarking and auto-fee

Current shipped defaults:

- `LAUNCHDECK_BENCHMARK_MODE=full` when blank
- `LAUNCHDECK_AUTO_FEE_HELIUS_PRIORITY_LEVEL=high` when blank
- `LAUNCHDECK_AUTO_FEE_JITO_TIP_PERCENTILE=p99` when blank
- `LAUNCHDECK_HELIUS_PRIORITY_REFRESH_INTERVAL_MS=6000` when blank

Practical default:

- keep `high + p99`
- use a Max Auto Fee in the UI if you want a hard cost cap

Benchmarking docs:

- `docs/BENCHMARKING.md`
- `Benchmarking/README.md`

## Host and daemon defaults

Default local ports:

- main host: `8789`
- follow daemon: `8790`

Default follow transport:

- `local-http`

You usually do not need to change those unless:

- another process already uses the port
- you are intentionally wiring a non-default deployment

## Local persistence

By default, LaunchDeck writes local state under `.local/launchdeck`, with the engine runtime file at `.local/engine-runtime.json`.

Common outputs:

- app config
- image library
- lookup tables
- uploads
- send reports
- follow-daemon state
- engine runtime state

## Settings saves and route changes

The settings modal saves your operator defaults and active presets locally.

Important runtime behavior:

- changing the effective execution routes causes an immediate rewarm of the new paths
- saving unchanged routes does not restart the current warm schedule
- changing env vars still requires a runtime restart

Restart when you change:

- wallets
- RPC URLs
- websocket URLs
- region overrides
- metadata provider credentials
- provider integration keys

Use:

```bash
npm restart
```

## Full variable reference

For the complete list of supported environment variables, effective defaults, and override behavior, use:

- `docs/ENV_REFERENCE.md`
- `.env.advanced`
