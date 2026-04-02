# LaunchDeck by Trench.tools

LaunchDeck is a self-hosted Solana launch and snipe tool built under the broader `Trench.tools` project.

[![Website](https://img.shields.io/badge/Website-trench.tools-2563eb?style=flat-square&logo=googlechrome&logoColor=white)](https://trench.tools/)
[![Trench.tools on X](https://img.shields.io/badge/X-@TrenchDotTools-111111?style=flat-square&logo=x&logoColor=white)](https://x.com/TrenchDotTools)
[![0xd3bt on X](https://img.shields.io/badge/X-@0xd3bt-111111?style=flat-square&logo=x&logoColor=white)](https://x.com/0xd3bt)
[![Trench.tools Community](https://img.shields.io/badge/X-Community-111111?style=flat-square&logo=x&logoColor=white)](https://x.com/i/communities/2038790841418838419)

> ### Contract Address
> `L73w5odyo5ZdJ1fPp319nfjqaFfHDdKifRmM8Kxpump`

Instead of paying fees to third-party launch platforms, LaunchDeck lets you run the launcher locally, use your own wallets and provider keys, and customize how launches are built, simulated, and sent. You can get started with free infrastructure, but for serious use we strongly recommend a Helius dev-tier setup because it delivers a major improvement in realtime watcher quality, execution reliability, and overall performance.

This repo is under active development. The README reflects the features we consider usable today.

LaunchDeck is open-source tooling provided as-is. Running it, configuring it, modifying it, deploying it, or using it in any way is entirely the user's own responsibility. By using this software, you accept full responsibility for your environment, infrastructure, wallets, keys, dependencies, third-party packages, and any outcomes that result from its use. Trench.tools is not responsible for losses, damages, exploits, malicious code, compromised packages, misconfiguration, misuse, downtime, failed transactions, or any other direct or indirect consequences related to the software or its dependencies.

## Current Recommendation

For most operators today, the best-supported and fastest setup is:

- [Helius](https://www.helius.dev/) for the full infrastructure stack
- `Helius` for `SOLANA_RPC_URL`
- `Helius` for `SOLANA_WS_URL`
- a separate [Shyft](https://shyft.to/) RPC with a free API key for `LAUNCHDECK_WARM_RPC_URL`
- `Helius Sender` as the creation, buy, and sell provider

Why this is the current recommended stack:

- `LAUNCHDECK_WARM_RPC_URL` offloads startup warmup and block-height observation away from your main execution RPC
- Shyft is a good fit for that warm path because you can use a free API key there
- Helius dev tier gives a very noticeable improvement in watcher performance and live execution quality versus a bare-minimum free setup

If you have Helius dev tier and your websocket endpoint supports it, also enable:

- `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true`

That unlocks the enhanced `transactionSubscribe` path for follow-daemon slot, signature, and market watchers while still falling back safely when unsupported.

## What It Does

LaunchDeck is built for operators who want to:

- launch locally instead of using a hosted launcher UI
- use their own RPC, websocket, sender, and bundle infrastructure
- control creation, buy, and sell execution settings separately
- run same-time or delayed launch-follow actions
- keep durable local reports for audit and reuse

## Runtime Model

LaunchDeck currently runs as two local Rust processes:

- the main host on `http://127.0.0.1:8789` by default
- the follow daemon on `http://127.0.0.1:8790` by default

The main host serves:

- the browser UI
- browser-facing `/api/*` routes
- internal `/engine/*` routes
- uploaded assets under `/uploads/*`

The follow daemon is responsible for:

- delayed and watcher-driven follow actions
- realtime slot, signature, and market watchers
- follow telemetry and timing profiles
- persisted follow-job state outside the main request lifecycle

## Current Support

### Verified Launchpads

#### Pump

Verified and Rust-native for:

- `regular`
- `cashback`
- `agent-custom`
- `agent-unlocked`
- `agent-locked`

Pump launch assembly, transaction shaping, simulation, send orchestration, and reporting are handled in the Rust engine.

#### Bonk

Verified for:

- `regular`
- `bonkers`
- quote assets `sol` and `usd1`
- immediate dev buy
- same-time sniper buys
- snipe buys
- snipe sells
- automatic dev sell

Bonk validation, transport planning, reporting, simulation, and send execution are Rust-owned. Launch assembly uses the Raydium LaunchLab-backed helper bridge.

Bonk `usd1` currently uses a pinned Raydium `SOL -> USD1` route pool, and same-time `usd1` sniper buys are assembled as atomic swap-and-buy transactions.

### Experimental

#### Bagsapp

Bagsapp is available when configured, but it is still experimental in this repo.

Available behavior today includes:

- fee modes `bags-2-2`, `bags-025-1`, and `bags-1-025`
- wallet-only identity
- linked Bags identity when the selected LaunchDeck wallet belongs to the authenticated Bags account
- immediate dev buy
- same-time sniper buys
- snipe buy and snipe sell execution
- automatic dev sell

See `docs/LAUNCHPADS.md` for the exact support matrix and restrictions.

## Quick Start

### 1. Install Dependencies

LaunchDeck uses:

- Rust for the engine and daemon
- Node.js for runtime helpers and launchpad helper scripts

Install the repo dependencies, then copy `.env.example` to `.env` and fill in the quick-start values. If you want the full variable list, use `.env.advanced` as the reference.

### 2. Configure The Minimum Required Env Vars

Most operators only need to set:

- `SOLANA_RPC_URL`
- `SOLANA_WS_URL`
- `LAUNCHDECK_WARM_RPC_URL` if you want startup warm and block-height observation off your main RPC
- `SOLANA_PRIVATE_KEY` or `SOLANA_PRIVATE_KEY*`
- `USER_REGION` for region-aware providers; this is usually better than pinning one specific sender or bundle endpoint because LaunchDeck can fan out across the endpoints in that region

Optional but common:

- `LAUNCHDECK_METADATA_UPLOAD_PROVIDER=pinata` ([Pinata](https://pinata.cloud/))
- `PINATA_JWT`
- `BAGS_API_KEY`
- `LAUNCHDECK_ENABLE_HELIUS_TRANSACTION_SUBSCRIBE=true` if you are on Helius dev tier and want the enhanced `transactionSubscribe` watcher path

Recommended practical setup:

- use a Helius mainnet RPC URL for `SOLANA_RPC_URL`
- use the matching Helius websocket URL for `SOLANA_WS_URL`
- use a Shyft RPC URL with a free API key for `LAUNCHDECK_WARM_RPC_URL`
- use `Helius Sender` as the provider in LaunchDeck

That combination is currently the strongest default operator path in LaunchDeck. The app can run without paid infra, but Helius dev tier is highly recommended if you care about maximum speed, better watcher behavior, and better execution consistency.

Full configuration reference: `docs/CONFIG.md`

### 3. Start The Runtime

Primary commands:

- `npm start`
- `npm stop`
- `npm restart`
- `npm run ui`

`npm start` uses the platform runtime helper:

- Linux: `start.sh`
- Windows: `start.ps1`

It stops old LaunchDeck processes, starts the main host and follow daemon together, waits for both health checks, and opens the UI when supported.

### 4. Open The UI

Default local URL:

- `http://127.0.0.1:8789`

Typical first-run workflow:

1. import or confirm a wallet from `SOLANA_PRIVATE_KEY*`
2. choose a launchpad and mode
3. select an image and fill token metadata
4. review creation, buy, and sell settings
5. optionally configure snipers or auto-sell
6. `Build`, `Simulate`, or `Deploy`

Detailed operator walkthrough: `docs/USAGE.md`

VPS deployment walkthrough: `docs/VPS_SETUP.md`

## Execution Providers

LaunchDeck exposes three current provider choices:

- `Helius Sender`
- `Standard RPC`
- `Jito Bundle`

Important rules:

- `Helius Sender` is the current default, fastest, and most reliable starting point for most operators
- `Helius Sender` requires `skipPreflight=true`, a positive compute-unit price, and a tip of at least `200000` lamports
- `Standard RPC` uses standard RPC semantics and does not use tip
- `Jito Bundle` uses bundle submission and status polling
- private relay integrations such as `bloxroute`, `astralane`, and `hello moon` are planned next

The UI collects intent, but the engine is the final source of truth for what gets applied to each transaction.

Examples:

- a stored tip value is ignored on `Standard RPC`
- `Helius Sender` hard-fails if Sender requirements are not satisfied
- `Jito Bundle` may drop creation priority in launch shapes where it would only add cost without helping

Provider details: `docs/PROVIDERS.md`

## Follow Automation

LaunchDeck supports launch-follow automation through the dedicated daemon.

Current follow behavior includes:

- same-time sniper buys
- delayed sniper buys with `On Submit + Delay`
- confirmed-block sniper buys with `On Confirmed Block`
- automatic dev sell with exclusive `Time` or `Market Cap` trigger families
- market-cap sell triggers with timeout in seconds and selectable timeout behavior (`Stop` or `Sell`)
- snipe sells
- same-time retry for eligible sniper buys

Current limitation:

- `followLaunch.snipes[].postBuySell` is not supported yet and is rejected by config validation

Follow system details: `docs/FOLLOW_DAEMON.md` and `docs/STRATEGIES.md`

## Reporting And Local Data

LaunchDeck writes durable local data under `.local/launchdeck` by default:

- `app-config.json`
- `image-library.json`
- `lookup-tables.json`
- `uploads/`
- `send-reports/`
- `follow-daemon-state.json`

Reports capture both requested settings and actual execution outcomes, including provider, transport type, endpoint information, signatures, confirmations, and timing breakdowns.

History/report usage: `docs/REPORTING.md`

## Documentation Map

Primary operator docs:

- `docs/USAGE.md`
- `docs/CONFIG.md`
- `docs/LAUNCHPADS.md`
- `docs/PROVIDERS.md`
- `docs/STRATEGIES.md`
- `docs/FOLLOW_DAEMON.md`
- `docs/REPORTING.md`
- `docs/TROUBLESHOOTING.md`
- `docs/ARCHITECTURE.md`
- `docs/VPS_SETUP.md`

Supporting or internal documents:

- `docs/EXECUTION_PROVIDER_PLAN.md`
- `docs/FRONTEND_REGRESSION_CHECKLIST.md`
