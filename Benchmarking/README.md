# Benchmarking (`Benchmark.js`)

This is the copy-paste command reference for the LaunchDeck RPC and websocket benchmark.

For what the numbers mean, use `docs/BENCHMARKING.md`.

## Setup

From the repo root:

```bash
cd /path/to/LaunchDeck
npm install
```

## Recommended first comparison

Benchmark the same stack LaunchDeck recommends first:

```bash
https://beta.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
https://rpc.fra.shyft.to?api_key=YOUR_SHYFT_API_KEY
```

Put your Helius key immediately after `api-key=`. Put your Shyft key immediately after `api_key=`.

## Quick start

Compare two websocket endpoints:

```bash
npm run ws-bench -- "wss://your-first-endpoint?api-key=YOUR_KEY" "wss://your-second-endpoint?api-key=YOUR_KEY"
```

Without npm:

```bash
node Benchmarking/Benchmark.js "wss://your-first-endpoint?api-key=YOUR_KEY" "wss://your-second-endpoint?api-key=YOUR_KEY"
```

The `--` after `npm run ws-bench` is required so npm forwards the arguments.

## Use values from `.env`

```bash
node Benchmarking/Benchmark.js --from-env
```

This appends:

- `SOLANA_WS_URL`
- `HELIUS_WS_URL`

when they are set.

## Useful commands

Compare Helius standard vs Gatekeeper:

```bash
npm run ws-bench -- --helius-both "wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
```

HTTP only:

```bash
npm run ws-bench -- --rpc-only "https://beta.helius-rpc.com/?api-key=YOUR_KEY"
```

Websocket only:

```bash
npm run ws-bench -- --ws-only "wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
```

Longer run:

```bash
npm run ws-bench -- --preset standard "wss://..."
```

Custom warmup and samples:

```bash
npm run ws-bench -- --warmup 15 --samples 100 "wss://..."
```

Respect low provider rate limits:

```bash
npm run ws-bench -- --max-rps 10 "wss://rpc.fra.shyft.to/?api_key=YOUR_KEY"
```

No saved output:

```bash
node Benchmarking/Benchmark.js --no-save "wss://..."
```

## Common flags

| Flag | Meaning |
| --- | --- |
| `--preset` | `quick`, `standard`, `long`, `extended` |
| `--warmup` | Warmup cycles before timing |
| `--samples` | Timed samples per metric |
| `--pause-ms` | Pause between multiple URLs |
| `--request-gap-ms` | Delay between individual requests or subscribe cycles |
| `--max-rps` | Convenience pacing flag |
| `--helius-both` | Compare Helius standard and Gatekeeper |
| `--rpc-only` | HTTP only |
| `--ws-only` | websocket only |
| `--from-env` | Use env websocket URLs |
| `--no-save` | Print only, skip saved files |

## Saved output

By default, each run writes to:

- `.local/launchdeck/rpc-ws-bench/run-<timestamp>.md`
- `.local/launchdeck/rpc-ws-bench/run-<timestamp>.json`

Query strings are redacted in the saved output.

## Practical note

Run the benchmark from the same box where you actually run LaunchDeck. That matters more than getting pretty numbers from a different machine.

