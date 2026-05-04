use secrets_rs::{Secret, SourceRegistry, bind_all};

#[derive(secrets_rs::Bindable)]
struct Config {
    api_key: Secret<String>,
    db_pass: Secret<String>,
    /// Non-secret field — must be ignored by the derive macro.
    timeout_secs: u32,
}

#[test]
fn derive_macro_binds_secret_fields_only() {
    unsafe {
        std::env::set_var("DERIVE_TEST_API_KEY", "key123");
        std::env::set_var("DERIVE_TEST_DB_PASS", "pass456");
    }

    let mut config = Config {
        api_key: Secret::new("urn:secrets-rs:env:DERIVE_TEST_API_KEY").unwrap(),
        db_pass: Secret::new("urn:secrets-rs:env:DERIVE_TEST_DB_PASS").unwrap(),
        timeout_secs: 30,
    };

    let registry = SourceRegistry::new();
    bind_all(&mut config, &registry).unwrap();

    assert_eq!(config.api_key.value().unwrap(), "key123");
    assert_eq!(config.db_pass.value().unwrap(), "pass456");
    assert_eq!(config.timeout_secs, 30);

    unsafe {
        std::env::remove_var("DERIVE_TEST_API_KEY");
        std::env::remove_var("DERIVE_TEST_DB_PASS");
    }
}

#[test]
fn derive_macro_collects_all_bind_errors() {
    unsafe {
        std::env::remove_var("DERIVE_MISSING_KEY");
        std::env::remove_var("DERIVE_MISSING_PASS");
    }

    let mut config = Config {
        api_key: Secret::new("urn:secrets-rs:env:DERIVE_MISSING_KEY").unwrap(),
        db_pass: Secret::new("urn:secrets-rs:env:DERIVE_MISSING_PASS").unwrap(),
        timeout_secs: 30,
    };

    let registry = SourceRegistry::new();
    let errors = bind_all(&mut config, &registry).unwrap_err();
    assert_eq!(errors.len(), 2);
}
