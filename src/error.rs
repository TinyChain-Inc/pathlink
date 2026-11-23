use std::fmt;
use std::num::ParseIntError;
use std::str::Utf8Error;

/// A failure to parse an identifier, path, host, or link.
///
/// Parser failures retain reasons and native causes without copying the input
/// or formatting a diagnostic. Callers may add input context when reporting one.
#[derive(Debug)]
pub struct ParseError(pub(super) ErrorKind);

#[derive(Debug)]
pub(super) enum ErrorKind {
    Protocol,
    Domain,
    Host,
    Userinfo,
    Components,
    MissingAddress,
    ExpectedPath,
    TrailingSlash,
    Escape {
        offset: usize,
    },
    Utf8(Utf8Error),
    Identifier(hr_id::ParseError),
    Url {
        context: &'static str,
        cause: url::ParseError,
    },
    Port(ParseIntError),
    Message(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorKind::Protocol => f.write_str("invalid protocol"),
            ErrorKind::Domain => f.write_str("invalid domain name"),
            ErrorKind::Host => f.write_str("invalid host"),
            ErrorKind::Userinfo => f.write_str("invalid host (userinfo not allowed)"),
            ErrorKind::Components => f.write_str("invalid host (unexpected URL components)"),
            ErrorKind::MissingAddress => f.write_str("invalid host (missing address)"),
            ErrorKind::ExpectedPath => f.write_str("expected a path but found a hosted link"),
            ErrorKind::TrailingSlash => f.write_str("a non-root path cannot end with a slash"),
            ErrorKind::Escape { offset } => {
                write!(f, "invalid URI segment escape at byte {offset}")
            }
            ErrorKind::Utf8(cause) => write!(f, "invalid URI segment UTF-8: {cause}"),
            ErrorKind::Identifier(cause) => write!(f, "invalid path segment: {cause}"),
            ErrorKind::Url { context, cause } => write!(f, "invalid {context}: {cause}"),
            ErrorKind::Port(cause) => write!(f, "invalid port: {cause}"),
            ErrorKind::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ErrorKind::Utf8(cause) => Some(cause),
            ErrorKind::Identifier(cause) => Some(cause),
            ErrorKind::Url { cause, .. } => Some(cause),
            ErrorKind::Port(cause) => Some(cause),
            _ => None,
        }
    }
}

impl From<hr_id::ParseError> for ParseError {
    fn from(cause: hr_id::ParseError) -> Self {
        Self(ErrorKind::Identifier(cause))
    }
}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        Self(ErrorKind::Message(message))
    }
}

impl From<&str> for ParseError {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
