# Contributing to Voryn

## Development Workflow

1. Create a feature branch from `main`
2. Make changes following the code style guides below
3. Write tests for new functionality
4. Ensure all CI checks pass
5. Open a pull request with a clear description

## Branch Naming

- `feature/<description>` — New features
- `fix/<description>` — Bug fixes
- `refactor/<description>` — Code improvements
- `docs/<description>` — Documentation changes

## Commit Messages

Use clear, concise commit messages that describe **what** and **why**:

```
Add Ed25519 identity generation via libsodium

Implements keypair generation in voryn-crypto using sodiumoxide.
Keys are zeroed on drop via the zeroize crate.
```

## Code Style

### Rust

- Format with `cargo fmt`
- Lint with `cargo clippy -- -D warnings`
- All `unsafe` blocks must have a `// SAFETY:` comment explaining why they are sound
- Use `thiserror` for error types, not string errors
- All crypto-sensitive data must use `zeroize` for secure cleanup

### TypeScript

- Format with Prettier
- Lint with ESLint (strict TypeScript rules)
- No `any` types — use proper typing
- Prefix unused parameters with `_`

## Testing

- **Rust:** `cargo test --workspace`
- **TypeScript:** `yarn test`
- New crypto code requires test vectors and property-based tests
- New network code requires integration tests with multiple nodes

## Security

- Never commit secrets, keys, or credentials
- All plaintext handling must use `SecureBytes`/`SecureString` (auto-zeroed)
- All network messages must be signed and encrypted
- Report security vulnerabilities privately to the maintainers
