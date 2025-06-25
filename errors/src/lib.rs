// Fix for the source field in errors/src/lib.rs
// Replace any line that looks like:
//     source: Option<String>,
// With:
//     #[source]
//     source: Option<Box<dyn std::error::Error + Send + Sync>>,

// The Io variant should look like this:


// And update the constructor method:
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TlError {
    #[error("I/O error: {message}")]
    #[diagnostic(code(t::io))]
    Io {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl TlError {
    /// Create an I/O error.
    #[track_caller]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            location: std::panic::Location::caller().to_string(),
        }
    }
    pub fn io(message: impl Into<String>, source: Option<std::io::Error>) -> Self {
        Self::Io {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }
    
}