//! Demonstrates using `#[derive(Bindable)]` to bind all secrets in a config
//! struct in one step, with non-secret fields left untouched.
//!
//! Run with: `cargo run --example config`

use secrets_rs::{Secret, SourceRegistry, bind_all};

#[derive(secrets_rs::Bindable)]
struct AppConfig {
    db_password: Secret<String>,
    api_key: Secret<String>,
    /// Plain fields are unaffected by `bind_all`.
    max_connections: u32,
}

fn main() {
    // In a real application these would be set in the process environment
    // before the application starts.
    unsafe {
        std::env::set_var("APP_DB_PASSWORD", "correct-horse-battery-staple");
        std::env::set_var("APP_API_KEY", "sk-prod-0xdeadbeef");
    }

    let mut config = AppConfig {
        db_password: Secret::new("urn:secrets-rs:env:APP_DB_PASSWORD").unwrap(),
        api_key: Secret::new("urn:secrets-rs:env:APP_API_KEY").unwrap(),
        max_connections: 10,
    };

    // Safe to log — all Secret fields show masked values.
    println!("db_password     : {}", config.db_password);
    println!("api_key         : {}", config.api_key);
    println!("max_connections : {}", config.max_connections);

    // EnvSource is registered under "env" by default.
    let registry = SourceRegistry::new();
    bind_all(&mut config, &registry).expect("one or more secrets could not be bound");

    println!("\nAfter bind_all:");
    println!("db_password     : {}", config.db_password);
    println!("api_key         : {}", config.api_key);

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
