use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum length of a display name, in characters.
pub const MAX_NAME_CHARS: usize = 200;

/// A validated, trimmed display name: non-empty, at most
/// [`MAX_NAME_CHARS`] characters, and free of control characters.
///
/// Deserialisation goes through the same validation, so untrusted input
/// (e.g. from the UI over IPC) cannot bypass it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Name(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NameError {
    #[error("name must not be empty")]
    Empty,
    #[error("name must be at most {MAX_NAME_CHARS} characters")]
    TooLong,
    #[error("name must not contain control characters")]
    ControlCharacter,
}

impl Name {
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(NameError::Empty);
        }
        if trimmed.chars().count() > MAX_NAME_CHARS {
            return Err(NameError::TooLong);
        }
        if trimmed.chars().any(char::is_control) {
            return Err(NameError::ControlCharacter);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Name {
    type Error = NameError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Name> for String {
    fn from(value: Name) -> Self {
        value.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_accepts_valid_names() {
        assert_eq!(
            Name::parse("  Acme Portal ").map(|n| n.0),
            Ok("Acme Portal".into())
        );
    }

    #[test]
    fn rejects_empty_and_whitespace() {
        assert_eq!(Name::parse(""), Err(NameError::Empty));
        assert_eq!(Name::parse("   \t "), Err(NameError::Empty));
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(MAX_NAME_CHARS + 1);
        assert_eq!(Name::parse(&long), Err(NameError::TooLong));
        assert!(Name::parse(&"a".repeat(MAX_NAME_CHARS)).is_ok());
    }

    #[test]
    fn rejects_control_characters() {
        assert_eq!(
            Name::parse("bad\u{0}name"),
            Err(NameError::ControlCharacter)
        );
        assert_eq!(Name::parse("line\nbreak"), Err(NameError::ControlCharacter));
    }

    #[test]
    fn deserialisation_is_validated() {
        assert!(serde_json::from_str::<Name>("\"\"").is_err());
        assert!(serde_json::from_str::<Name>("\"ok\"").is_ok());
    }
}
