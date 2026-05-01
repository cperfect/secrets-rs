//! Demonstrates loading TLS credentials from the filesystem using `FileSource`.
//! Absolute and relative paths are both supported; relative paths resolve
//! against the process's current working directory.
//!
//! Run with: `cargo run --example file`
//!
//! Requires test fixtures — generate them first if needed:
//!   bash tests/fixtures/generate.sh

use secrets_rs::{FileSource, Secret, SourceRegistry, bind_all};

#[derive(secrets_rs::Bindable)]
struct TlsConfig {
    /// PEM-encoded private key.
    private_key: Secret<String>,
    /// DER-encoded certificate (binary).
    certificate: Secret<Vec<u8>>,
}

fn main() {
    let fixtures = std::path::Path::new("tests/fixtures");
    if !fixtures.join("test.key").exists() {
        eprintln!("Test fixtures not found. Run: bash tests/fixtures/generate.sh");
        std::process::exit(1);
    }

    let mut config = TlsConfig {
        private_key: Secret::new("urn:secrets-rs:file:tests/fixtures/test.key").unwrap(),
        certificate: Secret::new("urn:secrets-rs:file:tests/fixtures/test.der").unwrap(),
    };

    // Safe to log — shows masked values before binding
    println!("Before bind:");
    println!("  private_key : {}", config.private_key);
    println!("  certificate : {}", config.certificate);

    let mut registry = SourceRegistry::new();
    registry.register("file", FileSource);

    bind_all(&mut config, &registry).expect("failed to load TLS credentials");

    // Still masked after binding — real values never leak via Display
    println!("\nAfter bind:");
    println!("  private_key : {}", config.private_key);
    println!("  certificate : {}", config.certificate);

    // Explicit opt-in to the real values
    let key_pem: &str = config.private_key.value().unwrap();
    let cert_der: &[u8] = config.certificate.value().unwrap();

    println!("\nKey starts with : {}", key_pem.lines().next().unwrap());
    println!("Cert DER tag    : 0x{:02X} (ASN.1 SEQUENCE)", cert_der[0]);
}
