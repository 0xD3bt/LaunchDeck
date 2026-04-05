# VPS Setup

This guide walks through a straightforward VPS deployment flow for LaunchDeck.

The recommended pattern is:

- run LaunchDeck on the VPS
- keep the UI bound to `127.0.0.1`
- use an SSH tunnel from your local machine to reach the UI

## Recommended location

Place the VPS near the provider endpoints you actually plan to use.

Recommended starting regions:

- EU: Frankfurt or Amsterdam
- US: Newark or Salt Lake City if you want exact Helius metro routing, or nearby metros such as New York / Virginia when that is the more practical hosting choice
- Asia: Singapore or Tokyo

Practical note:

- if you use grouped routing like `us` or `asia`, those metros are far apart
- the best practical result is usually to place the server in one of them and use the exact metro token in `USER_REGION`

## Recommended stack on a VPS

For most operators:

- `SOLANA_RPC_URL`: Helius Gatekeeper HTTP
- `SOLANA_WS_URL`: Helius standard websocket
- `LAUNCHDECK_WARM_RPC_URL`: Shyft
- provider: `Helius Sender` or `Hello Moon`

Helius dev tier is strongly recommended if you plan to run multiple snipes or watcher-heavy follow automation.

## Recommended server shape

Start simple unless you already know you need more:

- Ubuntu `24.04`
- 2 vCPU minimum
- 4 GB RAM minimum
- enough SSD for Rust builds, `node_modules`, uploads, and local reports

## 1. Create an SSH key

Linux or macOS:

```bash
ssh-keygen -t ed25519 -C "you@example.com"
```

Windows PowerShell with OpenSSH:

```powershell
ssh-keygen -t ed25519 -C "you@example.com"
```

Show the public key:

```bash
cat ~/.ssh/id_ed25519.pub
```

Do not share the private key.

## 2. Create the VPS

Suggested starting choices:

1. use a standard cloud VPS
2. choose `Ubuntu 24.04`
3. choose at least `2 vCPU / 4 GB RAM`
4. choose the region closest to your target provider endpoints
5. attach your SSH key

This guide uses [Vultr](https://www.vultr.com/?ref=9589308) as the worked example, but the same shape works on other providers too.

## 3. Bootstrap the server

If you use the repo bootstrap flow, the startup script is:

- `scripts/vps-bootstrap.sh`

Typical bootstrap result:

- installs system packages
- installs Rust
- installs Node.js
- clones the repo to `/opt/launchdeck`
- runs `npm install`
- copies `.env.example` to `.env` when needed
- installs a `systemd` service

## 4. SSH into the server

```bash
ssh root@YOUR_SERVER_IP
```

## 5. Fill `.env`

On the server:

```bash
cd /opt/launchdeck
nano .env
```

Start with the same easy-setup values from `.env.example`:

- `SOLANA_PRIVATE_KEY` or your `SOLANA_PRIVATE_KEY*` wallet set
- `SOLANA_RPC_URL`
- `SOLANA_WS_URL`
- `USER_REGION`
- `LAUNCHDECK_WARM_RPC_URL`

If you want exact copy-paste examples:

```bash
SOLANA_RPC_URL=https://beta.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
SOLANA_WS_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
LAUNCHDECK_WARM_RPC_URL=https://rpc.fra.shyft.to?api_key=YOUR_SHYFT_API_KEY
```

Optional but common:

- `HELLOMOON_API_KEY`
- `BAGS_API_KEY`
- `LAUNCHDECK_METADATA_UPLOAD_PROVIDER=pinata`
- `PINATA_JWT`

More detail:

- `docs/CONFIG.md`
- `docs/ENV_REFERENCE.md`

## 6. Start or restart LaunchDeck

If you are using the service install:

```bash
systemctl restart launchdeck
systemctl status launchdeck
```

Useful logs:

```bash
journalctl -u launchdeck -n 100 --no-pager
```

## 7. Open the UI through SSH tunneling

Because LaunchDeck binds locally by default, the recommended access pattern is an SSH tunnel:

```bash
ssh -L 8789:127.0.0.1:8789 root@YOUR_SERVER_IP
```

Then open:

```text
http://127.0.0.1:8789
```

This keeps the UI private instead of exposing it directly to the internet.

## 8. Updating later

```bash
cd /opt/launchdeck
git pull --ff-only
npm install
systemctl restart launchdeck
```

## Useful commands

Service status:

```bash
systemctl status launchdeck
```

Restart:

```bash
systemctl restart launchdeck
```

Tail logs:

```bash
journalctl -u launchdeck -f
```

## Notes

- do not expose the raw local bind publicly unless you intentionally add a reverse proxy and access controls
- restart LaunchDeck after env changes
- if you change ports or install paths, update the service and commands accordingly

