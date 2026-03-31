# VPS Setup

This guide walks through a simple VPS deployment flow for LaunchDeck.

You can use any provider. For the best latency, the current recommendation is:

- US: Newark, Virginia, or New York area
- EU: Frankfurt or Amsterdam

The examples below use [Vultr](https://www.vultr.com/?ref=9589308) because it has been a reliable option, but the same general process works on other VPS providers too.

## Why Use A VPS

Running LaunchDeck on a VPS is usually the better default for:

- lower latency to your RPC, sender, and bundle endpoints
- better separation from your everyday browsing machine
- easier repeatable deployments with SSH keys and startup scripts
- the ability to keep execution on the VPS while still operating the UI from your normal desktop over SSH

By default, LaunchDeck binds the UI to `127.0.0.1` on the server. That means the recommended access pattern is an SSH tunnel from your local machine to the VPS.

## Recommended Server Shape

Start simple unless you already know you need more:

- Ubuntu `24.04` LTS
- 2 vCPU minimum
- 4 GB RAM minimum
- enough SSD for Rust builds, node modules, logs, and uploads

## 1. Create Your SSH Key

If you do not already have an SSH key on your local machine, create one.

Linux or macOS:

```bash
ssh-keygen -t ed25519 -C "you@example.com"
```

Windows PowerShell with OpenSSH:

```powershell
ssh-keygen -t ed25519 -C "you@example.com"
```

Accept the default path unless you already manage multiple keys. This usually creates:

- private key: `~/.ssh/id_ed25519`
- public key: `~/.ssh/id_ed25519.pub`

Show the public key so you can copy it:

```bash
cat ~/.ssh/id_ed25519.pub
```

Do not share the private key.

## 2. Add The SSH Key To Vultr

In Vultr, add SSH keys here:

- [Vultr SSH Keys](https://my.vultr.com/sshkeys/)

1. Open `SSH Keys`
2. Choose `Add SSH Key`
3. Paste the contents of your public key
4. Save it with a label you will recognize later

## 3. Choose The VPS Region

When creating the VPS:

- use any provider if you prefer another one
- for US, pick Newark, Virginia, or New York area
- for EU, pick Frankfurt or Amsterdam

Those are the suggested starting points for better latency.

## 4. Create The Server

Suggested starting choices:

1. Product: standard VPS / cloud compute
2. OS: `Ubuntu 24.04`
3. Size: at least `2 vCPU / 4 GB RAM`
4. Region: one of the recommended regions above
5. SSH key: attach the key you added
6. Startup script: create or paste the script here first, then attach it during deployment:

- [Vultr Startup Scripts](https://my.vultr.com/startup/)

Use the contents of `scripts/vps-bootstrap.sh`.

If your provider supports custom startup variables, these are the ones used by the script:

- `LAUNCHDECK_REPO_URL`
- `LAUNCHDECK_REPO_BRANCH`
- `LAUNCHDECK_DIR`
- `LAUNCHDECK_SERVICE_NAME`
- `NODE_MAJOR`

Defaults are already included, so you usually do not need to change them.

## 5. Wait For Bootstrap To Finish

The startup script will:

- install system packages
- install Rust
- install Node.js `20`
- clone the repo into `/opt/launchdeck`
- run `npm install`
- copy `.env.example` to `.env` if needed
- install and enable a `systemd` service called `launchdeck`
- enable `ufw` and `fail2ban`

If you are newer to coding or Linux, you can also do the setup through an AI coding tool instead of handling everything manually in a raw terminal.

Useful options:

- [Cursor](https://cursor.com/)
- [Codex](https://openai.com/codex/)

You can SSH into the VPS through those tools and let the AI help with the rest of the setup, edit files more easily, handle `.env` changes, restart services, and troubleshoot issues.

Once the server is up, SSH in:

```bash
ssh root@YOUR_SERVER_IP
```

## 6. Edit The Env File

On the server:

```bash
cd /opt/launchdeck
nano .env
```

At minimum, most operators will want to fill in:

- `SOLANA_RPC_URL`
- `SOLANA_WS_URL`
- `SOLANA_PRIVATE_KEY` or your `SOLANA_PRIVATE_KEY*` set
- `USER_REGION`

Optional but common:

- `HELIUS_API_KEY`
- `PINATA_JWT`
- `BAGS_API_KEY`

Full env reference:

- `docs/CONFIG.md`

## 7. Start Or Restart LaunchDeck

After editing `.env`, restart the service:

```bash
systemctl restart launchdeck
systemctl status launchdeck
```

Useful logs:

```bash
journalctl -u launchdeck -n 100 --no-pager
```

The runtime helper writes logs under:

```bash
/opt/launchdeck/.local/launchdeck
```

## 8. Open The UI Safely With An SSH Tunnel

Because LaunchDeck binds to `127.0.0.1` on the VPS by default, open an SSH tunnel from your local machine:

```bash
ssh -L 8789:127.0.0.1:8789 root@YOUR_SERVER_IP
```

Then open this locally in your browser:

```text
http://127.0.0.1:8789
```

This keeps the UI private instead of exposing it directly to the public internet.

After that, you can just use LaunchDeck from your normal desktop browser while the runtime stays on the VPS. The SSH tunnel only carries the local UI connection, so you can still use the app normally, including things like popout windows and the regular operator workflow.

## 9. Updating The Server Later

SSH into the VPS and run:

```bash
cd /opt/launchdeck
git pull --ff-only
npm install
systemctl restart launchdeck
```

## 10. Common Commands

Service status:

```bash
systemctl status launchdeck
```

Restart:

```bash
systemctl restart launchdeck
```

Stop:

```bash
systemctl stop launchdeck
```

Tail logs:

```bash
journalctl -u launchdeck -f
```

## Notes

- This guide uses [Vultr](https://www.vultr.com/?ref=9589308) as the worked example, but LaunchDeck can run on any VPS provider.
- If you want a public hostname later, add your own reverse proxy on top intentionally. Do not expose the raw local bind by accident.
- If you change the install path or service name, update the commands accordingly.
