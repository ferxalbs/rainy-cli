use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Categories of shell commands based on risk level
#[derive(Debug, Clone, PartialEq)]
pub enum CommandCategory {
    /// Safe read-only operations
    Safe,
    /// File system modifications
    FileSystem,
    /// Package management operations
    PackageManagement,
    /// Network operations
    Network,
    /// System administration
    SystemAdmin,
    /// Potentially dangerous operations
    Dangerous,
}

/// Result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub success: bool,
}