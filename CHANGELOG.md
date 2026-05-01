# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] — 2026-05-01

### Added

- `FileSource` — built-in source backed by `std::fs::read`. Loads secrets from
  the local filesystem; absolute paths are used as-is, relative paths resolve
  against the process's current working directory. Primary use case is TLS keys
  and certificates.

## [0.1.0] — 2026-04-30

### Added

- `Secret<T>` — a generic wrapper that keeps secret values masked by default across `Display`, `Debug`, and serde serialization.
- `SecretValue` trait with built-in impls for `String`, `Vec<u8>`, and `serde_json::Value`.
- URN-based identity scheme: `urn:secrets-rs:<source_id>:<name>` (scheme and NID case-insensitive per RFC 8141).
- `Source` trait and `SourceRegistry` for pluggable secret backends.
- `EnvSource` — built-in source backed by `std::env::var`.
- `Bindable` trait and `bind_all` helper — bind every `Secret` field in a struct in one call, collecting all errors.
- `#[derive(Bindable)]` proc-macro (crate `secrets-rs-macros`) — auto-implements `Bindable` for structs with `Secret` fields.
- Serde `Deserialize` support — a URN string deserializes into an unbound `Secret`; non-URN strings are rejected.
