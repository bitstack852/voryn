# Development Status

Last updated: 2026-04-22 (end of session 8)

## Project Overview

**Voryn** — Private. Encrypted. Unreachable.

A fully decentralized, end-to-end encrypted messaging app for iOS and Android.
No central servers. No metadata exposure. No compromises.

**Goal: Production deployment for real users.**

---

## Infrastructure Status — ALL LIVE

| Component | Status | URL/Address |
|-----------|--------|-------------|
| Update Server | **LIVE** | `https://updates.voryn.bitstack.website` |
| Relay Node | **LIVE** | `ws://boot1.voryn.bitstack.website:4001/ws` |
| CI/CD Pipeline | **Working** | GitHub Actions (tag-triggered releases) |
| Android Keystore | **Configured** | Stored in GitHub Secrets |
| DNS (Cloudflare) | **Configured** | `updates.voryn` + `boot1.voryn` on `bitstack.website` |
| Coolify | **Running** | 2 services: nginx update server + WebSocket relay node |

### Relay Node (Session 8 — replaced libp2p)
- **Protocol:** WebSocket JSON relay — devices register by pubkey, relay routes encrypted payloads
- **Endpoints:** `/ws` (WebSocket), `/health` (HTTP), `/status` (HTTP peer count)
- **Identity:** Generated on first start, persisted at `/data/node-identity.key` in container
- **Deployment:** Coolify builds from `deploy/coolify/Dockerfile.bootstrap` (no pre-built image)
- **Verified:** `curl http://boot1.voryn.bitstack.website:4001/health` → `{"status":"ok","version":"0.2.0"}`
- **No volume mount** — old identity file from libp2p era was causing crashes; removed volume so identity regenerates fresh each deploy

---

## App Status

### Platforms

| Platform | Build Status | Tested on Device | Transport |
|----------|-------------|-----------------|-----------|
| iOS | **Builds & runs** | **Both iPhone-NST + Acumen-XR** | **WebSocket relay — working** |
| Android | **APK builds** (84MB) | Not tested (no Android phone) | Not yet connected |

### Screens Implemented (10 total)

| Screen | Status | Functionality |
|--------|--------|--------------|
| Onboarding | **Working** | Create identity (real Ed25519), auto-login if exists |
| Passcode Setup | **Working** | Create + confirm numeric passcode after identity creation, skip option |
| Passcode Lock | **Working** | Numeric keypad, attempt tracking, auto-wipe at 10 failures |
| Contacts | **Working** | List contacts with avatars, add contact, long-press for detail, bottom nav |
| Chat | **Working** | Send messages, timestamps, delivery status icons, keyboard-aware |
| Settings | **Working** | Public key display, passcode toggle (requires passcode to remove), debug link, delete identity |
| Add Contact | **Working** | Hex public key input with validation |
| Share Key | **Working** | Formatted key display, system share sheet, how-it-works guide |
| Contact Detail | **Working** | Avatar, full key, safety number, encryption status, remove contact |
| Debug | **Working** | Bootstrap connection test, passcode status, test contacts/messages, activity log |

### Data Flow

```
Create Identity → Real Ed25519 via Rust (sodiumoxide)
                → Keys persisted to AsyncStorage
                → Public key displayed in hex (64 chars)

Passcode Lock → Numeric keypad only
             → Hash stored in AsyncStorage
             → 10 failed attempts → full data wipe
             → Requires passcode to remove

Relay → WebSocket to ws://boot1.voryn.bitstack.website:4001/ws
      → Registers pubkey on connect, auto-reconnects every 5s
      → Ping/pong keepalive every 30s
      → Verified: health endpoint returns ok, both devices connected

Messages → Stored locally in AsyncStorage
         → Pending → Sent → Delivered status transitions
         → Encrypted payloads relayed between devices
```

---

## Rust Native Module — CONNECTED TO iOS

### C FFI Layer (`crates/voryn-core/src/ffi.rs`)

