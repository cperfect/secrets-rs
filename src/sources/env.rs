use crate::{error::SourceError, source::Source};

/// A [`Source`] that retrieves secrets from environment variables.
///
/// The secret `name` is used directly as the environment variable name.
/// Case sensitivity therefore matches the OS (case-sensitive on Unix,
/// case-insensitive on Windows).
pub struct EnvSource;

impl Source for EnvSource {
    fn get(&self, name: &str) -> Result<Vec<u8>, SourceError> {
        std::env::var(name)
            .map(|v| v.into_bytes())
            .map_err(|_| SourceError::NotFound {
                name: name.to_owned(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_bytes_for_present_var() {
        unsafe { std::env::set_var("SECRETS_RS_TEST_PRESENT", "hello") };
        let result = EnvSource.get("SECRETS_RS_TEST_PRESENT").unwrap();
        assert_eq!(result, b"hello");
        unsafe { std::env::remove_var("SECRETS_RS_TEST_PRESENT") };
    }

    #[test]
    fn returns_not_found_for_absent_var() {
        unsafe { std::env::remove_var("SECRETS_RS_TEST_ABSENT") };
        let err = EnvSource.get("SECRETS_RS_TEST_ABSENT").unwrap_err();
        assert!(matches!(err, SourceError::NotFound { .. }));
    }
}
