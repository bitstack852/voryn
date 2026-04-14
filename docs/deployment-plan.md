# Deployment Plan

## Overview

Voryn is a fully decentralized app — there are no application servers to deploy. Deployment consists of:

1. **Bootstrap nodes** — seed peers for DHT network discovery
2. **Update server** — static file hosting for version checks and APK downloads
3. **CI/CD pipeline** — automated build, test, sign, and publish
4. **App releases** — iOS (TestFlight) and Android (signed APK)

---

## 1. Infrastructure

### Bootstrap Nodes

Bootstrap nodes are regular Voryn libp2p nodes that remain online 24/7 to help new devices join the DHT network. They do **not** store messages or relay traffic — they only assist with peer discovery.

**Requirements:**
- 2-3 VPS instances across different providers/regions for redundancy
- Minimal resources: 1 vCPU, 512MB RAM, 10GB disk
- Static IP or DNS hostname
- Inbound TCP/UDP open on the libp2p listen port

**Recommended providers (privacy-focused):**
- Njalla (anonymous domain + VPS)
- 1984 Hosting (Iceland, privacy-respecting)
- BuyVM (offshore-friendly)

**Deployment:**
```bash
# On each bootstrap node
git clone https://github.com/bitstack852/voryn.git
cd voryn
cargo build --release -p voryn-core

# Run as systemd service
sudo cp deploy/voryn-bootstrap.service /etc/systemd/system/
sudo systemctl enable --now voryn-bootstrap
```

**Bootstrap node systemd service:**
```ini
# deploy/voryn-bootstrap.service
[Unit]
Description=Voryn Bootstrap Node
After=network.target

[Service]
Type=simple
User=voryn
ExecStart=/opt/voryn/voryn-bootstrap --listen /ip4/0.0.0.0/tcp/4001
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

**Hardcoded bootstrap peers** (compiled into the app):
```
/dns4/boot1.voryn.app/tcp/4001/p2p/<PEER_ID_1>
/dns4/boot2.voryn.app/tcp/4001/p2p/<PEER_ID_2>
/dns4/boot3.voryn.app/tcp/4001/p2p/<PEER_ID_3>
```

### Update Server

A simple static file server hosting `version.json` and Android APKs.

**Requirements:**
- Static site hosting (Cloudflare Pages, Netlify, or self-hosted nginx)
- HTTPS required (certificate pinning in the app)
- No dynamic backend — only static files

**Structure:**
```
updates.voryn.app/
  version.json           # Current version metadata
  releases/
    voryn-0.1.0.apk      # Signed Android APK
    voryn-0.1.0.apk.sha256
    voryn-0.2.0.apk
    voryn-0.2.0.apk.sha256
```

---

## 2. Environments

| Environment | Purpose | Bootstrap Nodes | Update Server |
|------------|---------|-----------------|---------------|
| **Development** | Local testing | Local nodes (localhost) | None |
| **Staging** | Pre-release testing | 1 staging bootstrap node | staging.updates.voryn.app |
| **Production** | Live users | 3 production bootstrap nodes | updates.voryn.app |

### Environment Configuration

The app determines its environment via a build-time flag:

```rust
// crates/voryn-network/src/config.rs
#[cfg(feature = "staging")]
pub const BOOTSTRAP_PEERS: &[&str] = &[
    "/dns4/boot-staging.voryn.app/tcp/4001/p2p/...",
];

#[cfg(not(feature = "staging"))]
pub const BOOTSTRAP_PEERS: &[&str] = &[
    "/dns4/boot1.voryn.app/tcp/4001/p2p/...",
    "/dns4/boot2.voryn.app/tcp/4001/p2p/...",
    "/dns4/boot3.voryn.app/tcp/4001/p2p/...",
];
```

---

## 3. CI/CD Release Pipeline

### Trigger

Releases are triggered by pushing a semver tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

### Pipeline Stages

```
Tag Push (v*)
  │
  ├─ Stage 1: Validate
  │   ├─ cargo fmt --check
  │   ├─ cargo clippy -- -D warnings
  │   ├─ cargo test --workspace
  │   ├─ eslint + prettier check
  │   └─ jest tests
  │
  ├─ Stage 2: Build Rust (parallel)
  │   ├─ iOS targets (aarch64-apple-ios, aarch64-apple-ios-sim)
  │   └─ Android targets (aarch64, armv7, x86_64, i686)
  │
  ├─ Stage 3: Build Apps (parallel)
  │   ├─ iOS: xcodebuild → IPA → TestFlight upload
  │   └─ Android: Gradle assembleRelease → signed APK
  │
  ├─ Stage 4: Sign & Hash
  │   ├─ Android APK signing (via Cloud KMS or encrypted keystore)
  │   └─ SHA-256 hash generation for APK
  │
  ├─ Stage 5: Publish
  │   ├─ Upload APK + hash to update server
  │   ├─ Update version.json
  │   ├─ Upload IPA to App Store Connect (TestFlight)
  │   └─ Create GitHub Release with changelog
  │
  └─ Stage 6: Verify
      ├─ Download published APK, verify SHA-256
      ├─ Install on test device, verify version string
      └─ Verify bootstrap node connectivity
