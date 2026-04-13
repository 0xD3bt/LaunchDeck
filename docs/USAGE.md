# Usage Guide

This guide covers the normal operator workflow in LaunchDeck, from first startup through deploy, history, and reuse.

Before using the UI, set up the basics from `.env.example` and `docs/CONFIG.md`.

If you are bringing up a brand-new server first, use `docs/VPS_SETUP.md` before this guide. That page covers the initial VPS instance, dependency install, bootstrap script, and SSH-tunnel workflow.

## Before you start

Make sure you have:

- configured `SOLANA_RPC_URL`
- configured `SOLANA_WS_URL`
- set `USER_REGION`
- loaded at least one wallet through `SOLANA_PRIVATE_KEY*`
- started the runtime with `npm start`

Default UI URL:

- `http://127.0.0.1:8789`

Useful setup paths:

- local or existing machine: use `README.md` plus `.env.example`
- fresh VPS: use `docs/VPS_SETUP.md`, then come back here once the UI is reachable

## Recommended first session

For the first live session, keep it simple:

1. load one wallet
2. choose `Pump` or `Bonk`
3. use the recommended stack: Helius Gatekeeper HTTP for `SOLANA_RPC_URL`, Helius standard websocket for `SOLANA_WS_URL`, Shyft for `LAUNCHDECK_WARM_RPC_URL`
4. leave the provider on `Helius Sender`
5. keep follow actions off
6. run `Build`
7. run `Simulate`
8. only then run `Deploy`

## Main UI flow

The normal LaunchDeck workflow is:

1. select a wallet
2. choose a launchpad and mode
3. fill token metadata or import an existing token with `Vamp`
4. choose or upload an image through the image library
5. review creation, buy, and sell settings in the Settings modal
6. optionally add snipers, fee split, vanity, or automatic sell logic
7. run `Build`, `Simulate`, or `Deploy`
8. review the saved result in `Dashboard`

## Runtime indicators

The launchpad row shows runtime status indicators before you send anything.

Current operator meaning:

- `Warm` summarizes startup warm, active endpoint warm, and watcher warm health
- green usually means the active warm targets are healthy
- yellow or blue usually means LaunchDeck is still starting, auto-paused, or partially degraded
- red means at least one active warm target is failing and you should inspect the tooltip or logs before trusting the path

The `Dashboard` button also reflects follow-daemon health and active job count.

## Image library and Vamp

LaunchDeck now treats images and imported metadata as first-class workflow tools.

Use the image panel to:

- upload a new image
- reuse an existing image from the local image library
- organize saved images with favorites, categories, and tags

Use `Vamp` when you want to seed the form from an existing token.

Current behavior:

- paste a Solana mint / contract address
- LaunchDeck imports the token image and metadata into the current form
- imported values can include the name, symbol, description, and social links
- the imported image is stored in the local image library so you can reuse it later

## Wallets

Wallets are loaded from:

- `SOLANA_PRIVATE_KEY`
- `SOLANA_PRIVATE_KEY2`
- `SOLANA_PRIVATE_KEY3`
- and any additional `SOLANA_PRIVATE_KEY<number>` slots you define

Practical notes:

- the selected wallet becomes the deployer wallet
- sniper rows can use other loaded wallet slots
- labeled wallet syntax is supported as `<privatekey>,<label>`

If the wallet list is empty, fix `.env` and restart the runtime.

## Launchpad and mode

Current launchpads:

- `Pump`
- `Bonk`
- `Bagsapp`

Practical guidance:

- use `Pump` for the most native LaunchDeck path
- use `Bonk` for the supported helper-backed Bonk path
- use `Bagsapp` when you want the supported Bags path and have Bags credentials configured
- the shipped UI treats Bags identity as `wallet-only` for the normal operator flow

See `docs/LAUNCHPADS.md` for the exact support matrix.

## Token metadata

Required:

- token name
- token symbol
- image

The launch cannot proceed without a token URI, so metadata upload needs to complete before deploy.

Optional fields:

- description
- website
- twitter
- telegram

## Metadata upload flow

Current providers:

- `pump-fun`
- `pinata`

Current behavior:

- blank metadata provider means `pump-fun`
- `pinata` requires `PINATA_JWT`
- Pinata failures fall back to `pump-fun`
- the UI surfaces that fallback as a warning

## Presets and settings

LaunchDeck uses three presets:

- `Preset 1`
- `Preset 2`
- `Preset 3`

Each preset stores:

- creation settings
- buy settings
- sell settings

The settings modal is where you set your normal defaults for:

- provider
- MEV mode where applicable
- tip
- priority fee
- auto-fee
- max auto fee
- slippage

Those settings persist locally in the app config.

## Provider selection

Current provider choices:

- `Helius Sender`
- `Hello Moon`
- `Standard RPC`
- `Jito Bundle`

Recommended usage:

- start with `Helius Sender`
- use `Hello Moon` when you want the alternate low-latency path
- use `Standard RPC` when you want plain RPC behavior
- use `Jito Bundle` when you explicitly want bundle behavior

Provider details live in `docs/PROVIDERS.md`.

## Dev buy

Dev buy is the deployer-wallet buy that runs as part of the launch flow.

Use it when you want the deployer wallet to buy immediately on launch.

This is separate from sniper rows.

## Snipers

Each sniper row controls:

- wallet
- buy amount
- trigger mode
- retry behavior where supported

Current trigger modes:

- `Same Time`
- `On Submit + Delay`
- `On Confirmed Block`

Practical usage:

- use `Same Time` only when you intentionally want the buy sent alongside launch creation
- use `On Submit + Delay` when you want a delay from observed submit time
- use `On Confirmed Block` when you want the safest normal default

## Automatic dev sell

Automatic dev sell is configured separately from sniper rows.

It lets you:

- enable or disable sell behavior for the deployer wallet
- choose a percent
- choose time-based or market-cap-based triggers
- choose delay-based or confirmed-block timing for time-triggered sells
- use market-cap triggers where supported

This action is daemon-executed and shows separately in reports.

## Build, Simulate, Deploy

LaunchDeck exposes three main actions:

- `Build`
- `Simulate`
- `Deploy`

Use them like this:

- `Build`: inspect the planned launch without sending it
- `Simulate`: check the launch through RPC simulation
- `Deploy`: send it live

The backend still owns final validation and transport shaping even if the UI fields looked valid.

## Dashboard and reuse

Open `Dashboard` when you want to inspect prior launches, transactions, live jobs, and recent logs.

Current dashboard views:

- `Transactions`
- `Launches`
- `Jobs`
- `Logs`

From `Launches` you can:

- inspect the report
- review timings and endpoints
- reuse a launch into the current form
- relaunch from saved history

Use `Jobs` when you want to watch active follow launches and cancel them if needed.

Use `Logs` when you want quick browser-side access to recent engine and daemon output before dropping to `journalctl`.

Reporting details live in `docs/REPORTING.md`.

## Follow actions

Follow behavior is handled by the dedicated follow daemon.

That includes:

- delayed buys
- confirmed-block buys
- automatic dev sell
- snipe sells
- watcher-driven follow behavior

Follow details live in `docs/FOLLOW_DAEMON.md`.

## When to restart

Restart LaunchDeck after changing:

- wallet env vars
- RPC URLs
- websocket URLs
- region overrides
- metadata provider credentials
- provider integration keys

Use:

```bash
npm restart
```

