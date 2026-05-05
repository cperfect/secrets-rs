//! Demonstrates loading TLS credentials from the filesystem using `FileSource`.
//!
//! Three patterns are shown:
//!
//! 1. **Single `with_base` source** — one `FileSource` registered under one
//!    `source_id`; all relative URN names resolve against its base directory.
//!
//! 2. **Multiple sources, different base directories** — two `FileSource`
//!    instances registered under separate IDs (`"file-keys"` and `"file-certs"`), each
//!    anchored to its own directory. URNs select which source to use via the
//!    `source_id` component, keeping key material and certificates in separate
//!    locations with independent permissions.
//!
//! 3. **`FileSource::new()` with an absolute path** — shows that absolute URN
//!    names bypass the base directory and work the same in both modes.
//!
//! Run with: `cargo run --example file`
//!
//! Requires test fixtures — run the integration tests first to generate them:
//!   cargo test --test file_source

use std::path::PathBuf;

use secrets_rs::{FileSource, Secret, SourceRegistry, bind_all};

#[derive(secrets_rs::Bindable)]
struct TlsConfig {
    /// PEM-encoded private key.
    private_key: Secret<String>,
    /// DER-encoded certificate (binary).
    certificate: Secret<Vec<u8>>,
}

fn main() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    if !fixtures.join("test.key").exists() {
        eprintln!("Test fixtures not found. Run: cargo test --test file_source");
        std::process::exit(1);
    }

    // ---- with_base: recommended for production --------------------------------
    //
    // URN names are just filenames; FileSource joins them against `fixtures`.
    // Resolution is stable even if another thread calls set_current_dir.

    let mut config = TlsConfig {
        private_key: Secret::new("urn:secrets-rs:file:test.key").unwrap(),
        certificate: Secret::new("urn:secrets-rs:file:test.der").unwrap(),
    };

    println!("Before bind:");
    println!("  private_key : {}", config.private_key);
    println!("  certificate : {}", config.certificate);

    let mut registry = SourceRegistry::new();
    registry
        .register("file", FileSource::with_base(&fixtures))
        .unwrap();

    bind_all(&mut config, &registry).expect("failed to load TLS credentials");

    println!("\nAfter bind (with_base):");
    println!("  private_key : {}", config.private_key);
    println!("  certificate : {}", config.certificate);

    let key_pem: &str = config.private_key.value().unwrap();
    let cert_der: &[u8] = config.certificate.value().unwrap();
    println!("\nKey starts with : {}", key_pem.lines().next().unwrap());
    println!("Cert DER tag    : 0x{:02X} (ASN.1 SEQUENCE)", cert_der[0]);

    // ---- multiple sources: separate base directories per source_id ------------
    //
    // Convention: prefix the source_id with the source type ("file-<qualifier>").
    // This makes the backend readable directly from the URN:
    //   urn:secrets-rs:file-keys:server.key  → FileSource anchored to the keys dir
    //   urn:secrets-rs:file-certs:server.der → FileSource anchored to the certs dir
    //
    // Key material and certificates live in separate directories with independent
    // filesystem permissions; the URN source_id component selects which to use.

    let keys_dir = tempfile::tempdir().unwrap();
    let certs_dir = tempfile::tempdir().unwrap();

    std::fs::copy(
        fixtures.join("test.key"),
        keys_dir.path().join("server.key"),
    )
    .unwrap();
    std::fs::copy(
        fixtures.join("test.der"),
        certs_dir.path().join("server.der"),
    )
    .unwrap();

    let mut multi_registry = SourceRegistry::new();
    multi_registry
        .register("file-keys", FileSource::with_base(keys_dir.path()))
        .unwrap();
    multi_registry
        .register("file-certs", FileSource::with_base(certs_dir.path()))
        .unwrap();

    let mut server_key: Secret<String> =
        Secret::new("urn:secrets-rs:file-keys:server.key").unwrap();
    let mut server_cert: Secret<Vec<u8>> =
        Secret::new("urn:secrets-rs:file-certs:server.der").unwrap();

    server_key.bind(&multi_registry).unwrap();
    server_cert.bind(&multi_registry).unwrap();

    println!("\nTwo sources, separate base directories:");
    println!("  server_key  : {}", server_key);
    println!("  server_cert : {}", server_cert);
    println!(
        "  Key starts  : {}",
        server_key.value().unwrap().lines().next().unwrap()
    );
    println!(
        "  Cert tag    : 0x{:02X} (ASN.1 SEQUENCE)",
        server_cert.value().unwrap()[0]
    );

    // ---- new(): absolute paths work the same regardless of mode ---------------
    //
    // When the URN name is an absolute path, both modes behave identically.

    let abs_urn = format!(
        "urn:secrets-rs:file:{}",
        fixtures.join("test.crt").display()
    );
    let mut cert_pem: Secret<String> = Secret::new(&abs_urn).unwrap();

    let mut registry2 = SourceRegistry::new();
    registry2.register("file", FileSource::new()).unwrap();
    cert_pem.bind(&registry2).unwrap();

    println!(
        "\nCert PEM (new + absolute path): {}",
        cert_pem.value().unwrap().lines().next().unwrap()
    );
}
