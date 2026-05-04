use std::collections::HashMap;

use crate::{EnvSource, error::SourceError};

/// A source from which secret values can be retrieved.
///
/// Implementations are responsible for looking up a secret by `name` and
/// returning its raw bytes. Case sensitivity of `name` is source-specific.
pub trait Source: Send + Sync {
    /// Retrieve the raw bytes for the secret identified by `name`.
    fn get(&self, name: &str) -> Result<Vec<u8>, SourceError>;
}

/// A registry that maps source IDs to their [`Source`] implementations.
///
/// [`EnvSource`] is registered under `"env"` by default. Additional sources
/// can be added with [`register`](SourceRegistry::register); registering an
/// already-used ID replaces the previous source.
///
/// Pass a registry to [`Secret::bind`](crate::Secret::bind) or
/// [`bind_all`](crate::bind_all) to resolve secrets.
pub struct SourceRegistry {
    sources: HashMap<String, Box<dyn Source>>,
}

impl SourceRegistry {
    /// Creates a registry with [`EnvSource`] pre-registered under `"env"`.
    pub fn new() -> Self {
        let mut registry = Self {
            sources: HashMap::new(),
        };
        registry.register("env", EnvSource);
        registry
    }

    /// Registers a source under the given `id`.
    ///
    /// The `id` must match the `source_id` component of the secret URN.
    /// If `id` is already registered, the previous source is replaced.
    pub fn register(&mut self, id: impl Into<String>, source: impl Source + 'static) {
        self.sources.insert(id.into(), Box::new(source));
    }

    /// Returns the source registered under `source_id`, if any.
    pub fn get(&self, source_id: &str) -> Option<&dyn Source> {
        self.sources.get(source_id).map(|s| s.as_ref())
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pre_registers_env_source() {
        let registry = SourceRegistry::new();
        assert!(
            registry.get("env").is_some(),
            "expected \"env\" to be registered by default"
        );
    }

    #[test]
    fn new_does_not_register_file_by_default() {
        let registry = SourceRegistry::new();
        assert!(registry.get("file").is_none());
    }

    #[test]
    fn env_source_resolves_without_explicit_registration() {
        unsafe { std::env::set_var("REGISTRY_DEFAULT_ENV_TEST", "works") };
        let result = SourceRegistry::new()
            .get("env")
            .unwrap()
            .get("REGISTRY_DEFAULT_ENV_TEST");
        unsafe { std::env::remove_var("REGISTRY_DEFAULT_ENV_TEST") };
        assert_eq!(result.unwrap(), b"works");
    }

    #[test]
    fn register_replaces_existing_id() {
        struct ConstSource(&'static [u8]);
        impl Source for ConstSource {
            fn get(&self, _name: &str) -> Result<Vec<u8>, SourceError> {
                Ok(self.0.to_vec())
            }
        }

        let mut registry = SourceRegistry::new();
        registry.register("env", ConstSource(b"replaced"));
        let result = registry.get("env").unwrap().get("anything").unwrap();
        assert_eq!(result, b"replaced");
    }
}
