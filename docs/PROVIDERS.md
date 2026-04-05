# Providers

This page explains the execution providers exposed in LaunchDeck, how routing works, and which endpoint groups are currently used.

For the recommended overall setup, start with `docs/CONFIG.md`. This page is the routing and provider reference.

## Current provider IDs

LaunchDeck currently exposes:

- `helius-sender`
- `hellomoon`
- `standard-rpc`
- `jito-bundle`

User-facing labels:

- `Helius Sender`
- `Hello Moon`
- `Standard RPC`
- `Jito Bundle`

Current recommendation:

- start with `Helius Sender`
- use `Hello Moon` when you want a strong alternate low-latency path
- use `Standard RPC` when you want plain RPC behavior
- use `Jito Bundle` when you explicitly want bundle semantics

## Provider resolution

LaunchDeck lets you choose send settings separately for:

- creation
- buy
- sell

From those selections, the engine resolves:

- the provider actually used
- the transport type
- the endpoint profile
- whether the send is single, sequential, or bundle-based
- provider-specific requirements such as tip, priority fee, and preflight rules

The UI stores your intent. The engine owns the final wire behavior.

## Send path vs watcher path

These are not the same thing.

- `execution.provider`, `execution.buyProvider`, and `execution.sellProvider` decide how transactions are sent
- `SOLANA_WS_URL` decides which websocket watcher path the follow daemon uses
- `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true` only affects the watcher websocket path

That means:

- a launch can send with `jito-bundle` and still use Helius watchers
- a launch can send with `hellomoon` and still use Helius watchers
- watcher warm is based on the active watcher websocket, not on provider-region fanout

## Endpoint profiles

Providers with endpoint-profile support:

- `Helius Sender`
- `Hello Moon`
- `Jito Bundle`

Supported profile tokens:

- `global`
- `us`
- `eu`
- `asia`
- `slc`
- `ewr`
- `lon`
- `fra`
- `ams`
- `sg`
- `tyo`
- comma-separated lists such as `fra,ams`

`ny` is still accepted as an alias for Newark / New York style mappings where applicable.

Resolution order:

1. provider-specific region override such as `USER_REGION_HELIUS_SENDER`
2. shared `USER_REGION`
3. provider fallback

If you set explicit endpoint overrides, profile routing is bypassed.

## Routing behavior by provider

### Helius Sender

Helius Sender supports exact metro routing where those metros exist.

Grouped profiles:

- `global`: global Sender front door
- `us`: Salt Lake City + Newark
- `eu`: Frankfurt + Amsterdam
- `asia`: Singapore + Tokyo

Exact metros:

- `slc`
- `ewr`
- `lon`
- `fra`
- `ams`
- `sg`
- `tyo`

### Hello Moon

Hello Moon supports region-aware routing, but it does not expose every metro that Helius Sender does.

Current mapping:

- `global` -> global Hello Moon endpoint
- `eu` -> Frankfurt + Amsterdam
- `fra` -> Frankfurt
- `ams` -> Amsterdam
- `us` -> New York + Ashburn
- `slc` -> New York + Ashburn
- `ewr` -> New York + Ashburn
- `asia` -> Tokyo
- `sg` -> Tokyo
- `tyo` -> Tokyo
- `lon` -> Frankfurt + Amsterdam

That means Helius Sender exact-metro routing is more precise than Hello Moon exact-metro routing in US, London, and Singapore cases.

### Jito Bundle

Jito Bundle uses the same profile tokens for regional filtering of its block-engine hosts.

In practice:

- `us` filters to the US Jito hosts
- `eu` filters to the EU Jito hosts
- `asia` filters to the Asia Jito hosts
- exact metro tokens narrow that list further where hostnames match

### Standard RPC

`standard-rpc` does not use endpoint profiles.

It uses:

- `SOLANA_RPC_URL` as the primary read/confirm RPC
- optional extra submit-only endpoints from `LAUNCHDECK_EXTRA_STANDARD_RPC_SEND_URLS`

## Recommended stack

For most operators, the current recommended stack is:

