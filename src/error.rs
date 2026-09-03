/// This type represents all possible errors that can occur when parsing frontmatter
#[derive(Debug)]
pub enum Error {
    /// Failure to read or write bytes on an I/O stream
    Io(std::io::Error),
    /// Unclosed YAML fence `---`
    UnclosedFence,
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
    /// Input that doesn't conform to the expected `[context, book]` structure mdBook sends
    MalformedInput(String),
}

/// Alias for a `Result` with the error type `mdbook_frontmatter_strip::Error`.
pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::UnclosedFence => write!(f, "unclosed fence"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::MalformedInput(msg) => write!(f, "malformed input: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::UnclosedFence | Self::MalformedInput(_) => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
