# Benchmarking (`Benchmark.js`)

RPC + WebSocket latency checks from your machine. Run these from the **repository root** so `node_modules` and `.local/` paths resolve correctly.

## Setup

```bash
cd /path/to/LaunchDeck
npm install
```

The script uses the **`ws`** package (devDependency).

## Quick start

Compare two websocket endpoints (HTTP cold/warm `getMultipleAccounts` + WebSocket are run for each `wss://` URL):

```bash
npm run ws-bench -- "wss://your-first-endpoint?api_key=YOUR_KEY" "wss://your-second-endpoint?api-key=YOUR_KEY"
```

Same thing without npm:

```bash
node Benchmarking/Benchmark.js "wss://your-first-endpoint?api_key=YOUR_KEY" "wss://your-second-endpoint?api-key=YOUR_KEY"
```

**Note:** The `--` after `npm run ws-bench` is required so npm passes your URLs and flags to the script.

## Recommended LaunchDeck comparison

Recommended LaunchDeck stack:

- HTTP / RPC reads and confirms: Helius Gatekeeper
- WebSocket watchers: Helius standard websocket
- Warmup RPC: Shyft

Example URLs:

```bash
https://beta.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
https://rpc.fra.shyft.to?api_key=YOUR_SHYFT_API_KEY
```

Put your Helius key immediately after `api-key=`. Put your Shyft key immediately after `api_key=`.

## Use URLs from `.env`

After loading env vars into your shell (however you normally do it), or with `dotenv` tooling:

```bash
node Benchmarking/Benchmark.js --from-env
```

This appends **`SOLANA_WS_URL`** then **`HELIUS_WS_URL`** when they are non-empty. You can mix CLI URLs first, then env:

```bash
node Benchmarking/Benchmark.js "wss://extra.example/ws" --from-env
```

## Helius standard vs Gatekeeper (`--helius-both`)

When you pass a Helius unified endpoint (`mainnet.helius-rpc.com` or `beta.helius-rpc.com`), this expands it into **both** hosts with the same path and query string so you can compare standard Helius vs Gatekeeper in one run:

```bash
npm run ws-bench -- --helius-both "wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
node Benchmarking/Benchmark.js --helius-both "https://beta.helius-rpc.com/?api-key=YOUR_KEY"
```

This works for both **WebSocket** and **HTTP** inputs. Non-Helius URLs are left unchanged.

This is the easiest way to compare Helius standard vs Gatekeeper from the same machine and same API key before you lock the values into `.env`.

## HTTP-only (JSON-RPC `getMultipleAccounts`)

Cold connection per request vs one keep-alive agent, using a fixed reusable 3-account basket sourced from recent LaunchDeck launch-history mints:

```bash
npm run ws-bench -- --rpc-only "https://beta.helius-rpc.com/?api-key=YOUR_KEY"
```

Or any HTTPS Solana RPC:

```bash
node Benchmarking/Benchmark.js --rpc-only "https://api.mainnet-beta.solana.com"
```

## WebSocket-only (`slotSubscribe` ack + first event, `accountSubscribe` ack, plus Helius `transactionSubscribe` ack + first event)

```bash
npm run ws-bench -- --ws-only "wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
```

For LaunchDeck’s recommended watcher setup, benchmark the standard Helius websocket and then enable `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true` in your `.env` if your Helius tier supports it.

For Helius unified hosts (`mainnet.helius-rpc.com` and `beta.helius-rpc.com`), the WebSocket benchmark now records both:

- standard **`slotSubscribe`** ack latency
- standard **`accountSubscribe`** ack latency rotating through the LaunchDeck-history account basket
- standard **`slotSubscribe`** first-notification wait on the same live socket
- Helius-style **`transactionSubscribe`** ack latency using a filtered request with `accountInclude=11111111111111111111111111111111`, `failed=false`, `vote=false`, `commitment=processed`, `encoding=jsonParsed`, and `transactionDetails=none`
- Helius-style **`transactionSubscribe`** first-notification wait on the same live socket

## Longer runs (`--preset`)

Use a preset for more warmup + more timed samples **per metric** (HTTP cold, HTTP warm, and WebSocket each use **`--samples`** iterations). **HTTP cold** opens a **new TCP+TLS connection every sample**, so `extended` can take a long time and many RPC quota units.

```bash
npm run ws-bench -- --preset standard "wss://..." "wss://..."
npm run ws-bench -- --preset long "wss://..."
npm run ws-bench -- --preset extended "wss://..."
```

| Preset | `--warmup` | `--samples` (each metric) |
|--------|------------|---------------------------|
| `quick` (default if omitted) | 10 | 80 |
| `standard` | 25 | 200 |
| `long` | 50 | 400 |
| `extended` | 75 | 800 |

Mix preset with explicit overrides (only the fields you pass are overridden):

```bash
npm run ws-bench -- --preset long --samples 250 "wss://..."
```

## Tune warmup and sample count manually

```bash
npm run ws-bench -- --warmup 15 --samples 100 "wss://..." "wss://..."
```

## Respect low provider rate limits

If a provider has a low request cap, add per-request pacing. For a **10 RPS** limit such as Shyft, use:

```bash
npm run ws-bench -- --max-rps 10 "wss://rpc.fra.shyft.to/?api_key=YOUR_KEY"
```

Equivalent explicit gap:

```bash
node Benchmarking/Benchmark.js --request-gap-ms 100 "wss://rpc.fra.shyft.to/?api_key=YOUR_KEY"
```

If you use Helius free / lower tiers and plan to run multiple sniper wallets or heavy follow automation, expect more rate-limit pressure than on Helius dev tier. Benchmarking does not remove those runtime plan limits.

| Flag | Default | Meaning |
|------|---------|---------|
| `--preset` | — | `quick` \| `standard` \| `long` \| `extended` |
| `--warmup` | 10 | Cycles before timing (HTTP warm path + WebSocket) |
| `--samples` | 80 | Timed samples **per** metric (HTTP cold, HTTP warm, WS each) |
| `--pause-ms` | 1000 | Pause between multiple URLs |
| `--request-gap-ms` | 0 | Delay between individual warmup/timed requests or subscribe cycles |
| `--max-rps` | — | Convenience pacing flag; `10` means ~`100ms` gap between requests |
| `--helius-both` | off | Expand Helius unified URLs into both `mainnet.helius-rpc.com` and `beta.helius-rpc.com` |

## Save / no save

By default, each run writes two files under `./.local/launchdeck/rpc-ws-bench/`:

- **`run-<timestamp>.md`** — screenshot-friendly markdown summary with setup, rate limit / pacing, and summary tables.
- **`run-<timestamp>.json`** — full report plus per-sample **`samplesMs`** arrays for spreadsheets or scripts.

Query strings are redacted as `?…` in saved reports.

```bash
node Benchmarking/Benchmark.js --no-save "wss://..."
```

## Help

```bash
npm run ws-bench -- --help
node Benchmarking/Benchmark.js --help
```

## More detail

See **`docs/BENCHMARKING.md`** for what each metric means and how to interpret results fairly (region, cold vs warm, `slotSubscribe` vs `transactionSubscribe`, etc.).
