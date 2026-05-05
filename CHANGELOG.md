# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] — 2026-05-05

### Added

- `FileSource::with_base(dir)` constructor — captures an absolute base directory at construction time and resolves all relative URN names against it, regardless of later `set_current_dir` calls. Recommended for multi-threaded programs.
- `Secret::urn(&self) -> &Urn` accessor — returns the identifying URN whether or not the secret is bound. The URN never contains the secret value, so it is safe to log, store, or compare.
- `examples/sharing.rs` — runnable example demonstrating all three patterns for sharing secrets across subsystems: `Arc<AppConfig>`, `Arc<Secret<T>>`, and `Secret::urn()` for independent binding.
- `SourceRegisterError` is now part of the public API, re-exported from the crate root.
- `EnvSource` is pre-registered under `"env"` by `SourceRegistry::new()` — env-backed secrets work without manual setup.

### Changed

- **Breaking:** `SourceRegistry::register()` now returns `Result<(), SourceRegisterError>` instead of `()`. The source id is validated against the RFC 8141 NSS pchar character set (ASCII alphanumerics and `-._~!$&'()*+,;=@/`); invalid ids are rejected with `SourceRegisterError::InvalidSourceId`.
- `FileSource` is now a proper struct supporting both `new()` (CWD-relative) and `with_base(dir)` (stable base). Previously it was a unit struct with CWD-only resolution.
- `FileSource` error messages now report the original URN name rather than the resolved filesystem path, preventing base directory disclosure in errors.
- URN parsing now validates the `source_id` component against the RFC 8141 NSS pchar character set, returning `UrnParseError::InvalidSourceId` for invalid ids.

### Documentation

- `SourceRegistry` design rationale: type-erased storage, `Send + Sync` requirement, decoupling of secrets from the registry, and source-id naming convention (`file-<qualifier>`, `env-<qualifier>`).
- `FileSource` security warning: using the URN name as a filesystem path with attacker-controlled input is an arbitrary file-read vulnerability; only bind URNs from trusted configuration.
- Documented why `Secret<T>` does not implement `Clone` — cloning would duplicate the secret value in memory, multiplying exposure in core dumps, swap, and cold-boot scenarios.
- Added sharing patterns guide covering `Arc<AppConfig>`, `Arc<Secret<T>>`, and `Secret::urn()` with code examples.

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
