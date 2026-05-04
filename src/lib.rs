//! `secrets-rs` — safe secret retrieval for Rust applications.
//!
//! Secrets are identified by URN (`urn:secrets-rs:<source_id>:<name>`) and
//! stored in [`Secret<T>`] structs. All default access paths (Display, Debug,
//! serde serialization) emit a **masked value** that is safe to log. The real
//! value must be requested explicitly via [`Secret::value`].
//!
//! # Built-in sources
//!
//! | Source | `source_id` | Backed by |
//! |--------|-------------|-----------|
//! | [`EnvSource`] | `"env"` (pre-registered) | `std::env::var` |
//! | [`FileSource`] | e.g. `"file"` | `std::fs::read` (use [`FileSource::with_base`] for stable resolution in multi-threaded programs) |
//!
//! # Quick start
//!
//! ```rust
//! use secrets_rs::{Secret, SourceRegistry, bind_all};
//!
//! #[derive(secrets_rs::Bindable)]
//! struct Config {
//!     api_key: Secret<String>,
//! }
//!
//! # unsafe { std::env::set_var("API_KEY", "s3cr3t") };
//! let mut config = Config {
//!     api_key: Secret::new("urn:secrets-rs:env:API_KEY").unwrap(),
//! };
//!
//! // EnvSource is registered under "env" by default.
//! let registry = SourceRegistry::new();
//! bind_all(&mut config, &registry).unwrap();
//!
//! // Masked value — safe to log
//! println!("{}", config.api_key);
//!
//! // Real value — explicit opt-in
//! let key: &str = config.api_key.value().unwrap();
//! # let _ = key;
//! ```

pub mod error;
pub mod source;
pub mod sources;
pub mod urn;

mod bindable;
mod secret;

pub use bindable::{Bindable, bind_all};
pub use error::{BindError, SourceError, UnboundError, UrnParseError};
pub use secret::{Secret, SecretValue};
pub use secrets_rs_macros::Bindable;
pub use source::{Source, SourceRegistry};
pub use sources::env::EnvSource;
pub use sources::file::FileSource;
pub use urn::Urn;
