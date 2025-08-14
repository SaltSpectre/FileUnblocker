//! Configuration management for the file unblocker utility.

use crate::error::{Result, UnblockerError};
use std::path::Path;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Enable verbose output to console
    pub verbose: bool,
    /// Optional path to log file
    pub log_path: Option<String>,
    /// Target file or directory path to process
    pub target_path: String,
    /// Whether elevation is required (set during runtime)
    pub requires_elevation: bool,
}

impl Config {
    /// Create a new configuration with validation
    pub fn new(
        verbose: bool,
        log_path: Option<String>,
        target_path: String,
    ) -> Result<Self> {
        let config = Self {
            verbose,
            log_path,
            target_path,
            requires_elevation: false,
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate target path exists
        if !Path::new(&self.target_path).exists() {
            return Err(UnblockerError::PathNotFound(self.target_path.clone()));
        }
        
        // Validate log directory exists if log path is specified
        if let Some(log_path) = &self.log_path {
            if let Some(parent) = Path::new(log_path).parent() {
                if !parent.exists() {
                    return Err(UnblockerError::Config(format!(
                        "Log directory does not exist: {}",
                        parent.display()
                    )));
                }
            }
        }
        
        Ok(())
    }
    
    /// Mark that elevation is required
    pub fn set_requires_elevation(&mut self) {
        self.requires_elevation = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_validation_valid() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().to_string_lossy().to_string();
        
        let config = Config::new(false, None, target_path);
        assert!(config.is_ok());
    }
    
    #[test]
    fn test_config_validation_invalid_target() {
        let config = Config::new(false, None, "/nonexistent/path".to_string());
        assert!(matches!(config, Err(UnblockerError::PathNotFound(_))));
    }
    
    #[test]
    fn test_config_validation_invalid_log_dir() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().to_string_lossy().to_string();
        let log_path = "/nonexistent/dir/log.txt".to_string();
        
        let config = Config::new(false, Some(log_path), target_path);
        assert!(matches!(config, Err(UnblockerError::Config(_))));
    }
}