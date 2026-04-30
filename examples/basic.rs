//! Demonstrates creating a `Secret<String>`, observing the masked value before
//! and after binding, and retrieving the real value via an explicit call.
//!
//! Run with: `cargo run --example basic`

use secrets_rs::{EnvSource, Secret, SourceRegistry};

fn main() {
    // Normally this env var would already be set in your environment.
    unsafe { std::env::set_var("MY_API_KEY", "super-secret-value-123") };

    let mut secret: Secret<String> =
        Secret::new("urn:secrets-rs:env:MY_API_KEY").expect("invalid URN");

    // Before binding: Display, Debug, and serde all emit the masked form.
    println!("Before bind : {secret}");
    println!("             {:?}", secret);

    let mut registry = SourceRegistry::new();
    registry.register("env", EnvSource);

    secret.bind(&registry).expect("binding failed");

    // After binding: still masked by default.
    println!("After bind  : {secret}");

    // Explicit opt-in to the real value.
    println!("Real value  : {}", secret.value().unwrap());

    unsafe { std::env::remove_var("MY_API_KEY") };
}
