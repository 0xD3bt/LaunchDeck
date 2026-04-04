# Providers

This page explains the current execution providers exposed in LaunchDeck, what they are good for, and the rules the engine enforces when you select them.

## Supported Provider IDs

- `helius-sender`
- `standard-rpc`
- `jito-bundle`

User-facing labels:

- `Helius Sender`
- `Standard RPC`
- `Jito Bundle`

All three providers are active in the runtime registry, but `Helius Sender` is the current default recommendation for most operators.

## How Provider Resolution Works

LaunchDeck lets you choose provider settings separately for:

- creation
- buy
- sell

From those selections, the engine resolves:

- the provider actually used
- the execution class: `single`, `sequential`, or `bundle`
- the transport type
- endpoint or endpoint profile
- send requirements such as tip, preflight behavior, and ordering

The UI stores your intent. The engine decides final wire behavior.

## Provider Profiles

Only these providers support endpoint profiles:

- `Helius Sender`
- `Jito Bundle`

Supported profile values:

- `global`
- `us`
- `eu`
- `asia`
- Metro codes (Helius regional senders; Jito uses the same tokens to filter block-engine bases): `slc`, `ewr`, `lon`, `fra`, `ams`, `sg`, `tyo`
- Optional comma-separated metro lists (e.g. `fra,ams`)
- `ny` accepted as an alias for Newark (`ewr` for Helius Sender; Jito `ny.` hosts)

The former `west` aggregate is removed—use `us`, `eu`, or explicit metros.

When a profile is selected, LaunchDeck fans out across the endpoints in that selected group or metro set. It does not simply pick one endpoint.

For most operators, this is the recommended setup. Using `USER_REGION` or a provider-specific region override is usually faster and more reliable than pinning a single endpoint because the runtime can fan out across the selected endpoint set instead of depending on one host.

Region resolution order:

1. provider-specific override such as `USER_REGION_HELIUS_SENDER`
2. shared `USER_REGION`
3. provider default or global fallback

If you set explicit endpoint overrides, profile-based routing is bypassed. Use explicit endpoints only when you have a specific reason to force one host or one private integration.

### Helius Sender regional endpoints

When `HELIUS_SENDER_ENDPOINT` / `HELIUS_SENDER_BASE_URL` are unset, profile fanout uses Helius regional Sender HTTP hosts (each submit path ends in `/fast`). The mapping is:

| Profile | Sender hosts used |
| --- | --- |
| `global` | default `sender.helius-rpc.com` |
| `us` | `slc-sender`, `ewr-sender` |
| `eu` | `fra-sender`, `ams-sender` |
| `asia` | `sg-sender`, `tyo-sender` |
| Single metro (e.g. `fra`, `lon`) | that region’s `*-sender` host only |
| Comma metros (e.g. `fra,lon`) | the union of those regional hosts |

London (`lon-sender`) is available only when you set `lon`, `eu` (which is fra+ams only), or an explicit list that includes `lon`. Override envs still bypass this table.

## Full endpoint catalog (reference)

This section lists **concrete URLs** operators may plug into env vars or provider-specific overrides. It matches what the LaunchDeck engine uses by default where applicable, and vendor-published endpoints elsewhere—hostnames can change, so verify with each provider’s documentation if something stops resolving.

### Helius Sender (execution)

Used when `execution.provider` is `helius-sender`. Send path is always `…/fast`; LaunchDeck’s Sender **warm** path uses **`…/ping`** on the same host (not JSON-RPC).

| Key | Location | Send URL | Warm URL |
| --- | --- | --- | --- |
| `global` | Global front door | `https://sender.helius-rpc.com/fast` | `https://sender.helius-rpc.com/ping` |
| `slc` | Salt Lake City | `http://slc-sender.helius-rpc.com/fast` | `http://slc-sender.helius-rpc.com/ping` |
| `ewr` | Newark | `http://ewr-sender.helius-rpc.com/fast` | `http://ewr-sender.helius-rpc.com/ping` |
| `lon` | London | `http://lon-sender.helius-rpc.com/fast` | `http://lon-sender.helius-rpc.com/ping` |
| `fra` | Frankfurt | `http://fra-sender.helius-rpc.com/fast` | `http://fra-sender.helius-rpc.com/ping` |
| `ams` | Amsterdam | `http://ams-sender.helius-rpc.com/fast` | `http://ams-sender.helius-rpc.com/ping` |
| `sg` | Singapore | `http://sg-sender.helius-rpc.com/fast` | `http://sg-sender.helius-rpc.com/ping` |
| `tyo` | Tokyo | `http://tyo-sender.helius-rpc.com/fast` | `http://tyo-sender.helius-rpc.com/ping` |

