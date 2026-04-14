# Voryn Threat Model

## Assets

1. **Message content** — plaintext of all communications
2. **Identity keys** — Ed25519 private keys (device identity)
3. **Contact graph** — who communicates with whom
4. **Group membership** — who belongs to which groups
5. **Metadata** — timestamps, message sizes, frequency patterns

## Threat Actors

### Passive Network Observer
- **Capability:** Can monitor all network traffic between devices
- **Goal:** Determine message content, contact graph, or communication patterns
- **Mitigations:**
  - End-to-end encryption (XSalsa20-Poly1305 / Double Ratchet)
  - Traffic padding to fixed-size buckets
  - Random send delays
  - Chaff/dummy traffic generation
  - All protocol messages (including ACKs) are encrypted

### Active Network Attacker (MITM)
- **Capability:** Can intercept, modify, or inject network traffic
- **Goal:** Forge messages, impersonate users, or disrupt communication
- **Mitigations:**
  - Ed25519 signatures on all messages
  - Safety number verification for contact authentication
  - libp2p Noise protocol for transport-layer authentication
  - Hash-chained group ledger detects tampered events

### Device Thief (Physical Access)
- **Capability:** Has physical possession of an unlocked or locked device
- **Goal:** Extract messages, keys, or contact information
- **Mitigations:**
  - SQLCipher encryption at rest
  - Hardware-bound keys (Secure Enclave / StrongBox)
  - App passcode with Argon2id key derivation
  - Failed attempt limit with full data wipe
  - Time lock requiring re-authentication
  - Memory zeroing of plaintext after display
  - Screenshot/screen recording prevention

### Coercive Attacker
- **Capability:** Forces the user to unlock the device
- **Goal:** Access real messages and contacts
- **Mitigations:**
  - Duress passcode shows empty/decoy state
  - Duress mode visually indistinguishable from legitimate empty state
  - Optional silent remote wipe trigger on duress entry

### Forensic Analyst
- **Capability:** Advanced tools to examine device storage and memory
- **Goal:** Recover deleted messages or key material
- **Mitigations:**
  - SQLCipher PRAGMA secure_delete = ON (overwrite before delete)
  - 3-pass random overwrite on data wipe
  - `zeroize` crate for all sensitive memory
  - Core dump disabled
  - iOS backup exclusion
  - **Known limitation:** Flash storage wear leveling may retain copies
    of data in cells not currently mapped. This is a fundamental hardware
    limitation that cannot be fully mitigated in software.

### Compromised Contact
- **Capability:** A verified contact whose device or key is compromised
- **Goal:** Impersonate the contact or access group keys
- **Mitigations:**
  - Forward secrecy (Double Ratchet) limits exposure of past messages
  - Identity revocation broadcast to exclude compromised keys
  - Automatic group key resharing on member removal
  - Invite token single-use prevents mass identity creation

## Non-Goals (Explicitly Out of Scope)

1. **Protection against rooted/jailbroken devices** — If the OS is compromised,
   all bets are off. We detect and warn but do not block.
2. **Protection against screen photography** — We prevent screenshots and
   screen recording via OS APIs, but cannot prevent a camera pointed at the screen.
3. **Anonymity at the network level** — Voryn does not hide IP addresses.
   Users who need IP-level anonymity should use a VPN or Tor.
4. **Protection against compromised hardware** — If the Secure Enclave or
   StrongBox is compromised at the hardware level, key extraction is possible.

## Security Review Schedule

- **Pre-release:** Full external security audit (cryptographic + code + protocol)
- **Quarterly:** Internal security review of all changes
- **On every membership change:** Automatic group key resharing
- **On identity compromise report:** Immediate revocation broadcast
