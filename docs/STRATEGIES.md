# Strategies

This page explains the operator-facing post-launch strategies and follow-action timing models available in LaunchDeck today.

## Shared Strategy Options

Current top-level post-launch strategies:

- `none`
- `dev-buy`
- `snipe-own-launch`
- `automatic-dev-sell`

In practice, the UI also exposes the richer `followLaunch` model for explicit sniper and sell actions.

## Dev Buy

`dev-buy` means the deployer wallet buys immediately as part of the launch flow rather than waiting for a separate delayed action.

Behavior depends on the launchpad:

- Pump can include the buy directly in the launch transaction shape
- Bonk supports immediate dev buy on the supported path
- Bagsapp supports immediate dev buy on the supported Bags path

This is not the same as a delayed sniper buy. It is part of the core launch execution flow.

## Snipe Own Launch

This strategy is for buying your own launch from one or more configured wallets.

LaunchDeck supports:

- same-time sniper buys
- delayed sniper buys from submit time
- confirmed-block sniper buys

### Trigger Modes

#### `Same Time`

Use this when your latency is high enough that waiting for observed submit timing may put you behind, for example when you are sending from a far region or a slower path to the chain.

This mode is best treated as a latency tool, not the default choice. Operators should benchmark their own setup first. If you are unsure which trigger to use, start with `On Confirmed Block`.

How it works:

- same-time sniper buys are sent alongside the launch flow
- Bonk `usd1` same-time sniper buys compile as atomic swap-and-buy transactions
- if a buy lands before the creation transaction, it fails
- same-time literally sends the launch path and selected buys at the same time
- same-time rows can arm a one-time retry through the daemon
- the retry is skipped if the wallet already holds the token
- the UI warns when same-time buy fees exceed launch fees

#### `On Submit + Delay`

Use this when you want the buy to start from observed launch submission time instead of racing creation.

How it works:

- `0ms` means send on observed submit
- positive delay values wait from observed submit plus the configured extra delay
- execution is daemon-managed rather than inline with the launch
- this mode is faster than `On Confirmed Block`, but it can still fail if the buy reaches chain execution before creation is live

#### `On Confirmed Block`

Use this when you want the safest currently shipped buy timing in LaunchDeck. This is the mode we recommend first for most users.

How it works:

- the daemon watches launch-relative block progress
- the action fires when the configured confirmed-block target is observed
- because it waits for observed launch state, it is safer than `Same Time` when your priority is execution safety rather than raw speed
- the exact allowed block-offset range is enforced by the current UI and backend validation

### Additional Sniper Behavior

- each sniper row is wallet-specific
- same-time sniper buys can optionally retry once through the daemon
- Bagsapp same-time sniper compilation happens after launch submission because the trade route needs the live mint
- if you are unsure which trigger to use, benchmark your own end-to-end latency and start with `On Confirmed Block`
- for buy-side timing, `On Confirmed Block` is the most conservative currently shipped option; confirmation-gated execution exists on supported sell-side follow actions instead

## Automatic Dev Sell

Automatic dev sell is the dev-wallet sell path managed by the follow daemon after launch.

Use it when you want:

- a percentage of the dev wallet sold after launch
- timing based on submit observation or block progression
- execution outside the main launch request path

### Trigger Modes

- `On Submit + Delay`
- `On Confirmed Block`
- `Market Cap`

How it works:

- `On Submit + Delay` supports `0ms`
- `On Confirmed Block` is watcher-driven
- `Market Cap` watches for the configured USD market-cap threshold and can stop or sell on timeout
- automatic dev sell state is persisted in UI settings
- agent-custom and agent-locked flows prefer the post-setup creator-vault authority path
- the daemon owns execution and reporting

Validation:

- percent must be within `1-100` in operator use

## Follow Sells

Follow sells are daemon-executed sells tied to a wallet or follow action after the launch has already been submitted.

Current sell trigger types include:

- delay-based timing
- market-cap based timing

These sells are reported independently from the original buy action.

## Sniper Autosell

`followLaunch.snipes[].postBuySell` is supported.

Current behavior:

- slot offsets mean `+N slots after the matching buy confirms`
- `+0` means the autosell becomes eligible immediately after that matching buy confirms
- market-cap autosells start watching only after the matching buy confirms