`HELIUS_SENDER_ENDPOINT` / `HELIUS_SENDER_BASE_URL` override the above and bypass profile fanout.

### Helius Solana RPC and WebSocket (reads / confirm / watchers)

These are **normal Solana JSON-RPC and websocket** endpoints—not Sender. Typical shapes:

| Usage | Example pattern |
| --- | --- |
| HTTPS RPC (`SOLANA_RPC_URL`) | `https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY` |
| Websocket (`SOLANA_WS_URL`) | `wss://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY` |

Exact paths and query parameter names follow [Helius](https://www.helius.dev/) documentation for your plan.

### Jito Block Engine (execution)

Used when `execution.provider` is `jito-bundle`. LaunchDeck derives bundle **send** from each regional **base**: `{base}/api/v1/bundles` and **status**: `{base}/api/v1/getBundleStatuses` (unless `JITO_SEND_BUNDLE_ENDPOINT` / `JITO_BUNDLE_STATUS_ENDPOINT` or `JITO_BUNDLE_BASE_URLS` override defaults).

| Region key | Location | Base URL |
| --- | --- | --- |
| `mainnet` | Global mainnet | `https://mainnet.block-engine.jito.wtf` |
| `new-york` / `ny` | New York | `https://ny.mainnet.block-engine.jito.wtf` |
| `salt-lake-city` / `slc` | Salt Lake City | `https://slc.mainnet.block-engine.jito.wtf` |
| `frankfurt` | Frankfurt | `https://frankfurt.mainnet.block-engine.jito.wtf` |
| `amsterdam` | Amsterdam | `https://amsterdam.mainnet.block-engine.jito.wtf` |
| `london` | London | `https://london.mainnet.block-engine.jito.wtf` |
| `dublin` | Dublin | `https://dublin.mainnet.block-engine.jito.wtf` |
| `singapore` | Singapore | `https://singapore.mainnet.block-engine.jito.wtf` |
| `tokyo` | Tokyo | `https://tokyo.mainnet.block-engine.jito.wtf` |

LaunchDeck **endpoint profile** metro tokens (`slc`, `ewr`, `fra`, etc.) filter this list by matching these hostnames (for example `ewr` / `ny` match the New York base).

### Hello Moon — Lunar Lander (reference)

Hello Moon is **not** a separate execution provider in the current runtime registry. Legacy config may still map `hellomoon` → `standard-rpc`. The table below documents **published Lunar Lander regional HTTP send** endpoints as operator reference data. Submission modes (batch send, bundle, QUIC) are described in Hello Moon’s own API docs, linked below.

| Key | Location | Send URL |
| --- | --- | --- |
| `global` | Geolocated / global path | `http://lunar-lander.hellomoon.io/send` |
| `fra` | Frankfurt | `http://fra.lunar-lander.hellomoon.io/send` |
| `ams` | Amsterdam | `http://ams.lunar-lander.hellomoon.io/send` |
| `nyc` | New York | `http://nyc.lunar-lander.hellomoon.io/send` |
| `ash` | Ashburn, Virginia | `http://ash.lunar-lander.hellomoon.io/send` |
| `tyo` | Tokyo | `http://tyo.lunar-lander.hellomoon.io/send` |

API references: [Batch Send](https://docs.hellomoon.io/reference/batch-send-api), [Send Bundle](https://docs.hellomoon.io/reference/send-bundle-api), [QUIC submission](https://docs.hellomoon.io/reference/quic-submission). For generic RPC, see `HELLOMOON_RPC_URL` in [CONFIG.md](CONFIG.md) if you use Hello Moon as a normal RPC provider.

### Shyft — regional Solana RPC (standard-RPC style)

[Shyft](https://shyft.to/) regional HTTPS RPC hosts are commonly used for `LAUNCHDECK_WARM_RPC_URL` and sometimes for `LAUNCHDECK_STANDARD_RPC_SEND_URLS`. Replace `YOUR_API_KEY` with your key.

| Key | Location | Endpoint |
| --- | --- | --- |
| `fra` | Frankfurt | `https://rpc.fra.shyft.to?api_key=YOUR_API_KEY` |
| `ams` | Amsterdam | `https://rpc.ams.shyft.to?api_key=YOUR_API_KEY` |
| `sgp` | Singapore | `https://rpc.sgp.shyft.to?api_key=YOUR_API_KEY` |
| `va` | Virginia | `https://rpc.va.shyft.to?api_key=YOUR_API_KEY` |
| `ny` | New York | `https://rpc.ny.shyft.to?api_key=YOUR_API_KEY` |

Shyft may publish additional regions; treat this table as a common regional set, not an exhaustive vendor list.

## Helius Sender

`Helius Sender` is the default, fastest, and most reliable starting point in the current runtime for most operators.

Recommended operator stack:

- use Helius for `SOLANA_RPC_URL`
- use Helius for `SOLANA_WS_URL`
- use a [Shyft](https://shyft.to/) RPC with a free API key for `LAUNCHDECK_WARM_RPC_URL`
- use `helius-sender` for creation, buy, and sell provider routing
- if you have Helius dev tier and websocket support for it, enable `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true`

Use it when you want:

- the main supported low-latency path
- endpoint-profile support
- predictable Sender-specific transport behavior
- instant execution in typical low-latency setups

How it works:

- supports `single` execution
- supports `sequential` execution
- does not support bundle execution
- supports endpoint profiles

Required behavior:

- inline tip is required
- inline compute-unit price is required
- `skipPreflight=true` is required
- incompatible requests are rejected rather than silently downgraded

Code-enforced requirements:

- `execution.skipPreflight` must be `true`
- `tx.computeUnitPriceMicroLamports` must be greater than `0`
- `tx.jitoTipLamports` must be at least `200000`

Practical note:

- if `SOLANA_RPC_URL` is not configured, LaunchDeck can still use the default Sender endpoint, but you should set a dedicated confirmation RPC for real operation
- in normal average-latency setups this is the provider we recommend first
- pairing Helius Sender with Helius RPC + Helius WS is currently the strongest overall default setup in LaunchDeck
- Helius dev tier is strongly recommended if you care about the best watcher quality and execution performance

### Helius Enhanced Realtime Watchers

When all of these are true:

- `SOLANA_WS_URL` points at a Helius websocket endpoint
- `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true`
- your Helius tier actually supports `transactionSubscribe`

the follow daemon upgrades slot, signature, and market watchers to use Helius `transactionSubscribe` instead of standard websocket subscriptions.

If any of those conditions are not met, LaunchDeck falls back to the standard websocket watcher path automatically.

Watcher routing note:

- send provider and watch endpoint are not the same thing
- provider selection decides how launch and trade transactions are sent
- `SOLANA_WS_URL` decides which websocket watch path the daemon uses
- that means a launch can send with `standard-rpc` or `jito-bundle` and still use Helius enhanced realtime watchers if the websocket watch endpoint is Helius

## Standard RPC

`Standard RPC` is the plain Solana RPC path.

Use it when you want:

- the most conventional transport behavior
- standard confirmation semantics
- no Sender or bundle-specific requirements

How it works:

- supports `single` execution
- supports `sequential` execution
- does not support `bundle`
- does not support endpoint profiles
- does not use tip

Practical note:

- this is the most predictable fallback if you want explicit RPC semantics, but it does not have Sender-specific low-latency behavior

## Jito Bundle

`Jito Bundle` is the bundle-oriented path.

Use it when you want:

- bundle submission semantics
- bundle-specific tip behavior
- regional Jito endpoint fanout

How it works:

- supports `single` execution
- does not support `sequential`
- supports `bundle`
- supports endpoint profiles

Practical note:

- bundle members are treated as an ordered grouped send
- bundle submission is fanned out across the selected profile group when profiles are used

## Engine-Owned Overrides

The provider selection is not a raw pass-through. The engine owns final shaping.

Examples:

- `standard-rpc` ignores tip even if an old preset still contains a tip value
- `helius-sender` rejects incompatible requests instead of silently falling back
- `jito-bundle` may accept both tip and priority in the UI, but the engine can intentionally drop creation priority in some multi-transaction creation flows

This is by design. Operators should treat the provider as a routing intent, not a guarantee that every individual fee field will be applied exactly as typed.

## Availability And Bootstrap

Provider availability is exposed through the runtime bootstrap and status APIs so the browser can initialize from the same backend that owns execution.

The important operator takeaway is simple:

- the UI reads provider availability from the Rust host
- execution still happens according to runtime validation and transport planning

## Legacy Provider Mapping

Older saved provider values are migrated forward when settings are loaded:

- `auto` -> `helius-sender`
- `helius` -> `helius-sender`
- `jito` -> `jito-bundle`
- `astralane` -> `standard-rpc`
- `bloxroute` -> `standard-rpc`
- `hellomoon` -> `standard-rpc`

These values should not be used as live provider IDs in current config.
