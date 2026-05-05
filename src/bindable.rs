use crate::{error::BindError, source::SourceRegistry};

/// Implemented by structs that contain one or more [`Secret`](crate::Secret) fields.
///
/// Call [`bind_all`] to bind every secret in the struct in one step, collecting
/// all errors rather than stopping at the first failure.
///
/// Implement this trait manually until a derive macro is available:
///
/// ```rust
/// use secrets_rs::{Bindable, BindError, Secret, SourceRegistry};
///
/// struct Config {
///     api_key: Secret<String>,
/// }
///
/// impl Bindable for Config {
///     fn bind_secrets(&mut self, registry: &SourceRegistry) -> Result<(), Vec<BindError>> {
///         let mut errors = Vec::new();
///         if let Err(e) = self.api_key.bind(registry) {
///             errors.push(e);
///         }
///         if errors.is_empty() { Ok(()) } else { Err(errors) }
///     }
/// }
/// ```
pub trait Bindable {
    /// Bind all [`Secret`](crate::Secret) fields using sources from `registry`.
    ///
    /// Collects every error rather than stopping at the first failure so that
    /// callers can report all missing secrets at once.
    fn bind_secrets(&mut self, registry: &SourceRegistry) -> Result<(), Vec<BindError>>;
}

/// Binds all secrets in `target` using `registry`.
///
/// This is a convenience wrapper around [`Bindable::bind_secrets`].
pub fn bind_all<T: Bindable>(
    target: &mut T,
    registry: &SourceRegistry,
) -> Result<(), Vec<BindError>> {
    target.bind_secrets(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Secret, SourceRegistry};

    struct Config {
        key_a: Secret<String>,
        key_b: Secret<String>,
    }

    impl Bindable for Config {
        fn bind_secrets(&mut self, registry: &SourceRegistry) -> Result<(), Vec<BindError>> {
            let mut errors = Vec::new();
            if let Err(e) = self.key_a.bind(registry) {
                errors.push(e);
            }
            if let Err(e) = self.key_b.bind(registry) {
                errors.push(e);
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    #[test]
    fn bind_all_populates_both_secrets() {
        unsafe {
            std::env::set_var("BINDABLE_TEST_A", "alpha");
            std::env::set_var("BINDABLE_TEST_B", "beta");
        }

        let mut config = Config {
            key_a: Secret::new("urn:secrets-rs:env:BINDABLE_TEST_A").unwrap(),
            key_b: Secret::new("urn:secrets-rs:env:BINDABLE_TEST_B").unwrap(),
        };

        let registry = SourceRegistry::new();
        bind_all(&mut config, &registry).unwrap();

        assert_eq!(config.key_a.value().unwrap(), "alpha");
        assert_eq!(config.key_b.value().unwrap(), "beta");

        unsafe {
            std::env::remove_var("BINDABLE_TEST_A");
            std::env::remove_var("BINDABLE_TEST_B");
        }
    }

    #[test]
    fn bind_all_collects_all_errors() {
        unsafe {
            std::env::remove_var("BINDABLE_MISSING_A");
            std::env::remove_var("BINDABLE_MISSING_B");
        }

        let mut config = Config {
            key_a: Secret::new("urn:secrets-rs:env:BINDABLE_MISSING_A").unwrap(),
            key_b: Secret::new("urn:secrets-rs:env:BINDABLE_MISSING_B").unwrap(),
        };

        let registry = SourceRegistry::new();
        let errors = bind_all(&mut config, &registry).unwrap_err();
        assert_eq!(errors.len(), 2);
    }
}
