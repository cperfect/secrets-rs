use std::io;
use std::path::{Path, PathBuf};

use crate::{error::SourceError, source::Source};

/// A [`Source`] that retrieves secrets from the local filesystem.
///
/// Two construction modes are available:
///
/// - **[`FileSource::new()`]** — resolves relative paths against the process's
///   current working directory at the time [`get`](Source::get) is called.
///   Simple, but the result is non-deterministic if any code calls
///   [`std::env::set_current_dir`] concurrently.
///
/// - **[`FileSource::with_base(dir)`](FileSource::with_base)** — captures an
///   absolute base directory at construction time and resolves all relative
///   paths against it. The resolved path is stable regardless of later CWD
///   changes. Absolute paths in the URN name are still used as-is.
///
/// The primary use case is loading keys and certificates, e.g.:
///
/// ```text
/// urn:secrets-rs:file:/etc/ssl/private/server.key   // absolute — same in both modes
/// urn:secrets-rs:file:certs/ca.crt                  // relative — stable only with with_base
/// ```
///
/// # Security
///
/// Because the URN name is used as a filesystem path without further
/// validation, binding a `FileSource` secret with an attacker-controlled URN
/// is an **arbitrary file-read** vulnerability. Only bind URNs that come from
/// **trusted configuration** (static code, operator-supplied config files with
/// restricted write permissions, etc.). Never construct or accept
/// `urn:secrets-rs:file:...` URNs from untrusted input such as API requests,
/// user-supplied data, or deserialized network payloads.
///
/// [`with_base`](FileSource::with_base) anchors relative resolution to a known
/// directory but does **not** prevent path-traversal sequences (`../`) in the
/// URN name from escaping that directory; the trusted-configuration requirement
/// still applies.
pub struct FileSource {
    base: Option<PathBuf>,
}

impl FileSource {
    /// Creates a `FileSource` that resolves relative paths against the
    /// process's current working directory at call time.
    pub fn new() -> Self {
        Self { base: None }
    }

    /// Creates a `FileSource` that resolves relative paths against `base`.
    ///
    /// `base` is captured at construction time, so subsequent calls to
    /// [`std::env::set_current_dir`] do not affect resolution. For stable
    /// behaviour `base` should be an absolute path; if it is relative it is
    /// stored as-is and still subject to CWD changes.
    ///
    /// Absolute paths in the URN name are used as-is regardless of `base`.
    pub fn with_base(base: impl Into<PathBuf>) -> Self {
        Self {
            base: Some(base.into()),
        }
    }

    fn resolve(&self, name: &str) -> PathBuf {
        let p = Path::new(name);
        if p.is_absolute() {
            p.to_path_buf()
        } else if let Some(base) = &self.base {
            base.join(p)
        } else {
            p.to_path_buf()
        }
    }
}

impl Default for FileSource {
    fn default() -> Self {
        Self::new()
    }
}

impl Source for FileSource {
    fn get(&self, name: &str) -> Result<Vec<u8>, SourceError> {
        let path = self.resolve(name);
        std::fs::read(&path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => SourceError::NotFound {
                name: name.to_owned(),
            },
            _ => SourceError::Other(format!("failed to read file `{}`: {}", name, e)),
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
        let result = FileSource::new().get(f.path().to_str().unwrap()).unwrap();
        assert_eq!(result, b"file-secret");
    }

    #[test]
    fn returns_not_found_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nonexistent.key");
        let err = FileSource::new()
            .get(missing.to_str().unwrap())
            .unwrap_err();
        assert!(matches!(err, SourceError::NotFound { .. }));
    }

    #[test]
    fn with_base_resolves_relative_against_base() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("secret.txt"), b"base-secret").unwrap();

        let src = FileSource::with_base(dir.path());
        let result = src.get("secret.txt").unwrap();
        assert_eq!(result, b"base-secret");
    }

    #[test]
    fn with_base_absolute_name_ignores_base() {
        let base_dir = tempfile::tempdir().unwrap();
        let other_dir = tempfile::tempdir().unwrap();
        std::fs::write(other_dir.path().join("abs.txt"), b"abs-secret").unwrap();

        let abs_path = other_dir.path().join("abs.txt");
        let src = FileSource::with_base(base_dir.path());
        let result = src.get(abs_path.to_str().unwrap()).unwrap();
        assert_eq!(result, b"abs-secret");
    }

    #[test]
    fn with_base_not_found_uses_original_name_in_error() {
        let base_dir = tempfile::tempdir().unwrap();
        let src = FileSource::with_base(base_dir.path());
        let err = src.get("missing.key").unwrap_err();
        assert!(
            matches!(&err, SourceError::NotFound { name } if name == "missing.key"),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn other_error_uses_original_name_not_resolved_path() {
        // Reading a directory produces EISDIR (an Other-class error).
        // The error message must contain the original URN name, not the
        // resolved path, so the base directory is never disclosed.
        let base_dir = tempfile::tempdir().unwrap();
        let sub_dir = base_dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        let src = FileSource::with_base(base_dir.path());
        let err = src.get("subdir").unwrap_err();

        let msg = match &err {
            SourceError::Other(m) => m.clone(),
            other => panic!("expected Other, got {other:?}"),
        };
        assert!(msg.contains("subdir"), "original name missing from: {msg}");
        assert!(
            !msg.contains(base_dir.path().to_str().unwrap()),
            "base directory disclosed in: {msg}"
        );
    }
}
