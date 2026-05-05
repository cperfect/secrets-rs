use std::{fmt, str::FromStr};

use crate::error::UrnParseError;

/// Returns `true` if every character in `s` is valid in the `source_id`
/// position of a `urn:secrets-rs:<source_id>:<name>` URN.
///
/// Accepts the *literal* characters from the RFC 8141 NSS pchar set, minus `:`
/// (which is our segment separator): ASCII letters, digits, and
/// `-._~!$&'()*+,;=@/`. Percent-encoded sequences (`%XX`) are **not**
/// accepted — source IDs are developer-defined identifiers where encoding adds
/// no value and would require normalisation logic (`%41` vs `A`, etc.).
pub(crate) fn is_valid_source_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(
                    c,
                    '-' | '.' | '_' | '~'  // unreserved mark chars
                    | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='  // sub-delims
                    | '@' | '/' // remaining pchar / NSS chars
                )
        })
}

/// A secret URN of the form `urn:secrets-rs:<source_id>:<name>`.
///
/// The scheme (`urn`) and NID (`secrets-rs`) are validated case-insensitively
/// per RFC 8141. The `source_id` and `name` are stored as-is; case sensitivity
/// is determined by the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Urn {
    /// Identifies which source holds this secret.
    pub source_id: String,
    /// The name of the secret within the source.
    pub name: String,
}

impl Urn {
    /// Creates a `Urn` by parsing a URN string.
    pub fn parse(s: &str) -> Result<Self, UrnParseError> {
        s.parse()
    }
}

impl FromStr for Urn {
    type Err = UrnParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(4, ':').collect();
        if parts.len() != 4 {
            // splitn(4) on "a:b:c" yields ["a","b","c"] — only 3 parts
            return Err(UrnParseError::WrongSegmentCount(parts.len()));
        }

        if !parts[0].eq_ignore_ascii_case("urn") {
            return Err(UrnParseError::InvalidScheme(parts[0].to_owned()));
        }
        if !parts[1].eq_ignore_ascii_case("secrets-rs") {
            return Err(UrnParseError::InvalidNid(parts[1].to_owned()));
        }
        if parts[2].is_empty() {
            return Err(UrnParseError::EmptySourceId);
        }
        if !is_valid_source_id(parts[2]) {
            return Err(UrnParseError::InvalidSourceId(parts[2].to_owned()));
        }
        if parts[3].is_empty() {
            return Err(UrnParseError::EmptyName);
        }

        Ok(Urn {
            source_id: parts[2].to_owned(),
            name: parts[3].to_owned(),
        })
    }
}

impl fmt::Display for Urn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "urn:secrets-rs:{}:{}", self.source_id, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_urn() {
        let urn: Urn = "urn:secrets-rs:env:MY_SECRET".parse().unwrap();
        assert_eq!(urn.source_id, "env");
        assert_eq!(urn.name, "MY_SECRET");
    }

    #[test]
    fn case_insensitive_scheme_and_nid() {
        let urn: Urn = "URN:SECRETS-RS:env:KEY".parse().unwrap();
        assert_eq!(urn.source_id, "env");
        assert_eq!(urn.name, "KEY");
    }

    #[test]
    fn display_uses_lowercase_prefix() {
        let urn: Urn = "urn:secrets-rs:env:KEY".parse().unwrap();
        assert_eq!(urn.to_string(), "urn:secrets-rs:env:KEY");
    }

    #[test]
    fn name_may_contain_colons() {
        // splitn(4) means everything after the third colon is the name
        let urn: Urn = "urn:secrets-rs:env:a:b:c".parse().unwrap();
        assert_eq!(urn.name, "a:b:c");
    }

    #[test]
    fn error_on_wrong_scheme() {
        let err = "nrn:secrets-rs:env:KEY".parse::<Urn>().unwrap_err();
        assert!(matches!(err, UrnParseError::InvalidScheme(_)));
    }

    #[test]
    fn error_on_wrong_nid() {
        let err = "urn:other:env:KEY".parse::<Urn>().unwrap_err();
        assert!(matches!(err, UrnParseError::InvalidNid(_)));
    }

    #[test]
    fn error_on_too_few_segments() {
        let err = "urn:secrets-rs:env".parse::<Urn>().unwrap_err();
        assert!(matches!(err, UrnParseError::WrongSegmentCount(_)));
    }

    #[test]
    fn error_on_empty_source_id() {
        let err = "urn:secrets-rs::KEY".parse::<Urn>().unwrap_err();
        assert_eq!(err, UrnParseError::EmptySourceId);
    }

    #[test]
    fn error_on_empty_name() {
        let err = "urn:secrets-rs:env:".parse::<Urn>().unwrap_err();
        assert_eq!(err, UrnParseError::EmptyName);
    }

    #[test]
    fn error_on_source_id_with_space() {
        let err = "urn:secrets-rs:bad id:KEY".parse::<Urn>().unwrap_err();
        assert!(matches!(err, UrnParseError::InvalidSourceId(_)));
    }

    #[test]
    fn error_on_source_id_with_non_ascii() {
        let err = "urn:secrets-rs:café:KEY".parse::<Urn>().unwrap_err();
        assert!(matches!(err, UrnParseError::InvalidSourceId(_)));
    }

    #[test]
    fn valid_source_id_with_punctuation() {
        let urn = "urn:secrets-rs:aws.sm-v2/prod:MY_SECRET"
            .parse::<Urn>()
            .unwrap();
        assert_eq!(urn.source_id, "aws.sm-v2/prod");
        assert_eq!(urn.name, "MY_SECRET");
    }
}
