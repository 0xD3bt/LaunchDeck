# Usage Guide

This guide walks through the normal operator workflow in LaunchDeck, from first startup through deploy, follow actions, and reuse.

## Before You Start

Make sure you have:

- configured `SOLANA_RPC_URL`
- configured `SOLANA_WS_URL`
- set `USER_REGION` to your nearest regional group or explicit metro list so region-aware providers can fan out across that selected endpoint set instead of relying on one pinned host
- imported at least one wallet through `SOLANA_PRIVATE_KEY*`
- started LaunchDeck with `npm start`

Default UI URL:

- `http://127.0.0.1:8789`

## Main Workflow

The normal LaunchDeck flow is:

1. select a wallet
2. choose a launchpad and mode
3. configure token metadata
4. review preset-backed execution settings
5. optionally add snipers or auto-sell
6. `Build`, `Simulate`, or `Deploy`
7. review the output and inspect History if needed

## Wallet Selection

Wallets are loaded from `SOLANA_PRIVATE_KEY*`.

In practice:

- the selected wallet becomes the deployer wallet
- sniper rows can target different wallet env keys
- Bags linked identity checks are tied to the currently selected LaunchDeck wallet

If your wallet list is empty, fix your env file before using the UI.

## Choose A Launchpad And Mode

LaunchDeck exposes:

- `Pump`
- `Bonk`
- `Bagsapp`

Typical choices:

- use `Pump` for the most native LaunchDeck path
- use `Bonk` for the supported Bonk and Bonkers path
- use `Bagsapp` only if you are comfortable using an experimental path

Mode selection changes the rest of the form. For example:

- Pump exposes `regular`, `cashback`, and agent modes
- Bonk exposes `regular` and `bonkers`
- Bagsapp exposes the Bags fee-mode variants

## Fill Token Metadata

Required fields:

- token name
- token symbol
- image

The launch cannot normalize successfully without a token URI, so LaunchDeck expects metadata upload to complete before deploy.

Optional fields:

- description
- website
- twitter
- telegram

### Image Library Workflow

The app includes an `Image Library` so you can reuse uploaded media instead of re-uploading each time.

Typical image flow:

1. open the image picker
2. upload or select an existing image
3. optionally tag, categorize, or favorite it
4. confirm the selection for the current launch

The image library is stored locally in `image-library.json`.

### Metadata Pre-Upload

When enough metadata is present, the UI can begin metadata upload before deploy.

That helps reduce total launch latency because the final deploy path does not have to wait as long for metadata upload.

## Presets And Settings

LaunchDeck uses three presets:

- `Preset 1`
- `Preset 2`
- `Preset 3`

Each preset stores three execution groups:

- creation settings
- buy settings
- sell settings

The settings modal exposes:

- provider
- tip
- priority fee
- slippage for buy and sell
- auto-fee
- max auto fee

If you want to change your default execution setup, edit your presets in Settings. That is the intended place to set your usual provider, fee, slippage, and auto-fee defaults.

Use presets when:

- you want one aggressive setup and one safer setup
- you switch between providers regularly
- you want quick access to different dev-buy amounts

## Dev Buy

Dev buy is the deployer wallet buy that happens as part of the launch flow.

Use it when you want the deployer wallet to buy immediately on launch.

This is separate from snipers:

- dev buy is part of the launch
- snipers are follow actions or same-time sniper buys from specific wallets

## Snipers

The sniper UI lets you configure sniper buys per wallet.

Each sniper row can control:

- target wallet
- buy amount
- trigger mode
- same-time retry behavior where supported

Current trigger modes:

- `Same Time`
- `On Submit + Delay`
- `On Confirmed Block`

Use `Same Time` when you want the buy sent alongside launch creation.

Use `On Submit + Delay` when you want the daemon to fire after observed submit time.

Use `On Confirmed Block` when you want the safest default buy timing. This is the mode we suggest first for most users.

## Automatic Dev Sell

Automatic dev sell is configured separately from sniper rows.

It lets you:

- enable or disable dev-wallet sell behavior
- set the sell percent
- choose delay-based or confirmed-block timing

Current trigger modes:

- `On Submit + Delay`
- `On Confirmed Block`

This action is daemon-executed and appears in reporting separately from the original launch.

## Bags Identity Flow

When using Bagsapp, the UI exposes a Bags identity flow.

Identity modes:

- `Wallet Only`
- `Linked Bags Identity`

Typical linked-identity flow:

1. open the Bags identity modal
2. initialize verification
3. complete the verification step
4. verify against the selected wallet

If the selected LaunchDeck wallet does not belong to the authenticated Bags account, linked mode will not remain enabled.

## Build, Simulate, And Deploy

LaunchDeck exposes three core execution actions:

- `Build`
- `Simulate`
- `Deploy`

Use `Build` when you want to inspect the planned launch shape without sending it.

Use `Simulate` when you want RPC simulation before real send.

Use `Deploy` when you want the launch sent live.

The main host normalizes the request, validates it, chooses transport behavior, and writes a report for the result.

## History, Reuse, And Relaunch

Open `History` to inspect previous runs.

The History UI has two main views:

- `Transactions`
- `Launches`

From History you can:

- inspect a previous run
- review report tabs
- reuse a launch into the current form
- relaunch from a previous saved entry

Detailed reporting guidance: `REPORTING.md`

## Recommended First Session

For a safe first session:

1. set up one wallet
2. choose `Pump` or `Bonk`
3. use the recommended stack first: Helius Gatekeeper HTTP for `SOLANA_RPC_URL`, Helius standard websocket for `SOLANA_WS_URL`, optional Shyft for `LAUNCHDECK_WARM_RPC_URL`, and leave the provider on `Helius Sender`
4. keep follow actions off
5. run `Build`
6. run `Simulate`
7. deploy only after the first two look correct
