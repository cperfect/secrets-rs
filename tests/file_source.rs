use std::sync::Once;

use secrets_rs::{FileSource, Secret, SourceRegistry, bind_all};

static FIXTURES: Once = Once::new();

/// Runs `tests/fixtures/generate.sh` once per test run if the fixtures are absent.
fn ensure_fixtures() {
    FIXTURES.call_once(|| {
        let key = std::path::Path::new("tests/fixtures/test.key");
        if !key.exists() {
            let status = std::process::Command::new("bash")
                .arg("tests/fixtures/generate.sh")
                .status()
                .expect("failed to run tests/fixtures/generate.sh");
            assert!(status.success(), "generate.sh exited with failure");
        }
    });
}

fn registry() -> SourceRegistry {
    let mut r = SourceRegistry::new();
    r.register("file", FileSource);
    r
}

// --- absolute path ---

#[test]
fn binds_absolute_path_as_bytes() {
    ensure_fixtures();
    let abs = std::fs::canonicalize("tests/fixtures/test.der")
        .expect("test.der not found — run tests/fixtures/generate.sh");
    let urn = format!("urn:secrets-rs:file:{}", abs.display());

    let mut secret: Secret<Vec<u8>> = Secret::new(&urn).unwrap();
    secret.bind(&registry()).unwrap();

    // DER files start with the ASN.1 SEQUENCE tag (0x30)
    assert_eq!(
        secret.value().unwrap()[0],
        0x30,
        "expected DER SEQUENCE tag"
    );
}

// --- relative path ---

#[test]
fn binds_relative_path_as_string() {
    ensure_fixtures();
    let mut secret: Secret<String> =
        Secret::new("urn:secrets-rs:file:tests/fixtures/test.crt").unwrap();
    secret.bind(&registry()).unwrap();

    assert!(
        secret
            .value()
            .unwrap()
            .contains("-----BEGIN CERTIFICATE-----")
    );
}

// --- relative path with parent component ---

#[test]
fn binds_relative_path_with_parent_component() {
    ensure_fixtures();
    // tests/fixtures/../fixtures/test.key resolves to tests/fixtures/test.key
    let mut secret: Secret<String> =
        Secret::new("urn:secrets-rs:file:tests/fixtures/../fixtures/test.key").unwrap();
    secret.bind(&registry()).unwrap();

    assert!(secret.value().unwrap().contains("-----BEGIN"));
}

// --- missing file ---

#[test]
fn missing_file_returns_name_not_found() {
    let mut secret: Secret<Vec<u8>> =
        Secret::new("urn:secrets-rs:file:/tmp/secrets-rs-no-such-file-xyz").unwrap();

    let err = secret.bind(&registry()).unwrap_err();
    assert!(matches!(err, secrets_rs::BindError::NameNotFound { .. }));
}

// --- derive(Bindable) with mixed file and non-secret fields ---

#[derive(secrets_rs::Bindable)]
struct TlsConfig {
    certificate: Secret<Vec<u8>>,
    private_key: Secret<String>,
    max_connections: u32,
}

#[test]
fn bindable_derive_binds_file_secrets() {
    ensure_fixtures();
    let abs_crt = std::fs::canonicalize("tests/fixtures/test.der").unwrap();

    let mut config = TlsConfig {
        certificate: Secret::new(&format!("urn:secrets-rs:file:{}", abs_crt.display())).unwrap(),
        private_key: Secret::new("urn:secrets-rs:file:tests/fixtures/test.key").unwrap(),
        max_connections: 4,
    };

    bind_all(&mut config, &registry()).unwrap();

    assert_eq!(config.certificate.value().unwrap()[0], 0x30);
    assert!(config.private_key.value().unwrap().contains("-----BEGIN"));
    assert_eq!(config.max_connections, 4);
}
