# RPC + WebSocket benchmarking

How to benchmark Solana **HTTP JSON-RPC** and **WebSocket** endpoints from the same machine where you run LaunchDeck, and how to read the numbers.

**Copy-paste commands:** see **`Benchmarking/README.md`**.

Recommended LaunchDeck stack to benchmark first:

- `SOLANA_RPC_URL=https://beta.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY`
- `SOLANA_WS_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY`
- `LAUNCHDECK_WARM_RPC_URL=https://rpc.fra.shyft.to?api_key=YOUR_SHYFT_API_KEY`

Put your Helius key immediately after `api-key=`. Put your Shyft key immediately after `api_key=`.

## Script

- **Path:** `Benchmarking/Benchmark.js`
- **npm script:** `ws-bench` (requires `npm install` so the `ws` devDependency is present)
- **Saved output (default):** `./.local/launchdeck/rpc-ws-bench/run-<timestamp>.md` (markdown summary for sharing/screenshots) and `.json` (full data, including per-sample `samplesMs`). From the repo root. Query strings are **redacted** (`?…`). Use **`--no-save`** to print only.
- The **`.local/`** directory is listed in `.gitignore` so results stay out of git.

The script is organized into **separate functions** for each mode:

1. **HTTP cold** — `getMultipleAccounts` over `https://` (or `http://`), using a fixed reusable 3-account basket sourced from recent LaunchDeck launch-history mints, **`agent: false`** on every request so each call establishes a **new TCP + TLS** connection (full cold path per sample).
2. **HTTP warm** — same `getMultipleAccounts` request, one **`keepAlive: true`** agent, **`--warmup`** requests then **`--samples`** timed requests on the **reused** connection.
3. **WebSocket** — `slotSubscribe` JSON-RPC ack → `slotUnsubscribe`, `accountSubscribe` ack → `accountUnsubscribe` against the fixed LaunchDeck-history account basket, and `slotSubscribe` first-notification timing. For Helius unified hosts (`mainnet.helius-rpc.com` / `beta.helius-rpc.com`), the script also measures `transactionSubscribe` ack → `transactionUnsubscribe` plus first-notification timing.

For a **`wss://`** URL, the HTTP sections use the **same host and query string** with **`https://`** (or `ws://` → `http://`). That matches typical setups where RPC and WSS share one API key on the same domain. If your HTTP RPC base URL differs from the WebSocket URL, pass **`https://...`** as a separate CLI argument or use **`--rpc-only`** with explicit HTTPS URLs.

Many providers **do not** expose unary methods (`getMultipleAccounts`, etc.) over **`wss://`** (`Method not found`), while **HTTP** works. WebSocket mode therefore uses **`slotSubscribe`** ack latency instead of an HTTP-style unary RPC on WSS.

## How to run

From the repository root:

```bash
npm install
npm run ws-bench -- "wss://endpoint-a?..." "wss://endpoint-b?..."
```

The `--` after `ws-bench` is required so npm forwards arguments to the script.

**Default order per `wss://` URL:** HTTP cold → HTTP warm → WebSocket.

### Options

| Flag | Default | Meaning |
|------|---------|---------|
| `--preset <name>` | off | `quick` \| `standard` \| `long` \| `extended` — sets warmup + samples (see below) |
| `--warmup <n>` | 10* | HTTP warm path + WebSocket: cycles before recording |
| `--samples <n>` | 80* | Recorded cycles **per** metric (cold HTTP, warm HTTP, WS each use this count) |
| `--request-gap-ms <n>` | 0 | Delay between individual warmup / timed requests or subscribe cycles |
| `--max-rps <n>` | off | Convenience pacing flag; converts to roughly `ceil(1000 / n)` ms between requests |

\*Defaults match **`quick`** unless you pass **`--preset`**. **`--warmup`** / **`--samples`** after **`--preset`** override only that value.

**Presets** (warmup / samples). **HTTP cold** performs **`samples`** full new connections — large presets take proportionally longer.

| Preset | Warmup | Samples |
|--------|--------|---------|
| `quick` (default) | 10 | 80 |
| `standard` | 25 | 200 |
| `long` | 50 | 400 |
| `extended` | 75 | 800 |
| `--pause-ms <n>` | 1000 | Delay between top-level URL entries |
| `--from-env` | off | After any CLI URLs, append `SOLANA_WS_URL` then `HELIUS_WS_URL` if non-empty |
| `--helius-both` | off | For any Helius unified URL, benchmark both `mainnet.helius-rpc.com` and `beta.helius-rpc.com` with the same path + query string |
| `--ws-only` | off | WebSocket `slotSubscribe` only |
| `--rpc-only` / `--http-only` | off | HTTP `getMultipleAccounts` only (cold + warm); for `wss://` inputs, still uses derived `https://` |
| `--no-save` | off | Skip writing `./.local/launchdeck/rpc-ws-bench/run-*.json` |
| `--help` | | Usage |

