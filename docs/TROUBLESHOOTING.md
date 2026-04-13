# Troubleshooting

This page covers the most common LaunchDeck problems and what to check first.

## First checks

Before debugging deeper, confirm:

- `npm start` completed successfully
- the main host is reachable at `http://127.0.0.1:8789`
- the follow daemon is reachable at `http://127.0.0.1:8790`
- `.env` includes `SOLANA_RPC_URL`, `SOLANA_WS_URL`, and at least one `SOLANA_PRIVATE_KEY*`

## No wallets in the UI

Common causes:

- no `SOLANA_PRIVATE_KEY` values are set
- the private key format is invalid
- `.env` changed after startup and the runtime was not restarted

What to do:

1. check `.env`
2. confirm `SOLANA_PRIVATE_KEY` or another wallet slot is present
3. run `npm restart`

## Follow daemon not ready

Symptoms:

- delayed snipers do not arm
- auto-sell does not trigger
- the UI reports daemon readiness issues

What to check:

- `LAUNCHDECK_FOLLOW_DAEMON_URL`
- `LAUNCHDECK_FOLLOW_DAEMON_PORT`
- `LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN`
- `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS`

Practical note:

- blank or `0` on the capacity limits means uncapped

## Realtime follow timing is poor

Common causes:

- missing or poor `SOLANA_WS_URL`
- websocket instability
- region mismatch between your VPS and your provider endpoints
- using a weaker Helius tier for watcher-heavy operation

What to do:

1. set `SOLANA_WS_URL` explicitly
2. set `USER_REGION` to your nearest region or exact metro
3. keep `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true`
4. use Helius dev tier if you are running multiple snipes or watcher-heavy follow logic

## Helius transactionSubscribe did not activate

Symptoms:

- `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE` is on
- reports or runtime still show standard websocket watcher behavior

What to check:

1. confirm the active watcher websocket is actually Helius
2. confirm the env change was applied with `npm restart`
3. confirm the watcher path is using websocket at all
4. remember that LaunchDeck probes first and falls back automatically if unsupported

Important:

- this is driven by the watcher websocket path, not by the send provider alone

## Helius Sender rejection

Common causes:

- `skipPreflight` is not true
- compute-unit price is zero or missing
- tip is below Sender minimum

Current Sender requirements:

- `execution.skipPreflight=true`
- positive compute-unit price
- tip of at least `200000` lamports

Recommended Sender stack:

1. use Helius Gatekeeper HTTP for `SOLANA_RPC_URL`
2. use Helius standard websocket for `SOLANA_WS_URL`
3. use Shyft for `LAUNCHDECK_WARM_RPC_URL`
4. use Helius dev tier if you care about the best runtime behavior

### Sender warm checks fail

LaunchDeck warms Sender hosts through `GET /ping` on the Sender host, not through `SOLANA_RPC_URL`.

If Sender warm fails while normal JSON-RPC looks fine, check:

- outbound access to `*-sender.helius-rpc.com`
- TLS or proxy interception
- firewall rules blocking those Sender hosts

## Hello Moon problems

Common causes:

- missing `HELLOMOON_API_KEY`
- tip or priority-fee requirements not met
- secure bundle path being used without a valid bundle tip
- Hello Moon routing assumptions not matching actual metro fallback behavior

What to check:

1. confirm `HELLOMOON_API_KEY` is set
2. confirm the active Hello Moon mode is the one you intended
3. remember that Hello Moon `us`, `slc`, and `ewr` route to New York + Ashburn
4. remember that Hello Moon `asia` and `sg` route to Tokyo

## Standard RPC is not using tip

This is expected.

`standard-rpc` does not use provider tip handling even if an old preset still has a tip value stored.

Current transport behavior:

- `skipPreflight=true`
- `maxRetries=0`
- optional fanout through `LAUNCHDECK_EXTRA_STANDARD_RPC_SEND_URLS`

## Jito Bundle is acting differently than creation settings suggest

This can be expected.

Bundle transports may apply engine-owned shaping rules that differ from plain single-send behavior.

Check the saved report to see:

- the actual transport used
- the actual endpoint list
- the actual fee/tip behavior

## Unsupported launchpad or mode

