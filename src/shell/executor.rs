use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

use super::command::{CommandCategory, CommandResult};
use super::config::ShellConfig;
use super::security::SecurityLevel;
use crate::ui;

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
    async fn execute_command(&self, _cmd: &str, _args: &[String], full_command: &str) -> Result<(i32, String, String)> {
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd_builder = TokioCommand::new("powershell");
            cmd_builder.args(&["-Command", full_command]);
            cmd_builder
        } else {
            let mut cmd_builder = TokioCommand::new("sh");
            cmd_builder.arg("-c").arg(full_command);
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
    pub async fn command_exists(&self, cmd: &str) -> bool {
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

    // High-level file management functions
    pub async fn read_file(&self, path: &str) -> Result<String> {
        let command = format!("cat {}", path);
        let result = self.execute(&command).await?;
        if result.success {
            Ok(result.stdout)
        } else {
            Err(anyhow!("Failed to read file: {}", result.stderr))
        }
    }

    pub async fn create_file(&self, path: &str, content: &str) -> Result<CommandResult> {
        let command = format!("echo '{}' > {}", content, path);
        self.execute(&command).await
    }

    pub async fn delete_file(&self, path: &str) -> Result<CommandResult> {
        let command = if cfg!(target_os = "windows") {
            format!("del {}", path)
        } else {
            format!("rm {}", path)
        };
        self.execute(&command).await
    }

    // High-level Git integration functions
    pub async fn git_clone(&self, repo_url: &str, path: &str) -> Result<CommandResult> {
        let command = format!("git clone {} {}", repo_url, path);
        self.execute(&command).await
    }

    pub async fn git_status(&self) -> Result<String> {
        let result = self.execute("git status").await?;
        if result.success {
            Ok(result.stdout)
        } else {
            Err(anyhow!("Failed to get git status: {}", result.stderr))
        }
    }

    pub async fn git_add(&self, files: &[&str]) -> Result<CommandResult> {
        let command = format!("git add {}", files.join(" "));
        self.execute(&command).await
    }

    pub async fn git_commit(&self, message: &str) -> Result<CommandResult> {
        let command = format!("git commit -m '{}'", message);
        self.execute(&command).await
    }

    pub async fn git_push(&self) -> Result<CommandResult> {
        self.execute("git push").await
    }
}