# Troubleshooting

This page covers the most common operator problems in LaunchDeck and where to look when something does not behave as expected.

## First Checks

Before debugging deeper, confirm:

- `npm start` completed successfully
- the UI host is reachable at `http://127.0.0.1:8789` or your configured port
- the follow daemon is reachable at `http://127.0.0.1:8790` or your configured port
- your `.env` includes `SOLANA_RPC_URL`, `SOLANA_WS_URL`, and at least one `SOLANA_PRIVATE_KEY*`

## No Wallets In The UI

Common causes:

- no `SOLANA_PRIVATE_KEY` values are set
- the key format is invalid
- the env file was changed after startup and the runtime was not restarted

What to do:

1. check `.env`
2. confirm `SOLANA_PRIVATE_KEY` or `SOLANA_PRIVATE_KEY2` exists
3. restart with `npm restart`

## Follow Daemon Not Ready

Symptoms:

- delayed snipers do not arm
- auto-sell does not trigger
- the UI reports daemon readiness issues

Common causes:

- the daemon process is not running
- the daemon URL or port is misconfigured
- auth token mismatch between host and daemon
- daemon capacity is exhausted

What to check:

- `LAUNCHDECK_FOLLOW_DAEMON_URL`
- `LAUNCHDECK_FOLLOW_DAEMON_PORT`
- `LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN`
- `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS`

If those follow capacity vars are blank or `0`, the daemon runs uncapped and capacity exhaustion is not the problem.

## Realtime Follow Timing Is Poor

Symptoms:

- confirmed-block actions feel late
- delayed actions are inconsistent
- watcher-driven actions look stale

Common causes:

- missing or poor `SOLANA_WS_URL`
- websocket instability
- regional mismatch between your provider choice and your actual location

What to do:

1. set `SOLANA_WS_URL` explicitly
2. set `USER_REGION` to your closest region
3. prefer region fanout over pinning one explicit sender or bundle endpoint
4. if needed, use provider-specific region overrides
5. if you are on Helius dev tier and LaunchDeck is watching through a Helius websocket endpoint, enable `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true`

## Helius `transactionSubscribe` Did Not Activate

Symptoms:

- you enabled `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true`
- reports or follow state still show `standard-ws`
- follow timing does not seem to use the enhanced Helius watcher path

What to check:

1. confirm `SOLANA_WS_URL` is actually a Helius websocket endpoint
2. confirm the env change was applied with `npm restart`
3. confirm the runtime is using websocket watchers at all and did not fall back because the watch endpoint is missing
4. inspect the persisted report or follow-daemon state for watcher mode and watcher health
5. if Helius `transactionSubscribe` was attempted but rejected, expect LaunchDeck to fall back automatically to `standard-ws`

Important:

- this feature is selected from the watch endpoint, not from the send provider alone
- the env flag allows the Helius attempt; it does not disable fallback safety

## Helius Sender Rejection

Symptoms:

- the launch fails before send on Sender
- the UI settings look reasonable but the backend rejects the request

Common causes:

- `skipPreflight` is not true
- compute-unit price is zero or missing
- tip is below Sender minimum

Current Sender requirements:

- `execution.skipPreflight=true`
- `tx.computeUnitPriceMicroLamports > 0`
- `tx.jitoTipLamports >= 200000`

If you do not want Sender rules, switch to `Standard RPC`.

For the best current Sender path:

1. use Helius for `SOLANA_RPC_URL`
2. use the matching Helius websocket for `SOLANA_WS_URL`
3. use a Shyft RPC with a free API key for `LAUNCHDECK_WARM_RPC_URL`
4. use Helius dev tier if you want the strongest performance and watcher behavior

## Standard RPC Not Using Tip

This is expected.

`Standard RPC` does not use tip even if an older preset still contains a tip value. The engine ignores it for that provider.

Current transport note:

- `Standard RPC` currently resolves to the optimized `standard-rpc-fanout` transport
- it sends with `skipPreflight=true` and `maxRetries=0`
- it can fan out to `SOLANA_RPC_URL` plus extra submit endpoints from `LAUNCHDECK_STANDARD_RPC_SEND_URLS`
- the report view is the best place to confirm the winning endpoint and full attempted endpoint list

## Jito Bundle Acting Differently Than Creation Settings Suggest

This can also be expected.