| Function | Purpose | Status |
|----------|---------|--------|
| `voryn_hello()` | Bridge test | **Working on iPhone** |
| `voryn_generate_identity()` | Real Ed25519 keypair | **Working on iPhone** |
| `voryn_compute_safety_number()` | Deterministic safety number | **Compiled** |
| `voryn_encrypt_message()` | DH + XSalsa20-Poly1305 encrypt | **Working on device** |
| `voryn_decrypt_message()` | Decrypt + verify sender sig | **Working on device** |
| `voryn_peer_id_from_public_key()` | Derive peer ID from pubkey | **Working on device** |
| `voryn_free_string()` | Memory cleanup | **Compiled** |

**Note (Session 8):** libp2p FFI functions (`voryn_start_node`, `voryn_stop_node`, `voryn_send_message`, `voryn_poll_event`, `voryn_node_status`) are no longer used. Network transport is now pure JS WebSocket via `NetworkService.ts`. The Rust bridge is only used for crypto operations.

### Bridge Architecture

```
React Native (TypeScript)
  → NetworkService.ts (WebSocket relay — pure JS, no Rust needed for transport)
  → TurboModuleRegistry.get('VorynCore')  ← WORKING — used for crypto only
    → VorynCoreModule.mm (ObjC++ TurboModule)
      → voryn_core.h (C FFI, extern "C" wrapped)
        → libvoryn_core.a (static Rust library, gitignored — build locally)
          → sodiumoxide (libsodium) Ed25519 + XSalsa20-Poly1305
```

### Bridge Layer (`crates/voryn-core/src/bridge.rs`)

| Function | Purpose | Tests |
|----------|---------|-------|
| `generate_identity()` | Real Ed25519 keypair via sodiumoxide | **4/4 passing** |
| `sign_message()` / `verify_signature()` | Ed25519 signing | **Passing** |
| `encrypt_for_peer()` / `decrypt_from_peer()` | DH + XSalsa20-Poly1305 | **Passing** |
| `derive_passcode_key()` | Argon2id key derivation | **Passing** |
| `compute_safety_number()` | Symmetric from two public keys | **Passing** |

### VorynBridge.ts (JavaScript → Native)

- Detects `NativeModules.VorynCore` availability
- Uses Rust for identity generation when bridge is available
- Falls back to JS random bytes when bridge is not available
- Stores `rustGenerated: true/false` flag with identity

---

## Rust Codebase Status

All Rust code compiles clean on macOS (`cargo check --workspace` — 0 errors).

### Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| `voryn-core` | FFI bridge, orchestration, auth, wipe | **Compiled + bridged to iOS** |
| `voryn-crypto` | Ed25519, X25519, XSalsa20-Poly1305, Argon2id | **Compiled, all tests pass** |
| `voryn-network` | libp2p DHT node (UNUSED — replaced by WebSocket relay in Session 8) | **Compiles, not used in app** |
| `voryn-storage` | SQLCipher database, migrations, CRUD | **Compiled, all tests pass** |
| `voryn-protocol` | Double Ratchet, Shamir, group ledger, invites | **Compiled, all tests pass** |
| `voryn-bootstrap` | WebSocket relay server (replaced libp2p DHT in Session 8) | **Live on boot1.voryn.bitstack.website:4001** |

---

## What's Been Completed

### Session 1: Foundation + Infrastructure
- All 7 phases of Rust code (crypto, protocol, storage, network)
- Monorepo with Cargo workspace + Yarn workspaces
- React Native app shell with TypeScript
- CI/CD pipelines on GitHub Actions
- Bootstrap node deployed on Coolify
- Update server deployed on Coolify
- DNS configured on Cloudflare
- Android APK builds successfully
- All documentation written

### Session 2: App + Rust Bridge + iOS
- React Native initialized with iOS + Android native projects
- 10 screens with full dark theme UI
- Real Ed25519 identity via Rust native module on iOS
- Passcode lock system (setup, lock, auto-wipe, settings toggle)
- Network service connecting to bootstrap node
- iOS app tested on physical iPhone
- Bootstrap connection verified from app
- VorynBridge.ts with Rust/JS fallback pattern
- C FFI layer for Rust ↔ Objective-C ↔ React Native

### Session 4: TurboModule Migration — COMPLETED in Session 5

