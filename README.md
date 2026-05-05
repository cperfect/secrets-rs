# secrets-rs

A Rust library for safely retrieving and using secrets in applications, primarily for configuration.

A secret's real value must be explicitly requested. Every default access path — `Display`, `Debug`, and serde serialization — emits a **masked value** that is safe to include in logs and error reports.

## Concepts

### Secret

`Secret<T>` is a generic wrapper around a typed value. The supported types are:

| Type | `T` |
|------|-----|
| UTF-8 string | `String` |
| Raw bytes | `Vec<u8>` |
| JSON | `serde_json::Value` |

A secret is identified by a URN of the form:

```
urn:secrets-rs:<source_id>:<name>
```

The scheme (`urn`) and NID (`secrets-rs`) are case-insensitive per RFC 8141. The case sensitivity of `source_id` and `name` depends on the source.

### Masked value

Until a secret is bound, or whenever it is displayed by default, it shows a masked value:

```
urn:secrets-rs:env:MY_API_KEY [UNBOUND]        # before binding
urn:secrets-rs:env:MY_API_KEY [string:22]      # after binding
```

The format is `<urn> [<type>:<size>]`. Calling `.value()` before binding returns an error.

### Sources

A source is anything that can look up a secret by name and return its raw bytes. Sources are registered in a `SourceRegistry` keyed by the `source_id` from the URN.

**Built-in sources:**

| Source | `source_id` convention | Backed by | Primary use case |
|--------|----------------------|-----------|-----------------|
| `EnvSource` | e.g. `"env"` | `std::env::var` | API keys, passwords |
| `FileSource` | e.g. `"file"` | `std::fs::read` | TLS keys and certificates |

`FileSource` uses the secret `name` directly as a file path. Two construction modes are available:

- **`FileSource::new()`** — resolves relative paths against the process's current working directory at call time. Simple, but non-deterministic if any code calls `set_current_dir` concurrently.
- **`FileSource::with_base(dir)`** — captures an absolute base directory at construction time and resolves all relative paths against it, regardless of later CWD changes. Absolute paths in the URN name are still used as-is.

```
urn:secrets-rs:file:/etc/ssl/private/server.key   // absolute — same in both modes
urn:secrets-rs:file:certs/ca.crt                  // relative — stable only with with_base
```

```rust
// Stable resolution — recommended for multi-threaded programs
registry.register("file", FileSource::with_base("/etc/ssl/private")).unwrap();
```

> **Security:** Because the URN name is used as a filesystem path without validation, binding a `FileSource` secret with an attacker-controlled URN is an **arbitrary file-read** vulnerability. Only bind URNs that come from **trusted configuration** (static code, operator-supplied config files with restricted write permissions, etc.). Never accept `urn:secrets-rs:file:...` URNs from untrusted input such as API requests, user-supplied data, or deserialized network payloads. `with_base` anchors relative resolution to a known directory but does **not** prevent path-traversal sequences (`../`) from escaping it; the trusted-configuration requirement still applies.

### SourceRegistry

`SourceRegistry` maps the `source_id` component of a URN to the `Source` implementation that resolves it. When `bind` is called on a secret, the registry looks up the source by the `source_id` extracted from the secret's URN and delegates to its `get` method.

```rust
// EnvSource is already registered. Add FileSource for filesystem secrets.
let mut registry = SourceRegistry::new();
registry.register("file", FileSource::with_base("/run/secrets")).unwrap();
// urn:secrets-rs:env:...  → resolved by EnvSource (default)
// urn:secrets-rs:file:... → resolved by FileSource
```

A few design points worth knowing:

**`EnvSource` is pre-registered under `"env"`.** `SourceRegistry::new()` registers `EnvSource` automatically, so env-backed secrets work without any manual setup. Calling `register("env", ...)` again replaces it, which is useful in tests or when you need a custom env implementation.

