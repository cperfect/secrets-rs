# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Test (all unit, integration, and doc tests)
cargo test

# Run a single test by name
cargo test <test_name>

# Run examples
cargo run --example basic
cargo run --example config

# Format (must produce no output to pass CI)
cargo fmt --all -- --check

# Lint (must produce no warnings to pass CI)
cargo clippy --all-targets -- -D warnings
```

Run `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` before every commit and fix any issues.

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/#summary).

## Workspace structure

This is a Cargo workspace with two crates:

- **`secrets-rs`** (root) — the public library
- **`secrets-rs-macros`** (`macros/`) — a proc-macro crate that provides `#[derive(Bindable)]`; re-exported from the root crate as `secrets_rs::Bindable`

The macro crate is a separate compilation unit because Rust requires proc-macros to live in their own crate. Changes to `macros/src/lib.rs` affect the derive macro only; changes to `src/` affect the library.

## Architecture

`secrets-rs` is a Rust library for safely surfacing secrets (strings, JSON, raw bytes) to applications — primarily for configuration. The central design constraint is that a secret's actual value must be explicitly requested; all default access paths (`Display`, `Debug`, serde serialization) return a **masked value** instead.

### Key concepts

- **`Secret<T>`** — wraps a typed value. Before binding it exposes only the masked form; after binding the raw value is accessible via `Secret::value()`. `T` must implement `SecretValue`, which provides type label, byte conversion, and size reporting. Built-in impls: `String`, `Vec<u8>`, `serde_json::Value`.
- **Masked value** — `<urn> [<type>:<size>]` when bound, `<urn> [UNBOUND]` when not. Safe for logs.
- **URN identity** — each secret is addressed as `urn:secrets-rs:<source_id>:<name>`. Scheme and NID are case-insensitive (RFC 8141); `source_id` and `name` case-sensitivity depends on the source.
- **`Source` trait** — anything that resolves a name to raw bytes. Registered in a `SourceRegistry` keyed by `source_id`. Built-in sources: `EnvSource` (reads `std::env::var`) and `FileSource` (reads `std::fs::read`; supports `new()` for CWD-relative resolution or `with_base(dir)` for stable base-anchored resolution).
- **Binding** — `Secret::bind(&mut self, registry)` resolves the secret from its source. Accessing the value before binding returns `UnboundError`.
- **`Bindable` + `bind_all`** — `Bindable` is a trait for structs with `Secret` fields; `bind_all` calls `bind_secrets`, collecting all errors instead of stopping at the first. `#[derive(Bindable)]` (from the macros crate) generates the impl by detecting fields whose type path ends in `Secret`.

### Serde integration

- **Serialize** — always emits the masked value string; safe to use in any serialization context.
- **Deserialize** — accepts a URN string and produces an unbound `Secret`. Non-URN strings are rejected. This means a config file can hold URNs and be deserialized directly into a typed struct; `bind_all` is then called separately to resolve values.

### Adding a new source

Implement `Source` (in `src/source.rs`) for the new backend, add it under `src/sources/`, register it via `SourceRegistry::register("your-id", YourSource)`, and re-export it from `src/lib.rs`.

### Adding a new `SecretValue` type

Implement `SecretValue` on the type in `src/secret.rs`: provide `type_name()`, `from_bytes()`, and `masked_size()`.

## Code style

- Doc-comments (`///`) are **required** for all public functions, types, and modules.
- Inline comments should explain *why*, not what — use them for complex algorithms, non-obvious choices, or library constraints.
- Follow the [Rust Style Guide](https://doc.rust-lang.org/style-guide/index.html) and [API Design Guidelines](https://rust-lang.github.io/api-guidelines/) for public APIs.
- Errors should follow the [NRC Error Design Guidelines](https://nrc.github.io/error-docs/error-design/index.html).

## Out of scope

Writing secrets back to sources and in-memory encryption are explicitly out of scope.