- `SOLANA_RPC_URL`: Helius Gatekeeper HTTP
- `SOLANA_WS_URL`: Helius standard websocket
- `LAUNCHDECK_WARM_RPC_URL`: Shyft
- send provider: `helius-sender` or `hellomoon`

If you are watcher-heavy or running multiple snipes, Helius dev tier is strongly recommended.

## Provider details

## Helius Sender

`Helius Sender` is the easiest production default in LaunchDeck right now.

Use it when you want:

- the main supported low-latency path
- exact metro routing
- predictable Sender-specific transport behavior

Requirements enforced by the engine:

- `skipPreflight=true`
- positive compute-unit price
- tip of at least `200000` lamports

Warm behavior:

- LaunchDeck warms Sender via `GET /ping` on the same host as the `/fast` send path

## Hello Moon

`Hello Moon` is the alternate low-latency provider path in LaunchDeck.

It requires:

- `HELLOMOON_API_KEY`
- positive compute-unit price
- tip that satisfies the active Hello Moon path
- `skipPreflight=true`

Hello Moon modes in practice:

- `off`: standard QUIC path without the stronger Hello Moon protection route
- `reduced`: QUIC path with the reduced-protection behavior used by the app
- `secure`: bundle-oriented Hello Moon path with bundle constraints and the stronger protection-focused route

Current runtime behavior:

- non-secure Hello Moon sends use the QUIC path
- secure Hello Moon sends use the Hello Moon bundle path
- warm logic follows the active mode, so secure and non-secure Hello Moon legs warm different endpoints when needed

Bundle-path note:

- Hello Moon bundle mode requires at least one valid Hello Moon tip in the bundle
- LaunchDeck validates that locally before submission

## Standard RPC

`Standard RPC` is the plain RPC send path.

Use it when you want:

- explicit RPC semantics
- no Sender or bundle-specific provider requirements
- the optimized LaunchDeck standard-RPC fanout transport

Current behavior:

- `skipPreflight=true`
- `maxRetries=0`
- no provider tip handling
- optional submit fanout through `LAUNCHDECK_EXTRA_STANDARD_RPC_SEND_URLS`

## Jito Bundle

`Jito Bundle` is the bundle-oriented provider path.

Use it when you want:

- bundle semantics
- ordered grouped execution
- Jito block-engine routing

Current behavior:

- bundle submission with status polling
- regional bundle endpoint filtering
- provider-specific bundle/tip rules enforced by the engine

## Full endpoint catalog

This section is the concrete endpoint reference.

### Helius Sender

Send path uses `/fast`. Warm path uses `/ping`.

| Key | Location | Send URL | Warm URL |
| --- | --- | --- | --- |
| `global` | Global | `https://sender.helius-rpc.com/fast` | `https://sender.helius-rpc.com/ping` |
| `slc` | Salt Lake City | `http://slc-sender.helius-rpc.com/fast` | `http://slc-sender.helius-rpc.com/ping` |
| `ewr` | Newark | `http://ewr-sender.helius-rpc.com/fast` | `http://ewr-sender.helius-rpc.com/ping` |
| `lon` | London | `http://lon-sender.helius-rpc.com/fast` | `http://lon-sender.helius-rpc.com/ping` |
| `fra` | Frankfurt | `http://fra-sender.helius-rpc.com/fast` | `http://fra-sender.helius-rpc.com/ping` |
| `ams` | Amsterdam | `http://ams-sender.helius-rpc.com/fast` | `http://ams-sender.helius-rpc.com/ping` |
| `sg` | Singapore | `http://sg-sender.helius-rpc.com/fast` | `http://sg-sender.helius-rpc.com/ping` |
| `tyo` | Tokyo | `http://tyo-sender.helius-rpc.com/fast` | `http://tyo-sender.helius-rpc.com/ping` |

### Helius Solana RPC and websocket

Recommended LaunchDeck values:

| Usage | Example |
| --- | --- |
| `SOLANA_RPC_URL` | `https://beta.helius-rpc.com/?api-key=YOUR_API_KEY` |
| `SOLANA_WS_URL` | `wss://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY` |
| optional `HELIUS_RPC_URL` | `https://beta.helius-rpc.com/?api-key=YOUR_API_KEY` |
| optional `HELIUS_WS_URL` | `wss://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY` |

