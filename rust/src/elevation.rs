//! Windows elevation and privilege management.

use crate::error::{Result, UnblockerError};

#[cfg(windows)]
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Security::*,
    Win32::System::Threading::*,
    Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*,
};

/// RAII wrapper for Windows handles to ensure proper cleanup
#[cfg(windows)]
pub struct HandleGuard(pub HANDLE);

#[cfg(windows)]
impl Drop for HandleGuard {
    fn drop(&mut self) {
        if !self.0.0.is_null() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

/// Check if the current process is running with elevated privileges
#[cfg(windows)]
pub fn is_elevated() -> Result<bool> {
    unsafe {
        let mut token = HANDLE(std::ptr::null_mut());
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
            .map_err(|e| UnblockerError::WindowsApi(format!("Failed to open process token: {:?}", e)))?;
        
        let _guard = HandleGuard(token);
        
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0u32;

        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        )
        .map_err(|e| UnblockerError::WindowsApi(format!("Failed to get token information: {:?}", e)))?;

        Ok(elevation.TokenIsElevated != 0)
    }
}

#[cfg(not(windows))]
pub fn is_elevated() -> Result<bool> {
    Ok(false)
}

/// Properly escape command line arguments to prevent injection
#[cfg(windows)]
fn escape_argument(arg: &str) -> String {
    // Escape quotes and backslashes according to Windows command line rules
    let escaped = arg.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

/// Relaunch the application with administrator privileges
#[cfg(windows)]
pub fn relaunch_as_admin() -> Result<()> {
    use std::env;
    use std::process;
    
    let current_exe = env::current_exe()
        .map_err(|e| UnblockerError::WindowsApi(format!("Failed to get current executable path: {}", e)))?;

    let args: Vec<String> = env::args().collect();
    
    // Properly escape arguments to prevent injection attacks
    let escaped_args: Vec<String> = args[1..].iter()
        .map(|arg| escape_argument(arg))
        .collect();
    let arguments = escaped_args.join(" ");

    log::info!("Relaunching with elevated privileges");
    log::debug!("Executable: {}", current_exe.display());
    log::debug!("Arguments: {}", arguments);

    unsafe {
        let exe_path: Vec<u16> = current_exe.to_string_lossy().encode_utf16().chain(Some(0)).collect();
        let params: Vec<u16> = arguments.encode_utf16().chain(Some(0)).collect();
        let operation: Vec<u16> = "runas".encode_utf16().chain(Some(0)).collect();

        let result = ShellExecuteW(
            HWND(std::ptr::null_mut()),
            PCWSTR(operation.as_ptr()),
            PCWSTR(exe_path.as_ptr()),
            PCWSTR(params.as_ptr()),
            PCWSTR::null(),
            SHOW_WINDOW_CMD(1), // SW_NORMAL
        );

        if result.0 as isize <= 32 {
            return Err(UnblockerError::ElevationFailed);
        }
    }
    
    process::exit(0);
}

#[cfg(not(windows))]
pub fn relaunch_as_admin() -> Result<()> {
    Err(UnblockerError::WindowsApi("Elevation not supported on this platform".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_escape_argument() {
        assert_eq!(escape_argument("simple"), "\"simple\"");
        assert_eq!(escape_argument("with space"), "\"with space\"");
        assert_eq!(escape_argument("with\"quote"), "\"with\\\"quote\"");
        assert_eq!(escape_argument("with\\backslash"), "\"with\\\\backslash\"");
        assert_eq!(escape_argument("with\\\"both"), "\"with\\\\\\\"both\"");
    }
    
    #[test]
    fn test_escape_argument_injection_attempts() {
        // Test various injection attempts
        assert_eq!(escape_argument("\" && del *"), "\"\\\" && del *\"");
        assert_eq!(escape_argument("'; rm -rf /"), "\"'; rm -rf /\"");
        assert_eq!(escape_argument("$(malicious)"), "\"$(malicious)\"");
    }
}