### Examples

```bash
npm run ws-bench -- --preset long "wss://..." "wss://..."
npm run ws-bench -- --preset standard --samples 300 "wss://..."
npm run ws-bench -- --warmup 15 --samples 100 "wss://..." "wss://..."
npm run ws-bench -- --helius-both "wss://mainnet.helius-rpc.com/?api-key=..."
npm run ws-bench -- --max-rps 10 "wss://rpc.fra.shyft.to/?api_key=..."
npm run ws-bench -- --rpc-only "https://beta.helius-rpc.com/?api-key=..."
npm run ws-bench -- --ws-only "wss://..."
node Benchmarking/Benchmark.js --from-env
```

Put **secrets only in your environment or shell**; do not commit API keys in URLs to git.

## Metrics printed

### HTTP JSON-RPC (`getMultipleAccounts`)

- **Cold:** each sample pays **new connection** cost (often dominates vs warm).
- **Warm:** after warmup, latency is mostly **server + TLS session reuse** (still includes request/response on the wire).

Stats: **min / max / mean / stdev / p50 / p95** (ms).

### WebSocket

1. **`connect_handshake_ms`** — Open WebSocket, then close, **no** JSON-RPC. This is the connection lifecycle timing for that open/close cycle, so it is a good proxy for handshake cost but not a pure upgrade-only number.

2. **`slotSubscribe_ack_ms`** (after warmup, same connection) — send `slotSubscribe` → JSON response with subscription id → `slotUnsubscribe`, repeated for **`--samples`**.

3. **`accountSubscribe_ack_ms`** — send `accountSubscribe` against the rotating fixed LaunchDeck-history account basket → JSON response with subscription id → `accountUnsubscribe`, repeated for **`--samples`**. This is a more realistic standard WebSocket registration path than `slotSubscribe`, but still an ack metric only.

4. **`slotSubscribe_first_notification_ms`** — send `slotSubscribe`, wait for the subscribe ack, keep the same socket live until the first `slotNotification` arrives, then unsubscribe. This is the wait from **ack -> first stream event**, not pure transport latency.

5. **`transactionSubscribe_ack_ms`** (Helius unified hosts only) — send `transactionSubscribe` with a LaunchDeck-style filter/config (`accountInclude=11111111111111111111111111111111`, `failed=false`, `vote=false`, `commitment=processed`, `encoding=jsonParsed`, `transactionDetails=none`) → JSON response with subscription id → `transactionUnsubscribe`, repeated for **`--samples`**.

6. **`transactionSubscribe_first_notification_ms`** (Helius unified hosts only) — same request/filter as above, but keep the live subscription open until the first matching `transactionNotification` arrives, then unsubscribe. This is also measured from **ack -> first matching stream event**.

HTTP **warm** `getMultipleAccounts` and WebSocket subscription ack metrics are **not** the same RPC method; they are complementary views of **HTTP unary** vs **WSS subscription** behavior from your host.

## Interpreting results fairly

- **Run from the machine you actually use** (e.g. the same VPS as the engine).
- **Match regions** when comparing providers (EU vs US endpoints).
- Benchmark your recommended split directly: Helius Gatekeeper for HTTP and Helius standard websocket for watcher behavior.
- **Cold HTTP** vs **warm HTTP** isolates connection setup; compare warm HTTP to **WS subscribe ack** cautiously (different stacks).
- **Wide spread** suggests jitter or load; tight clusters are more stable.
- The new `*_first_notification_ms` metrics are useful for live-stream behavior, but they still depend on how quickly the chain produces the next matching event, so they are not a pure provider-internal latency number.
- `--helius-both` is useful when comparing Helius standard vs Gatekeeper from the exact same box and API key.
- For providers with strict rate caps, use `--max-rps` or `--request-gap-ms` so the benchmark stays below the provider limit instead of measuring 429 behavior.
- Helius dev tier is strongly recommended if you plan to run multiple snipes or watcher-heavy follow automation, because lower tiers are more likely to rate-limit both live traffic and benchmark traffic.

## Related LaunchDeck settings

- **`SOLANA_RPC_URL`**, **`HELIUS_RPC_URL`** — HTTP JSON-RPC used by the engine.
- **`SOLANA_WS_URL`**, **`HELIUS_WS_URL`** — WebSocket watchers; see `transport.rs`.
- **`LAUNCHDECK_WARM_RPC_URL`** — alternate warmup and block-height RPC; Shyft is still a good default for this path.

Use `ws-bench` to compare candidate URLs before locking them in `.env`.
