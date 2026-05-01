use std::io;

use crate::{error::SourceError, source::Source};

/// A [`Source`] that retrieves secrets from the local filesystem.
///
/// The secret `name` is used directly as a file path. Absolute paths are used
/// as-is; relative paths are resolved against the process's current working
/// directory at the time [`get`](Source::get) is called.
///
/// The primary use case is loading keys and certificates, e.g.:
///
/// ```text
/// urn:secrets-rs:file:/etc/ssl/private/server.key
/// urn:secrets-rs:file:certs/ca.crt
/// ```
pub struct FileSource;

impl Source for FileSource {
    fn get(&self, name: &str) -> Result<Vec<u8>, SourceError> {
        std::fs::read(name).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => SourceError::NotFound {
                name: name.to_owned(),
            },
            _ => SourceError::Other(e.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn returns_bytes_for_existing_file() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"file-secret").unwrap();
        let result = FileSource.get(f.path().to_str().unwrap()).unwrap();
        assert_eq!(result, b"file-secret");
    }

    #[test]
    fn returns_not_found_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nonexistent.key");
        let err = FileSource.get(missing.to_str().unwrap()).unwrap_err();
        assert!(matches!(err, SourceError::NotFound { .. }));
    }
}
