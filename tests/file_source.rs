use std::path::PathBuf;
use std::sync::{Mutex, Once};

use secrets_rs::{BindError, FileSource, Secret, SourceRegistry, bind_all};

static FIXTURES: Once = Once::new();
// set_current_dir is process-global; serialise all tests that touch it.
static CWD_LOCK: Mutex<()> = Mutex::new(());

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures_dir() -> PathBuf {
    manifest_dir().join("tests/fixtures")
}

fn ensure_fixtures() {
    FIXTURES.call_once(|| {
        let dir = fixtures_dir();
        let all_present = ["test.key", "test.crt", "test.der"]
            .iter()
            .all(|f| dir.join(f).exists());
        if !all_present {
            let status = std::process::Command::new("bash")
                .arg(dir.join("generate.sh"))
                .status()
                .expect("failed to run generate.sh");
            assert!(status.success(), "generate.sh exited with failure");
        }
    });
}

/// Restores the working directory on drop so that a panicking test cannot
/// leave the process in the wrong directory.
struct CwdGuard(PathBuf);

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

/// Runs `f` with the working directory set to the crate root, then restores it.
fn with_cwd_as_manifest<F: FnOnce()>(f: F) {
    // Recover from a poisoned mutex so a panicking test does not block others.
    let _lock = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = std::env::current_dir().unwrap();
    let _guard = CwdGuard(original);
    std::env::set_current_dir(manifest_dir()).unwrap();
    f();
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
    let urn = format!(
        "urn:secrets-rs:file:{}",
        fixtures_dir().join("test.der").display()
    );

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
    with_cwd_as_manifest(|| {
        let mut secret: Secret<String> =
            Secret::new("urn:secrets-rs:file:tests/fixtures/test.crt").unwrap();
        secret.bind(&registry()).unwrap();

        assert!(
            secret
                .value()
                .unwrap()
                .contains("-----BEGIN CERTIFICATE-----")
        );
    });
}

// --- relative path with parent component ---

#[test]
fn binds_relative_path_with_parent_component() {
    ensure_fixtures();
    with_cwd_as_manifest(|| {
        // tests/fixtures/../fixtures/test.key resolves to tests/fixtures/test.key
        let mut secret: Secret<String> =
            Secret::new("urn:secrets-rs:file:tests/fixtures/../fixtures/test.key").unwrap();
        secret.bind(&registry()).unwrap();

        assert!(secret.value().unwrap().contains("-----BEGIN"));
    });
}

// --- missing file ---

#[test]
fn missing_file_returns_name_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nonexistent.key");
    // `missing` is guaranteed not to exist — the tempdir was just created empty.
    let urn = format!("urn:secrets-rs:file:{}", missing.display());

    let mut secret: Secret<Vec<u8>> = Secret::new(&urn).unwrap();
    let err = secret.bind(&registry()).unwrap_err();
    assert!(matches!(err, BindError::NameNotFound { .. }));
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
    let abs_der = fixtures_dir().join("test.der");

    with_cwd_as_manifest(|| {
        let mut config = TlsConfig {
            certificate: Secret::new(&format!("urn:secrets-rs:file:{}", abs_der.display()))
                .unwrap(),
            private_key: Secret::new("urn:secrets-rs:file:tests/fixtures/test.key").unwrap(),
            max_connections: 4,
        };

        bind_all(&mut config, &registry()).unwrap();

        assert_eq!(config.certificate.value().unwrap()[0], 0x30);
        assert!(config.private_key.value().unwrap().contains("-----BEGIN"));
        assert_eq!(config.max_connections, 4);
    });
}
