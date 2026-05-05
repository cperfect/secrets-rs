//! Demonstrates three patterns for sharing secrets across subsystems when
//! the same struct instance cannot be passed everywhere.
//!
//! The patterns are shown in order of preference:
//!
//! 1. `Arc<AppConfig>` — bind once at startup, share the whole config.
//!    No secret values are duplicated; all subsystems read from one allocation.
//!
//! 2. `Arc<Secret<T>>` — share a single bound secret across threads or
//!    subsystems without duplicating the value.
//!
//! 3. `Secret::urn()` — construct an independent unbound secret from an
//!    existing one. Use this only when subsystems must bind separately (e.g.
//!    different registries or initialization lifetimes).
//!
//! Run with: `cargo run --example sharing`

use std::sync::Arc;

use secrets_rs::{Secret, SourceRegistry, bind_all};

#[derive(secrets_rs::Bindable)]
struct AppConfig {
    api_key: Secret<String>,
    db_password: Secret<String>,
}

fn main() {
    unsafe {
        std::env::set_var("SHARE_API_KEY", "key-shared-across-subsystems");
        std::env::set_var("SHARE_DB_PASSWORD", "hunter2");
    }

    let registry = SourceRegistry::new();

    // ---- Pattern 1: Arc<AppConfig> ----------------------------------------
    //
    // Build and bind the config once, then wrap it in Arc. Each subsystem
    // receives a clone of the Arc — a pointer copy, not a value copy.

    let mut config = AppConfig {
        api_key: Secret::new("urn:secrets-rs:env:SHARE_API_KEY").unwrap(),
        db_password: Secret::new("urn:secrets-rs:env:SHARE_DB_PASSWORD").unwrap(),
    };
    bind_all(&mut config, &registry).unwrap();
    let config = Arc::new(config);

    println!("Pattern 1 — Arc<AppConfig>:");
    println!("  main thread  api_key : {}", config.api_key);

    let config_for_worker = Arc::clone(&config);
    let handle = std::thread::spawn(move || {
        // No binding needed — the value is already resolved.
        println!("  worker thread api_key : {}", config_for_worker.api_key);
        println!(
            "  worker sees value len : {}",
            config_for_worker.api_key.value().unwrap().len()
        );
    });
    handle.join().unwrap();

    // ---- Pattern 2: Arc<Secret<T>> ----------------------------------------
    //
    // When only one secret needs to be shared, wrap just that secret in Arc
    // after binding. The value is fetched once and never duplicated.

    let mut api_key: Secret<String> = Secret::new("urn:secrets-rs:env:SHARE_API_KEY").unwrap();
    api_key.bind(&registry).unwrap();
    let api_key = Arc::new(api_key);

    println!("\nPattern 2 — Arc<Secret<T>>:");
    println!("  main thread  : {}", api_key);

    let key_for_worker = Arc::clone(&api_key);
    let handle = std::thread::spawn(move || {
        println!("  worker thread : {}", key_for_worker);
        println!("  worker sees   : {}", key_for_worker.value().unwrap());
    });
    handle.join().unwrap();

    // ---- Pattern 3: Secret::urn() -----------------------------------------
    //
    // When two subsystems must bind independently — e.g. different registries
    // or initialization lifetimes — use urn() to obtain the URN and construct
    // a second unbound secret. Each subsystem then fetches the value itself.
    //
    // Use this only when patterns 1 or 2 are not applicable: it results in two
    // source lookups instead of one.

    let original: Secret<String> = Secret::new("urn:secrets-rs:env:SHARE_API_KEY").unwrap();

    // Subsystem B builds its own registry and binds its own copy.
    let mut for_subsystem_b: Secret<String> = Secret::new(&original.urn().to_string()).unwrap();

    assert_eq!(original.urn(), for_subsystem_b.urn());

    let registry_b = SourceRegistry::new();
    for_subsystem_b.bind(&registry_b).unwrap();

    println!("\nPattern 3 — Secret::urn() for independent binding:");
    println!("  original (unbound) : {}", original);
    println!("  subsystem B (bound): {}", for_subsystem_b);
    println!(
        "  subsystem B value  : {}",
        for_subsystem_b.value().unwrap()
    );

    unsafe {
        std::env::remove_var("SHARE_API_KEY");
        std::env::remove_var("SHARE_DB_PASSWORD");
    }
}
