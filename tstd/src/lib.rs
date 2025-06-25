// Fix for the source field in errors/src/lib.rs
// Replace any line that looks like:
//     source: Option<String>,
// With:
//     #[source]
//     source: Option<Box<dyn std::error::Error + Send + Sync>>,

// The Io variant should look like this:
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("I/O error: {message}")]
#[diagnostic(code(t::io))]
pub enum TlError {
    Io {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

// And update the constructor method:
impl TlError {
    /// Create an I/O error.
    pub fn io(message: impl Into<String>, source: Option<std::io::Error>) -> Self {
        Self::Io {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }
}