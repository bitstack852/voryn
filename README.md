# Voryn

**Private. Encrypted. Unreachable.**

Voryn is a fully decentralized, end-to-end encrypted messaging application for iOS and Android. No central servers. No metadata exposure. No compromises.

## Architecture

```
┌─────────────────────────────────────────────┐
│           React Native (TypeScript)          │
│         UI Shell · Navigation · State        │
├──────────────────┬──────────────────────────┤
│                  │   UniFFI Bridge           │
├──────────────────┴──────────────────────────┤
│              Rust Core (voryn-core)          │
├────────────┬───────────┬────────────────────┤
│ voryn-     │ voryn-    │ voryn-   │ voryn-  │
│ crypto     │ network   │ storage  │ protocol│
│ (libsodium)│ (libp2p)  │(SQLCipher)│(Ratchet)│
├────────────┴───────────┴──────────┴─────────┤
│     Secure Enclave (iOS) / StrongBox (Android)│
└─────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Cross-Platform | React Native (TypeScript) |
| Crypto & Networking | Rust via UniFFI |
| P2P Network | rust-libp2p (QUIC + Kademlia DHT) |
| Encryption | libsodium (Ed25519, X25519, XSalsa20-Poly1305) |
| Messaging Protocol | Double Ratchet Algorithm |
| Group Keys | Shamir's Secret Sharing |
| Local Storage | SQLCipher (encrypted SQLite) |
| Hardware Security | Secure Enclave (iOS) / StrongBox (Android) |

## Project Structure

```
voryn/
├── apps/mobile/          # React Native app
│   └── src/
│       ├── screens/      # UI screens
│       ├── navigation/   # React Navigation
│       ├── hooks/        # React hooks
│       ├── services/     # Rust bridge wrappers
│       └── theme/        # Design tokens
├── crates/
│   ├── voryn-core/       # UniFFI entry point, orchestration
│   ├── voryn-crypto/     # Ed25519, X25519, encryption, KDF
│   ├── voryn-network/    # libp2p node, DHT, protocols
│   ├── voryn-storage/    # SQLCipher database, migrations
│   └── voryn-protocol/   # Double Ratchet, Shamir, groups
├── packages/
│   └── shared-types/     # Shared TypeScript definitions
├── .github/workflows/    # CI/CD pipelines
├── docs/                 # Architecture & setup docs
└── scripts/              # Build & release scripts
```

## Development

### Prerequisites

- **Rust** (stable, via rustup)
- **Node.js** 20+ and Yarn
- **Xcode** 15+ (for iOS)
- **Android Studio** + NDK (for Android)
- **cargo-ndk** (`cargo install cargo-ndk`)

### Quick Start

```bash
# Clone and enter repo
git clone https://github.com/bitstack852/voryn.git
cd voryn

# Install JS dependencies
yarn install

# Check Rust workspace compiles
cargo check --workspace

# Run Rust tests
cargo test --workspace
```

See [docs/environment-setup.md](docs/environment-setup.md) for full setup instructions.

## Security Principles

1. **Fully Decentralized** — Every device is a full DHT node
2. **Zero Trust** — All identities and messages cryptographically verified
3. **No Plaintext Outside Memory** — Encrypted from composition to storage
4. **Invite-Only** — Closed community with cryptographic identity
5. **No Metadata Exposure** — Even delivery confirmations are encrypted
6. **Hardware-Bound Keys** — Private keys never leave the Secure Enclave/StrongBox

## License

MIT License — see [LICENSE](LICENSE) for details.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

---

*BitStack Labs — building tools for absolute communication privacy.*
