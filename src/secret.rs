use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{
    error::{BindError, UnboundError, UrnParseError},
    source::SourceRegistry,
    urn::Urn,
};

/// Implemented by types that can be stored inside a [`Secret`].
///
/// Provides the conversion from raw bytes returned by a source, the type label
/// used in masked values, and a measurement of the value's size.
pub trait SecretValue: Sized {
    /// Short label included in the masked value, e.g. `"string"`, `"bytes"`, `"json"`.
    fn type_name() -> &'static str;

    /// Construct a value from the raw bytes returned by a source.
    fn from_bytes(bytes: Vec<u8>, urn: &str) -> Result<Self, BindError>;

    /// Human-readable size/length included in the masked value, e.g. `"12"`.
    fn masked_size(&self) -> String;
}

impl SecretValue for String {
    fn type_name() -> &'static str {
        "string"
    }

    fn from_bytes(bytes: Vec<u8>, urn: &str) -> Result<Self, BindError> {
        String::from_utf8(bytes).map_err(|e| BindError::TypeConversion {
            urn: urn.to_owned(),
            detail: e.to_string(),
        })
    }

    fn masked_size(&self) -> String {
        self.chars().count().to_string()
    }
}

impl SecretValue for Vec<u8> {
    fn type_name() -> &'static str {
        "bytes"
    }

    fn from_bytes(bytes: Vec<u8>, _urn: &str) -> Result<Self, BindError> {
        Ok(bytes)
    }

    fn masked_size(&self) -> String {
        self.len().to_string()
    }
}

impl SecretValue for serde_json::Value {
    fn type_name() -> &'static str {
        "json"
    }

    fn from_bytes(bytes: Vec<u8>, urn: &str) -> Result<Self, BindError> {
        serde_json::from_slice(&bytes).map_err(|e| BindError::TypeConversion {
            urn: urn.to_owned(),
            detail: e.to_string(),
        })
    }

    fn masked_size(&self) -> String {
        // Use the compact serialized length as the size metric.
        self.to_string().len().to_string()
    }
}

/// A secret value identified by a URN.
///
/// Before [`bind`](Secret::bind) is called the secret is *unbound*; accessing
/// its value returns an [`UnboundError`]. All default display paths (
/// [`Display`](fmt::Display), [`Debug`], serde serialization) emit the masked
/// value, which is safe to include in logs.
///
/// # Masked value format
///
/// - Unbound: `urn:secrets-rs:env:KEY [UNBOUND]`
/// - Bound:   `urn:secrets-rs:env:KEY [string:12]`
pub struct Secret<T: SecretValue> {
    urn: Urn,
    value: Option<T>,
}

impl<T: SecretValue> Secret<T> {
    /// Creates an unbound secret from a URN string.
    pub fn new(urn_str: &str) -> Result<Self, UrnParseError> {
        Ok(Self {
            urn: urn_str.parse()?,
            value: None,
        })
    }

    /// Returns the underlying URN.
    pub fn urn(&self) -> &Urn {
        &self.urn
    }

    /// Returns the secret value, or an [`UnboundError`] if not yet bound.
    pub fn value(&self) -> Result<&T, UnboundError> {
        self.value.as_ref().ok_or_else(|| UnboundError {
            urn: self.urn.to_string(),
        })
    }

    /// Returns the masked value string — safe to log or serialize by default.
    pub fn masked_value(&self) -> String {
        match &self.value {
            None => format!("{} [UNBOUND]", self.urn),
            Some(v) => format!("{} [{}:{}]", self.urn, T::type_name(), v.masked_size()),
        }
    }

