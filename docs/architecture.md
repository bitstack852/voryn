# Voryn Architecture

## Overview

Voryn is a decentralized encrypted messaging app where every device runs a full P2P node. There are no central servers — the network consists entirely of user devices communicating directly.

## Layer Architecture

### 1. React Native UI Layer (TypeScript)
- Thin UI shell — all business logic lives in Rust
- Communicates with Rust via UniFFI-generated Turbo Modules
- Screens: Onboarding, Contacts, Chat, Settings, Admin

### 2. Rust Core (voryn-core)
- Entry point for all UniFFI bindings
- Orchestrates crypto, network, storage, and protocol layers
- Manages app lifecycle (start/stop node, auth state)

### 3. Crypto Layer (voryn-crypto)
- **Identity:** Ed25519 keypair generation and management
- **Key Exchange:** X25519 Diffie-Hellman via Ed25519→X25519 conversion
- **Symmetric Encryption:** XSalsa20-Poly1305 (authenticated encryption)
- **Key Derivation:** HKDF (from shared secrets), Argon2id (from passcodes)
- **Secure Memory:** Auto-zeroing types for all sensitive data
- Built on libsodium via `sodiumoxide` Rust bindings

### 4. Network Layer (voryn-network)
- **Transport:** QUIC (primary, good NAT traversal) + TCP/Noise (fallback)
- **Discovery:** Kademlia DHT + mDNS for local networks
- **Protocols:** Custom request-response protocols for messaging, ACKs, wipe, sync
- **Obfuscation:** Message padding, timing jitter, chaff traffic
- Built on rust-libp2p

### 5. Storage Layer (voryn-storage)
- SQLCipher-encrypted database (all data encrypted at rest)
- Schema: identities, contacts, messages, key_material, groups, invites
- Migration framework for schema evolution
- Secure delete (overwrite before unlink)

### 6. Protocol Layer (voryn-protocol)
- **Double Ratchet:** Forward-secret 1-to-1 messaging (new key per message)
- **X3DH:** Initial key agreement with pre-key bundles
- **Shamir's Secret Sharing:** Threshold-based group key distribution
- **Group Ledger:** Append-only, hash-chained event log for group management
- **Invite Tokens:** Cryptographically signed, single-use onboarding tokens

### 7. Hardware Security
- **iOS:** Secure Enclave Processor (private keys never leave hardware)
- **Android:** StrongBox Keystore (hardware-backed, fallback to TEE)
- Abstracted via `HardwareKeyStore` trait in Rust

## Message Flow

```
Compose → Encrypt (Double Ratchet) → Sign (Ed25519) → Pad → Send (libp2p)
                                                              ↓
Receive → Verify (Ed25519) → Decrypt (Double Ratchet) → Display → Zero Memory
                                                              ↓
                                                    Re-encrypt → SQLCipher
```

## Key Architectural Decisions

1. **Rust-first core:** All security-critical code in Rust for memory safety
2. **rust-libp2p over js-libp2p:** Better mobile support, no Node.js polyfills
3. **UniFFI bridge:** Mozilla's tool generates React Native Turbo Modules from Rust
4. **Hardware key abstraction:** Single trait with platform-specific implementations
5. **Monorepo with Yarn workspaces:** Single repo for all code (TS + Rust)
