# Development Status

Last updated: 2026-04-15

## Project Overview

**Voryn** — Private. Encrypted. Unreachable.

A fully decentralized, end-to-end encrypted messaging app for iOS and Android.
No central servers. No metadata exposure. No compromises.

---

## Infrastructure Status

| Component | Status | URL/Address |
|-----------|--------|-------------|
| Update Server | **Live** | `https://updates.voryn.bitstack.website` |
| Bootstrap Node | **Live** | `boot1.voryn.bitstack.website:4001` |
| CI/CD Pipeline | **Working** | GitHub Actions (tag-triggered releases) |
| Android Keystore | **Configured** | Stored in GitHub Secrets |
| DNS (Cloudflare) | **Configured** | `updates.voryn` + `boot1.voryn` on `bitstack.website` |
| Coolify | **Running** | 2 services: nginx update server + bootstrap node |

### Bootstrap Node Identity
- **PeerId:** `f85f6881ffa135afee8d29194c4498af69ead527e91c87add55e13e033bbd7ba`
- **Binary:** Cross-compiled from macOS → Linux (x86_64-unknown-linux-musl)
- **Persistent:** Identity key stored in Docker volume at `/data/node-identity.key`

---

## App Status

### Platforms Tested

| Platform | Build Status | Tested on Device |
|----------|-------------|-----------------|
| iOS | **Builds & runs** | Tested on iPhone (via Xcode 16.2) |
| Android | **APK builds** (84MB) | Not tested on device (no Android phone available) |

### Screens Implemented

| Screen | Status | Functionality |
|--------|--------|--------------|
| Onboarding | **Working** | Create identity, auto-login if identity exists |
| Contacts | **Working** | List contacts, add contact, long-press for detail |
| Chat | **Working** | Send messages, timestamps, delivery status icons |
| Settings | **Working** | Public key display, contact count, delete identity |
| Add Contact | **Working** | Hex public key input with validation |
| Share Key | **Working** | Formatted key display, system share sheet |
| Contact Detail | **Working** | Avatar, full key, safety number, remove contact |
| Debug | **Working** | Test bootstrap, add test contacts, send test messages, activity log |

### Data Persistence

| Feature | Storage | Status |
|---------|---------|--------|
| Identity (public/secret key) | AsyncStorage | **Working** |
| Contacts | AsyncStorage | **Working** |
| Messages | AsyncStorage | **Working** |
| Identity deletion (full wipe) | AsyncStorage.multiRemove | **Working** |

---

## Rust Codebase Status

All Rust code compiles clean (`cargo check --workspace` passes).

### Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| `voryn-core` | UniFFI entry point, orchestration | **Compiles** — not yet bridged to RN |
| `voryn-crypto` | Ed25519, X25519, XSalsa20-Poly1305, Argon2id | **Compiles** — full implementation with tests |
| `voryn-network` | libp2p DHT node, transport, discovery | **Compiles** — stubs (libp2p disabled in CI due to yanked deps) |
| `voryn-storage` | SQLCipher database, migrations, CRUD | **Compiles** — full implementation with tests |
| `voryn-protocol` | Double Ratchet, Shamir, group ledger, invites | **Compiles** — full implementation with tests |
| `voryn-bootstrap` | Standalone DHT bootstrap server | **Compiles & runs** — deployed on Coolify |

### Crypto Implementations

| Algorithm | Crate | Status |
|-----------|-------|--------|
| Ed25519 keypair generation | `voryn-crypto` | **Implemented** (via sodiumoxide) |
| Ed25519 signing/verification | `voryn-crypto` | **Implemented** with tests |
| X25519 Diffie-Hellman | `voryn-crypto` | **Implemented** (Ed25519 → X25519 conversion) |
| XSalsa20-Poly1305 encryption | `voryn-crypto` | **Implemented** with encrypt/decrypt roundtrip tests |
| HKDF key derivation | `voryn-crypto` | **Implemented** |
| Argon2id passcode derivation | `voryn-crypto` | **Implemented** with key wrapping/unwrapping |
| Double Ratchet protocol | `voryn-protocol` | **Implemented** — session init, encrypt, decrypt, out-of-order delivery, forward secrecy |
| X3DH key agreement | `voryn-protocol` | **Implemented** — pre-key bundles, initiator-side computation |
| Shamir's Secret Sharing | `voryn-protocol` | **Implemented** — GF(256) arithmetic, split/reconstruct with threshold property |
| Safety numbers | `voryn-protocol` | **Implemented** — symmetric computation from two public keys |

### Protocol Implementations

| Feature | Crate | Status |
|---------|-------|--------|
| Encrypted message format | `voryn-protocol` | **Implemented** — bincode serialization |
| Delivery acknowledgment | `voryn-protocol` | **Implemented** — ACK protocol |
| Group ledger (hash-chained) | `voryn-protocol` | **Implemented** — tamper detection, member tracking |
| Group admin controls | `voryn-protocol` | **Implemented** — permission model, event authorization |
| Invite tokens (Base58) | `voryn-protocol` | **Implemented** — single-use, time-limited |
| Identity revocation | `voryn-protocol` | **Implemented** — revocation list, broadcast notice |
| Remote wipe | `voryn-protocol` | **Implemented** — signed command with replay prevention |
| Auto-delete timers | `voryn-protocol` | **Implemented** — per-conversation and group policies |
| Message queue | `voryn-storage` | **Implemented** — persistent outbound queue with retry |

