//! Main entry point for SaltSpectre's File Unblocker

#![windows_subsystem = "windows"]

#[cfg(not(windows))]
compile_error!("This application is designed for Windows only. Use cross-compilation targets like x86_64-pc-windows-msvc or aarch64-pc-windows-msvc");

use clap::{Arg, Command};
use std::process;

use unblocker::{
    config::Config,
    elevation::{is_elevated, relaunch_as_admin},
    error::{Result, UnblockerError},
    ui::{log_message, show_error},
    unblocker::process_target,
    APP_NAME, APP_VERSION, APP_DESCRIPTION,
};

#[cfg(windows)]
use windows::Win32::System::Console::{AttachConsole, AllocConsole, ATTACH_PARENT_PROCESS};

/// Attach to parent console or allocate new one if needed
#[cfg(windows)]
fn ensure_console() {
    unsafe {
        // Try to attach to parent process console (if launched from cmd/powershell)
        if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
            // If no parent console, try to allocate a new one
            let _ = AllocConsole();
        }
    }
}

#[cfg(not(windows))]
fn ensure_console() {
    // No-op on non-Windows platforms
}

fn main() {
    // Check if --verbose flag is present before parsing full arguments
    let needs_console = std::env::args().any(|arg| arg == "--verbose" || arg == "-v");

    if needs_console {
        ensure_console();
    }

    env_logger::init();
    
    if let Err(e) = run() {
        let error_msg = e.user_message();
        eprintln!("Error: {}", error_msg);
        
        // Try to create a minimal config for error display
        let config = Config::new(true, None, ".".to_string()).unwrap_or_else(|_| {
            // Fallback config that should always work
            Config {
                verbose: true,
                log_path: None,
                target_path: ".".to_string(),
                requires_elevation: false,
            }
        });
        
        show_error(&error_msg, &config);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = Command::new(APP_NAME)
        .version(APP_VERSION)
        .author("SaltSpectre")
        .about(APP_DESCRIPTION)
        .arg(
            Arg::new("path")
                .help("File or directory path to unblock")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("log")
                .long("log")
                .help("Log file path")
                .value_name("FILE")
                .num_args(1),
        )
        .get_matches();

    let target_path = matches.get_one::<String>("path")
        .ok_or_else(|| UnblockerError::Config("Path argument is required".to_string()))?
        .clone();
    
    let mut config = Config::new(
        matches.get_flag("verbose"),
        matches.get_one::<String>("log").cloned(),
        target_path,
    )?;

    let stats = process_target(&config.target_path.clone(), &mut config)?;
    
    log_message(&format!("Operation completed. {}", stats.summary()), &config)?;

    if config.requires_elevation && !is_elevated()? {
        log_message("Some files could not be unblocked due to permission issues. Retrying with admin privileges...", &config)?;
        relaunch_as_admin()?;
    }
    
    Ok(())
}

