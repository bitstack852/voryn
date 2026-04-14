#!/usr/bin/env bash
# Voryn Update Server — Provisioning Script
# Run as root on a fresh Debian 12 / Ubuntu 22.04 VPS
#
# Usage: sudo bash provision.sh

set -euo pipefail

DOMAIN="updates.voryn.app"
WEB_ROOT="/var/www/voryn-updates"
DEPLOY_USER="voryn-deploy"

echo "=== Voryn Update Server Provisioning ==="

# ── 1. Install nginx + certbot ────────────────────────────────────
echo "[1/6] Installing nginx and certbot..."
apt-get update -qq
apt-get install -y -qq nginx certbot python3-certbot-nginx ufw

# ── 2. Create directory structure ─────────────────────────────────
echo "[2/6] Creating web root..."
mkdir -p "${WEB_ROOT}/releases"

# Seed version.json
cat > "${WEB_ROOT}/version.json" << 'VERSIONJSON'
{
  "latest": "0.1.0",
  "minimum": "0.1.0",
  "android_url": "https://updates.voryn.app/releases/voryn-0.1.0.apk",
  "ios_url": "",
  "changelog": "Initial release",
  "released_at": "2026-04-14T00:00:00Z"
}
VERSIONJSON

chown -R www-data:www-data "${WEB_ROOT}"

# ── 3. Create deploy user (for CI/CD uploads) ────────────────────
echo "[3/6] Creating deploy user..."
if ! id -u "${DEPLOY_USER}" &>/dev/null; then
    useradd -r -m -s /bin/bash "${DEPLOY_USER}"
    mkdir -p "/home/${DEPLOY_USER}/.ssh"
    chmod 700 "/home/${DEPLOY_USER}/.ssh"
    chown -R "${DEPLOY_USER}:${DEPLOY_USER}" "/home/${DEPLOY_USER}/.ssh"
fi

# Allow deploy user to write to web root
usermod -aG www-data "${DEPLOY_USER}"
chmod 775 "${WEB_ROOT}" "${WEB_ROOT}/releases"

echo "  Add your CI/CD SSH public key to /home/${DEPLOY_USER}/.ssh/authorized_keys"

# ── 4. Configure firewall ────────────────────────────────────────
echo "[4/6] Configuring firewall..."
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 80/tcp comment "HTTP (redirect to HTTPS)"
ufw allow 443/tcp comment "HTTPS"
ufw --force enable

# ── 5. Install nginx config ──────────────────────────────────────
echo "[5/6] Installing nginx configuration..."
cat > /etc/nginx/sites-available/voryn-updates << 'NGINX'
server {
    listen 80;
    listen [::]:80;
    server_name updates.voryn.app;
    root /var/www/voryn-updates;

    location = /version.json {
        expires 5m;
        add_header Cache-Control "public, max-age=300";
        add_header Access-Control-Allow-Origin "*" always;
    }

    location /releases/ {
        expires 365d;
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header Access-Control-Allow-Origin "*" always;
    }

    location / {
        return 404;
    }
}
NGINX

ln -sf /etc/nginx/sites-available/voryn-updates /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default
nginx -t && systemctl reload nginx

# ── 6. TLS Certificate ──────────────────────────────────────────
echo "[6/6] Requesting TLS certificate..."
echo ""
echo "  Run the following command after DNS is configured:"
echo "    sudo certbot --nginx -d ${DOMAIN}"
echo ""
echo "  Or for staging/test:"
echo "    sudo certbot --nginx -d ${DOMAIN} --staging"
echo ""

# ── Summary ──────────────────────────────────────────────────────
echo "=== Update Server Summary ==="
echo "  Domain:     ${DOMAIN}"
echo "  Web root:   ${WEB_ROOT}"
echo "  Deploy user: ${DEPLOY_USER}"
echo ""
echo "=== Upload a release ==="
echo "  scp voryn-0.1.0.apk ${DEPLOY_USER}@${DOMAIN}:${WEB_ROOT}/releases/"
echo "  Then update ${WEB_ROOT}/version.json"
echo ""
echo "=== DNS Setup ==="
echo "  Create an A record: ${DOMAIN} -> $(curl -s ifconfig.me || echo '<this-server-ip>')"