### Security Implementations

| Feature | Location | Status |
|---------|----------|--------|
| Attempt limiter (brute force) | `voryn-core` | **Implemented** — configurable limit + exponential backoff |
| Duress passcode | `voryn-core` | **Implemented** — empty/decoy state on coercion |
| Time lock | `voryn-core` | **Implemented** — re-auth after inactivity |
| Secure data wipe | `voryn-core` | **Implemented** — 3-pass overwrite + key deletion |
| Memory zeroing | `voryn-crypto` | **Implemented** — SecureBytes/SecureString with zeroize |
| Traffic padding | `voryn-network` | **Implemented** — fixed-size bucket padding |
| Timing obfuscation | `voryn-network` | **Implemented** — random send delays |
| Chaff traffic | `voryn-network` | **Implemented** — dummy message generation |
| Hardware keystore trait | `voryn-core` | **Implemented** — trait + software fallback |

---

## CI/CD Status

| Workflow | Trigger | Status |
|----------|---------|--------|
| `ci.yml` | Push to main, PRs | **Working** — Rust fmt + clippy + tests |
| `release.yml` | Tag push (v*) | **Working** — validate + build Android/iOS + publish release |
| `build-android.yml` | Manual (workflow_dispatch) | **Available** |
| `build-ios.yml` | Manual (workflow_dispatch) | **Available** |

### GitHub Secrets Configured

- `VORYN_KEYSTORE_BASE64` — Android release keystore
- `VORYN_KEYSTORE_PASSWORD` — Keystore password
- `VORYN_KEY_ALIAS` — Key alias (`voryn-release`)
- `VORYN_KEY_PASSWORD` — Key password

---

## Documentation Status

| Document | Path | Status |
|----------|------|--------|
| README | `/README.md` | **Complete** |
| Contributing guide | `/CONTRIBUTING.md` | **Complete** |
| Architecture | `/docs/architecture.md` | **Complete** |
| Environment setup | `/docs/environment-setup.md` | **Complete** |
| Deployment plan | `/docs/deployment-plan.md` | **Complete** |
| Distribution guide | `/docs/distribution.md` | **Complete** |
| Coolify setup guide | `/docs/coolify-setup-guide.md` | **Complete** |
| Infrastructure setup | `/docs/infrastructure-setup-guide.md` | **Complete** |
| Threat model | `/docs/security/threat-model.md` | **Complete** |
| Audit checklist | `/docs/security/audit-checklist.md` | **Complete** |
| Technical spec | `/docs/Voryn_Technical_Development_Plan.docx` | **Original spec** |

---

## What's Been Completed

### Phase 0: Project Scaffolding
- Monorepo structure (Cargo workspace + Yarn workspaces)
- 5 Rust crates + 1 binary crate
- React Native app with TypeScript
- CI/CD pipelines (GitHub Actions)
- Developer tooling (ESLint, Prettier, Clippy, rustfmt)
- Documentation suite

### Phase 1: Foundation
- Ed25519 identity generation
- X25519 key exchange
- XSalsa20-Poly1305 encryption
- SQLCipher storage with schema v1
- libp2p node structure (stubbed)
- Basic encrypted message exchange logic
- Minimal UI shell

### Phase 2: Core Messaging
- Double Ratchet protocol (full implementation)
- X3DH key agreement
- Message queue with delivery confirmation
- Argon2id passcode key derivation
- Safety numbers for contact verification

### Phase 3: Security Hardening
- Attempt limiter with data wipe trigger
- Duress passcode (decoy state)
- Time lock (re-auth on inactivity)
- Secure data wipe (3-pass overwrite)
- Traffic padding, timing jitter, chaff traffic
- Remote wipe protocol

### Phase 4: Group Messaging
- Hash-chained cryptographic ledger
- Shamir's Secret Sharing (GF(256))
- Group key management and resharing
- Admin role permissions
- Group message sync

### Phase 5: Invite System
- Base58 invite tokens (single-use, time-limited)
- Token validation
- New member onboarding protocol
- Identity revocation and broadcast

### Phase 6: Auto Delete
- Per-conversation timers (1h, 24h, 7d, 30d, custom)
- Group auto-delete policies
- Delivery-triggered timer start

### Phase 7: Polish & Hardening
- Threat model document
- Security audit checklist
- Distribution guide