Root cause found and fixed: `RCTModuleProviders.mm` codegen generates an empty `moduleMapping` dictionary for app-level modules. The module was compiled but never registered with the bridgeless runtime.

Fix: Podfile `post_install` hook patches `RCTModuleProviders.mm` after codegen to inject `@"VorynCore": @"VorynCoreModule"`.

### Session 8: WebSocket Relay — Both Devices Messaging

**Goal:** Replace libp2p with a simple WebSocket relay and get real device-to-device messaging working.

**Why:** libp2p circuit relay was unreliable behind iOS NAT. WebSocket is natively supported in React Native JS — no Rust bridge needed for transport. Simpler, auditable, and it works.

**What was done:**

1. **Rewrote `voryn-bootstrap`** — from libp2p DHT to a WebSocket JSON relay server using `warp`. Protocol: devices register by hex pubkey, relay routes encrypted payloads between them. HTTP `/health` and `/status` endpoints added.

2. **Rewrote `NetworkService.ts`** — from libp2p polling to WebSocket relay client. Connects to `ws://boot1.voryn.bitstack.website:4001/ws`, registers identity on connect, auto-reconnects every 5s, ping/pong keepalive every 30s. Backward-compatible API maintained.

3. **Wired `ChatScreen.tsx`** — `handleSend` now stores via `VorynBridge.sendMessage` then calls `NetworkService.sendToPeer` if connected. Incoming messages trigger reload via `NetworkService.onMessage` listener.

4. **Updated `App.tsx`** — single `NetworkService.connect()` call on mount with cleanup on unmount.

5. **Updated `deploy/coolify/docker-compose.yml`** — changed `image: ghcr.io/...` to `build: context:` so Coolify builds from Dockerfile instead of pulling a stale pre-built image. Removed `bootstrap-data` volume (was persisting old libp2p identity file that crashed new binary).

6. **Dead code removed:** `PasscodeEntryScreen.tsx`, `PasscodeSetupScreen.tsx`, `useAuth.ts`, `useMessages.ts`, `MessageStatus.tsx` — all orphaned, never referenced.

7. **Theme color fixes** — `ScanQRScreen.tsx`, `ContactDetailScreen.tsx`, `ShareKeyScreen.tsx` all updated to use `colors.*` constants instead of hardcoded hex values.

**Verified:**
- `curl http://boot1.voryn.bitstack.website:4001/health` → `{"status":"ok","version":"0.2.0"}`
- App built and installed on both iPhone-NST and Acumen-XR
- Both devices connect to relay

**Build commands (Session 8 — updated):**
```bash
# From repo root on Mac
git pull

# Build Rust static lib for iOS
cargo build --release --target aarch64-apple-ios -p voryn-core
cp target/aarch64-apple-ios/release/libvoryn_core.a apps/mobile/ios/VorynRust/libvoryn_core.a

# CocoaPods
cd apps/mobile/ios && pod install && cd ../../..

# Install to device (use --device with display name, NOT --udid)
cd apps/mobile
npx react-native run-ios --device "iPhone - NST"
npx react-native run-ios --device "Acumen-XR"
```

**Devices:**
- iPhone-NST: CoreDevice ID `307AE499-AE17-4FBF-8CBF-0AA226467217`
- Acumen-XR: CoreDevice ID `9D82AFFE-7F98-4E32-BD8C-F1C096E09636`

---

### Session 7: Two-Device Messaging — NAT Traversal + Relay

**Goal:** Get iPhone-NST and Acumen-XR to exchange messages.

**Devices:**
- iPhone-NST: UDID `00008101-001C4D8C2251001E`, CoreDevice ID `307AE499-AE17-4FBF-8CBF-0AA226467217`
- Acumen-XR: UDID `00008020-000C49D93E98002E`, CoreDevice ID `9D82AFFE-7F98-4E32-BD8C-F1C096E09636`

**Root causes identified and fixed:**

1. **mDNS peers never dialed** — `Discovered` handler added peers to Kademlia but never called `swarm.dial()`. Fix: dial each discovered peer.

