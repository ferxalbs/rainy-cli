use super::security::SecurityLevel;
use std::collections::HashMap;

/// Configuration for shell command execution
#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub security_level: SecurityLevel,
    pub timeout_seconds: u64,
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
    pub working_directory: Option<std::path::PathBuf>,
    pub environment_vars: HashMap<String, String>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Medium,
            timeout_seconds: 300, // 5 minutes default timeout
            allowed_commands: vec![
                // Safe commands
                "ls".to_string(), "dir".to_string(), "pwd".to_string(), "echo".to_string(),
                "cat".to_string(), "type".to_string(), "head".to_string(), "tail".to_string(),
                "grep".to_string(), "find".to_string(), "where".to_string(),
                // File management
                "cp".to_string(), "copy".to_string(), "mv".to_string(), "move".to_string(),
                "mkdir".to_string(), "rmdir".to_string(), "rm".to_string(), "del".to_string(),
                "touch".to_string(),
                // Development tools
                "git".to_string(), "cargo".to_string(), "npm".to_string(), "yarn".to_string(),
                "pip".to_string(), "python".to_string(), "node".to_string(), "rustc".to_string(),
                "gcc".to_string(), "make".to_string(), "cmake".to_string(), "docker".to_string(),
                "docker-compose".to_string(), "kubectl".to_string(), "terraform".to_string(),
                "ansible".to_string(), "dotnet".to_string(),
                // Package managers
                "apt".to_string(), "yum".to_string(), "dnf".to_string(), "pacman".to_string(),
                "brew".to_string(), "choco".to_string(), "winget".to_string(),
                // Testing and building
                "test".to_string(), "build".to_string(), "run".to_string(),
            ],
            blocked_commands: vec![
                // Dangerous system commands
                "format".to_string(), "fdisk".to_string(), "mkfs".to_string(), "dd".to_string(),
                "shutdown".to_string(), "reboot".to_string(), "halt".to_string(),
                "poweroff".to_string(), "init".to_string(),
                // User management
                "useradd".to_string(), "userdel".to_string(), "usermod".to_string(),
                "passwd".to_string(), "su".to_string(), "sudo".to_string(),
                // Network security
                "iptables".to_string(), "netsh".to_string(), "route".to_string(),
                // Service management
                "systemctl".to_string(), "service".to_string(), "sc".to_string(),
            ],
            working_directory: None,
            environment_vars: HashMap::new(),
        }
    }
}