**The `source_id` is application-defined for everything else, with a recommended naming convention.** Prefix the id with the source type, separated by `-`:

| Pattern | Example | When to use |
|---------|---------|-------------|
| `"file"` | `"file"` | Single `FileSource` instance |
| `"file-<qualifier>"` | `"file-certs"`, `"file-keys"` | Multiple `FileSource` instances with different base directories |
| `"env-<qualifier>"` | `"env-prod"`, `"env-staging"` | Multiple env sources pointing to different environments |

This makes the backend immediately readable from the URN itself — `urn:secrets-rs:file-certs:server.der` is unambiguously file-backed, without inspecting the registry. Any string that passes character validation is accepted; the convention is not enforced by the library.

**Sources are type-erased.** `SourceRegistry` stores `Box<dyn Source>`, so you can mix arbitrary `Source` implementations in the same registry without the registry itself being generic. The `Source` trait requires `Send + Sync`, which means a registry in an `Arc` is safe to share across threads.

**Secrets and the registry are decoupled.** A `Secret` is just a URN until `bind` is called — it holds no reference to any source. This means secrets can be constructed or deserialized freely before sources are configured, and you can supply a different registry in tests without changing how secrets are declared.

### Binding

Binding resolves a secret from its source and stores the typed value inside the `Secret<T>` struct. You can bind secrets individually with `Secret::bind`, or bind every secret in a struct at once with `bind_all`.

## Usage

### Add the dependency

```toml
[dependencies]
secrets-rs = "1.0"
```

### Individual binding

```rust
use secrets_rs::{Secret, SourceRegistry};

let mut api_key: Secret<String> =
    Secret::new("urn:secrets-rs:env:MY_API_KEY")?;

// EnvSource is registered under "env" by default.
let registry = SourceRegistry::new();
api_key.bind(&registry)?;

// Safe to log — shows the masked value
println!("{api_key}");

// Explicit opt-in to the real value
let key: &str = api_key.value()?;
```

### Config struct with `#[derive(Bindable)]`

