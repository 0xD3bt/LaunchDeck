# Execution Do's And Don'ts

This document is the shared reference for how LaunchDeck should treat low-latency execution providers and watcher infrastructure.

It is intentionally operational rather than marketing-oriented:

- how each provider actually works
- what must be present in the transaction
- what order and transport behavior matters
- warmup and keep-alive expectations
- MEV and bundle-protection details
- what we should never do in the app

Use this alongside `docs/PROVIDERS.md` when implementing or changing execution paths.

## Scope

The notes below were compiled from the provider documentation indexes plus the highest-signal pages relevant to LaunchDeck's architecture:

- [Jito docs](https://docs.jito.wtf/)
- [Jito low latency transaction send](https://docs.jito.wtf/lowlatencytxnsend/)
- [Hello Moon Lunar Lander](https://docs.hellomoon.io/reference/lunar-lander)
- [Hello Moon QUIC Submission](https://docs.hellomoon.io/reference/quic-submission)
- [Hello Moon Batch Send API](https://docs.hellomoon.io/reference/batch-send-api)
- [Hello Moon Send Bundle API](https://docs.hellomoon.io/reference/send-bundle-api)
- [Harmonic docs](https://docs.harmonic.gg/)
- [Harmonic Bundles](https://docs.harmonic.gg/searchers/harmonic-bundles)
- [Harmonic Bundle Control Accounts](https://docs.harmonic.gg/searchers/bundle-control-accounts)
- [Harmonic NTP Configuration](https://docs.harmonic.gg/ntp-configuration)
- [Harmonic Endpoints](https://docs.harmonic.gg/endpoints)
- [Helius docs](https://www.helius.dev/docs)
- [Helius docs index](https://www.helius.dev/docs/llms.txt)
- [Helius Sender API](https://www.helius.dev/docs/api-reference/sender/llms.txt)
- [Helius transaction sending overview](https://www.helius.dev/docs/sending-transactions/overview)
- [Helius transaction optimization guide](https://www.helius.dev/docs/sending-transactions/optimizing-transactions)
- [Helius Priority Fee API](https://www.helius.dev/docs/api-reference/priority-fee/llms.txt)
- [Helius transactionSubscribe](https://www.helius.dev/docs/enhanced-websockets/transaction-subscribe)

## Global Rules

### Do

- Treat transaction sending and transaction confirmation as separate concerns.
- Build and sign transactions locally whenever the provider supports low-latency raw submission.
- Prefer `base64` over `base58` anywhere a provider allows both.
- Implement our own retry logic instead of delegating retries to provider-side default retry loops.
- Use provider-specific transports only when the transaction satisfies that provider's hard requirements.
- Keep endpoint selection region-aware and fan out when the provider supports it.
- Keep a dedicated read/confirm RPC separate from the execution transport when that gives better control or observability.
- Make MEV protection explicit in the UI and runtime, not hidden in environment defaults alone.
- Reuse warm connections wherever the transport is connection-oriented.
- Preserve atomicity guarantees only when the provider explicitly gives them.

### Don't

- Do not assume all "fast send" providers behave like normal RPC `sendTransaction`.
- Do not silently downgrade from a provider with hard requirements into a weaker path.
- Do not treat "request accepted" as "landed on-chain".
- Do not use one provider's tip accounts with another provider.
- Do not assume bundle semantics for batch APIs.
- Do not assume a fire-and-forget transport returns signatures or per-send receipts.
- Do not reconnect for every submission on QUIC or long-lived socket-style paths.
- Do not let the app emit transactions with missing tip or missing priority fee when the selected provider requires them.

## Provider Snapshot

| Provider | Primary LaunchDeck use | Transport style | Atomic multi-tx support | Tip requirement | Priority fee requirement | Warmup pattern |
| --- | --- | --- | --- | --- | --- | --- |
| Helius Sender | fastest default single/sequential execution | HTTP Sender path | No bundle path | Yes | Yes | `GET /ping` |
| Hello Moon Lunar Lander QUIC | low-latency Hello Moon execution | QUIC, fire-and-forget | No via QUIC; yes via separate HTTP bundle endpoint | Yes | Yes in practice for competitive sends and enforced by LaunchDeck | keep QUIC connection open |
| Jito Block Engine | bundle or protected low-latency execution | JSON-RPC or gRPC | Yes, bundles up to 5 | Yes | Yes for competitive landing | no published ping endpoint; keep client hot and monitor status |
| Harmonic Block Engine | alternative bundle engine | gRPC | Yes | no separate tip account; tip is priority fee | Yes | parallel regional submission + accurate clock sync |
| Helius staked RPC / standard RPC | confirmation, reads, non-specialized sending | JSON-RPC / WSS | No | No provider tip | Dynamic priority fee recommended | `getHealth` warm loop for staked regional cache if used for send |

## Shared Transaction Construction Rules

### Always include compute budget intentionally

- Add `ComputeBudgetProgram.setComputeUnitLimit(...)`.
- Add `ComputeBudgetProgram.setComputeUnitPrice(...)` whenever the chosen path benefits from priority fees.
- For high-competition routes, compute-unit price should be refreshed dynamically from live market data instead of hard-coded forever.

### Separate "provider fee" from "program logic"

- Provider-required tip transfers should be treated as transport requirements, not business logic.
- Priority fee selection should be provider-aware.
- Bundle protection and anti-front-run markers should be treated as optional execution modifiers.

### Retry safely

- Retry until blockhash expiry, not forever.
- Poll confirmation state between retries.
- Re-sign only after the current blockhash is no longer valid.
- Never blindly blast duplicate transactions with fresh signatures while the original blockhash is still live.

## Helius

### How it works

Helius exposes two different ideas that matter to LaunchDeck:

- standard/staked RPC send paths
- Sender, which is an ultra-low-latency send path routed to validators and Jito simultaneously

Sender is the Helius product that most closely maps to LaunchDeck's specialized execution path. Standard Helius RPC is still important for:

- blockhash fetch
- confirmation
- reads
- watcher streams
- priority fee estimation

### Helius Sender: operational rules

#### Do

- Send to `https://sender.helius-rpc.com/fast` for browser-safe/global frontend use.
- Use regional backend endpoints when operating servers close to the target region.
- Include both:
  - a priority fee instruction
  - a tip transfer to a Helius Sender tip account
- Set `skipPreflight: true`.
- Set `maxRetries: 0`.
- Keep Sender warm with `GET /ping`.
- Use Sender when latency is critical and you want dual routing to validators and Jito.

#### Don't

- Do not use Sender without a tip.
- Do not use Sender without a compute-unit price instruction.
- Do not rely on provider-side automatic retries.
- Do not treat Sender as a generic RPC endpoint.

### Helius Sender tip accounts

These are the documented mainnet tip accounts and should be randomized to reduce contention:

- `4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE`
- `D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ`
- `9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta`
- `5VY91ws6B2hMmBFRsXkoAAdsPHBJwRfBht4DXox3xkwn`
- `2nyhqdwKcJZR2vcqCyrYsaPVdAnFoJjiksCXJ7hfEYgD`
- `2q5pghRs6arqVjRvT5gfgWfWcHWmw1ZuCzphgd5KfWGJ`
- `wyvPkWjVZz1M8fHQnMMCDTQDbkManefNNhweYk5WkcF`
- `3KCKozbAaF75qEU33jtzozcJ29yJuaLJTy2jFdzUY8bT`
- `4vieeGHPYPG2MmyPRcYjdiDmmhN3ww7hsFNap8pVN3Ey`
- `4TQLFNWK8AovT1gFvda5jfw2oJeRMKEmw7aH6MGBJ3or`

### Helius Sender minimums and behavior

- Minimum dual-route tip: `0.0002 SOL`
- SWQoS-only minimum tip: `0.000005 SOL`
- Default rate limit: `50 TPS`
- Credits: `0`
- Recommended encoding: `base64`

### Helius standard/staked send guidance

Use Helius staked connections when you want high reliability but not necessarily specialized HFT-style transport behavior.

#### Do

- Fetch the latest blockhash at `confirmed`.
- Use dynamic priority fees.
- Simulate to estimate compute-unit usage, then add margin.
- Set `maxRetries: 0` and rebroadcast yourself.
- Warm regional caches with `getHealth` once per second if you are optimizing staked send latency.

#### Don't

- Do not use one static priority fee forever.
- Do not rely on preflight simulation as the only safety mechanism for trading paths.
- Do not run multiple redundant cache-warming threads per region.

### Helius watcher guidance

Helius `transactionSubscribe` is useful when LaunchDeck needs low-latency transaction-aware watchers.

#### Do

- Use unified endpoints like `wss://mainnet.helius-rpc.com/?api-key=...`.
- Send websocket pings regularly to avoid inactivity disconnects.
- Set `maxSupportedTransactionVersion: 0` when you want both legacy and v0 coverage with detailed payloads.
- Use `accountInclude`, `accountExclude`, and `accountRequired` deliberately.

#### Don't

- Do not use legacy Atlas websocket endpoints for new work.
- Do not assume the connection will stay alive without pings.
- Do not ask for `full` or `accounts` details without setting `maxSupportedTransactionVersion`.

## Hello Moon Lunar Lander

### How it works

Lunar Lander offers multiple send paths:

- HTTP `/send`
- HTTP batch `/sendBatch`
- HTTP atomic bundle `/sendBundle`
- QUIC submission

For LaunchDeck, QUIC is the closest conceptual match to Helius Sender because it is:

- low-latency
- local-signing friendly
- connection-oriented
- fire-and-forget

### Hello Moon QUIC: operational rules

#### Do

- Use QUIC on UDP port `16888`.
- Reuse long-lived QUIC connections.
- Send one transaction per unidirectional stream.
- Use the closest regional hostname possible.
- Treat QUIC as fire-and-forget and confirm via RPC afterward.
- Keep MEV protection per connection, not per individual stream.
- Use the official Rust client unless there is a strong reason to own the entire QUIC/TLS stack.

#### Don't

- Do not expect a per-stream response body.
- Do not batch multiple transactions into one QUIC stream.
- Do not reconnect for every transaction.
- Do not attempt QUIC without a valid API key.
- Do not send tipless transactions over QUIC.

### Hello Moon QUIC connection details

- ALPN must be `lunar-lander-tpu`
- One transaction per unidirectional stream
- Max transaction size: `1232` bytes
- Max concurrent unidirectional streams per connection: `64`
- Idle timeout: `30s`
- Max concurrent connections per API key: `10`
- Max connection attempts per source IP: `10/min`

### Hello Moon QUIC MEV protection

MEV protection is enabled per connection.

#### Do

- Use `ClientOptions { mev_protect: true }` with the official Rust client when protection is desired.
- Reconnect if the protection setting changes.
- Surface this as a route-specific runtime choice in the UI.

#### Don't

- Do not pretend MEV protection is per-transaction on QUIC.
- Do not change the toggle without rebuilding or switching the underlying QUIC connection state.

### Hello Moon QUIC certificate behavior

For custom clients, MEV protection is activated by a non-critical X.509 extension:

- OID: `2.999.1.1`
- critical: `false`
- value: DER BOOLEAN TRUE (`0x01 0x01 0xFF`)

If that extension is absent, MEV protection is disabled for that connection.

### Hello Moon connection and stream failure handling

Connection-level failures:

- `bad_cert`
- `invalid_auth`
- `connection_limit`

Stream-level dropped submissions include:

- `tip_required`
- `invalid_payload`
- oversized payload
- transaction-level rate-limit drop

Interpretation:

- connection failures require reconnect or credential/config correction
- stream failures mean the submission did not enter the pipeline and must be handled by our app logic

### Hello Moon HTTP paths

#### `/sendBatch`

Use only for multiple independent transactions where atomicity is not required.

#### Do

- Treat `/sendBatch` as throughput optimization, not atomic execution.
- Use the documented binary frame format.
- Expect only a summary response: attempted, accepted, rejected, parse error.

#### Don't

- Do not use `/sendBatch` if you need per-tx signatures from the response.
- Do not use `/sendBatch` if you need all-or-nothing semantics.
- Do not exceed `16` transactions.

#### `/sendBundle`

Use only when you explicitly need atomic multi-transaction execution.

#### Do

- Ensure every tx is fully signed before submission.
- Include at least one valid Lunar Lander tip in the bundle.
- Confirm signatures on-chain via normal RPC after submission.

#### Don't

- Do not assume a successful HTTP response means the bundle landed.
- Do not exceed `4` transactions.
- Do not submit if any bundled transaction would fail simulation.

### Hello Moon endpoints

- `http://fra.lunar-lander.hellomoon.io/send`
- `http://ams.lunar-lander.hellomoon.io/send`
- `http://nyc.lunar-lander.hellomoon.io/send`
- `http://ash.lunar-lander.hellomoon.io/send`
- `http://tyo.lunar-lander.hellomoon.io/send`
- `http://lunar-lander.hellomoon.io/send`

QUIC uses those same regional hostnames on `:16888`.

### Hello Moon tip requirement

- Lunar Lander QUIC is tip-enforced.
- Bundle tip minimum: `1,000,000` lamports (`0.001 SOL`)
- LaunchDeck should continue to treat Hello Moon execution as requiring both tip and priority fee.

### Hello Moon tip accounts

These are the tip accounts currently mirrored in LaunchDeck and should be randomized:

- `moon17L6BgxXRX5uHKudAmqVF96xia9h8ygcmG2sL3F`
- `moon26Sek222Md7ZydcAGxoKG832DK36CkLrS3PQY4c`
- `moon7fwyajcVstMoBnVy7UBcTx87SBtNoGGAaH2Cb8V`
- `moonBtH9HvLHjLqi9ivyrMVKgFUsSfrz9BwQ9khhn1u`
- `moonCJg8476LNFLptX1qrK8PdRsA1HD1R6XWyu9MB93`
- `moonF2sz7qwAtdETnrgxNbjonnhGGjd6r4W4UC9284s`
- `moonKfftMiGSak3cezvhEqvkPSzwrmQxQHXuspC96yj`
- `moonQBUKBpkifLcTd78bfxxt4PYLwmJ5admLW6cBBs8`
- `moonXwpKwoVkMegt5Bc776cSW793X1irL5hHV1vJ3JA`
- `moonZ6u9E2fgk6eWd82621eLPHt9zuJuYECXAYjMY1C`

## Jito

### How it works

Jito's low-latency infrastructure exposes:

- direct low-latency `sendTransaction`
- atomic `sendBundle`
- bundle status APIs
- tip floor and tip stream data

Jito transactions and bundles compete in an auction environment. Bundle ordering and landing are shaped by:

- tip size
- compute unit usage
- account lock overlap
- region latency

Parallel auctions are run at `50ms` ticks for intersecting lock sets.

### Current LaunchDeck behavior

Today LaunchDeck warms Jito by calling `getTipAccounts` against the currently selected Jito block-engine endpoints on the same shared continuous warm cadence used for other providers.

Important nuance:

- this is HTTP keep-warm through the shared reqwest client pool
- it is not a persistent Jito gRPC/searcher channel
- if an idle pooled HTTP connection is dropped, the next warm probe or next send re-establishes it automatically

Warm failures are tracked in runtime warm-target telemetry as:

- `healthy`
- `rate-limited`
- `error`

Per-target warm status includes the last attempt timestamp, last success timestamp, last error text, and consecutive failure count.

### Jito sendTransaction rules

#### Do

- Use Jito when you want validator-direct low-latency send with Jito-side protection features.
- Use `base64`.
- Expect `skipPreflight=true`.
- Use both priority fee and Jito tip for competitive execution.
- Consider `bundleOnly=true` if you want revert protection as a single-tx bundle.

#### Don't

- Do not assume the absolute documented minimum tip is competitive in busy periods.
- Do not use Jito tip accounts in ALTs.
- Do not send standalone tip transactions without state assertions when avoidable.

### Jito bundle rules

#### Do

- Use bundles when you need strict sequential all-or-nothing execution.
- Keep related tips inside the same bundle logic when possible.
- Query `getBundleStatuses` or `getInflightBundleStatuses` after submission.
- Randomize the Jito tip account.

#### Don't

- Do not exceed `5` transactions per bundle.
- Do not assume bundle privacy solves uncled-block rebroadcast risk completely.
- Do not separate the tip from the protected strategy unless you understand the uncle-bandit risk.

### Jito sandwich mitigation and bundle control

Jito supports `jitodontfront`-style protection on transactions.

#### Do

- Add a valid read-only pubkey beginning with `jitodontfront` when you want a bundle to reject any attempt to place your transaction behind another tx in the same bundle.
- Put that account in the transaction in a way that survives the final compiled message.
- Treat it as a bundle constraint, not as universal global ordering protection.

#### Don't

- Do not assume this protects transactions outside Jito-controlled block-engine behavior.
- Do not assume it solves every ordering attack.

### Jito tip accounts

- `96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5`
- `HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe`
- `Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY`
- `ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49`
- `DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh`
- `ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt`
- `DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL`
- `3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT`

### Jito fee guidance

Documented guidance for `sendTransaction` is to think in combined fee budget terms and often split roughly:

- `70%` priority fee
- `30%` Jito tip

This is guidance, not a rule. Real auction conditions should override stale heuristics.

### Jito status and rate limits

- Default rate limit: `1 request/sec per IP per region`
- Authentication may be required depending on endpoint or product mode
- `getInflightBundleStatuses` looks back only a short recent window

### Jito operational caveat: uncled blocks

Jito explicitly documents uncle/rebroadcast risk.

#### Do

- Add state assertions and post-conditions when using bundles.
- Design bundles so partial rebroadcast exposure is survivable.

#### Don't

- Do not assume bundle atomicity still protects you if bundle contents are later rebroadcast out of an uncled block context.

## Harmonic

### How it works

Harmonic is an alternative block engine with bundle submission and auction infrastructure. For searchers it is intentionally close to Jito conceptually, but not economically identical.

Important differences:

- bundles are private to Harmonic's trusted builder network
- revert protection is provided
- there is no separate "Jito tip account" model
- tips are regular priority fees

### Harmonic searcher rules

#### Do

- Treat Harmonic as a bundle engine, not as a generic RPC replacement.
- Send bundles to all regions at once for best performance.
- Keep system time tightly synchronized.
- Reuse Jito-like client code where useful, but account for auth-role differences.

#### Don't

- Do not assume Jito auth role numbers work unchanged.
- Do not design Harmonic integrations around separate tip transfer instructions.

### Harmonic auth and proto caveat

Harmonic documents interface compatibility with Jito-style searcher flows, but searcher auth role values differ.

If reusing Jito proto-based code:

- pay attention to the searcher role mismatch
- do not blindly reuse Jito auth constants

### Harmonic bundle economics

#### Do

- Think of the bid as priority fee value to the validator.
- Prefer priority-fee-based bidding instead of separate tip transfer patterns.

#### Don't

- Do not add a Jito-style tip transfer just because the code path used to target Jito.

### Harmonic bundle control accounts

Harmonic exposes strong bundle-control-account semantics.

#### `dontfront` / `jitodontfront`

- transaction must be at bundle index `0`

#### `dontbund1e`

- transaction must be at bundle index `0`
- all later bundle txs may only invoke:
  - System Program
  - Compute Budget Program
  - Memo V1
  - Memo V3

#### Do

- Put the control account as the first account on the first compute budget instruction the engine sees.
- Mark the control account read-only.
- Use this when you explicitly want bundle-placement restrictions.

#### Don't

- Do not append the control account to an arbitrary later instruction and expect the engine to honor it.
- Do not assume more than the first matched compute-budget placement matters.

### Harmonic timing and region rules

Harmonic explicitly recommends good clock sync.

#### Do

- Keep chrony or equivalent accurately configured.
- Maintain low clock skew.
- Send to all regions in parallel for searcher performance.

#### Don't

- Do not run latency-sensitive searcher logic with poor NTP discipline.

### Harmonic endpoints

Block engine endpoints documented by Harmonic:

- `https://fra.be.harmonic.gg`
- `https://lon.be.harmonic.gg`
- `https://ams.be.harmonic.gg`
- `https://ewr.be.harmonic.gg`
- `https://tyo.be.harmonic.gg`
- `https://sgp.be.harmonic.gg`

Additional network/infra endpoints:

- Amsterdam auction: `https://ams.auction.harmonic.gg`
- Newark auction: `https://ewr.auction.harmonic.gg`
- Frankfurt auction: `https://fra.auction.harmonic.gg`
- London auction: `https://lon.auction.harmonic.gg`
- Tokyo auction: `https://tyo.auction.harmonic.gg`
- Singapore auction: `https://sgp.auction.harmonic.gg`

## Keep-Warm And Ping Guidance

### Helius Sender

- Use `GET /ping` every `30-60s` during idle periods to avoid cold-start latency.

### Helius staked regional send optimization

- Use `getHealth` once per second on the same endpoint and API key if intentionally warming a staked regional path.
- Only one warming thread per region is useful.

### Hello Moon QUIC

- No HTTP ping equivalent is documented for QUIC.
- The keep-warm strategy is connection reuse, not ping endpoints.
- Avoid idle expiration by keeping the connection active and recreating it on timeout.

### Jito

- No documented dedicated ping endpoint for low-latency send.
- Keep clients region-aware, maintain hot blockhash/fee/tip state, and use status APIs instead of assuming a warm transport guarantee.

### Harmonic

- Warmth is less about pinging a special endpoint and more about:
  - authenticated ready clients
  - parallel regional submission
  - accurate NTP
  - persistent channel reuse

## App-Level Implementation Rules For LaunchDeck

### Route selection

- Keep provider choice separate for creation, buy, and sell.
- Enforce provider-specific validation before submission.
- Surface provider capabilities in UI, especially:
  - tip required
  - priority fee required
  - slippage relevance
  - bundle support
  - MEV protection mode

### Fee handling

- Never use one universal minimum fee table for all providers.
- Keep provider-specific minimums in one canonical place.
- Randomize provider-specific tip accounts.
- Distinguish:
  - inline provider tip
  - priority fee
  - auto-fee cap

### MEV protection

- Keep Hello Moon protection route-specific and expose it as explicit MEV modes, not one global switch.
- Recommended LaunchDeck wording:
  - `Off`: fastest path, no Hello Moon MEV filtering, no `jitodontfront`.
  - `Reduced`: uses Hello Moon QUIC with `mev_protect=true` and adds `jitodontfront` where the transaction builder supports it.
  - `Secure`: uses the Hello Moon bundle path with stronger protection-focused routing, but slower execution and bundle-specific constraints.
- Support Jito/Harmonic bundle control accounts as transaction modifiers, not as interchangeable provider-level guarantees.
- Document clearly that protection semantics differ per provider and that `jitodontfront` is bundle-position protection, not universal chain-wide protection.

### Confirmation and observability

- Fire-and-forget execution paths must always be paired with explicit confirmation RPC strategy.
- Track transport used, endpoint selected, and whether MEV protection was enabled.
- For bundle engines, record submission receipt separately from landing confirmation.

### Fallback behavior

- Do not silently fall back from specialized fast paths into generic RPC.
- If we choose to add optional fallback in the future, it must be explicit, observable, and operator-controlled.

## Hard Don'ts

- Do not use `base58` when `base64` is available and recommended.
- Do not use Address Lookup Tables for Jito tip accounts.
- Do not send Hello Moon QUIC multi-tx payloads on one stream.
- Do not assume Hello Moon QUIC returns signatures.
- Do not assume `/sendBatch` means bundle semantics on any provider.
- Do not assume `jitodontfront` is universal chain-wide protection.
- Do not use Jito-style tip transfer logic on Harmonic.
- Do not mutate transport requirements silently in the UI.
- Do not re-sign a still-valid transaction just because confirmation is slow.
- Do not keep stale fee recommendations around during active trading windows.

## Practical Default Recommendations

### Best default overall

- Helius Sender for execution
- Helius RPC for reads/blockhash/confirmation
- Helius enhanced websockets when available for watcher quality
- dynamic priority fee estimation
- explicit app-side retry loop

### Fast Hello Moon stack

- Hello Moon QUIC for execution
- separate RPC for blockhash/confirmation
- route-specific MEV protection toggle
- connection cache and reconnect-on-failure behavior

### Bundle-engine work

- Jito when you want the most established bundle ecosystem and status tooling
- Harmonic when you want bundle privacy, revert protection, and priority-fee-based bidding without separate tip-account economics

## Source Links

- [Jito docs home](https://docs.jito.wtf/)
- [Jito low latency tx send](https://docs.jito.wtf/lowlatencytxnsend/)
- [Hello Moon Lunar Lander](https://docs.hellomoon.io/reference/lunar-lander)
- [Hello Moon QUIC Submission](https://docs.hellomoon.io/reference/quic-submission)
- [Hello Moon Batch Send API](https://docs.hellomoon.io/reference/batch-send-api)
- [Hello Moon Send Bundle API](https://docs.hellomoon.io/reference/send-bundle-api)
- [Harmonic docs home](https://docs.harmonic.gg/)
- [Harmonic Bundles](https://docs.harmonic.gg/searchers/harmonic-bundles)
- [Harmonic Bundle Control Accounts](https://docs.harmonic.gg/searchers/bundle-control-accounts)
- [Harmonic NTP Configuration](https://docs.harmonic.gg/ntp-configuration)
- [Harmonic Endpoints](https://docs.harmonic.gg/endpoints)
- [Helius docs home](https://www.helius.dev/docs)
- [Helius docs index](https://www.helius.dev/docs/llms.txt)
- [Helius Sender API](https://www.helius.dev/docs/api-reference/sender/llms.txt)
- [Helius transaction sending overview](https://www.helius.dev/docs/sending-transactions/overview)
- [Helius transaction optimization guide](https://www.helius.dev/docs/sending-transactions/optimizing-transactions)
- [Helius Priority Fee API](https://www.helius.dev/docs/api-reference/priority-fee/llms.txt)
- [Helius transactionSubscribe](https://www.helius.dev/docs/enhanced-websockets/transaction-subscribe)
