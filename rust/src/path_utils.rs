//! Path validation and sanitization utilities.

use crate::error::{Result, UnblockerError};
use std::path::{Path, PathBuf};

/// Validate and sanitize a file path to prevent directory traversal attacks
pub fn validate_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    
    // Check for path traversal attempts
    if path.to_string_lossy().contains("..") {
        return Err(UnblockerError::InvalidPath(
            "Path contains directory traversal sequences".to_string()
        ));
    }
    
    // Ensure path is absolute on Windows (required for ADS operations)
    #[cfg(windows)]
    if !path.is_absolute() {
        return Err(UnblockerError::InvalidPath(
            "Path must be absolute on Windows".to_string()
        ));
    }
    
    // Check for invalid characters
    let path_str = path.to_string_lossy();
    if path_str.chars().any(|c| matches!(c, '<' | '>' | '|' | '\0')) {
        return Err(UnblockerError::InvalidPath(
            "Path contains invalid characters".to_string()
        ));
    }
    
    // Check path length (Windows has limits)
    #[cfg(windows)]
    if path_str.len() > 260 {
        log::warn!("Path length exceeds 260 characters, may cause issues on older Windows versions");
    }
    
    Ok(path.to_path_buf())
}

/// Generate the ADS (Alternate Data Stream) path for Zone.Identifier
pub fn get_ads_path(file_path: &Path) -> Result<PathBuf> {
    let file_path = validate_path(&file_path.to_string_lossy())?;
    let mut ads_path = file_path.as_os_str().to_os_string();
    ads_path.push(":Zone.Identifier");
    Ok(PathBuf::from(ads_path))
}

/// Check if a path is safe to process (additional security checks)
pub fn is_safe_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Don't process system directories
    let dangerous_prefixes = [
        "C:\\Windows\\System32",
        "C:\\Windows\\SysWOW64", 
        "C:\\Program Files\\Windows",
        "\\\\?\\",  // Raw device paths
    ];
    
    for prefix in &dangerous_prefixes {
        if path_str.starts_with(prefix) {
            log::warn!("Skipping potentially dangerous system path: {}", path_str);
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_path_traversal() {
        assert!(validate_path("../../../etc/passwd").is_err());
        assert!(validate_path("..\\..\\windows\\system32").is_err());
        assert!(validate_path("normal/path/file.txt").is_ok());
    }
    
    #[test]
    fn test_validate_path_invalid_chars() {
        assert!(validate_path("file<.txt").is_err());
        assert!(validate_path("file>.txt").is_err());
        assert!(validate_path("file|.txt").is_err());
        assert!(validate_path("file\0.txt").is_err());
    }
    
    #[cfg(windows)]
    #[test]
    fn test_validate_path_absolute_required() {
        assert!(validate_path("relative/path").is_err());
        assert!(validate_path("C:\\absolute\\path").is_ok());
    }
    
    #[test]
    fn test_get_ads_path() {
        let file_path = Path::new("C:\\test\\file.txt");
        let ads_path = get_ads_path(file_path).unwrap();
        assert_eq!(ads_path.to_string_lossy(), "C:\\test\\file.txt:Zone.Identifier");
    }
    
    #[test]
    fn test_is_safe_path() {
        assert!(!is_safe_path(Path::new("C:\\Windows\\System32\\kernel32.dll")));
        assert!(!is_safe_path(Path::new("C:\\Windows\\SysWOW64\\ntdll.dll")));
        assert!(is_safe_path(Path::new("C:\\Users\\test\\file.txt")));
    }
}