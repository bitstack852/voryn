#!/usr/bin/env bash
# Voryn Bootstrap Node — Server Provisioning Script
# Run as root on a fresh Debian 12 / Ubuntu 22.04 VPS
#
# Usage: curl -sSf https://raw.githubusercontent.com/bitstack852/voryn/main/deploy/bootstrap/provision.sh | sudo bash

set -euo pipefail

VORYN_USER="voryn"
VORYN_DIR="/opt/voryn"
VORYN_DATA="${VORYN_DIR}/data"
VORYN_BIN="${VORYN_DIR}/bin"
LISTEN_PORT=4001

echo "=== Voryn Bootstrap Node Provisioning ==="

# ── 1. System Updates & Dependencies ──────────────────────────────
echo "[1/8] Updating system packages..."
apt-get update -qq
apt-get upgrade -y -qq
apt-get install -y -qq \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    ufw \
    unattended-upgrades \
    fail2ban

# ── 2. Create voryn user ─────────────────────────────────────────
echo "[2/8] Creating voryn system user..."
if ! id -u "${VORYN_USER}" &>/dev/null; then
    useradd -r -m -d "${VORYN_DIR}" -s /bin/false "${VORYN_USER}"
fi

mkdir -p "${VORYN_BIN}" "${VORYN_DATA}"
chown -R "${VORYN_USER}:${VORYN_USER}" "${VORYN_DIR}"

# ── 3. Install Rust ──────────────────────────────────────────────
echo "[3/8] Installing Rust toolchain..."
if ! command -v rustup &>/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "${HOME}/.cargo/env"
fi

# ── 4. Build Voryn ───────────────────────────────────────────────
echo "[4/8] Cloning and building Voryn..."
TEMP_BUILD=$(mktemp -d)
git clone --depth 1 https://github.com/bitstack852/voryn.git "${TEMP_BUILD}/voryn"
cd "${TEMP_BUILD}/voryn"
cargo build --release -p voryn-network

# Copy binary (when bootstrap binary exists)
# cp target/release/voryn-bootstrap "${VORYN_BIN}/"
echo "  Note: Bootstrap binary will be available after Phase 1 implementation."
echo "  For now, the network library is compiled successfully."

# Cleanup
cd /
rm -rf "${TEMP_BUILD}"

# ── 5. Firewall Configuration ────────────────────────────────────
echo "[5/8] Configuring firewall (ufw)..."
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow ${LISTEN_PORT}/tcp comment "Voryn libp2p TCP"
ufw allow ${LISTEN_PORT}/udp comment "Voryn libp2p QUIC"
ufw --force enable

echo "  Firewall rules:"
ufw status numbered

# ── 6. System Hardening ──────────────────────────────────────────
echo "[6/8] Applying system hardening..."

# Disable root SSH login
if grep -q "^PermitRootLogin" /etc/ssh/sshd_config; then
    sed -i 's/^PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config
else
    echo "PermitRootLogin no" >> /etc/ssh/sshd_config
fi

# Disable password auth (require SSH keys)
if grep -q "^PasswordAuthentication" /etc/ssh/sshd_config; then
    sed -i 's/^PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
else
    echo "PasswordAuthentication no" >> /etc/ssh/sshd_config
fi

systemctl restart sshd

# Enable automatic security updates
cat > /etc/apt/apt.conf.d/20auto-upgrades << 'AUTOUPDATE'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
APT::Periodic::AutocleanInterval "7";
AUTOUPDATE

# Kernel hardening
cat >> /etc/sysctl.d/99-voryn.conf << 'SYSCTL'
# Disable IP forwarding
net.ipv4.ip_forward = 0
net.ipv6.conf.all.forwarding = 0

# Ignore ICMP redirects
net.ipv4.conf.all.accept_redirects = 0
net.ipv6.conf.all.accept_redirects = 0

# Ignore source-routed packets
net.ipv4.conf.all.accept_source_route = 0

# SYN flood protection
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_max_syn_backlog = 2048

# Increase file descriptor limit
fs.file-max = 100000
SYSCTL
sysctl --system --quiet

# ── 7. Install systemd service ───────────────────────────────────
echo "[7/8] Installing systemd service..."
cat > /etc/systemd/system/voryn-bootstrap.service << 'SERVICE'
[Unit]
Description=Voryn DHT Bootstrap Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=voryn
Group=voryn
WorkingDirectory=/opt/voryn
ExecStart=/opt/voryn/bin/voryn-bootstrap --listen /ip4/0.0.0.0/tcp/4001 --identity-file /opt/voryn/data/node-identity.key --log-level info
Restart=always
RestartSec=5
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/voryn/data
PrivateTmp=yes
LimitNOFILE=65536
MemoryMax=256M

[Install]
WantedBy=multi-user.target
SERVICE

systemctl daemon-reload
# Don't enable yet — binary not available until build
# systemctl enable --now voryn-bootstrap

# ── 8. Summary ───────────────────────────────────────────────────
echo "[8/8] Provisioning complete!"
echo ""
echo "=== Bootstrap Node Summary ==="
echo "  User:      ${VORYN_USER}"
echo "  Directory: ${VORYN_DIR}"
echo "  Data:      ${VORYN_DATA}"
echo "  Port:      ${LISTEN_PORT} (TCP + UDP)"
echo "  Service:   voryn-bootstrap.service"
echo ""
echo "=== Next Steps ==="
echo "  1. Build and install the bootstrap binary to ${VORYN_BIN}/voryn-bootstrap"
echo "  2. Run: systemctl enable --now voryn-bootstrap"
echo "  3. Check: systemctl status voryn-bootstrap"
echo "  4. Get PeerId: journalctl -u voryn-bootstrap | grep PeerId"
echo "  5. Add the PeerId to the app's hardcoded bootstrap peer list"
echo ""
echo "=== DNS Setup ==="
echo "  Create an A record: boot1.voryn.app -> $(curl -s ifconfig.me || echo '<this-server-ip>')"
