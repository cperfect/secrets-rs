//! Demonstrates loading TLS credentials from the filesystem using `FileSource`.
//!
//! Two construction modes are shown:
//!
//! - `FileSource::with_base(dir)` — captures a base directory at construction
//!   time; relative URN names are resolved against it, which is stable in
//!   multi-threaded programs regardless of later `set_current_dir` calls.
//!   Recommended for production code.
//!
//! - `FileSource::new()` — resolves relative paths against the process's CWD
//!   at call time. Fine for single-threaded or CLI contexts where CWD is fixed.
//!
//! Absolute URN paths work the same in both modes.
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
    registry.register("file", FileSource::with_base(&fixtures));

    bind_all(&mut config, &registry).expect("failed to load TLS credentials");

    println!("\nAfter bind (with_base):");
    println!("  private_key : {}", config.private_key);
    println!("  certificate : {}", config.certificate);

    let key_pem: &str = config.private_key.value().unwrap();
    let cert_der: &[u8] = config.certificate.value().unwrap();
    println!("\nKey starts with : {}", key_pem.lines().next().unwrap());
    println!("Cert DER tag    : 0x{:02X} (ASN.1 SEQUENCE)", cert_der[0]);

    // ---- new(): absolute paths work the same regardless of mode ---------------
    //
    // When the URN name is an absolute path, both modes behave identically.

    let abs_urn = format!(
        "urn:secrets-rs:file:{}",
        fixtures.join("test.crt").display()
    );
    let mut cert_pem: Secret<String> = Secret::new(&abs_urn).unwrap();

    let mut registry2 = SourceRegistry::new();
    registry2.register("file", FileSource::new());
    cert_pem.bind(&registry2).unwrap();

    println!(
        "\nCert PEM (new + absolute path): {}",
        cert_pem.value().unwrap().lines().next().unwrap()
    );
}
