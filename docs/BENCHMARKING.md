# RPC + WebSocket Benchmarking

This page explains how to benchmark Solana HTTP RPC and websocket endpoints from the same machine where you actually run LaunchDeck.

For copy-paste commands, use `Benchmarking/README.md`.

## What to benchmark first

Benchmark the setup you actually plan to run:

- `SOLANA_RPC_URL`: Helius Gatekeeper HTTP
- `SOLANA_WS_URL`: Helius standard websocket
- `LAUNCHDECK_WARM_RPC_URL`: Shyft

Example values:

```bash
SOLANA_RPC_URL=https://beta.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
SOLANA_WS_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
LAUNCHDECK_WARM_RPC_URL=https://rpc.fra.shyft.to?api_key=YOUR_SHYFT_API_KEY
```

Put your Helius key immediately after `api-key=`. Put your Shyft key immediately after `api_key=`.

## Script location

- script: `Benchmarking/Benchmark.js`
- npm script: `npm run ws-bench`
- saved output: `.local/launchdeck/rpc-ws-bench/`

Saved output includes:

- a markdown summary
- a JSON file with full timings and sample arrays

`.local/` is gitignored, so benchmark output does not get committed.

## What the benchmark measures

### HTTP

The HTTP section measures `getMultipleAccounts` in two ways:

- cold HTTP: a new TCP/TLS connection per sample
- warm HTTP: a reused keep-alive connection

This helps separate connection setup cost from normal steady-state request cost.

### WebSocket

The websocket section measures:

- connection handshake timing
- `slotSubscribe` ack timing
- `accountSubscribe` ack timing
- `slotSubscribe` first-notification timing

For Helius unified websocket hosts, it also measures:

- `transactionSubscribe` ack timing
- `transactionSubscribe` first-notification timing

## Why this matters for LaunchDeck

LaunchDeck does not use one endpoint for everything.

In practice:

- HTTP RPC timing helps you choose `SOLANA_RPC_URL`
- websocket timing helps you choose `SOLANA_WS_URL`
- warm-path benchmarking helps you decide whether a separate `LAUNCHDECK_WARM_RPC_URL` is worth it

That is why the recommended split is:

- Helius Gatekeeper HTTP for main RPC
- Helius standard websocket for watchers
- Shyft for warm/cache/block-height traffic

## How to compare fairly

Benchmarking is only useful when the comparison is fair.

Use these rules:

1. run the benchmark from the same machine or VPS where you actually run LaunchDeck
2. compare endpoints in the same region when possible
3. benchmark the exact URL shapes you plan to put in `.env`
4. compare HTTP against HTTP and websocket against websocket before drawing conclusions
5. use pacing if a provider has strict rate limits

## Helius note

If you are comparing Helius variants, `--helius-both` is the quickest way to compare Helius standard and Gatekeeper from the same machine with the same API key.

If you plan to run multiple snipes or watcher-heavy follow automation, Helius dev tier is strongly recommended. Lower tiers are more likely to rate-limit both live traffic and benchmark traffic.

## Reading the results

Useful patterns:

- lower warm HTTP numbers usually matter more than cold numbers once the app is already running
- tighter latency spread usually matters as much as raw minimum latency
- websocket subscribe acks and first-notification timing matter more for watcher quality than plain HTTP numbers
- Shyft is often a strong warm-path choice even when it is not the main execution RPC

## Common uses

Use the benchmark when you want to:

- compare Helius standard vs Gatekeeper
- compare multiple candidate websocket endpoints
- verify whether a separate warm RPC is worth using
- test a VPS region before locking it into production

## Related docs

- `Benchmarking/README.md`
- `docs/CONFIG.md`
- `docs/ENV_REFERENCE.md`

