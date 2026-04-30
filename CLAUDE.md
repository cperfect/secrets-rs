# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Test
cargo test

# Run a single test
cargo test <test_name>

# Format (must produce no output to pass CI)
cargo fmt --all -- --check

# Lint (must produce no warnings to pass CI)
cargo clippy --all-targets -- -D warnings
```

Run `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` before every commit and fix any issues.

## Architecture

`secrets-rs` is a Rust library for safely surfacing secrets (strings, JSON, raw bytes) to applications — primarily for configuration. The central design constraint is that a secret's actual value must be explicitly requested; all default access paths (Display, serde serialization, Debug) return a **masked value** instead.

### Key concepts

- **Secret** — a generic struct `Secret<T>` that wraps a typed value. Before binding it exposes only the masked form; after binding the raw value is accessible via an explicit method.
- **Masked value** — the URN of the secret plus its type and length/size. Unbound secrets include `"UNBOUND"` in place of the length. Safe for logs.
- **URN identity** — each secret is addressed as `urn:secret-rs:<source_id>:<name>`. Scheme and NID are case-insensitive (RFC 8141); case-sensitivity of source_id and name depends on the source.
- **Source** — where secrets are fetched from. The first supported source is environment variables; future sources may include AWS Secrets Manager, Azure Key Vault, etc. Each source has a unique `source_id`.
- **Binding** — the act of resolving a secret from its source and storing the value inside the `Secret<T>` struct. Accessing the value before binding is an error.
- **`#[secret("urn:...")]` attribute macro** — decorates fields of config structs to replace serde deserialization with binding and serialization with the masked value. Serializing/deserializing directly to/from a `Secret` field is an error.

Helper functions will recursively discover all `Secret` fields in a struct, bind them in one call, and collect errors.

### Out of scope
Writing secrets back to sources and in-memory encryption are explicitly out of scope.

## Code style

- Doc-comments (`///`) are **required** for all public functions, types, and modules.
- Inline comments should explain *why*, not what — use them for complex algorithms, non-obvious choices, or library constraints.
- Follow the [Rust Style Guide](https://doc.rust-lang.org/style-guide/index.html) and [API Design Guidelines](https://rust-lang.github.io/api-guidelines/) for public APIs.
- Errors should follow the [NRC Error Design Guidelines](https://nrc.github.io/error-docs/error-design/index.html).