2. **DHT peers never dialed** — `GetClosestPeers` handler listed returned peers but only called `swarm.dial()` on the first and broke. Fix: iterate all addresses.

3. **Stale DHT addresses / Connection refused** — Random port 0 means each app restart gets a new port; old address stays in DHT. Fix: fixed port 47777 in `VorynBridge.ts`.

4. **Direct TCP always fails between phones** — Both phones are behind iOS NAT; neither can accept inbound TCP. Fix: libp2p circuit relay.
   - Bootstrap now runs `relay::Behaviour` (server)
   - Phones run `relay::client::Behaviour` via `with_relay_client()`
   - On bootstrap connect, phone calls `swarm.listen_on(bootstrap_addr/p2p-circuit)` to reserve relay slot
   - Relay address advertised via `add_external_address` + Identify push

5. **Doubled messages in chat** — `ChatScreen.handleSend` called `VorynBridge.sendMessage` (stores+sends) AND `NetworkService.sendToPeer` (which called `VorynBridge.sendMessage` internally). Fix: removed redundant `sendToPeer` call.

6. **Relay address not propagating** — `identify::Config` had `push_listen_addr_updates=false` (default), so the relay address was never pushed to bootstrap after reservation. Fix: `with_push_listen_addr_updates(true)`.

7. **Relay address skipped in DHT dial** — DHT dial loop broke after first `Ok()` from `swarm.dial()`, meaning relay address (3rd or 4th in list) was never tried. Fix: removed `break`, now tries all addresses.

8. **SwarmBuilder phase order wrong** — `with_relay_client()` was called before `with_dns_config()`. In libp2p 0.54's type-state builder, DNS must wrap TCP first; relay client is layered on top. Fix: swapped order to `tcp → dns_config → relay_client`.

9. **CI pipeline failures** — Two clippy errors: `unused mut` on `|mut b|` in bootstrap, and relay event pattern matching on wrong field names. Fix: removed `mut`, simplified relay events to `debug!("Relay event: {:?}", event)`.

10. **Duplicate messages on retry** — When a send fails, the message is marked 'failed' in storage. User retries, creating a second copy. Fix: `sendMessage` now removes prior failed messages with same content before adding the new attempt.

**CI/CD:**
- `ci.yml` runs `cargo fmt`, `cargo clippy --workspace --all-targets -- -W warnings`, `cargo test --workspace`
- `bootstrap-docker.yml` builds Docker image and pushes to `ghcr.io/bitstack852/voryn-bootstrap` on changes to crates/voryn-bootstrap, crates/voryn-network, Cargo.toml
- Bootstrap image auto-deploys via Coolify watching the registry

**Build commands for iOS (run from Mac in repo root):**
```bash
git pull  # ALWAYS pull latest before building
cargo build --release --target aarch64-apple-ios -p voryn-core
cp target/aarch64-apple-ios/release/libvoryn_core.a apps/mobile/ios/VorynRust/libvoryn_core.a
cd apps/mobile/ios && pod install

# Build (run for each device — use UDID in destination, not CoreDevice UUID)
xcodebuild -workspace Voryn.xcworkspace -scheme Voryn \
  -configuration Debug \
  -destination "id=00008101-001C4D8C2251001E" \
  -allowProvisioningUpdates CODE_SIGN_STYLE=Automatic \
  DEVELOPMENT_TEAM=$(xcodebuild -workspace Voryn.xcworkspace -scheme Voryn \
    -showBuildSettings 2>/dev/null | grep DEVELOPMENT_TEAM | head -1 | awk '{print $3}') \
  2>&1 | grep -E "error:|BUILD FAILED|BUILD SUCCEEDED"

# Install (use CoreDevice UUID from `xcrun devicectl list devices`)
xcrun devicectl device install app \
  --device 9D82AFFE-7F98-4E32-BD8C-F1C096E09636 \
  /Users/nstorres/Library/Developer/Xcode/DerivedData/Voryn-heddhmcluvfafmfwmhmgdtkwwfbf/Build/Products/Debug-iphoneos/Voryn.app
```

