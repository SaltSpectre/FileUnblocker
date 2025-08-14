//! Core file unblocking functionality.

use crate::config::Config;
use crate::error::{Result, UnblockerError};
use crate::path_utils::{get_ads_path, is_safe_path, validate_path};
use crate::ui::{log_message, show_warning};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Statistics about the unblocking operation
#[derive(Debug, Default)]
pub struct UnblockStats {
    pub files_processed: usize,
    pub files_unblocked: usize,
    pub files_no_ads: usize,
    pub files_failed: usize,
    pub permission_errors: usize,
}

impl UnblockStats {
    /// Create a summary message for the statistics
    pub fn summary(&self) -> String {
        format!(
            "Processed {} files: {} unblocked, {} had no ADS, {} failed ({} permission errors)",
            self.files_processed,
            self.files_unblocked,
            self.files_no_ads,
            self.files_failed,
            self.permission_errors
        )
    }
}

/// Unblock a single file by removing its Zone.Identifier ADS
pub fn unblock_file(file_path: &str, config: &mut Config) -> Result<bool> {
    let file_path = validate_path(file_path)?;
    
    if !is_safe_path(&file_path) {
        show_warning(
            &format!("Skipping potentially dangerous system path: {}", file_path.display()),
            config
        );
        return Ok(false);
    }
    
    let ads_path = get_ads_path(&file_path)?;
    
    match fs::remove_file(&ads_path) {
        Ok(_) => {
            log_message(&format!("Unblocked: {}", file_path.display()), config)?;
            Ok(true)
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                log_message(&format!("No ADS found: {}", file_path.display()), config)?;
                Ok(false)
            }
            std::io::ErrorKind::PermissionDenied => {
                config.set_requires_elevation();
                log_message(
                    &format!("Access denied, requires elevation: {}", file_path.display()),
                    config
                )?;
                Err(UnblockerError::PermissionDenied(file_path.display().to_string()))
            }
            _ => {
                log_message(
                    &format!("Failed to unblock: {} — {}", file_path.display(), e),
                    config
                )?;
                Err(UnblockerError::Io(e))
            }
        }
    }
}

/// Unblock all files in a directory recursively
pub fn unblock_directory(dir_path: &str, config: &mut Config) -> Result<UnblockStats> {
    let dir_path = validate_path(dir_path)?;
    let mut stats = UnblockStats::default();
    
    log_message(&format!("Processing directory: {}", dir_path.display()), config)?;
    
    for entry in WalkDir::new(&dir_path) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    stats.files_processed += 1;
                    
                    let path_str = entry.path().to_string_lossy();
                    match unblock_file(&path_str, config) {
                        Ok(true) => stats.files_unblocked += 1,
                        Ok(false) => stats.files_no_ads += 1,
                        Err(UnblockerError::PermissionDenied(_)) => {
                            stats.permission_errors += 1;
                            stats.files_failed += 1;
                        }
                        Err(e) => {
                            log_message(
                                &format!("Error processing {}: {}", entry.path().display(), e),
                                config
                            )?;
                            stats.files_failed += 1;
                        }
                    }
                }
            }
            Err(e) => {
                let error_path = e.path().map(|p| p.display().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                    
                if e.io_error()
                    .map(|io_err| io_err.kind() == std::io::ErrorKind::PermissionDenied)
                    .unwrap_or(false)
                {
                    config.set_requires_elevation();
                    log_message(
                        &format!("Access denied to directory: {}", error_path),
                        config
                    )?;
                    stats.permission_errors += 1;
                } else {
                    log_message(
                        &format!("Failed to enumerate directory: {} — {}", error_path, e),
                        config
                    )?;
                }
                stats.files_failed += 1;
            }
        }
    }
    
    log_message(&stats.summary(), config)?;
    Ok(stats)
}

/// Process a target path (either file or directory)
pub fn process_target(target_path: &str, config: &mut Config) -> Result<UnblockStats> {
    let path = Path::new(target_path);
    
    if path.is_file() {
        let mut stats = UnblockStats::default();
        stats.files_processed = 1;
        
        match unblock_file(target_path, config) {
            Ok(true) => stats.files_unblocked = 1,
            Ok(false) => stats.files_no_ads = 1,
            Err(UnblockerError::PermissionDenied(_)) => {
                stats.permission_errors = 1;
                stats.files_failed = 1;
            }
            Err(e) => {
                stats.files_failed = 1;
                return Err(e);
            }
        }
        
        Ok(stats)
    } else if path.is_dir() {
        unblock_directory(target_path, config)
    } else {
        Err(UnblockerError::PathNotFound(target_path.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    
    #[cfg(windows)]
    #[test]
    fn test_unblock_file_no_ads() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        
        let mut config = Config::new(
            true,
            None,
            temp_dir.path().to_string_lossy().to_string(),
        ).unwrap();
        
        let result = unblock_file(&file_path.to_string_lossy(), &mut config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // No ADS to remove
    }
    
    #[cfg(windows)]
    #[test]
    fn test_unblock_file_with_ads() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        
        // Create ADS file
        let ads_path = format!("{}:Zone.Identifier", file_path.to_string_lossy());
        let mut ads_file = File::create(&ads_path).unwrap();
        writeln!(ads_file, "[ZoneTransfer]\nZoneId=3").unwrap();
        
        let mut config = Config::new(
            true, 
            None,
            temp_dir.path().to_string_lossy().to_string(),
        ).unwrap();
        
        let result = unblock_file(&file_path.to_string_lossy(), &mut config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true); // ADS was removed
        
        // Verify ADS is gone
        assert!(!Path::new(&ads_path).exists());
    }
    
    #[test]
    fn test_process_target_directory() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();
        
        let mut config = Config::new(
            true,
            None,
            temp_dir.path().to_string_lossy().to_string(),
        ).unwrap();
        
        let stats = process_target(&temp_dir.path().to_string_lossy(), &mut config).unwrap();
        assert_eq!(stats.files_processed, 2);
    }
    
    #[test]
    fn test_unblock_stats_summary() {
        let stats = UnblockStats {
            files_processed: 10,
            files_unblocked: 5,
            files_no_ads: 3,
            files_failed: 2,
            permission_errors: 1,
        };
        
        let summary = stats.summary();
        assert!(summary.contains("10 files"));
        assert!(summary.contains("5 unblocked"));
        assert!(summary.contains("3 had no ADS"));
        assert!(summary.contains("2 failed"));
        assert!(summary.contains("1 permission errors"));
    }
}