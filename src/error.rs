//! Error types for the omareel component.

/// Errors in timeline/render planning.
#[derive(Debug, thiserror::Error)]
pub enum OmareelError {
    /// Clip overlaps or is out of bounds.
    #[error("timeline error: {0}")]
    Timeline(String),

    /// Invalid render setting.
    #[error("render error: {0}")]
    Render(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
