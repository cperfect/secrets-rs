//! Demonstrates loading a config from JSON where secret fields are URN strings.
//! `serde` deserializes each URN into an unbound `Secret`; `bind_all` then
//! resolves the real values from their sources.
//!
//! Run with: `cargo run --example serde`

use secrets_rs::{Secret, SourceRegistry, bind_all};

#[derive(serde::Deserialize, secrets_rs::Bindable)]
struct AppConfig {
    db_password: Secret<String>,
    api_key: Secret<String>,
    max_connections: u32,
}

fn main() {
    // In a real application these would already be set in the environment.
    unsafe {
        std::env::set_var("APP_DB_PASSWORD", "correct-horse-battery-staple");
        std::env::set_var("APP_API_KEY", "sk-prod-0xdeadbeef");
    }

    // Simulates a config file — secret fields hold URNs, not raw values.
    let config_json = r#"{
        "db_password":     "urn:secrets-rs:env:APP_DB_PASSWORD",
        "api_key":         "urn:secrets-rs:env:APP_API_KEY",
        "max_connections": 10
    }"#;

    // Deserialize: Secret fields become unbound secrets identified by their URN.
    let mut config: AppConfig = serde_json::from_str(config_json).expect("invalid config");

    // Still safe to log — secrets are unbound and show masked values.
    println!("Before bind:");
    println!("  db_password     : {}", config.db_password);
    println!("  api_key         : {}", config.api_key);
    println!("  max_connections : {}", config.max_connections);

    // EnvSource is registered under "env" by default.
    let registry = SourceRegistry::new();
    bind_all(&mut config, &registry).expect("one or more secrets could not be bound");

    // Secrets are now bound — masked values show type and length.
    println!("\nAfter bind_all:");
    println!("  db_password : {}", config.db_password);
    println!("  api_key     : {}", config.api_key);

    // Real values only where explicitly needed.
    start_db(config.db_password.value().unwrap());

    unsafe {
        std::env::remove_var("APP_DB_PASSWORD");
        std::env::remove_var("APP_API_KEY");
    }
}

fn start_db(password: &str) {
    println!("\nConnecting to DB (password length: {})", password.len());
}