**Known gotcha:** `cargo build` output "Finished in 0.53s" means it was CACHED. Run `git pull` first or the phones will have old Rust code.

**Status:** Both phones connect to bootstrap. Relay reservation should be occurring. E2E message delivery under active investigation — relay addresses may not be propagating fast enough on first connect.

---

### Session 6: E2E Encryption Wired

- **3 new C FFI functions** — `voryn_encrypt_message`, `voryn_decrypt_message`, `voryn_peer_id_from_public_key`
- **Encryption algorithm** — DH (X25519 via Ed25519 conversion) + KDF + XSalsa20-Poly1305 + Ed25519 signature over nonce+ciphertext
- **Wire format** — JSON envelope `{v, from, nonce, ct, sig}` hex-encoded, sent as raw bytes over libp2p
- **`voryn_peer_id_from_public_key`** — derives libp2p PeerId deterministically from a contact's 32-byte Ed25519 public key; exposed via `peer_id_from_ed25519_public_key` in `voryn-network`
- **`VorynBridge.sendMessage`** — now encrypts, derives recipient PeerId, and sends via P2P; marks status `sent` on success, `failed` on error (replaces fake 500ms timeout)
- **`NetworkService.decryptAndStore`** — decrypts inbound envelopes, verifies sender signature, stores plaintext via `VorynBridge.receiveMessage`
- **TurboModule spec updated** — 3 new methods in `NativeVorynCore.ts`; codegen regenerated `VorynCoreSpec.h` on pod install
- **Bootstrap still connected** — peers: 1 confirmed after rebuild

#### What was shelved
- Android bridge — deferred indefinitely

### Session 5: Rust Bridge Live + Bootstrap Connected

**All goals achieved this session:**