    /// Fetches the secret from the appropriate source in `registry` and stores
    /// it. Returns [`BindError`] if the source is not registered or the lookup
    /// fails.
    pub fn bind(&mut self, registry: &SourceRegistry) -> Result<(), BindError> {
        let urn_str = self.urn.to_string();
        let source =
            registry
                .get(&self.urn.source_id)
                .ok_or_else(|| BindError::SourceNotFound {
                    source_id: self.urn.source_id.clone(),
                })?;

        let bytes = source.get(&self.urn.name).map_err(|e| {
            use crate::error::SourceError;
            match e {
                SourceError::NotFound { name } => BindError::NameNotFound {
                    source_id: self.urn.source_id.clone(),
                    name,
                },
                other => BindError::Source {
                    urn: urn_str.clone(),
                    source: other,
                },
            }
        })?;

        self.value = Some(T::from_bytes(bytes, &urn_str)?);
        Ok(())
    }
}

impl<T: SecretValue> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.masked_value())
    }
}

/// Always displays the masked value — the real value is never revealed via Debug.
impl<T: SecretValue> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret({})", self.masked_value())
    }
}

impl<T: SecretValue> Serialize for Secret<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.masked_value())
    }
}

/// Deserialization of `Secret<T>` is intentionally unsupported. Use
/// [`bind_all`](crate::bind_all) / [`Bindable`](crate::Bindable) instead.
impl<'de, T: SecretValue> Deserialize<'de> for Secret<T> {
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Err(de::Error::custom(
            "direct deserialization of Secret is not supported; use bind_all() instead",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceRegistry;
    use crate::sources::env::EnvSource;

    #[test]
    fn unbound_masked_value() {
        let s: Secret<String> = Secret::new("urn:secrets-rs:env:MY_KEY").unwrap();
        assert_eq!(s.masked_value(), "urn:secrets-rs:env:MY_KEY [UNBOUND]");
    }

    #[test]
    fn display_shows_masked_value() {
        let s: Secret<String> = Secret::new("urn:secrets-rs:env:MY_KEY").unwrap();
        assert_eq!(s.to_string(), "urn:secrets-rs:env:MY_KEY [UNBOUND]");
    }

    #[test]
    fn debug_shows_masked_value() {
        let s: Secret<String> = Secret::new("urn:secrets-rs:env:MY_KEY").unwrap();
        assert_eq!(
            format!("{s:?}"),
            "Secret(urn:secrets-rs:env:MY_KEY [UNBOUND])"
        );
    }

    #[test]
    fn value_before_bind_is_error() {
        let s: Secret<String> = Secret::new("urn:secrets-rs:env:MY_KEY").unwrap();
        assert!(s.value().is_err());
    }

    #[test]
    fn bound_masked_value_includes_type_and_length() {
        unsafe { std::env::set_var("SECRET_TEST_MASKED", "hello") };
        let mut s: Secret<String> = Secret::new("urn:secrets-rs:env:SECRET_TEST_MASKED").unwrap();
        let mut registry = SourceRegistry::new();
        registry.register("env", EnvSource);
        s.bind(&registry).unwrap();
        assert_eq!(
            s.masked_value(),
            "urn:secrets-rs:env:SECRET_TEST_MASKED [string:5]"
        );
        unsafe { std::env::remove_var("SECRET_TEST_MASKED") };
    }

    #[test]
    fn value_after_bind_returns_correct_value() {
        unsafe { std::env::set_var("SECRET_TEST_VALUE", "s3cr3t") };
        let mut s: Secret<String> = Secret::new("urn:secrets-rs:env:SECRET_TEST_VALUE").unwrap();
        let mut registry = SourceRegistry::new();
        registry.register("env", EnvSource);
        s.bind(&registry).unwrap();
        assert_eq!(s.value().unwrap(), "s3cr3t");
        unsafe { std::env::remove_var("SECRET_TEST_VALUE") };
    }

    #[test]
    fn serialize_produces_masked_string() {
        let s: Secret<String> = Secret::new("urn:secrets-rs:env:MY_KEY").unwrap();
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#""urn:secrets-rs:env:MY_KEY [UNBOUND]""#);
    }

    #[test]
    fn deserialize_always_errors() {
        let result = serde_json::from_str::<Secret<String>>(r#""anything""#);
        assert!(result.is_err());
    }
}
