# Configuration

## Environment

Provider and launchpad credentials stay env-only.

Key variables:

- `SOLANA_PRIVATE_KEY`
- `SOLANA_RPC_URL`
- `SOLANA_WS_URL`
- `USER_REGION`
- `USER_REGION_HELIUS_SENDER`
- `USER_REGION_JITO_BUNDLE`
- `LAUNCHDECK_METADATA_UPLOAD_PROVIDER`
- `PINATA_JWT`
- `BAGS_API_KEY`
- `HELIUS_SENDER_ENDPOINT`
- `HELIUS_SENDER_BASE_URL`
- `JITO_BUNDLE_BASE_URLS`
- `JITO_SEND_BUNDLE_ENDPOINT`
- `JITO_BUNDLE_STATUS_ENDPOINT`
- `LAUNCHDECK_SEND_LOG_DIR`

Metadata upload behavior:

- default provider: `pump-fun`
- optional provider: `pinata`
- supported values for `LAUNCHDECK_METADATA_UPLOAD_PROVIDER`: `pump-fun`, `pinata`
- `PINATA_JWT` is required when `pinata` is selected

## Host Runtime

LaunchDeck now runs as a single Rust host process on the local UI port.

Primary runtime variables:

- `LAUNCHDECK_PORT`
- `LAUNCHDECK_ENGINE_AUTH_TOKEN`
- `LAUNCHDECK_FOLLOW_DAEMON_TRANSPORT`
- `LAUNCHDECK_FOLLOW_DAEMON_URL`
- `LAUNCHDECK_FOLLOW_DAEMON_PORT`
- `LAUNCHDECK_FOLLOW_DAEMON_AUTH_TOKEN`
- `LAUNCHDECK_FOLLOW_MAX_ACTIVE_JOBS`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_COMPILES`
- `LAUNCHDECK_FOLLOW_MAX_CONCURRENT_SENDS`
- `LAUNCHDECK_FOLLOW_CAPACITY_WAIT_MS`

Current behavior:

- `LAUNCHDECK_PORT` is the primary host port for both `/api/*` and `/engine/*`
- `npm start` starts the Rust host and follow daemon through `start.ps1`
- `npm stop` stops both local processes through `stop.ps1`
- `npm restart` performs the same clean recycle explicitly
- `npm run ui` starts the Rust host directly
- the follow daemon defaults to `http://127.0.0.1:8790`

## Persisted App Config

The UI settings file lives at:

`LaunchDeck/.local/launchdeck/app-config.json`

It stores:

- default launch execution settings
- default buy execution settings
- default sell execution settings
- preset provider selections
- default post-launch strategy
- default automatic dev-sell settings
- sniper draft state
- selected launch mode and fee-split drafts

Legacy provider values are migrated forward when persisted config is read, so older `auto` or removed provider IDs do not remain live in settings.

The Rust host preserves the browser contract for both:

- `POST /api/settings`
- `POST /api/settings/save`

That keeps older UI save paths working during the Rust-only cutover.

## Launch Config

`launch.example.yml` shows the build/simulate/send launch config shape.

Important runtime fields:

- `launchpad`
- `mode`
- `execution.provider`
- `execution.skipPreflight`
- `execution.buyProvider`
- `execution.sellProvider`
- `tx.computeUnitPriceMicroLamports`
- `tx.jitoTipLamports`
- `postLaunch.strategy`
- `presets.selectedLaunchPresetId`
- `presets.selectedSniperPresetId`

## Safe Defaults

The app defaults are designed so a normal user can open the UI and get a sensible baseline without manually tuning every field:

- launch provider: `helius-sender`
- buy provider: `helius-sender`
- sell provider: `helius-sender`
- post-launch strategy: `none`
- automatic dev sell: off

## Provider-Specific Behavior

Not every entered field is always applied exactly as typed. The engine decides final transport shaping based on provider and launch shape.

Examples:

- `Helius Sender` requires tip, priority fee, and `skipPreflight=true`.
- `Standard RPC` ignores tip.
- `Jito Bundle` creation may accept a priority fee input in the UI, but the engine can drop it for multi-transaction creation flows where it would only waste money.
- explicit sender or bundle endpoint overrides bypass normal region fanout

## Endpoint Profiles

`Endpoint Profile` is only shown for providers with multiple documented endpoint groups.

Current supported values:

- `Global`
- `US`
- `EU`
- `West`
- `Asia`

Current providers with endpoint-profile support:

- `Helius Sender`
- `Jito Bundle`

Current runtime behavior uses the selected profile as an endpoint group and fans out submission across that group.

Current region resolution order:

- provider-specific override such as `USER_REGION_HELIUS_SENDER`
- shared `USER_REGION`
- provider default/global fallback

## Runtime Reports

Durable send reports are written under the local runtime area by default:

`LaunchDeck/.local/launchdeck/send-reports`

Each report captures the planned transport strategy and the actual send outcome per transaction.

When follow behavior is enabled, persisted reports can also include:

- follow-job snapshot
- follow-action outcomes
- watcher health
- follow timing profiles

The local runtime area also stores:

- `LaunchDeck/.local/launchdeck/uploads`
- `LaunchDeck/.local/launchdeck/image-library.json`
- `LaunchDeck/.local/launchdeck/lookup-tables.json`

## Metadata Upload Providers

LaunchDeck supports configurable off-chain metadata upload behavior before deploy.

### `pump-fun`

- default when no metadata provider env var is set
- uploads image and metadata together through Pump's upload API
- full metadata URI reuse is supported when the whole metadata fingerprint is unchanged

### `pinata`

- enabled with `LAUNCHDECK_METADATA_UPLOAD_PROVIDER=pinata`
- uploads the image to Pinata and pins metadata JSON separately
- reuses the uploaded image CID across metadata-only edits during the current app session
- this reduces deploy-time wait when name, symbol, description, or socials change but the image does not
- if Pinata upload fails, LaunchDeck automatically falls back to the `pump-fun` upload path

Pinata can also be used on its free tier for local LaunchDeck testing:

- storage: `1 GB`
- pinned files: `500`
- API rate limit: `60 requests/minute`
- dedicated gateway: `1`
- gateway bandwidth: `10 GB/month`
- gateway requests: `10,000/month`

## Runtime Reports

Benchmark timing output now separates user-visible wait from backend execution:

- `total`: end-to-end click-to-finish duration
- `backendTotal`: Rust backend duration after request receipt
- `preRequest`: browser-side wait before `/api/run`
- `form`, `normalize`, `wallet`, `compile`, `send`, `persist`
- compile sub-timings: `altLoad`, `blockhash`, `global`, `followUpPrep`, `serialize`
- send sub-timings: `submit`, `confirm`

Follow timing profiles can also surface historical submit percentiles such as:

- `P50 Submit`
- `P75 Submit`
- `P90 Submit`
