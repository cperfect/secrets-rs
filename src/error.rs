/// Errors that can occur when parsing a secret URN string.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum UrnParseError {
    #[error(
        "URN must have exactly 4 colon-separated segments (urn:secrets-rs:<source_id>:<name>), got {0}"
    )]
    WrongSegmentCount(usize),

    #[error("URN scheme must be 'urn', got '{0}'")]
    InvalidScheme(String),

    #[error("URN NID must be 'secrets-rs', got '{0}'")]
    InvalidNid(String),

    #[error("source_id must not be empty")]
    EmptySourceId,

    #[error(
        "source_id '{0}' contains characters that are invalid in a URN; \
         allowed: ASCII letters, digits, and `-._~!$&'()*+,;=@/`"
    )]
    InvalidSourceId(String),

    #[error("name must not be empty")]
    EmptyName,
}

/// Errors produced by a [`crate::Source`] during secret retrieval.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("secret '{name}' not found in source")]
    NotFound { name: String },

    #[error("source error: {0}")]
    Other(String),
}

/// Errors returned by [`crate::SourceRegistry::register`].
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SourceRegisterError {
    #[error(
        "source id '{0}' is empty or contains characters that are invalid in a URN source_id; \
         allowed: ASCII letters, digits, and `-._~!$&'()*+,;=@/`"
    )]
    InvalidSourceId(String),
}

/// Errors that can occur while binding a secret to its value.
#[derive(Debug, thiserror::Error)]
pub enum BindError {
    #[error("no source registered with id '{source_id}'")]
    SourceNotFound { source_id: String },

    #[error("secret '{name}' not found in source '{source_id}'")]
    NameNotFound { source_id: String, name: String },

    #[error("type conversion failed for secret '{urn}': {detail}")]
    TypeConversion { urn: String, detail: String },

    #[error("source error for '{urn}': {source}")]
    Source { urn: String, source: SourceError },
}

/// Error returned when the value of a [`crate::Secret`] is accessed before it has been bound.
#[derive(Debug, thiserror::Error)]
#[error("secret '{urn}' has not been bound; call bind() or bind_all() first")]
pub struct UnboundError {
    pub urn: String,
}
