#!/usr/bin/env bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "Run this script as root." >&2
  exit 1
fi

LAUNCHDECK_REPO_URL="${LAUNCHDECK_REPO_URL:-https://github.com/0xD3bt/Trench-Tools.git}"
LAUNCHDECK_REPO_BRANCH="${LAUNCHDECK_REPO_BRANCH:-master}"
LAUNCHDECK_DIR="${LAUNCHDECK_DIR:-/opt/launchdeck}"
LAUNCHDECK_SERVICE_NAME="${LAUNCHDECK_SERVICE_NAME:-launchdeck}"
NODE_MAJOR="${NODE_MAJOR:-20}"

install_base_packages() {
  apt-get update
  apt-get install -y \
    ca-certificates \
    curl \
    git \
    wget \
    unzip \
    tmux \
    htop \
    jq \
    build-essential \
    pkg-config \
    libssl-dev \
    ufw \
    fail2ban \
    gnupg
}

install_rust() {
  if [[ ! -x /root/.cargo/bin/rustup ]]; then
    curl https://sh.rustup.rs -sSf | sh -s -- -y
  fi

  if ! grep -q '.cargo/env' /root/.bashrc 2>/dev/null; then
    echo '. "$HOME/.cargo/env"' >> /root/.bashrc
  fi

  # shellcheck disable=SC1091
  source /root/.cargo/env
  rustup default stable
}

install_node() {
  if [[ ! -f /etc/apt/keyrings/nodesource.gpg ]]; then
    mkdir -p /etc/apt/keyrings
    curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key \
      | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
  fi

  echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" \
    > /etc/apt/sources.list.d/nodesource.list

  apt-get update
  apt-get install -y nodejs
}

sync_repo() {
  mkdir -p "$(dirname "$LAUNCHDECK_DIR")"

  if [[ ! -d "$LAUNCHDECK_DIR/.git" ]]; then
    git clone --branch "$LAUNCHDECK_REPO_BRANCH" "$LAUNCHDECK_REPO_URL" "$LAUNCHDECK_DIR"
  else
    git -C "$LAUNCHDECK_DIR" fetch origin "$LAUNCHDECK_REPO_BRANCH"
    git -C "$LAUNCHDECK_DIR" checkout "$LAUNCHDECK_REPO_BRANCH"
    git -C "$LAUNCHDECK_DIR" pull --ff-only origin "$LAUNCHDECK_REPO_BRANCH"
  fi

  cd "$LAUNCHDECK_DIR"
  npm install

  if [[ ! -f "$LAUNCHDECK_DIR/.env" ]]; then
    cp "$LAUNCHDECK_DIR/.env.example" "$LAUNCHDECK_DIR/.env"
  fi
}

write_systemd_service() {
  cat >/etc/systemd/system/${LAUNCHDECK_SERVICE_NAME}.service <<EOF
[Unit]
Description=LaunchDeck runtime
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=${LAUNCHDECK_DIR}
Environment=HOME=/root
ExecStart=/usr/bin/env bash -lc 'cd "${LAUNCHDECK_DIR}" && npm start'
ExecStop=/usr/bin/env bash -lc 'cd "${LAUNCHDECK_DIR}" && npm stop'
ExecReload=/usr/bin/env bash -lc 'cd "${LAUNCHDECK_DIR}" && npm restart'
TimeoutStartSec=300
TimeoutStopSec=180

[Install]
WantedBy=multi-user.target
EOF

  systemctl daemon-reload
  systemctl enable "${LAUNCHDECK_SERVICE_NAME}.service"
}

configure_host_security() {
  systemctl enable fail2ban
  systemctl restart fail2ban

  ufw allow OpenSSH
  ufw --force enable
}

start_launchdeck() {
  systemctl restart "${LAUNCHDECK_SERVICE_NAME}.service"
}

print_next_steps() {
  cat <<EOF

LaunchDeck bootstrap complete.

Project path:
  ${LAUNCHDECK_DIR}

Service commands:
  systemctl status ${LAUNCHDECK_SERVICE_NAME}
  systemctl restart ${LAUNCHDECK_SERVICE_NAME}
  journalctl -u ${LAUNCHDECK_SERVICE_NAME} -n 100 --no-pager

Next steps:
  1. Edit ${LAUNCHDECK_DIR}/.env
  2. Restart the service
  3. Open an SSH tunnel from your local machine:
       ssh -L 8789:127.0.0.1:8789 root@YOUR_SERVER_IP
  4. Visit http://127.0.0.1:8789 in your browser

The UI stays bound to localhost on the VPS by default. Use the SSH tunnel unless you intentionally add your own reverse proxy later.
EOF
}

install_base_packages
install_rust
install_node
sync_repo
write_systemd_service
configure_host_security
start_launchdeck
print_next_steps
