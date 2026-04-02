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
- `west`
- `asia`

When a profile is selected, LaunchDeck fans out across the endpoints in that profile group. It does not simply pick one endpoint.

For most operators, this is the recommended setup. Using `USER_REGION` or a provider-specific region override is usually faster and more reliable than pinning a single endpoint because the runtime can fan out across the region's endpoint group instead of depending on one host.

Region resolution order:

1. provider-specific override such as `USER_REGION_HELIUS_SENDER`
2. shared `USER_REGION`
3. provider default or global fallback

If you set explicit endpoint overrides, profile-based routing is bypassed. Use explicit endpoints only when you have a specific reason to force one host or one private integration.

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

## Upcoming Relay Integrations

Additional private relay integrations are planned but not yet live in the current runtime.

Current roadmap includes:

- `bloxroute`
- `astralane`
- `hello moon`

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