```

### GitHub Actions Workflow

```yaml
# .github/workflows/release.yml (to be created)
name: Release
on:
  push:
    tags: ['v*']

jobs:
  validate:
    # ... lint + test (same as ci.yml)

  build-android:
    needs: validate
    runs-on: ubuntu-latest
    steps:
      - # Build Rust for Android
      - # Gradle assembleRelease
      - # Sign APK
      - # Upload artifact

  build-ios:
    needs: validate
    runs-on: macos-latest
    steps:
      - # Build Rust for iOS
      - # xcodebuild
      - # Upload to TestFlight

  publish:
    needs: [build-android, build-ios]
    steps:
      - # Upload APK to update server
      - # Update version.json
      - # Create GitHub Release
```

---

## 4. Versioning Strategy

**Semantic Versioning (semver):**
- `MAJOR.MINOR.PATCH` (e.g., `0.1.0`, `0.2.0`, `1.0.0`)
- **MAJOR:** Breaking protocol changes (requires all users to update)
- **MINOR:** New features, backward-compatible protocol changes
- **PATCH:** Bug fixes, security patches

**Pre-1.0 (current):**
- `0.x.y` — API and protocol are unstable
- Breaking changes expected between minor versions

**Version 1.0.0 criteria:**
- External security audit passed
- Penetration test passed
- 30-day staging period with no critical bugs
- All Phase 7 milestones complete

---

## 5. Release Checklist

### Before Tagging

- [ ] All CI checks pass on `main`
- [ ] Changelog updated in `CHANGELOG.md`
- [ ] Version bumped in `Cargo.toml`, `package.json`, app configs
- [ ] Staging build tested on physical iOS + Android devices
- [ ] No known critical or high-severity bugs
- [ ] Security-sensitive changes reviewed by second developer

### After Publishing

- [ ] APK SHA-256 matches build output
- [ ] TestFlight build available for download
- [ ] version.json updated on update server
- [ ] GitHub Release created with changelog
- [ ] Bootstrap nodes verified online and responsive
- [ ] Smoke test: fresh install → create identity → send message

---

## 6. Rollback Procedure

Since Voryn is decentralized, "rollback" means reverting the app to a previous version.

### Android
1. Update `version.json` to point to the previous APK
2. Set `minimum` version to allow the old version
3. Users will be prompted to "update" to the older version

### iOS (TestFlight)
1. Stop the new build's distribution in App Store Connect
2. The previous build automatically becomes the active version
3. Users on the bad build can reinstall from TestFlight

### Protocol Rollback
If a protocol change causes incompatibility:
1. Tag and release a hotfix that supports both old and new protocol
2. Set `minimum` version to the hotfix
3. Phase out old protocol in the next minor release

---

## 7. Monitoring

Since there are no central servers, monitoring is limited to:

### Bootstrap Node Health
- Uptime monitoring (UptimeRobot, Hetrix, or self-hosted)
- Ping check: can a fresh libp2p node connect and discover peers?
- Alert on: node unreachable for > 5 minutes

### Update Server Health
- HTTPS endpoint monitoring
- Alert on: `version.json` unreachable or invalid JSON

### Crash Reporting (Opt-In)
- **Not enabled by default** (privacy commitment)
- If user opts in: anonymous crash reports via Sentry or self-hosted equivalent
- No message content, contact data, or identity information in crash reports
- Only: stack trace, device model, OS version, app version

### Network Health Metrics (Aggregated, Anonymous)
- DHT peer count (reported by bootstrap nodes only)
- Average peer discovery time
- No per-user metrics collected
