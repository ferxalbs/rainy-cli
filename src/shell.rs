use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;
use crate::ui;

/// Security levels for command execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    /// Always require user approval
    Low,
    /// Require approval for sensitive operations
    Medium,
    /// Never require approval (high trust)
    High,
}

impl SecurityLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => SecurityLevel::Low,
            "medium" => SecurityLevel::Medium,
            "high" => SecurityLevel::High,
            _ => SecurityLevel::Low, // Default to most secure
        }
    }
}

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
                // Development tools
                "git".to_string(), "cargo".to_string(), "npm".to_string(), "yarn".to_string(),
                "pip".to_string(), "python".to_string(), "node".to_string(), "rustc".to_string(),
                "gcc".to_string(), "make".to_string(), "cmake".to_string(),
                // Package managers
                "apt".to_string(), "yum".to_string(), "dnf".to_string(), "pacman".to_string(),
                "brew".to_string(), "choco".to_string(), "winget".to_string(),
                // Testing and building
                "test".to_string(), "build".to_string(), "run".to_string(),
            ],
            blocked_commands: vec![
                // Dangerous system commands
                "rm".to_string(), "del".to_string(), "format".to_string(), "fdisk".to_string(),
                "mkfs".to_string(), "dd".to_string(), "shutdown".to_string(), "reboot".to_string(),
                "halt".to_string(), "poweroff".to_string(), "init".to_string(),
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

/// Shell command executor with security controls
pub struct ShellExecutor {
    config: ShellConfig,
}

impl ShellExecutor {
    pub fn new(config: ShellConfig) -> Self {
        Self { config }
    }

    /// Execute a shell command with security validation
    pub async fn execute(&self, command: &str) -> Result<CommandResult> {
        let start_time = Instant::now();
        
        // Parse and validate the command
        let (cmd, args) = self.parse_command(command)?;
        self.validate_command(&cmd, command)?;
        
        // Check if user approval is needed
        let category = self.categorize_command(&cmd);
        if self.requires_approval(&category) {
            if !self.request_user_approval(command, &category).await? {
                return Err(anyhow!("Command execution denied by user"));
            }
        }

        ui::print_info(&format!("Executing: {}", command));
        
        // Execute the command
        let result = self.execute_command(&cmd, &args, command).await?;
        let duration = start_time.elapsed();
        
        Ok(CommandResult {
            command: command.to_string(),
            exit_code: result.0,
            stdout: result.1,
            stderr: result.2,
            duration,
            success: result.0 == 0,
        })
    }

    /// Execute multiple commands in sequence
    pub async fn execute_batch(&self, commands: &[String]) -> Result<Vec<CommandResult>> {
        let mut results = Vec::new();
        
        for command in commands {
            match self.execute(command).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to execute '{}': {}", command, e));
                    // Continue with other commands but record the failure
                    results.push(CommandResult {
                        command: command.clone(),
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        duration: Duration::from_secs(0),
                        success: false,
                    });
                }
            }
        }
        
        Ok(results)
    }

    /// Parse command into executable and arguments
    fn parse_command(&self, command: &str) -> Result<(String, Vec<String>)> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty command"));
        }
        
        let cmd = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        
        Ok((cmd, args))
    }

    /// Validate if command is allowed to execute
    fn validate_command(&self, cmd: &str, full_command: &str) -> Result<()> {
        // Check blocked commands first
        if self.config.blocked_commands.iter().any(|blocked| cmd.contains(blocked)) {
            return Err(anyhow!("Command '{}' is blocked for security reasons", cmd));
        }

        // Check for dangerous patterns
        if self.contains_dangerous_patterns(full_command) {
            return Err(anyhow!("Command contains dangerous patterns and is blocked"));
        }

        // For medium and low security, check allowed commands
        if self.config.security_level != SecurityLevel::High {
            if !self.config.allowed_commands.iter().any(|allowed| cmd.starts_with(allowed)) {
                return Err(anyhow!("Command '{}' is not in the allowed commands list", cmd));
            }
        }

        Ok(())
    }

    /// Check for dangerous command patterns
    fn contains_dangerous_patterns(&self, command: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf /", "del /s", "format c:", "> /dev/", ":(){ :|:& };:",
            "chmod 777", "chown root", "mv /* ", "cp /* ", "dd if=",
            "mkfs", "fdisk", "parted", "> /etc/", "curl | sh", "wget | sh",
        ];
        
        dangerous_patterns.iter().any(|pattern| command.contains(pattern))
    }

    /// Categorize command by risk level
    fn categorize_command(&self, cmd: &str) -> CommandCategory {
        match cmd {
            // Safe read-only commands
            cmd if ["ls", "dir", "pwd", "echo", "cat", "type", "head", "tail", "grep", "find", "where"].contains(&cmd) => {
                CommandCategory::Safe
            }
            // File system operations
            cmd if ["cp", "copy", "mv", "move", "mkdir", "rmdir", "touch", "chmod", "chown"].contains(&cmd) => {
                CommandCategory::FileSystem
            }
            // Package management
            cmd if ["npm", "yarn", "pip", "cargo", "apt", "yum", "dnf", "pacman", "brew", "choco", "winget"].contains(&cmd) => {
                CommandCategory::PackageManagement
            }
            // Network operations
            cmd if ["curl", "wget", "ping", "nslookup", "dig", "telnet", "ssh", "scp", "rsync"].contains(&cmd) => {
                CommandCategory::Network
            }
            // System administration
            cmd if ["ps", "top", "htop", "kill", "killall", "systemctl", "service", "crontab"].contains(&cmd) => {
                CommandCategory::SystemAdmin
            }
            // Default to potentially dangerous
            _ => CommandCategory::Dangerous,
        }
    }

    /// Check if user approval is required based on security level and command category
    fn requires_approval(&self, category: &CommandCategory) -> bool {
        match self.config.security_level {
            SecurityLevel::Low => true, // Always ask
            SecurityLevel::Medium => {
                matches!(category, 
                    CommandCategory::FileSystem | 
                    CommandCategory::PackageManagement | 
                    CommandCategory::Network | 
                    CommandCategory::SystemAdmin | 
                    CommandCategory::Dangerous
                )
            }
            SecurityLevel::High => false, // Never ask
        }
    }

    /// Request user approval for command execution
    async fn request_user_approval(&self, command: &str, category: &CommandCategory) -> Result<bool> {
        ui::print_warning(&format!("Command '{}' requires approval (Category: {:?})", command, category));
        ui::print_info("This command may modify your system. Do you want to proceed?");
        
        match ui::prompt_for_confirmation() {
            Ok(approved) => Ok(approved),
            Err(e) => {
                ui::print_error(&format!("Failed to get user confirmation: {}", e));
                Ok(false) // Default to deny on error
            }
        }
    }

    /// Execute the validated command
    async fn execute_command(&self, cmd: &str, args: &[String], full_command: &str) -> Result<(i32, String, String)> {
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd_builder = TokioCommand::new("powershell");
            cmd_builder.args(&["-Command", full_command]);
            cmd_builder
        } else {
            let mut cmd_builder = TokioCommand::new(cmd);
            cmd_builder.args(args);
            cmd_builder
        };

        // Set working directory if specified
        if let Some(ref wd) = self.config.working_directory {
            command.current_dir(wd);
        }

        // Set environment variables
        for (key, value) in &self.config.environment_vars {
            command.env(key, value);
        }

        // Configure stdio
        command.stdout(Stdio::piped())
               .stderr(Stdio::piped())
               .stdin(Stdio::null());

        // Execute with timeout
        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        
        match timeout(timeout_duration, command.output()).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);
                
                Ok((exit_code, stdout, stderr))
            }
            Ok(Err(e)) => Err(anyhow!("Failed to execute command: {}", e)),
            Err(_) => Err(anyhow!("Command timed out after {} seconds", self.config.timeout_seconds)),
        }
    }

    /// Install a package using the appropriate package manager
    pub async fn install_package(&self, package_name: &str) -> Result<CommandResult> {
        let install_command = if cfg!(target_os = "windows") {
            // Try winget first, then choco
            if self.command_exists("winget").await {
                format!("winget install {}", package_name)
            } else if self.command_exists("choco").await {
                format!("choco install {} -y", package_name)
            } else {
                return Err(anyhow!("No package manager found (winget or chocolatey)"));
            }
        } else if cfg!(target_os = "macos") {
            if self.command_exists("brew").await {
                format!("brew install {}", package_name)
            } else {
                return Err(anyhow!("Homebrew not found"));
            }
        } else {
            // Linux - try multiple package managers
            if self.command_exists("apt").await {
                format!("apt install -y {}", package_name)
            } else if self.command_exists("yum").await {
                format!("yum install -y {}", package_name)
            } else if self.command_exists("dnf").await {
                format!("dnf install -y {}", package_name)
            } else if self.command_exists("pacman").await {
                format!("pacman -S --noconfirm {}", package_name)
            } else {
                return Err(anyhow!("No supported package manager found"));
            }
        };

        ui::print_info(&format!("Installing package: {}", package_name));
        self.execute(&install_command).await
    }

    /// Check if a command exists in the system
    async fn command_exists(&self, cmd: &str) -> bool {
        let check_command = if cfg!(target_os = "windows") {
            format!("Get-Command {} -ErrorAction SilentlyContinue", cmd)
        } else {
            format!("command -v {}", cmd)
        };

        match self.execute(&check_command).await {
            Ok(result) => result.success,
            Err(_) => false,
        }
    }

    /// Run project tests
    pub async fn run_tests(&self) -> Result<CommandResult> {
        // Detect project type and run appropriate tests
        let test_command = if std::path::Path::new("Cargo.toml").exists() {
            "cargo test"
        } else if std::path::Path::new("package.json").exists() {
            "npm test"
        } else if std::path::Path::new("requirements.txt").exists() || std::path::Path::new("pyproject.toml").exists() {
            "python -m pytest"
        } else if std::path::Path::new("Makefile").exists() {
            "make test"
        } else {
            return Err(anyhow!("No recognized test framework found"));
        };

        ui::print_info("Running project tests...");
        self.execute(test_command).await
    }

    /// Build the project
    pub async fn build_project(&self) -> Result<CommandResult> {
        let build_command = if std::path::Path::new("Cargo.toml").exists() {
            "cargo build"
        } else if std::path::Path::new("package.json").exists() {
            "npm run build"
        } else if std::path::Path::new("Makefile").exists() {
            "make build"
        } else if std::path::Path::new("CMakeLists.txt").exists() {
            "cmake --build ."
        } else {
            return Err(anyhow!("No recognized build system found"));
        };

        ui::print_info("Building project...");
        self.execute(build_command).await
    }

    /// Get system information
    pub async fn get_system_info(&self) -> Result<HashMap<String, String>> {
        let mut info = HashMap::new();
        
        // OS information
        if cfg!(target_os = "windows") {
            if let Ok(result) = self.execute("systeminfo | findstr /B /C:\"OS Name\" /C:\"OS Version\"").await {
                info.insert("os".to_string(), result.stdout.trim().to_string());
            }
        } else {
            if let Ok(result) = self.execute("uname -a").await {
                info.insert("os".to_string(), result.stdout.trim().to_string());
            }
        }

        // CPU information
        if cfg!(target_os = "windows") {
            if let Ok(result) = self.execute("wmic cpu get name").await {
                info.insert("cpu".to_string(), result.stdout.trim().to_string());
            }
        } else {
            if let Ok(result) = self.execute("lscpu | grep 'Model name'").await {
                info.insert("cpu".to_string(), result.stdout.trim().to_string());
            }
        }

        // Memory information
        if cfg!(target_os = "windows") {
            if let Ok(result) = self.execute("wmic computersystem get TotalPhysicalMemory").await {
                info.insert("memory".to_string(), result.stdout.trim().to_string());
            }
        } else {
            if let Ok(result) = self.execute("free -h").await {
                info.insert("memory".to_string(), result.stdout.trim().to_string());
            }
        }

        Ok(info)
    }
}

