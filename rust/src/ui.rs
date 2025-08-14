//! User interface utilities for message boxes and logging.

use crate::config::Config;
use crate::error::{Result, UnblockerError};
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;

#[cfg(windows)]
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
};

/// Show a Windows message box with the specified text and style
#[cfg(windows)]
pub fn show_message_box(text: &str, caption: &str, flags: MESSAGEBOX_STYLE) {
    unsafe {
        let text_wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let caption_wide: Vec<u16> = caption.encode_utf16().chain(Some(0)).collect();
        
        MessageBoxW(
            HWND(std::ptr::null_mut()),
            PCWSTR(text_wide.as_ptr()),
            PCWSTR(caption_wide.as_ptr()),
            flags,
        );
    }
}

#[cfg(not(windows))]
pub fn show_message_box(text: &str, _caption: &str, _flags: u32) {
    eprintln!("{}", text);
}

/// Log a message with proper formatting and timestamps
pub fn log_message(message: &str, config: &Config) -> Result<()> {
    // Only format timestamp and log if we're actually going to use it
    let should_log = config.verbose || config.log_path.is_some();
    if !should_log {
        return Ok(());
    }
    
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let formatted_message = format!("[{}] {}", timestamp, message);
    
    if config.verbose {
        println!("{}", formatted_message);
    }

    if let Some(log_path) = &config.log_path {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| UnblockerError::Io(e))?;
            
        writeln!(file, "{}", formatted_message)
            .map_err(|e| UnblockerError::Io(e))?;
    }
    
    Ok(())
}

/// Display an error message to the user via appropriate channel
pub fn show_error(message: &str, config: &Config) {
    if config.verbose {
        eprintln!("ERROR: {}", message);
    } else if config.log_path.is_none() {
        #[cfg(windows)]
        show_message_box(message, "SaltSpectre's File Unblocker", MB_OK | MB_ICONERROR);
        #[cfg(not(windows))]
        show_message_box(message, "SaltSpectre's File Unblocker", 0);
    }
    
    // Always try to log errors if possible
    let _ = log_message(&format!("ERROR: {}", message), config);
}

/// Display a warning message to the user
pub fn show_warning(message: &str, config: &Config) {
    if config.verbose {
        println!("WARNING: {}", message);
    } else if config.log_path.is_none() {
        #[cfg(windows)]
        show_message_box(message, "SaltSpectre's File Unblocker", MB_OK | MB_ICONWARNING);
        #[cfg(not(windows))]
        show_message_box(message, "SaltSpectre's File Unblocker", 0);
    }
    
    // Always try to log warnings if possible
    let _ = log_message(&format!("WARNING: {}", message), config);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::fs;
    
    #[test]
    fn test_log_message_to_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config::new(
            false,
            Some(temp_file.path().to_string_lossy().to_string()),
            temp_file.path().to_string_lossy().to_string(),
        ).unwrap();
        
        log_message("Test message", &config).unwrap();
        
        let contents = fs::read_to_string(temp_file.path()).unwrap();
        assert!(contents.contains("Test message"));
        assert!(contents.contains("UTC"));
    }
    
    #[test]
    fn test_log_message_verbose() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = Config::new(
            true,
            None,
            temp_dir.path().to_string_lossy().to_string(),
        ).unwrap();
        
        // Should not error even without log file when verbose is true
        assert!(log_message("Test message", &config).is_ok());
    }
    
    #[test] 
    fn test_log_message_no_output() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = Config::new(
            false,
            None,
            temp_dir.path().to_string_lossy().to_string(),
        ).unwrap();
        
        // Should not do anything when neither verbose nor log file is set
        assert!(log_message("Test message", &config).is_ok());
    }
}