Examples:

- Bonk with unsupported Pump-only modes
- Bagsapp with unsupported quote asset or mode combinations

These combinations are rejected by backend validation. Fix the launchpad or mode choice rather than retrying the same request.

## Fee-sharing validation failure

Common causes:

- recipients do not total `10000` bps
- later setup is enabled without recipients
- later setup is used outside the supported mode

What to check:

- recipient percentages
- selected mode
- fee-sharing settings

## Pump `creator_vault` / `Custom 2006` retries

Current behavior:

- Pump pre-signed sniper buys can rebuild/retry automatically on the special `creator_vault` / `Custom 2006` failure path
- Pump dev-auto-sells and Pump sniper-sells can also rebuild/retry automatically on that same failure path
- both of those recovery paths default to enabled

What to check:

- whether the report or follow-daemon state shows `creator_vault`, `ConstraintSeeds`, or `Custom: 2006`
- `LAUNCHDECK_ENABLE_PUMP_BUY_CREATOR_VAULT_AUTO_RETRY`
- `LAUNCHDECK_ENABLE_PUMP_SELL_CREATOR_VAULT_AUTO_RETRY`

What to do:

1. leave both flags enabled unless you intentionally want LaunchDeck to stop resending that recovery buy or sell
2. if you want to disable the recovery resend path, set the relevant flag to `false` and restart with `npm restart`
3. inspect the saved report / follow-daemon state to confirm whether the action failed before retry, retried, or stayed failed after the retry path was disabled

## postBuySell semantics

`followLaunch.snipes[].postBuySell` is supported in the current runtime.

Current behavior:

- slot offsets mean `+N slots after the matching buy confirms`
- market-cap autosells start watching after the matching buy confirms
- older saved/imported payloads keep the same field names and now follow the same buy-relative meaning

## Metadata upload problems

Common causes:

- `PINATA_JWT` missing while provider is `pinata`
- Pinata failed and LaunchDeck fell back to `pump-fun`

What to check:

- `LAUNCHDECK_METADATA_UPLOAD_PROVIDER`
- `PINATA_JWT`
- the UI warning message
- the final report output

## Market-cap follow trigger did not fire

Common causes:

- the threshold or timeout action was configured differently than you expected
- no usable Helius RPC URL was available for the primary SOL price lookup
- outbound access to the configured SOL/USD HTTP price source failed
- the market-cap watch timed out before the threshold was reached

What to check:

- `HELIUS_RPC_URL`
- whether `SOLANA_RPC_URL` is already Helius-hosted
- `LAUNCHDECK_SOL_USD_HTTP_PRICE_URL`
- follow-job outcome details in the saved report
- daemon logs for price-source or watcher errors

## Bonk or Bags helper worker problems

What to check:

- `LAUNCHDECK_ENABLE_BONK_HELPER_WORKER`
- `LAUNCHDECK_ENABLE_BAGS_HELPER_WORKER`
- `LAUNCHDECK_LAUNCHPAD_HELPER_TIMEOUT_MS`
- `LAUNCHDECK_LAUNCHPAD_HELPER_MAX_CONCURRENCY`

What to do:

1. leave helper workers on unless you have a specific reason not to
2. if worker mode is behaving badly, temporarily set the relevant helper worker flag to `false` and restart
3. increase helper timeout if legitimate helper work is timing out

## Report and local state inspection

When the UI is unclear, inspect:

- `.local/launchdeck/send-reports`
- `.local/launchdeck/follow-daemon-state.json`
- `.local/launchdeck/app-config.json`
- `.local/launchdeck/lookup-tables.json`

Use these to confirm:

- which provider actually ran
- which endpoints were attempted
- whether the follow daemon armed the job
- whether watcher or warm paths were healthy

## When to restart

Restart LaunchDeck after changing:

- wallet env vars
- RPC or websocket URLs
- region overrides
- metadata provider credentials
- provider integration keys

Use:

```bash
npm restart
```

## Related docs

- `docs/CONFIG.md`
- `docs/ENV_REFERENCE.md`
- `docs/PROVIDERS.md`
- `docs/FOLLOW_DAEMON.md`
- `docs/VPS_SETUP.md`

