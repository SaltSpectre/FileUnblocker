//! Error handling for the file unblocker utility.

use thiserror::Error;

/// Custom error type for the unblocker application
#[derive(Error, Debug)]
pub enum UnblockerError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Path not found: {0}")]
    PathNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Elevation required but failed")]
    ElevationFailed,
    
    #[error("Windows API error: {0}")]
    WindowsApi(String),
    
    #[error("Walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, UnblockerError>;

impl UnblockerError {
    /// Check if this error indicates elevation might help
    pub fn requires_elevation(&self) -> bool {
        matches!(self, UnblockerError::PermissionDenied(_))
    }
    
    /// Convert to a user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            UnblockerError::InvalidPath(path) => {
                format!("Invalid or unsafe path: {}", path)
            }
            UnblockerError::PathNotFound(path) => {
                format!("Path not found: {}", path)
            }
            UnblockerError::PermissionDenied(details) => {
                format!("Access denied. {}", details)
            }
            UnblockerError::Io(e) => {
                format!("File operation failed: {}", e)
            }
            UnblockerError::Config(msg) => {
                format!("Configuration error: {}", msg)
            }
            UnblockerError::ElevationFailed => {
                "Failed to restart with administrator privileges".to_string()
            }
            UnblockerError::WindowsApi(msg) => {
                format!("Windows system error: {}", msg)
            }
            UnblockerError::WalkDir(e) => {
                format!("Directory traversal error: {}", e)
            }
        }
    }
}