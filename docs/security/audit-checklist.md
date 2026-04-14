# Security Audit Checklist

## Cryptographic Audit

- [ ] Ed25519 key generation uses proper entropy source (libsodium CSPRNG)
- [ ] Ed25519 → X25519 conversion follows RFC 7748
- [ ] XSalsa20-Poly1305 nonces are never reused
- [ ] Double Ratchet implementation matches Signal specification
- [ ] X3DH key agreement follows Signal specification
- [ ] Shamir's Secret Sharing uses correct GF(256) arithmetic
- [ ] All secret key material uses zeroize-on-drop
- [ ] Argon2id parameters provide adequate brute-force resistance
- [ ] HKDF derivation uses appropriate salt and context
- [ ] No weak or deprecated cryptographic primitives

## Code Audit

- [ ] All `unsafe` blocks in Rust are sound and documented
- [ ] FFI boundaries properly handle null pointers and errors
- [ ] No integer overflow in cryptographic operations
- [ ] No buffer over-reads/writes in message parsing
- [ ] bincode deserialization has size limits (DoS prevention)
- [ ] SQLCipher queries use parameterized statements (no SQL injection)
- [ ] No sensitive data in log output
- [ ] Error messages do not leak implementation details

## Protocol Audit

- [ ] Message replay prevention (nonces, sequence numbers)
- [ ] Group ledger hash chain is tamper-evident
- [ ] Remote wipe command has proper authentication and replay prevention
- [ ] Invite tokens are truly single-use
- [ ] Revocation notices cannot be forged
- [ ] Auto-delete timers start on delivery, not send
- [ ] Peer discovery does not leak unintended metadata

## Platform Audit

- [ ] Secure Enclave key generation uses proper attributes
- [ ] StrongBox key is non-exportable
- [ ] App passcode wrapping is correctly layered
- [ ] Failed attempt counter persists across app restarts
- [ ] Duress mode is indistinguishable from legitimate use
- [ ] Screenshot prevention active on all sensitive screens
- [ ] App switcher shows blank/safe content
- [ ] Clipboard auto-cleared after timeout