The engine may intentionally change creation-side fee shaping on bundle paths, especially in multi-transaction launch flows where a priority value would only add cost without helping.

Review the persisted report to see what was actually applied.

## Bonk USD1 Route Or Buy Problems

Common causes:

- the pinned `SOL -> USD1` Raydium route pool is unavailable or no longer matches the expected config
- the wallet does not have enough SOL headroom for the required USD1 top-up
- an atomic immediate dev-buy assembly overflows and falls back to split transactions

What happens now:

- Bonk `usd1` uses a pinned `SOL -> USD1` route pool instead of silently picking another pool
- Bonk `usd1` same-time sniper buys use atomic swap-and-buy assembly
- immediate dev buy on Bonk `usd1` attempts atomic launch-plus-buy assembly first

What to check:

- the persisted report warnings for any atomic `usd1` fallback note
- wallet SOL balance after reserve requirements
- the helper error text if the pinned pool or config check fails

## Unsupported Launchpad Or Mode Combination

Typical examples:

- Bonk with `cashback`
- Bonk with Pump agent modes
- Bagsapp with non-Bags modes
- Bagsapp with non-`sol` quote asset

These combinations are rejected by config normalization. Fix the launchpad or mode choice instead of retrying the same request.

Current runtime note:

- LaunchDeck is now `rust-native-only`
- unsupported combinations do not fall back to a generic JS compile path anymore
- if a launchpad/mode pair is not in the shipped support surface, the engine should be expected to hard-fail early

## Fee-Sharing Validation Failure

Common causes:

- recipients do not total `10000` bps
- later fee-sharing setup is enabled without recipients
- later fee-sharing setup is used outside Pump `regular`

What to check:

- recipient percentages
- selected mode
- creator-fee mode

## `postBuySell` Rejected

This is expected in the current shipped runtime.

`followLaunch.snipes[].postBuySell` is not a shipped operator feature yet and is explicitly rejected by validation.

Use separate snipe sell behavior that is currently supported instead.

## Metadata Upload Problems

Common causes:

- no metadata provider configured as expected
- `PINATA_JWT` missing when [`pinata`](https://pinata.cloud/) is selected
- upload failed and fell back to `pump-fun`

What to check:

- `LAUNCHDECK_METADATA_UPLOAD_PROVIDER`
- `PINATA_JWT`
- the final report output to see which upload path was actually used

## Bags Identity Problems

Symptoms:

- linked mode will not stay enabled
- verification succeeds but the app falls back to wallet-only

Common causes:

- selected LaunchDeck wallet does not belong to the authenticated Bags account
- missing or invalid Bags auth material
- identity was not fully verified

What to check:

- `BAGS_API_KEY`
- selected wallet in the UI
- linked identity status in the Bags modal

## Bonk Or Bags Helper Worker Problems

Symptoms:

- Bonk or Bags helper-backed actions intermittently time out
- helper-backed launches work once, then fail on later requests
- worker-mode experiments behave worse than one-shot helper calls

What to check:

- `LAUNCHDECK_ENABLE_BONK_HELPER_WORKER`
- `LAUNCHDECK_ENABLE_BAGS_HELPER_WORKER`
- `LAUNCHDECK_LAUNCHPAD_HELPER_TIMEOUT_MS`
- `LAUNCHDECK_LAUNCHPAD_HELPER_MAX_CONCURRENCY`

What to do:

1. leave helper worker flags unset unless you intentionally want persistent helper processes
2. if worker mode is enabled and behaving badly, disable it and restart
3. increase `LAUNCHDECK_LAUNCHPAD_HELPER_TIMEOUT_MS` if helper-backed requests are legitimately taking longer than your current timeout

## Report And Local State Inspection

If the UI result is unclear, inspect local state:

- `.local/launchdeck/send-reports`
- `.local/launchdeck/follow-daemon-state.json`
- `.local/launchdeck/app-config.json`
- `.local/launchdeck/lookup-tables.json`

Use these when you need to answer:

- what provider was actually used
- whether the daemon armed the follow job
- whether settings persisted correctly
- whether a cache or warm-up path was active

## When To Restart

Restart LaunchDeck if you change:

- wallet env vars
- RPC or websocket env vars
- region or endpoint overrides
- metadata provider env vars
- Bags credentials

Use:

- `npm restart`

That is usually enough to pick up new env values and refresh both the main host and follow daemon.