Put your Helius key immediately after `api-key=`.

### Hello Moon QUIC endpoints

Used for non-secure Hello Moon execution.

| Key | Location | QUIC endpoint |
| --- | --- | --- |
| `global` | Global | `lunar-lander.hellomoon.io:16888` |
| `fra` | Frankfurt | `fra.lunar-lander.hellomoon.io:16888` |
| `ams` | Amsterdam | `ams.lunar-lander.hellomoon.io:16888` |
| `nyc` | New York | `nyc.lunar-lander.hellomoon.io:16888` |
| `ash` | Ashburn | `ash.lunar-lander.hellomoon.io:16888` |
| `tyo` | Tokyo | `tyo.lunar-lander.hellomoon.io:16888` |

### Hello Moon HTTP endpoints

Used for Hello Moon HTTP send and bundle paths.

| Location | `/send` | `/sendBundle` |
| --- | --- | --- |
| Frankfurt | `http://fra.lunar-lander.hellomoon.io/send` | `http://fra.lunar-lander.hellomoon.io/sendBundle` |
| Amsterdam | `http://ams.lunar-lander.hellomoon.io/send` | `http://ams.lunar-lander.hellomoon.io/sendBundle` |
| New York | `http://nyc.lunar-lander.hellomoon.io/send` | `http://nyc.lunar-lander.hellomoon.io/sendBundle` |
| Ashburn | `http://ash.lunar-lander.hellomoon.io/send` | `http://ash.lunar-lander.hellomoon.io/sendBundle` |
| Tokyo | `http://tyo.lunar-lander.hellomoon.io/send` | `http://tyo.lunar-lander.hellomoon.io/sendBundle` |
| Global | `http://lunar-lander.hellomoon.io/send` | `http://lunar-lander.hellomoon.io/sendBundle` |

### Jito block engine

| Region key | Base URL |
| --- | --- |
| `mainnet` | `https://mainnet.block-engine.jito.wtf` |
| `ny` / `new-york` | `https://ny.mainnet.block-engine.jito.wtf` |
| `slc` / `salt-lake-city` | `https://slc.mainnet.block-engine.jito.wtf` |
| `frankfurt` | `https://frankfurt.mainnet.block-engine.jito.wtf` |
| `amsterdam` | `https://amsterdam.mainnet.block-engine.jito.wtf` |
| `london` | `https://london.mainnet.block-engine.jito.wtf` |
| `dublin` | `https://dublin.mainnet.block-engine.jito.wtf` |
| `singapore` | `https://singapore.mainnet.block-engine.jito.wtf` |
| `tokyo` | `https://tokyo.mainnet.block-engine.jito.wtf` |

### Shyft RPC

Common Shyft regional RPC patterns:

| Key | Example |
| --- | --- |
| `fra` | `https://rpc.fra.shyft.to?api_key=YOUR_API_KEY` |
| `ams` | `https://rpc.ams.shyft.to?api_key=YOUR_API_KEY` |
| `sgp` | `https://rpc.sgp.shyft.to?api_key=YOUR_API_KEY` |
| `va` | `https://rpc.va.shyft.to?api_key=YOUR_API_KEY` |
| `ny` | `https://rpc.ny.shyft.to?api_key=YOUR_API_KEY` |

## Override variables

These bypass normal profile-based routing:

- `HELIUS_SENDER_ENDPOINT`
- `HELIUS_SENDER_BASE_URL`
- `HELLOMOON_QUIC_ENDPOINT`
- `JITO_BUNDLE_BASE_URLS`
- `JITO_SEND_BUNDLE_ENDPOINT`
- `JITO_BUNDLE_STATUS_ENDPOINT`

Use them only when you intentionally want to force one endpoint or one private integration.

## Related docs

- `docs/CONFIG.md`
- `docs/ENV_REFERENCE.md`
- `docs/FOLLOW_DAEMON.md`
- `docs/EXECUTION_DOS_AND_DONTS.md`

