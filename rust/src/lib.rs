//! SaltSpectre's File Unblocker - A Windows utility for removing Zone.Identifier alternate data streams
//! 
//! This crate provides functionality to unblock files that have been marked as downloaded
//! from the internet by Windows, by removing their Zone.Identifier alternate data stream.

pub mod config;
pub mod elevation;
pub mod error;
pub mod path_utils;
pub mod ui;
pub mod unblocker;

pub use config::Config;
pub use error::{Result, UnblockerError};
pub use unblocker::{process_target, UnblockStats};

/// Application metadata
pub const APP_NAME: &str = "SaltSpectre's File Unblocker";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");