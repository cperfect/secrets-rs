# secrets-rs

A Rust library for safely retrieving and using secrets in applications, primarily for configuration.

The core guarantee: a secret's real value must be explicitly requested. Every default access path — `Display`, `Debug`, and serde serialization — emits a **masked value** that is safe to include in logs and error reports.

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

`FileSource` uses the secret `name` directly as a file path. Absolute paths are used as-is; relative paths resolve against the process's current working directory.

```
urn:secrets-rs:file:/etc/ssl/private/server.key
urn:secrets-rs:file:certs/ca.crt
urn:secrets-rs:file:../shared/client.crt
```

### Binding

Binding resolves a secret from its source and stores the typed value inside the `Secret<T>` struct. You can bind secrets individually with `Secret::bind`, or bind every secret in a struct at once with `bind_all`.

## Usage

### Add the dependency

```toml
[dependencies]
secrets-rs = "0.1"
```

### Individual binding

```rust
use secrets_rs::{EnvSource, Secret, SourceRegistry};

let mut api_key: Secret<String> =
    Secret::new("urn:secrets-rs:env:MY_API_KEY")?;

let mut registry = SourceRegistry::new();
registry.register("env", EnvSource);

api_key.bind(&registry)?;

// Safe to log — shows the masked value
println!("{api_key}");

// Explicit opt-in to the real value
let key: &str = api_key.value()?;
```

### Config struct with `#[derive(Bindable)]`

For structs that contain multiple secrets, derive `Bindable` to generate `bind_all` support automatically. Non-`Secret` fields are ignored. The derive macro is provided by the [`secrets-rs-macros`](https://crates.io/crates/secrets-rs-macros) crate, re-exported as `secrets_rs::Bindable`.

```rust
use secrets_rs::{EnvSource, Secret, SourceRegistry, bind_all};

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

let mut registry = SourceRegistry::new();
registry.register("env", EnvSource);

// Binds db_password and api_key; collects all errors rather than
// stopping at the first failure.
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

## Examples

Runnable examples are in the [`examples/`](https://github.com/cperfect/secrets-rs/tree/main/examples) directory:

| Example | Description |
|---------|-------------|
| [`basic.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/basic.rs) | Secret lifecycle: masked vs real value |
| [`config.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/config.rs) | `#[derive(Bindable)]` with a config struct |
| [`serde.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/serde.rs) | Deserialize URNs from JSON, then bind |
| [`file.rs`](https://github.com/cperfect/secrets-rs/blob/main/examples/file.rs) | Load TLS key and certificate with `FileSource` |

```sh
cargo run --example basic
cargo run --example config
cargo run --example serde
cargo run --example file   # requires: bash tests/fixtures/generate.sh
```

## Out of scope

- Writing secrets back to sources
- In-memory encryption
