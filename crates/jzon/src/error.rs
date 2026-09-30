use crate::__private::*;
use core::fmt;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    UnexpectedEof,
    UnexpectedToken,
    MissingField(&'static str),
    InvalidNumber,
    InvalidUtf8,
    InvalidEscape,
    UnknownField,
    UnknownVariant,
    /// A `&str` field requested zero-copy but the JSON string has escape
    /// sequences — use `String` for that field to allow allocated unescaping.
    EscapedString,
    /// An object key contained escape sequences (extremely rare).
    EscapedKey,
    /// Nesting reached the 128-level recursion boundary (mirrors `serde_json`'s recursion limit).
    RecursionLimit,
    /// Trailing comma in an object or array (strict core or native Serde paths).
    TrailingComma,
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedEof => write!(f, "unexpected end of JSON input"),
            Error::UnexpectedToken => write!(f, "unexpected token in JSON input"),
            Error::MissingField(n) => write!(f, "missing JSON field `{n}`"),
            Error::InvalidNumber => write!(f, "invalid JSON number"),
            Error::InvalidUtf8 => write!(f, "invalid UTF-8 in JSON string"),
            Error::InvalidEscape => write!(f, "invalid JSON escape sequence"),
            Error::UnknownField => write!(f, "unknown JSON field (deny_unknown_fields)"),
            Error::UnknownVariant => write!(f, "unknown enum variant"),
            Error::EscapedString => write!(
                f,
                "string contains escape sequences — borrow is impossible; use `String`"
            ),
            Error::EscapedKey => write!(f, "JSON object key contains escape sequences"),
            Error::RecursionLimit => write!(f, "recursion limit exceeded"),
            Error::TrailingComma => write!(f, "trailing comma in JSON input"),
            Error::Custom(m) => write!(f, "{m}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
