# Deployment Plan — Infrastructure Setup & App Deployments

## Table of Contents

1. [Infrastructure Overview](#1-infrastructure-overview)
2. [Bootstrap Node Setup](#2-bootstrap-node-setup)
3. [Update Server Setup](#3-update-server-setup)
4. [DNS & Domain Configuration](#4-dns--domain-configuration)
5. [CI/CD Secrets Configuration](#5-cicd-secrets-configuration)
6. [App Build & Signing](#6-app-build--signing)
7. [Release Process](#7-release-process)
8. [Post-Deployment Verification](#8-post-deployment-verification)
9. [Environments](#9-environments)
10. [Monitoring & Alerting](#10-monitoring--alerting)
11. [Rollback Procedures](#11-rollback-procedures)
12. [Maintenance Runbook](#12-maintenance-runbook)

---

## 1. Infrastructure Overview

Voryn is decentralized — there are no application servers. The infrastructure consists of:

```
┌──────────────────────────────────────────────────────┐
│                    DNS (voryn.app)                    │
│  boot1.voryn.app  boot2.voryn.app  boot3.voryn.app  │
│                updates.voryn.app                      │
└──────────────┬───────────────────────┬───────────────┘
               │                       │
    ┌──────────▼──────────┐  ┌────────▼─────────┐
    │   Bootstrap Nodes   │  │   Update Server   │
    │  (3x VPS, minimal)  │  │  (1x VPS, nginx)  │
    │                     │  │                    │
    │  - DHT peer routing │  │  - version.json    │
    │  - No msg storage   │  │  - APK hosting     │
    │  - No relay traffic │  │  - SHA-256 hashes  │
    └─────────────────────┘  └────────────────────┘

    ┌─────────────────────────────────────────────┐
    │              GitHub Actions CI/CD            │
    │  - Build: Rust + React Native (iOS/Android) │
    │  - Sign: APK keystore / Xcode signing       │
    │  - Publish: GitHub Release + Update Server   │
    └─────────────────────────────────────────────┘
```

**Total servers: 4** (3 bootstrap + 1 update server)
**Monthly cost estimate: ~$20-40** (4x $5-10 VPS)

---

## 2. Bootstrap Node Setup

Bootstrap nodes help new devices find peers on the DHT. They store no user data.

### 2.1 Provision a VPS

**Specs:** 1 vCPU, 512MB RAM, 10GB SSD, Debian 12 or Ubuntu 22.04

**Recommended providers:**
| Provider | Location | Price | Notes |
|----------|----------|-------|-------|
| Njalla | Sweden | ~$15/mo | Anonymous registration, privacy-focused |
| 1984 Hosting | Iceland | ~$5/mo | Strong privacy laws |
| BuyVM | Luxembourg/US | ~$5/mo | DDoS-protected, affordable |

Deploy 3 nodes across different providers/regions for redundancy.

### 2.2 Run the Provisioning Script

```bash
# SSH into the VPS
ssh root@<VPS_IP>

# Download and run the provisioning script
curl -sSf https://raw.githubusercontent.com/bitstack852/voryn/main/deploy/bootstrap/provision.sh | sudo bash
```

The script does:
1. Installs system packages (build tools, ufw, fail2ban)
2. Creates `voryn` system user under `/opt/voryn`
3. Installs Rust and builds the project
4. Configures firewall (SSH + port 4001 TCP/UDP only)
5. Hardens SSH (key-only auth, no root login)
6. Enables automatic security updates
7. Applies kernel hardening (SYN flood protection, no IP forwarding)
8. Installs the systemd service (not yet started)

### 2.3 Build and Start the Node

```bash
# Build the bootstrap binary (after Phase 1 is production-ready)
cd /opt/voryn
sudo -u voryn git clone https://github.com/bitstack852/voryn.git src
cd src
cargo build --release -p voryn-network

# Install the binary
sudo cp target/release/voryn-bootstrap /opt/voryn/bin/

# Start the service
sudo systemctl enable --now voryn-bootstrap

# Verify it's running
sudo systemctl status voryn-bootstrap

# Get the PeerId (needed for hardcoded bootstrap list)
journalctl -u voryn-bootstrap | grep "PeerId"
# Output: Voryn node starting with PeerId: 12D3KooW...
```

### 2.4 Record Node Information

For each node, record:
```
Node:     boot1
IP:       203.0.113.10
PeerId:   12D3KooWxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
Port:     4001
Provider: Njalla
Region:   Sweden
```

### 2.5 Update the App Config

Add the PeerIds to `crates/voryn-network/src/config.rs`:
```rust
pub const BOOTSTRAP_PEERS: &[&str] = &[
    "/dns4/boot1.voryn.app/tcp/4001/p2p/12D3KooWxxxx...",
    "/dns4/boot2.voryn.app/tcp/4001/p2p/12D3KooWyyyy...",
    "/dns4/boot3.voryn.app/tcp/4001/p2p/12D3KooWzzzz...",
];
```

---

## 3. Update Server Setup

Hosts `version.json` and signed APK files. Static content only — no dynamic backend.

### 3.1 Provision a VPS

**Specs:** 1 vCPU, 512MB RAM, 20GB SSD (for APK storage), Debian 12

### 3.2 Run the Provisioning Script

```bash
ssh root@<UPDATE_SERVER_IP>
curl -sSf https://raw.githubusercontent.com/bitstack852/voryn/main/deploy/update-server/provision.sh | sudo bash
```

The script does:
1. Installs nginx + certbot
2. Creates `/var/www/voryn-updates/` with releases subdirectory
3. Seeds `version.json` with v0.1.0
4. Creates `voryn-deploy` user for CI/CD uploads
5. Configures firewall (SSH + HTTP/HTTPS)
6. Installs nginx config (HTTP→HTTPS redirect, caching headers)

### 3.3 Configure TLS

```bash
# After DNS is configured (see section 4)
sudo certbot --nginx -d updates.voryn.app

# Verify auto-renewal
sudo certbot renew --dry-run
```

### 3.4 Configure CI/CD Access

```bash
# On your local machine, generate a deploy key
ssh-keygen -t ed25519 -f voryn-deploy-key -C "voryn-ci-deploy"

# Copy the public key to the update server
ssh-copy-id -i voryn-deploy-key.pub voryn-deploy@updates.voryn.app

# Add the PRIVATE key as a GitHub secret (see section 5)
cat voryn-deploy-key
# Copy this output → GitHub repo → Settings → Secrets → UPDATE_SERVER_SSH_KEY
```

### 3.5 Verify

```bash
curl -sf https://updates.voryn.app/version.json | jq .
# Should return: { "latest": "0.1.0", ... }
```

---

## 4. DNS & Domain Configuration

### Required DNS Records

| Type | Name | Value | TTL |
|------|------|-------|-----|
| A | boot1.voryn.app | `<Bootstrap Node 1 IP>` | 300 |
| A | boot2.voryn.app | `<Bootstrap Node 2 IP>` | 300 |
| A | boot3.voryn.app | `<Bootstrap Node 3 IP>` | 300 |
| A | updates.voryn.app | `<Update Server IP>` | 300 |
| AAAA | (same as above) | `<IPv6 if available>` | 300 |

### Registrar Recommendation

Use a privacy-respecting registrar:
- **Njalla** — anonymous domain registration (recommended)
- **Gandi** — WHOIS privacy included
- **Cloudflare Registrar** — at-cost pricing

---

## 5. CI/CD Secrets Configuration

Go to: GitHub repo → Settings → Secrets and variables → Actions

| Secret Name | Description | How to Get |
|-------------|-------------|-----------|
| `VORYN_KEYSTORE_BASE64` | Android release keystore (base64) | `base64 -w0 release.keystore` |
| `VORYN_KEYSTORE_PASSWORD` | Keystore password | Set during keytool creation |
| `VORYN_KEY_ALIAS` | Key alias in keystore | Set during keytool creation |
| `VORYN_KEY_PASSWORD` | Key password | Set during keytool creation |
| `UPDATE_SERVER_SSH_KEY` | Private SSH key for deploy user | Generated in section 3.4 |
| `APP_STORE_CONNECT_API_KEY` | Apple API key for TestFlight | App Store Connect → Users → Keys |

### Generate the Android Keystore

```bash
keytool -genkeypair \
    -v \
    -keystore release.keystore \
    -alias voryn-release \
    -keyalg RSA \
    -keysize 4096 \
    -validity 36500 \
    -dname "CN=BitStack Labs, O=BitStack Labs, L=London, C=GB"

# Base64 encode for GitHub secret
base64 -w0 release.keystore > release.keystore.b64

# Store release.keystore securely (password manager, HSM)
# NEVER commit it to the repository
```

---

## 6. App Build & Signing

### Android (Local)

```bash
./scripts/build-release-android.sh
# Output: build/voryn-0.1.0-release.apk + .sha256
```

### iOS (Local, requires macOS)

```bash
./scripts/build-release-ios.sh
# Output: build/voryn-0.1.0-release.ipa
```

### Reproducible Build (Docker)

```bash
docker build -t voryn-builder .
docker run -v $(pwd)/build:/output voryn-builder scripts/build-release-android.sh
sha256sum build/voryn-0.1.0-release.apk  # Compare with published hash
```

---

## 7. Release Process

### Standard Release

```bash
# 1. Ensure main is clean and all tests pass
git checkout main && git pull
cargo test --workspace && yarn lint

# 2. Bump version in Cargo.toml + package.json, update CHANGELOG.md
# 3. Commit the version bump
git add -A && git commit -m "Release v0.2.0"

# 4. Tag and push (triggers CI/CD)
git tag v0.2.0
git push origin main && git push origin v0.2.0

# 5. CI/CD automatically builds, signs, and publishes
# 6. Run post-deployment verification (section 8)
```

### Emergency Hotfix

```bash
git checkout -b hotfix/v0.2.1 v0.2.0
# ... apply fix ...
git commit -m "Fix critical security issue"
git tag v0.2.1
git push origin hotfix/v0.2.1 && git push origin v0.2.1
git checkout main && git merge hotfix/v0.2.1 && git push origin main
```

---

## 8. Post-Deployment Verification

Run after every release:

```bash
# 1. Verify update server
curl -sf https://updates.voryn.app/version.json | jq .latest
# Expected: "0.2.0"

# 2. Verify APK download + hash
curl -sf -o /tmp/test.apk "https://updates.voryn.app/releases/voryn-0.2.0-release.apk"
sha256sum /tmp/test.apk  # Compare with published hash

# 3. Verify bootstrap nodes
for node in boot1 boot2 boot3; do
    nc -zv ${node}.voryn.app 4001 && echo "${node}: OK" || echo "${node}: FAIL"
done

# 4. Smoke test on physical device
#    Fresh install → Create identity → Connect to network → Send message
```

---

## 9. Environments

| | Development | Staging | Production |
|---|---|---|---|
| **Branch** | feature/* | staging | main (tagged) |
| **Bootstrap** | localhost | boot-staging.voryn.app | boot{1,2,3}.voryn.app |
| **Update Server** | none | staging.updates.voryn.app | updates.voryn.app |
| **Build Flag** | `--features dev` | `--features staging` | (default) |
| **Signing** | debug keystore | debug keystore | release keystore |

---

## 10. Monitoring & Alerting

### Setup (UptimeRobot — free tier)

1. Create account at uptimerobot.com
2. Add TCP monitors for boot{1,2,3}.voryn.app:4001 (60s interval)
3. Add HTTPS monitor for updates.voryn.app/version.json (keyword: `"latest"`)
4. Configure alert contacts (email + Telegram/Slack)

### Alert Response

| Condition | Action |
|-----------|--------|
| 1 bootstrap node down | Non-critical. SSH in, `systemctl restart voryn-bootstrap` |
| All bootstrap nodes down | Critical. New users can't join. Fix immediately. |
| Update server down | Low priority. Users can't check updates. Fix within 24h. |
| TLS cert expiry < 14 days | Run `sudo certbot renew` on the server |

---

## 11. Rollback Procedures

### Bad Android Release

```bash
# Point version.json back to previous version
ssh voryn-deploy@updates.voryn.app
echo '{"latest":"0.1.0","minimum":"0.1.0",...}' > /var/www/voryn-updates/version.json
```

### Bad iOS Release

1. App Store Connect → TestFlight → Builds → Stop the bad build
2. Previous build becomes active automatically

### Corrupted Bootstrap Node

```bash
sudo systemctl stop voryn-bootstrap
sudo rm -rf /opt/voryn/data/*       # New identity generated on restart
sudo systemctl start voryn-bootstrap
# Get new PeerId and update app config in next release
```

---

## 12. Maintenance Runbook

### Weekly
- [ ] Check uptime dashboard for all monitors
- [ ] Review bootstrap node logs for errors

### Monthly
- [ ] OS security updates: `apt-get update && apt-get upgrade`
- [ ] Check disk usage: `df -h`
- [ ] Verify TLS certs: `openssl s_client -connect updates.voryn.app:443 </dev/null 2>/dev/null | openssl x509 -noout -dates`

### On Each Release
- [ ] Run section 8 verification checklist
- [ ] Monitor for 24h post-release

### Quarterly
- [ ] Audit SSH access logs
- [ ] Review VPS costs
- [ ] Update this runbook
