# Strategies

## Shared Post-Launch Strategies

- `none`
- `dev-buy`
- `snipe-own-launch`
- `automatic-dev-sell`

## Snipe Own Launch

Intent:

- submit one or more follow-up buy transactions around a launch
- support immediate same-time submits and daemon-managed delayed submits
- keep wallet-level sniper behavior configurable without blocking normal launch flow

Current sniper trigger modes:

- `Same Time`
- `On Submit + Delay`
- `Block Offset`

Current behavior:

- `Same Time` submits alongside the launch creation path
- `On Submit + Delay` schedules from observed launch submit time
- `Block Offset` sends when the configured observed block is reached
- current block-offset range is `0-5`
- same-time sniper rows can arm a one-time fallback retry through the daemon
- same-time rows show an inline safeguard if sniper buy fees are higher than launch fees

This strategy should only be enabled where the selected launchpad supports the required follow-up buy flow.

## Automatic Dev Sell

Intent:

- sell a configured percentage of the dev wallet after launch
- support launch-relative timing rather than only fixed post-launch delay
- keep execution inside the dedicated follow-daemon path

Current trigger modes:

- `On Submit + Delay`
- `Block Offset`

Current behavior:

- `On Submit + Delay` supports `0ms` for immediate post-submit scheduling
- `Block Offset` sends when the configured launch-relative block is observed
- the UI keeps automatic dev sell state persisted across refreshes
- agent-custom and agent-locked sell flows prefer the post-setup creator-vault authority path

Validation targets:

- percent: `1-100`

## Follow Sells

Sniper wallets can also chain sell behavior after a buy.

Current supported follow-sell trigger types include:

- delay-based sell timing
- market-cap based sell timing

These actions are daemon-executed and reported independently from the original sniper buy.