For structs that contain multiple secrets, derive `Bindable` to generate `bind_all` support automatically. Non-`Secret` fields are ignored. The derive macro is provided by the [`secrets-rs-macros`](https://crates.io/crates/secrets-rs-macros) crate, re-exported as `secrets_rs::Bindable`.

```rust
use secrets_rs::{Secret, SourceRegistry, bind_all};

#[derive(secrets_rs::Bindable)]
struct AppConfig {
    db_password:     Secret<String>,
    api_key:         Secret<String>,
    max_connections: u32,            // ignored — not a Secret
}

let mut config = AppConfig {
    db_password:     Secret::new("urn:secrets-rs:env:DB_PASSWORD")?,
    api_key:         Secret::new("urn:secrets-rs:env:API_KEY")?,
    max_connections: 10,
};

// EnvSource is registered under "env" by default.
// Binds db_password and api_key; collects all errors rather than
// stopping at the first failure.
let registry = SourceRegistry::new();
bind_all(&mut config, &registry)?;
```

Without the derive macro, implement `Bindable` manually:

```rust
use secrets_rs::{Bindable, BindError, SourceRegistry};

impl Bindable for AppConfig {
    fn bind_secrets(&mut self, registry: &SourceRegistry) -> Result<(), Vec<BindError>> {
        let mut errors = Vec::new();
        if let Err(e) = self.db_password.bind(registry) { errors.push(e); }
        if let Err(e) = self.api_key.bind(registry)     { errors.push(e); }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

### Serde integration

`Secret<T>` implements both `Serialize` and `Deserialize`:

- **Serialize** — always produces the masked value string, safe to use in any context.
- **Deserialize** — accepts a `urn:secrets-rs:<source_id>:<name>` string and produces an unbound secret. Non-URN strings are rejected with an error.

This means a config file can hold URN strings and be deserialized directly into a typed struct; `bind_all` is then called to resolve the actual values from their sources.

```rust
#[derive(serde::Deserialize, secrets_rs::Bindable)]
struct AppConfig {
    db_password:     Secret<String>,  // deserializes from "urn:secrets-rs:env:DB_PASSWORD"
    max_connections: u32,
}

// Deserialize URNs from config file, then bind to real values
let mut config: AppConfig = serde_json::from_str(&config_json)?;
bind_all(&mut config, &registry)?;

// Serializes as "urn:secrets-rs:env:DB_PASSWORD [string:28]"
println!("{}", serde_json::to_string(&config.db_password)?);
```

### Sharing secrets across subsystems

`Secret<T>` deliberately does not implement `Clone`. Cloning a bound secret would create a second full copy of the secret value in memory, multiplying the number of locations an attacker could read it from in a core dump, swap, or cold-boot scenario. The library's position is that there should be one copy of each secret value in memory at a time, shared by reference.

When multiple parts of an application need access to the same secret, prefer keeping one bound copy and distributing a reference to it rather than creating multiple independent copies. Three patterns apply, in order of preference:

**1. `Arc<AppConfig>` — bind once, share the whole config (recommended)**

The simplest approach: build and bind your config struct at startup, wrap it in an `Arc`, and hand clones of the `Arc` to each subsystem. No secret values are duplicated; all subsystems read from the same allocation.

```rust
use std::sync::Arc;
use secrets_rs::{Secret, SourceRegistry, bind_all};

#[derive(secrets_rs::Bindable)]
struct AppConfig {
    api_key:     Secret<String>,
    db_password: Secret<String>,
}

let mut config = AppConfig { /* ... */ };
bind_all(&mut config, &SourceRegistry::new())?;

let config = Arc::new(config);          // bind_all consumed &mut, now freeze
let config_for_worker = Arc::clone(&config);   // zero-copy share
```

**2. `Arc<Secret<T>>` — share a single bound secret**

When only one secret needs to be shared (rather than an entire config struct), wrap just that secret in an `Arc` after binding.

```rust
use std::sync::Arc;
use secrets_rs::{Secret, SourceRegistry};

let mut api_key: Secret<String> = Secret::new("urn:secrets-rs:env:API_KEY")?;
api_key.bind(&SourceRegistry::new())?;

let api_key = Arc::new(api_key);
let key_for_worker = Arc::clone(&api_key);
```

**3. `Secret::urn()` — construct an independent unbound secret**

When two subsystems must bind independently — for example because they use different registries or have different initialization lifetimes — use [`Secret::urn`] to obtain the URN from an existing secret and pass it to `Secret::new` to create a second unbound instance. Each subsystem then binds its own copy.

```rust
let original: Secret<String> = Secret::new("urn:secrets-rs:env:API_KEY")?;

// Subsystem B gets its own unbound secret with the same URN.
let for_subsystem_b: Secret<String> = Secret::new(&original.urn().to_string())?;
```

Note that this creates two independent bindings, so each subsystem fetches the value from the source separately. Prefer patterns 1 or 2 when a single fetch is sufficient.

## Examples

Runnable examples are in the [`examples/`](https://github.com/cperfect/secrets-rs/tree/main/examples) directory:

| Example | Description |
|---------|-------------|
| [`basic.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/basic.rs) | Secret lifecycle: masked vs real value |
| [`config.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/config.rs) | `#[derive(Bindable)]` with a config struct |
| [`serde.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/serde.rs) | Deserialize URNs from JSON, then bind |
| [`file.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/file.rs) | Load TLS key and certificate with `FileSource` |
| [`sharing.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/sharing.rs) | Share secrets across subsystems: `Arc<AppConfig>`, `Arc<Secret<T>>`, `Secret::urn()` |

```sh
cargo run --example basic
cargo run --example config
cargo run --example serde
cargo run --example file      # requires: cargo test --test file_source
cargo run --example sharing
```

## Out of scope

- Writing secrets back to sources
- In-memory encryption
