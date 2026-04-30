use std::collections::HashMap;

use crate::error::SourceError;

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
/// Pass a populated registry to [`Secret::bind`](crate::Secret::bind) or
/// [`bind_all`](crate::bind_all) to resolve secrets.
pub struct SourceRegistry {
    sources: HashMap<String, Box<dyn Source>>,
}

impl SourceRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Registers a source under the given `id`.
    ///
    /// The `id` must match the `source_id` component of the secret URN.
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