### Infrastructure Deployment
- Update server on Coolify (nginx, HTTPS via Let's Encrypt)
- Bootstrap node on Coolify (custom Rust binary, port 4001)
- DNS configured on Cloudflare
- Android APK build pipeline working
- iOS build tested on physical iPhone

### App Functionality
- Identity generation and persistence
- Contact management (add, remove, list)
- Message sending and local storage
- 8 screens with dark theme UI
- Share public key via system share sheet
- Contact detail with safety number
- Debug console

---

## What Still Needs To Be Done

### Priority 1: UniFFI Bridge (Rust ↔ React Native)
- **What:** Connect the Rust crypto crate to the React Native app via UniFFI
- **Why:** Currently the app uses random bytes for keys, not real Ed25519
- **Effort:** 2-3 sessions
- **Tasks:**
  - Define UniFFI `.udl` interface for identity, encryption, and storage
  - Cross-compile `voryn-core` for iOS (`aarch64-apple-ios`) and Android (`aarch64-linux-android`)
  - Generate React Native Turbo Module bindings via `uniffi-bindgen-react-native`
  - Replace `VorynBridge.ts` mock functions with real UniFFI calls
  - Test real Ed25519 key generation on both platforms

### Priority 2: libp2p Integration
- **What:** Enable real P2P networking so devices can discover and message each other
- **Why:** Currently messages are stored locally only — no actual network communication
- **Effort:** 2-3 sessions
- **Tasks:**
  - Resolve libp2p dependency issues (yanked crates)
  - Implement the full libp2p swarm with Kademlia DHT + mDNS
  - Wire bootstrap node PeerId into app config
  - Test device discovery between two phones on same network (mDNS)
  - Test device discovery across networks (via bootstrap node)

### Priority 3: Real Encrypted Messaging
- **What:** Two phones actually exchange encrypted messages over the P2P network
- **Why:** This is the core value proposition of Voryn
- **Effort:** 1-2 sessions (after UniFFI + libp2p are working)
- **Tasks:**
  - Implement message send over libp2p custom protocol
  - Implement message receive and decryption
  - Implement delivery confirmation (ACK)
  - Test full flow: Phone A sends → network → Phone B receives + decrypts
  - Implement offline message queue (store-and-forward when recipient comes online)

### Priority 4: Passcode Lock Screen
- **What:** Require passcode/biometric on app open
- **Why:** Security layer protecting the encryption keys
- **Effort:** 1 session
- **Tasks:**
  - Wire PasscodeSetupScreen and PasscodeEntryScreen into the navigation flow
  - Implement Argon2id key derivation on device (via Rust bridge)
  - Store wrapped key in device keychain

### Priority 5: Production Polish
- **What:** Final hardening before any real users
- **Effort:** 1-2 sessions
- **Tasks:**
  - External security audit of crypto implementations
  - Penetration testing
  - Signed APK uploaded to update server
  - TestFlight distribution for iOS
  - Network resilience testing (churn, partition, NAT traversal)
  - UI/UX refinement based on testing feedback

---

## Development Environment

### Required Tools
- **macOS** (for iOS builds)
- **Xcode 16.2** (installed)
- **Rust 1.94.1** (installed, with `aarch64-apple-ios` and `x86_64-unknown-linux-musl` targets)
- **Node.js 24** + Yarn 1.22 (installed)
- **Java 17** (OpenJDK via Homebrew)
- **Android SDK** (command line tools via Homebrew)
- **CocoaPods** (via Homebrew Ruby 4.0)

### Build Commands

```bash
# Rust — check all crates compile
cargo check --workspace

# Rust — run tests
cargo test --workspace

# Rust — build bootstrap binary for Linux
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc \
  cargo build --release --target x86_64-unknown-linux-musl -p voryn-bootstrap

# Android APK
cd apps/mobile
yarn install  # from repo root first
mkdir -p node_modules
ln -sf /path/to/repo/node_modules/react-native node_modules/react-native
ln -sf /path/to/repo/node_modules/@react-native node_modules/@react-native
ln -sf /path/to/repo/node_modules/@react-native-async-storage node_modules/@react-native-async-storage
export JAVA_HOME=/opt/homebrew/opt/openjdk@17
export ANDROID_HOME=/opt/homebrew/share/android-commandlinetools
cd android && ./gradlew assembleRelease

# iOS — open in Xcode
cd apps/mobile/ios
pod install  # using /opt/homebrew/lib/ruby/gems/4.0.0/bin/pod
open Voryn.xcworkspace  # then Cmd+R to build and run
```

---

## Repository Structure

```
voryn/
├── apps/mobile/              # React Native app (iOS + Android)
│   ├── src/screens/          # 8 screens (Onboarding, Contacts, Chat, etc.)
│   ├── src/services/         # VorynBridge (app logic layer)
│   ├── android/              # Android native project
│   └── ios/                  # iOS native project
├── crates/
│   ├── voryn-core/           # UniFFI entry point, auth, wipe, keystore
│   ├── voryn-crypto/         # Ed25519, X25519, encryption, passcode
│   ├── voryn-network/        # libp2p node, transport, obfuscation
│   ├── voryn-storage/        # SQLCipher database, migrations, CRUD
│   ├── voryn-protocol/       # Double Ratchet, Shamir, groups, invites
│   └── voryn-bootstrap/      # Standalone bootstrap server binary
├── deploy/                   # Coolify + Docker deployment configs
├── docs/                     # All documentation
├── scripts/                  # Build and release scripts
└── .github/workflows/        # CI/CD pipelines
```
