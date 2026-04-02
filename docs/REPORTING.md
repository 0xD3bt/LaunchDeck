# Reporting

This page explains how LaunchDeck stores and presents historical launch data, transaction reports, and follow-action outcomes.

## Where Reports Live

By default, LaunchDeck stores reports under:

- `.local/launchdeck/send-reports`

Related local data:

- `.local/launchdeck/app-config.json`
- `.local/launchdeck/image-library.json`
- `.local/launchdeck/lookup-tables.json`
- `.local/launchdeck/follow-daemon-state.json`
- `.local/launchdeck/uploads/`

You can override the report location with `LAUNCHDECK_SEND_LOG_DIR`.

## History In The UI

Open `History` from the main app to browse saved activity.

The History interface exposes two views:

- `Transactions`
- `Launches`

Use `Transactions` when you want to inspect raw execution output for a run.

Use `Launches` when you want the higher-level launch history and reuse flow.

Current report detail highlights in the UI include:

- `Winning Endpoint` for the endpoint that actually landed a multi-endpoint send first
- `Attempted Endpoints` for the full fanout list when the transport tried more than one endpoint
- `Auto-Fee Sources` for the live fee inputs that contributed to the resolved auto-fee result

Operator meaning:

- `Winning Endpoint` is the endpoint that appears to have landed or returned first on a multi-endpoint path
- `Attempted Endpoints` shows the full send fanout list, not just the winner
- watcher mode fields in follow data tell you whether LaunchDeck used `helius-transaction-subscribe`, `standard-ws`, or another watcher mode for that action family

## What Reports Capture

LaunchDeck reports are meant to answer two different questions:

- what did I ask LaunchDeck to do
- what actually happened on the wire

Typical report data includes:

- requested provider
- resolved provider
- transport type
- endpoint or endpoint profile information
- winning endpoint and attempted endpoint list where applicable
- send order
- transaction signatures
- confirmation state
- applied tip and compute-unit settings
- auto-fee source summaries when auto-fee was used
- benchmark timing data
- optional send/confirm block-height snapshots when `execution.trackSendBlockHeight` is enabled

When follow behavior is enabled, reports can also include:

- follow-job snapshot
- follow-action outcomes
- watcher health
- watcher mode
- timing profiles
- follow telemetry samples

## Timing Breakdown

LaunchDeck separates total visible latency from backend work.

Key timing fields:

- `total`
  Full click-to-finish time from the operator perspective.
- `backendTotal`
  Rust-side processing time after the request is received.
- `preRequest`
  Browser-side wait before `/api/run` is dispatched.

You may also see compile and send breakdowns such as:

- `altLoad`
- `blockhash`
- `global`
- `followUpPrep`
- `serialize`
- `submit`
- `confirm`

This helps distinguish:

- metadata upload delay
- backend compile time
- chain confirmation time

Benchmark detail levels:

- `off` disables benchmark payloads for the report
- `light` records core timings without benchmark-only diagnostics
- `full` records grouped timings plus reporting-overhead timing

Legacy compatibility:

- `basic` is still accepted and maps to `light`
- the UI labels the current non-full mode as `Light`

`LAUNCHDECK_BENCHMARK_MODE` controls this report timing detail. It does not enable block-height capture by itself.

For block-height capture in reports:

- use `execution.trackSendBlockHeight` per launch or preset
- or set `LAUNCHDECK_TRACK_SEND_BLOCK_HEIGHT=true` to make that the default when `LAUNCHDECK_BENCHMARK_MODE=full`
- `off` and `light` keep block-height capture off by default, but an explicit `execution.trackSendBlockHeight` value can still enable it intentionally

## Follow Telemetry

When the daemon is involved, reports can capture more than a single launch outcome.

Examples:

- trigger type
- delay and jitter settings
- launch-to-action latency
- submit latency
- confirm latency
- confirmed-block timing
- action outcome and quality labels

Timing profile summaries can include:

- `P50 Submit`
- `P75 Submit`
- `P90 Submit`

These values are historical visibility data, not automatic throttles.

## Reuse And Relaunch

History is also an operator workflow tool, not just an audit log.

From the UI you can:

- `Reuse` an entry to load its values back into the current form
- `Relaunch` from a previous entry

Use `Reuse` when you want to edit a prior launch before sending again.

Use `Relaunch` when you want to repeat a prior flow more directly.

## What Reports Are Good For

Reports are especially useful for:

- checking which provider was actually used
- confirming whether the engine changed a setting during transport planning
- comparing compile time versus confirm time
- reviewing follow-action outcomes separately from the launch itself
- debugging whether a failure happened before send, at submit, or at confirmation

## Practical Review Order

If a run behaves unexpectedly, review it in this order:

1. provider and transport section
2. signatures and confirmation state
3. benchmark timings
4. follow-job outcomes if follow actions were enabled
5. raw output for exact backend messages