- **TurboModule fully working** — `TurboModuleRegistry.get('VorynCore')` returns the real Rust module. Confirmed by "Test Rust Bridge (hello)" showing `"Voryn Core v0.1.0 — Private. Encrypted. Unreachable."` without any `(JS fallback)` suffix.
- **Root cause of null bridge fixed** — `RCTModuleProviders.mm` empty `moduleMapping` patched via Podfile `post_install` hook
- **Header fixed** — Correct generated header is `VorynCoreSpec/VorynCoreSpec.h`; added `$(SRCROOT)/build/generated/ios/ReactCodegen` to `HEADER_SEARCH_PATHS` in `project.pbxproj`
- **Linker fixed** — Added `extern "C"` to `voryn_core.h` (prevents C++ name mangling); added `SystemConfiguration`, `CFNetwork`, `Security`, `CoreFoundation` frameworks to `OTHER_LDFLAGS`
- **DNS fixed for iOS** — Replaced `with_dns()` (reads `/etc/resolv.conf`, doesn't exist on iOS) with `with_dns_config(ResolverConfig::cloudflare(), ResolverOpts::default())`; added `hickory-resolver = "0.24"` to `voryn-network/Cargo.toml`
- **Bootstrap connection verified** — App log shows `"Network status: connected, peers: 1"` and `/dns4/boot1.voryn.bitstack.website/tcp/4001/p2p/12D3KooWMnagsbtuh6ytx5VWUPDhq9BePidVwmpEU7GG9ZHTnv3X`
- **Debug UX** — Added "Copy" button next to LOG section header; tapping opens iOS share sheet with full log text for easy pasting

#### Important Notes
- `libvoryn_core.a` is gitignored — must be built locally with `cargo build --release --target aarch64-apple-ios -p voryn-core` then copied to `apps/mobile/ios/VorynRust/`
- All changes on `main` branch
- Build: `xcodebuild -workspace Voryn.xcworkspace -scheme Voryn -configuration Release -destination 'platform=iOS,id=00008101-001C4D8C2251001E' build`
- Install: `xcrun devicectl device install app --device 00008101-001C4D8C2251001E <path/to/Voryn.app>`

### Session 3: P2P Networking Implementation
- Re-enabled libp2p in `voryn-network` (was disabled pending Cargo.lock)
- Added `request-response` + `json` features to workspace
- Full libp2p swarm: Kademlia DHT + mDNS + Identify + TCP/Noise/Yamux
- `/voryn/message/1.0.0` custom protocol via `request_response::json`
- Pending-message queue: messages queued while peer is unreachable, flushed on connect
- Rewritten `voryn-bootstrap` with real libp2p (was TCP stub) — needs redeploy
- 5 new C FFI exports: `voryn_start_node`, `voryn_stop_node`, `voryn_send_message`, `voryn_poll_event`, `voryn_node_status`
- Updated iOS header (`voryn_core.h`) and `VorynCoreModule.m` with new RCT methods
- `NetworkService.ts` now drives the Rust node (poll every 500ms, event handlers)
- `VorynBridge.ts` network functions wired to native module
- `useNetwork` hook reads live state from `NetworkService`

---

## What Still Needs To Be Done (Road to Production)

### Phase A: P2P Networking (Critical Path)

**Goal:** Two phones discover each other and exchange messages.

| Task | Effort | Status |
|------|--------|--------|
| Resolve libp2p yanked dependency | 1 hour | ✅ Done (Session 3) |
| Implement full libp2p swarm (Kademlia DHT + mDNS + TCP/Noise) | 1-2 days | ✅ Done (Session 3) — superseded |
| Wire libp2p node as background thread via FFI | 1 day | ✅ Done (Session 3) — superseded |
| Fix TurboModule registration so Rust bridge loads | 1 day | ✅ Done (Session 5) |
| Fix DNS resolver for iOS (no `/etc/resolv.conf`) | 1 hour | ✅ Done (Session 5) |
| **Replace libp2p with WebSocket relay** | 1 day | ✅ Done (Session 8) — works reliably |
| Deploy relay server | 1 hour | ✅ Done (Session 8) — live at boot1.voryn.bitstack.website:4001 |
| Install on both devices and verify relay connection | 1 hour | ✅ Done (Session 8) — iPhone-NST + Acumen-XR |
| End-to-end message delivery (Phone A → Phone B via relay) | 2 hours | **Next — relay connected, needs two-device message test** |

### Phase B: Real Encrypted Messaging

**Goal:** Messages encrypted with Double Ratchet, delivered over P2P.

| Task | Effort | Status |
|------|--------|--------|
| Wire `encrypt_for_peer`/`decrypt_from_peer` into send/receive path | 4 hours | ✅ Done (Session 6) |
| Wire Double Ratchet session to message send/receive | 1 day | Rust code exists — post-MVP |
| Implement X3DH initial key agreement between two devices | 1 day | Rust code exists — post-MVP |
| Implement delivery ACK protocol | 4 hours | Protocol defined — post-MVP |
| Implement offline message queue (store-and-forward) | 4 hours | Queue code exists — post-MVP |
| Test full flow: Phone A → encrypt → P2P → decrypt → Phone B | 2 hours | **Next — requires two devices** |

### Phase C: Android Rust Bridge

**Goal:** Same Rust crypto on Android.

| Task | Effort | Status |
|------|--------|--------|
| Cross-compile libvoryn_core.a for aarch64-linux-android | 1 hour | cargo-ndk installed |
| Create JNI bridge (Java/Kotlin native module) | 4 hours | Not started |
| Wire to React Native NativeModules on Android | 2 hours | Not started |
| Test Ed25519 identity generation on Android | 1 hour | Not started |

### Phase D: Production Polish

**Goal:** Ready for real users.

| Task | Effort | Status |
|------|--------|--------|
| Sign APK with release keystore + upload to update server | 1 hour | Keystore ready |
| TestFlight distribution for iOS | 2 hours | Xcode configured |
| Add additional bootstrap nodes (2-3 for redundancy) | 2 hours | Coolify ready |
| Wire Argon2id passcode derivation via Rust (replace JS hash) | 2 hours | Rust code exists |
| Screenshot/screen recording prevention | 2 hours | Rust code exists |
| App icon and splash screen | 2 hours | Not started |
| Push notifications for incoming messages | 1 day | Not started |
| Network resilience testing (churn, partition, reconnect) | 4 hours | Not started |
| External security audit of crypto implementations | 2 weeks | Not started |
| Penetration testing | 1 week | Not started |

### Phase E: Group Messaging + Advanced Features

**Goal:** Full feature set from the spec.

| Task | Effort | Status |
|------|--------|--------|
| Wire group ledger to UI (create group, add members) | 2 days | Rust code exists |
| Wire Shamir's Secret Sharing for group key distribution | 1 day | Rust code exists |
| Implement invite token generation and redemption UI | 1 day | Rust code exists |
| Implement auto-delete timer UI | 4 hours | Rust code exists |
| Implement identity revocation flow | 4 hours | Rust code exists |
| Implement duress passcode setup | 2 hours | Rust code exists |
| Implement remote wipe from trusted contact | 4 hours | Rust code exists |

---

## Priority Order for Production

```
1. Phase A: P2P Networking        ← CRITICAL — without this, no messaging
2. Phase B: Real Encrypted Messaging ← CRITICAL — the core product
3. Phase D: Production Polish      ← Required for real users
4. Phase C: Android Rust Bridge    ← Expands to Android users
5. Phase E: Group + Advanced       ← Feature expansion
```

**Estimated time to MVP (Phases A+B+D): 1 focused session + two-device testing** (encryption done, P2P done, needs real device-to-device verification then distribution)

---

## Development Environment

### Required Tools (all installed on Mac)
- **macOS 15.6.1** (Sequoia)
- **Xcode 16.2** (in /Applications)
- **Rust 1.94.1** (targets: aarch64-apple-ios, x86_64-unknown-linux-musl)
- **Node.js 24.8.0** + Yarn 1.22.22
- **Java 17** (OpenJDK via Homebrew)
- **Android SDK** (command line tools via Homebrew)
- **CocoaPods** (via Homebrew Ruby 4.0.2)
- **musl-cross** (for Linux cross-compilation)

### Build Commands

```bash
# ── Rust ──────────────────────────────────────────────
cargo check --workspace              # Verify all crates compile
cargo test --workspace               # Run all tests
cargo test -p voryn-core -- bridge   # Run bridge tests only

# Build iOS static library for Rust crypto bridge
cargo build --release --target aarch64-apple-ios -p voryn-core
cp target/aarch64-apple-ios/release/libvoryn_core.a apps/mobile/ios/VorynRust/libvoryn_core.a

# ── iOS — Install to physical device (Session 8 method) ─────
cd /Users/nstorres/Documents/GitHub/voryn
git pull   # ALWAYS pull first

cargo build --release --target aarch64-apple-ios -p voryn-core
cp target/aarch64-apple-ios/release/libvoryn_core.a apps/mobile/ios/VorynRust/libvoryn_core.a

cd apps/mobile/ios
pod install
cd ..

# Build and install — use --device with display name (NOT --udid)
npx react-native run-ios --device "iPhone - NST"
npx react-native run-ios --device "Acumen-XR"

# ── iOS — Metro bundler (if needed separately) ────────
cd apps/mobile && npx react-native start

# ── Android APK ──────────────────────────────────────
cd apps/mobile
mkdir -p node_modules
ln -sf /Users/nstorres/Documents/GitHub/voryn/node_modules/react-native node_modules/react-native
ln -sf /Users/nstorres/Documents/GitHub/voryn/node_modules/@react-native node_modules/@react-native
ln -sf /Users/nstorres/Documents/GitHub/voryn/node_modules/@react-native-async-storage node_modules/@react-native-async-storage
export JAVA_HOME=/opt/homebrew/opt/openjdk@17
export ANDROID_HOME=/opt/homebrew/share/android-commandlinetools
cd android && ./gradlew assembleRelease

# ── Coolify relay redeploy ────────────────────────────
# Coolify builds from Dockerfile on each deploy trigger.
# After pushing code to main: Coolify → bootstrap service → Redeploy
# DO NOT just "Restart" — that reuses the existing container without rebuilding.
# "Redeploy" = rebuild from Dockerfile + start fresh container.
```

### Known Gotchas (Session 8 — lessons learned)

| Gotcha | Detail |
|--------|--------|
| Coolify Restart ≠ Redeploy | "Restart" reuses the running container. "Redeploy" rebuilds from Dockerfile. Always use Redeploy after pushing code changes. |
| `image:` vs `build:` in docker-compose | If compose has `image: ghcr.io/...`, Coolify pulls the old image on redeploy. Change to `build: context:` so it builds from the Dockerfile in the repo. |
| Old identity file in Docker volume | `voryn-bootstrap` had a volume at `/data`. Old libp2p identity file had wrong JSON schema, crashing the new WebSocket binary on startup. Fix: removed volume entirely from compose. |
| Can't SSH into crash-looping container | Container exits before you can exec into it. Fix the code, redeploy, or add graceful error handling (regenerate instead of crash) in the binary itself. |
| `--udid` flag is for simulators | `npx react-native run-ios --udid` does not work for physical devices. Use `--device "Display Name"` instead. |
| `git pull` before building | If `cargo build` says "Finished in 0.4s", it used the cache. Always pull latest first or the device gets stale Rust code. |
| macOS BSD `sed` can't insert real newlines | `sed 's/foo/bar\n/g'` on macOS inserts literal `\n`, not a newline. Use Python or write the file directly instead. |
| Never `--ff-only` on diverged branches | If server's `origin/main` is stale vs GitHub's main, `--ff-only` fails. Use `git merge --no-ff` or rebase. |
| Git remote proxy vs GitHub | The server's git remote proxies GitHub. It can lag behind. Always `git fetch origin` and check before assuming branch state. |

### Known Build Issues

| Issue | Workaround |
|-------|-----------|
| Yarn workspace hoists node_modules to root | Create symlinks in apps/mobile/node_modules |
| Xcode `PhaseScriptExecution` fails | Set `REACT_NATIVE_PATH` in .xcode.env.local |
| Xcode sandbox errors on Pod scripts | Set User Script Sandboxing = No in both project and Pods |
| `ENABLE_USER_SCRIPT_SANDBOXING` resets on pod install | Re-run `sed` command after pod install |
| 22MB libvoryn_core.a too large for git | Added to .gitignore, build locally |

---

## Repository Structure

```
voryn/
├── apps/mobile/                # React Native app (iOS + Android)
│   ├── src/
│   │   ├── screens/            # 10 screens
│   │   ├── services/           # VorynBridge, NetworkService, PasscodeService
│   │   ├── navigation/         # RootNavigator with passcode flow
│   │   ├── components/         # MessageStatus
│   │   ├── hooks/              # useNetwork, useMessages, useAuth
│   │   └── theme/              # colors
│   ├── android/                # Android native project (Gradle)
│   ├── ios/                    # iOS native project (Xcode)
│   │   ├── Voryn/              # VorynCoreModule.m, voryn_core.h
│   │   ├── VorynRust/          # libvoryn_core.a (built locally, gitignored)
│   │   └── Pods/               # CocoaPods dependencies
│   ├── index.js                # RN entry point
│   └── package.json            # RN 0.85.1, React 19.2.3
├── crates/
│   ├── voryn-core/             # FFI bridge, auth, wipe, keystore (staticlib)
│   │   ├── src/bridge.rs       # Rust functions callable from RN
│   │   ├── src/ffi.rs          # C FFI exports (#[no_mangle])
│   │   ├── voryn-core.udl      # UniFFI interface definition
│   │   └── include/            # C header
│   ├── voryn-crypto/           # Ed25519, X25519, encryption, passcode
│   ├── voryn-network/          # libp2p node, transport, obfuscation
│   ├── voryn-storage/          # SQLCipher database, migrations, CRUD
│   ├── voryn-protocol/         # Double Ratchet, Shamir, groups, invites
│   └── voryn-bootstrap/        # Standalone bootstrap server binary
├── deploy/
│   ├── coolify/                # Docker Compose for Coolify
│   └── bootstrap/              # Provision scripts, systemd, Dockerfile
├── docs/                       # All documentation (10 docs)
├── scripts/                    # Build and release scripts
└── .github/workflows/          # CI/CD pipelines (4 workflows)
```
