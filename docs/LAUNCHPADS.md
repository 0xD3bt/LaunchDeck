# Launchpads

This page documents the launchpads exposed in LaunchDeck and separates the supported paths from the experimental ones.

Treat this page as the most explicit public support matrix for launchpad behavior. Other pages such as `README.md` and `ARCHITECTURE.md` summarize the same surface more briefly.

## Support Summary

### Supported

- `pump`
- `bonk`

### Experimental

- `bagsapp` when Bags credentials are configured

If you want the most predictable path, start with Pump or Bonk before trying Bagsapp.

## Pump

Status: `supported`

Pump is the most native launch path in LaunchDeck today.

Supported Pump coverage:

- `regular`
- `cashback`
- `agent-custom`
- `agent-unlocked`
- `agent-locked`
- immediate dev buy
- same-time sniper buys
- delayed sniper buys
- snipe sells
- automatic dev sell

Current Pump characteristics:

- launch assembly is native Rust
- transaction shaping is native Rust
- versioned transaction choice is measured rather than hard-coded
- default lookup tables are warmed and persisted locally
- immediate dev-buy support can include the account extension and ATA setup needed before the buy path

### Pump Mode Guide

#### `regular`

Use this for the standard Pump launch path.

- normal Pump creation flow
- creator fee stays on the deployer by default
- can include an immediate dev buy
- can optionally generate later fee-sharing setup

#### `cashback`

Use this for the Pump cashback creation variant.

- same core creation flow as `regular`
- cashback behavior is marked in the Pump creation payload
- later fee-sharing setup can still be used when configured

#### `agent-unlocked`

Use this when you want an agent launch without a later agent setup transaction.

- agent is initialized during creation
- creator rewards are not rerouted into the locked escrow model
- no follow-up `agent-setup` transaction is emitted
- `buybackBps` is required

#### `agent-custom`

Use this when you want a custom post-launch recipient setup.

- agent is initialized during creation
- final custom recipient setup is deferred to `agent-setup`
- the follow-up applies the configured recipient split
- `buybackBps` is required

#### `agent-locked`

Use this when you want the locked agent escrow path.

- agent is initialized during creation
- creator rewards route through the locked escrow model
- a follow-up `agent-setup` transaction is emitted
- `buybackBps` is required

### Pump Restrictions And Rules

- `feeSharing.generateLaterSetup` is supported only in `regular`
- later fee-sharing setup requires fee recipients
- fee-sharing recipients must total `10000` bps
- agent modes require `buybackBps`

## Bonk

Status: `supported`

Supported Bonk coverage:

- `regular`
- `bonkers`
- quote asset `sol`
- quote asset `usd1`
- immediate dev buy
- same-time sniper buys
- delayed sniper buys
- snipe sells
- automatic dev sell

Current Bonk characteristics:

- validation, transport planning, reporting, simulation, and send orchestration are Rust-owned
- launch assembly uses the Raydium LaunchLab-backed helper bridge
- `regular` routes through LetsBonk
- `bonkers` routes through the Bonkers path on Raydium LaunchLab
- `usd1` uses a pinned Raydium `SOL -> USD1` route pool for top-up behavior when the wallet needs USD1 before buying
- `usd1` same-time sniper buys are assembled as atomic swap-and-buy transactions
- same-time sniper buys use launch-first safeguards so the buy path does not outrun the creation path
- immediate dev buy on `usd1` attempts atomic launch-plus-buy assembly and can fall back to split transactions if the combined message is too large

### Bonk Restrictions

- only `regular` and `bonkers` are supported
- Pump-only modes such as `cashback`, `agent-custom`, `agent-unlocked`, and `agent-locked` are rejected
- fee-sharing setup is rejected
- `mayhem` is rejected
- per-sniper `postBuySell` chaining is not supported yet

## Bagsapp

Status: `experimental` (`configured-required` until Bags credentials are present; `unverified` once enabled)

Bagsapp is available when Bags credentials are configured, but it is still experimental in this repo.

Current Bags behavior includes:

- `bags-2-2`
- `bags-025-1`
- `bags-1-025`
- quote asset `sol`
- wallet-only identity
- linked identity when the selected LaunchDeck wallet belongs to the authenticated Bags account
- immediate dev buy
- same-time sniper buys
- delayed sniper buys
- snipe sells
- automatic dev sell
- LaunchDeck fee-split UI mapped into Bags fee-claimer rows

Current Bags characteristics:

- launch assembly uses the hosted Bags API or SDK bridge
- linked identity is validated against the selected LaunchDeck wallet
- same-time buy compilation is deferred until after launch submission so the trade route can resolve against the live mint
- history persists the identity mode and display name, but not sensitive auth material

### Bags Restrictions

- `sol` is the only current quote asset
- Pump-only modes are rejected
- `mayhem` is rejected
- creator fee must remain the deployer wallet

## Launchpad Choice Guidance

Choose `pump` if you want the most native LaunchDeck path.

Choose `bonk` if you want the supported Bonk and Bonkers path, including `usd1` and follow automation.

Choose `bagsapp` only if you are comfortable using an experimental path and have Bags credentials configured.