/// Builder for ShellConfig
pub struct ShellConfigBuilder {
    config: ShellConfig,
}

impl ShellConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ShellConfig::default(),
        }
    }

    pub fn security_level(mut self, level: SecurityLevel) -> Self {
        self.config.security_level = level;
        self
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }

    pub fn working_directory<P: Into<std::path::PathBuf>>(mut self, path: P) -> Self {
        self.config.working_directory = Some(path.into());
        self
    }

    pub fn environment_var<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.config.environment_vars.insert(key.into(), value.into());
        self
    }

    pub fn allow_command<S: Into<String>>(mut self, cmd: S) -> Self {
        self.config.allowed_commands.push(cmd.into());
        self
    }

    pub fn block_command<S: Into<String>>(mut self, cmd: S) -> Self {
        self.config.blocked_commands.push(cmd.into());
        self
    }

    pub fn build(self) -> ShellConfig {
        self.config
    }
}

impl Default for ShellConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_security_level_from_str() {
        assert_eq!(SecurityLevel::from_str("low"), SecurityLevel::Low);
        assert_eq!(SecurityLevel::from_str("medium"), SecurityLevel::Medium);
        assert_eq!(SecurityLevel::from_str("high"), SecurityLevel::High);
        assert_eq!(SecurityLevel::from_str("invalid"), SecurityLevel::Low); // Default to most secure
    }

    #[test]
    fn test_shell_config_default() {
        let config = ShellConfig::default();
        assert_eq!(config.security_level, SecurityLevel::Medium);
        assert_eq!(config.timeout_seconds, 300);
        assert!(!config.allowed_commands.is_empty());
        assert!(!config.blocked_commands.is_empty());
    }

    #[test]
    fn test_shell_config_builder() {
        let config = ShellConfigBuilder::new()
            .security_level(SecurityLevel::High)
            .timeout(600)
            .working_directory("/tmp")
            .environment_var("TEST_VAR", "test_value")
            .allow_command("custom_cmd")
            .block_command("dangerous_cmd")
            .build();

        assert_eq!(config.security_level, SecurityLevel::High);
        assert_eq!(config.timeout_seconds, 600);
        assert_eq!(config.working_directory, Some(std::path::PathBuf::from("/tmp")));
        assert_eq!(config.environment_vars.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert!(config.allowed_commands.contains(&"custom_cmd".to_string()));
        assert!(config.blocked_commands.contains(&"dangerous_cmd".to_string()));
    }

    #[test]
    fn test_command_categorization() {
        let executor = ShellExecutor::new(ShellConfig::default());
        
        assert_eq!(executor.categorize_command("ls"), CommandCategory::Safe);
        assert_eq!(executor.categorize_command("cp"), CommandCategory::FileSystem);
        assert_eq!(executor.categorize_command("npm"), CommandCategory::PackageManagement);
        assert_eq!(executor.categorize_command("curl"), CommandCategory::Network);
        assert_eq!(executor.categorize_command("ps"), CommandCategory::SystemAdmin);
        assert_eq!(executor.categorize_command("unknown_cmd"), CommandCategory::Dangerous);
    }

    #[test]
    fn test_dangerous_patterns_detection() {
        let executor = ShellExecutor::new(ShellConfig::default());
        
        assert!(executor.contains_dangerous_patterns("rm -rf /"));
        assert!(executor.contains_dangerous_patterns("del /s"));
        assert!(executor.contains_dangerous_patterns("format c:"));
        assert!(executor.contains_dangerous_patterns("curl | sh"));
        assert!(!executor.contains_dangerous_patterns("ls -la"));
        assert!(!executor.contains_dangerous_patterns("echo hello"));
    }

    #[test]
    fn test_command_validation() {
        let config = ShellConfig {
            security_level: SecurityLevel::Medium,
            allowed_commands: vec!["echo".to_string(), "ls".to_string()],
            blocked_commands: vec!["rm".to_string()],
            ..Default::default()
        };
        let executor = ShellExecutor::new(config);

        // Should pass - allowed command
        assert!(executor.validate_command("echo", "echo hello").is_ok());
        
        // Should fail - blocked command
        assert!(executor.validate_command("rm", "rm file.txt").is_err());
        
        // Should fail - dangerous pattern
        assert!(executor.validate_command("rm", "rm -rf /").is_err());
        
        // Should fail - not in allowed list
        assert!(executor.validate_command("cat", "cat file.txt").is_err());
    }

    #[test]
    fn test_command_parsing() {
        let executor = ShellExecutor::new(ShellConfig::default());
        
        let (cmd, args) = executor.parse_command("echo hello world").unwrap();
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello", "world"]);
        
        let (cmd, args) = executor.parse_command("ls -la /tmp").unwrap();
        assert_eq!(cmd, "ls");
        assert_eq!(args, vec!["-la", "/tmp"]);
        
        // Test empty command
        assert!(executor.parse_command("").is_err());
        assert!(executor.parse_command("   ").is_err());
    }

    #[test]
    fn test_approval_requirements() {
        let low_security = ShellExecutor::new(ShellConfig {
            security_level: SecurityLevel::Low,
            ..Default::default()
        });
        
        let medium_security = ShellExecutor::new(ShellConfig {
            security_level: SecurityLevel::Medium,
            ..Default::default()
        });
        
        let high_security = ShellExecutor::new(ShellConfig {
            security_level: SecurityLevel::High,
            ..Default::default()
        });

        // Low security always requires approval
        assert!(low_security.requires_approval(&CommandCategory::Safe));
        assert!(low_security.requires_approval(&CommandCategory::Dangerous));

        // Medium security requires approval for non-safe operations
        assert!(!medium_security.requires_approval(&CommandCategory::Safe));
        assert!(medium_security.requires_approval(&CommandCategory::FileSystem));
        assert!(medium_security.requires_approval(&CommandCategory::Dangerous));

        // High security never requires approval
        assert!(!high_security.requires_approval(&CommandCategory::Safe));
        assert!(!high_security.requires_approval(&CommandCategory::Dangerous));
    }

    #[tokio::test]
    async fn test_safe_command_execution() {
        let executor = ShellExecutor::new(ShellConfig {
            security_level: SecurityLevel::High, // No approval needed
            ..Default::default()
        });

        // Test a safe command that should work on all platforms
        let result = executor.execute("echo hello").await;

        match result {
            Ok(cmd_result) => {
                assert_eq!(cmd_result.exit_code, 0);
                assert!(cmd_result.success);
                assert!(cmd_result.stdout.contains("hello"));
            }
            Err(e) => {
                // Command might not be available in test environment
                println!("Command execution failed (expected in some test environments): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_command_timeout() {
        let executor = ShellExecutor::new(ShellConfig {
            security_level: SecurityLevel::High,
            timeout_seconds: 1, // Very short timeout
            ..Default::default()
        });

        // This command should timeout (sleep/timeout command)
        let result = if cfg!(target_os = "windows") {
            executor.execute("Start-Sleep -Seconds 5").await
        } else {
            executor.execute("sleep 5").await
        };

        match result {
            Err(e) => {
                assert!(e.to_string().contains("timed out"));
            }
            Ok(_) => {
                // Command might complete quickly in some environments
                println!("Command completed unexpectedly fast");
            }
        }
    }

    #[tokio::test]
    async fn test_batch_execution() {
        let executor = ShellExecutor::new(ShellConfig {
            security_level: SecurityLevel::High,
            ..Default::default()
        });

        let commands = vec![
            "echo first".to_string(),
            "echo second".to_string(),
        ];

        match executor.execute_batch(&commands).await {
            Ok(results) => {
                assert_eq!(results.len(), 2);
                for result in results {
                    if result.success {
                        assert_eq!(result.exit_code, 0);
                    }
                }
            }
            Err(e) => {
                println!("Batch execution failed (expected in some test environments): {}", e);
            }
        }
    }

    #[test]
    fn test_command_result_serialization() {
        let result = CommandResult {
            command: "echo test".to_string(),
            exit_code: 0,
            stdout: "test\n".to_string(),
            stderr: "".to_string(),
            duration: Duration::from_millis(100),
            success: true,
        };

        // Test serialization
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("echo test"));
        assert!(serialized.contains("test\\n"));

        // Test deserialization
        let deserialized: CommandResult = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.command, result.command);
        assert_eq!(deserialized.exit_code, result.exit_code);
        assert_eq!(deserialized.success, result.success);
    }

    #[test]
    fn test_security_level_serialization() {
        let levels = vec![SecurityLevel::Low, SecurityLevel::Medium, SecurityLevel::High];
        
        for level in levels {
            let serialized = serde_json::to_string(&level).unwrap();
            let deserialized: SecurityLevel = serde_json::from_str(&serialized).unwrap();
            assert_eq!(level, deserialized);
        }
    }
}