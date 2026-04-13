# VPS Setup

This guide walks through the current VPS deployment flow for LaunchDeck, starting from a brand-new Ubuntu instance.

The recommended pattern is:

- run LaunchDeck on the VPS
- keep the UI bound to `127.0.0.1`
- use an SSH tunnel from your local machine to reach the UI

Use this page when you need to:

- create the initial VPS instance
- install the required system dependencies
- bootstrap LaunchDeck with the repo setup script
- keep the runtime private behind SSH instead of exposing the raw UI port

## Recommended location

Place the VPS near the provider endpoints and RPCs you actually plan to use.

Recommended starting regions:

- EU: Frankfurt or Amsterdam
- US: New York / Newark area or Salt Lake City area
- Asia: Singapore or Tokyo

Practical note:

- if you use grouped routing like `us` or `asia`, those metros are far apart
- the best practical result is usually to place the server in one of them and use the exact metro token in `USER_REGION`
- for the US path, that usually means `ewr` for the New York / Newark side or `slc` for Salt Lake City
- for Asia, the same rule applies: keep the VPS close to the Asian provider endpoints and RPCs you plan to use, which usually means Singapore or Tokyo

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

## Fresh-server checklist

For a new VPS, the working order is:

1. create the instance with `Ubuntu 24.04`
2. attach your SSH key
3. SSH in as `root` or another sudo-capable user
4. run the bootstrap script or install the same dependencies manually
5. fill `/opt/launchdeck/.env`
6. start or restart the `launchdeck` service
7. open the UI only through an SSH tunnel

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

This guide uses [Vultr](https://www.vultr.com/?ref=9589308) as the worked example because it is easy to deploy quickly across a wide range of regions, supports standard fiat/card payment flows as well as crypto, and has been reliable in long-term use. If you use Vultr, please use [my referral link](https://www.vultr.com/?ref=9589308). The same overall VPS shape works on other providers too.

## 3. Bootstrap the server

The repo bootstrap script is:

- `scripts/vps-bootstrap.sh`

What it does:

- installs system packages
- installs Rust
- installs Node.js
- clones the repo to `/opt/launchdeck`
- runs `npm install`
- copies `.env.example` to `.env` when needed
- installs a `systemd` service

### Recommended bootstrap path

On a fresh Ubuntu VPS:

```bash
ssh root@YOUR_SERVER_IP
apt-get update
apt-get install -y curl
curl -fsSL https://raw.githubusercontent.com/0xD3bt/Trench-Tools/master/scripts/vps-bootstrap.sh -o /root/vps-bootstrap.sh
bash /root/vps-bootstrap.sh
```

Optional overrides:

```bash
LAUNCHDECK_REPO_BRANCH=master \
LAUNCHDECK_DIR=/opt/launchdeck \
LAUNCHDECK_SERVICE_NAME=launchdeck \
bash /root/vps-bootstrap.sh
```

The script assumes a root-run install and is meant to leave you with a working repo checkout plus a `launchdeck` `systemd` service.

### What gets installed

The bootstrap script currently installs these base packages:

- `ca-certificates`
- `curl`
- `git`
- `wget`
- `unzip`
- `tmux`
- `htop`
- `jq`
- `build-essential`
- `pkg-config`
- `libssl-dev`
- `ufw`
- `fail2ban`
- `gnupg`

It also installs:

- Rust stable through `rustup`
- Node.js `20` through NodeSource

### Manual install path

If you intentionally do not want to use the bootstrap script, install the same prerequisites first:

```bash
apt-get update
apt-get install -y ca-certificates curl git wget unzip tmux htop jq build-essential pkg-config libssl-dev ufw fail2ban gnupg
curl https://sh.rustup.rs -sSf | sh -s -- -y
mkdir -p /etc/apt/keyrings
curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" > /etc/apt/sources.list.d/nodesource.list
apt-get update
apt-get install -y nodejs
```

Then clone the repo, run `npm install`, and create your own service around `npm start` / `npm stop` / `npm restart`.

## 4. SSH into the server

```bash
ssh root@YOUR_SERVER_IP
```

If you already ran the bootstrap command over SSH, you are already on the box and can continue with the next steps.

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
- `.env.advanced`

Recommended first-pass rule:

- fill only the starter values from `.env.example`
- leave advanced warm, follow, provider, and capacity tuning alone until the runtime is healthy

Quick verification after bootstrap:

```bash
node -v
npm -v
source /root/.cargo/env && cargo -V && rustc -V
```

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

Practical note:

- the very first service start may take a while because the Rust binaries still need to compile
- after the first build, restarts should be much faster

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

If dependencies changed heavily and you want to verify the service after an update:

```bash
systemctl status launchdeck
journalctl -u launchdeck -n 100 --no-pager
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
- do not open `8789` or `8790` directly to the internet for the normal operator setup
- the bootstrap script enables `ufw` and allows `OpenSSH`; if you use a custom SSH port or stricter firewall policy, adjust that before disconnecting
- restart LaunchDeck after env changes
- if you change ports or install paths, update the service and commands accordingly

