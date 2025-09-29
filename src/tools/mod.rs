use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::config::Config;
use walkdir::WalkDir;
use crate::shell::{ShellExecutor, ShellConfig, SecurityLevel};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "tool", content = "parameters", rename_all = "snake_case")]
pub enum ToolCall {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    PatchFile { path: String, instructions: String },
    DeleteFile { path: String },
    ListFiles { path: String },
    Grep { pattern: String, path: Option<String> },
    ExecuteCommand { command: String, security_level: Option<String> },
    ExecuteBatch { commands: Vec<String> },
    InstallPackage { package_name: String },
    RunTests,
    BuildProject,
    GetSystemInfo,
    GitClone { repo_url: String, path: String },
    GitStatus,
    GitAdd { files: Vec<String> },
    GitCommit { message: String },
    GitPush,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

use crate::utils::diff::FileModification;

pub async fn execute_tool(
    tool_call: ToolCall,
    config: &Config,
    file_modifications: &mut Vec<FileModification>,
) -> Result<ToolResult> {
    match tool_call {
        ToolCall::ReadFile { path } => read_file(&path, config).await,
        ToolCall::WriteFile { path, content } => {
            write_file(&path, &content, config, file_modifications).await
        }
        ToolCall::PatchFile { path, instructions } => {
            patch_file(&path, &instructions, file_modifications).await
        }
        ToolCall::DeleteFile { path } => delete_file(&path, config).await,
        ToolCall::ListFiles { path } => list_files(&path).await,
        ToolCall::Grep { pattern, path } => grep_files(&pattern, path.as_deref()).await,
        ToolCall::ExecuteCommand {
            command,
            security_level,
        } => execute_shell_command(&command, security_level.as_deref(), config).await,
        ToolCall::ExecuteBatch { commands } => execute_batch(&commands, config).await,
        ToolCall::InstallPackage { package_name } => install_package(&package_name, config).await,
        ToolCall::RunTests => run_tests(config).await,
        ToolCall::BuildProject => build_project(config).await,
        ToolCall::GetSystemInfo => get_system_info(config).await,
        ToolCall::GitClone { repo_url, path } => git_clone(&repo_url, &path, config).await,
        ToolCall::GitStatus => git_status(config).await,
        ToolCall::GitAdd { files } => {
            let file_strs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
            git_add(&file_strs, config).await
        }
        ToolCall::GitCommit { message } => git_commit(&message, config).await,
        ToolCall::GitPush => git_push(config).await,
    }
}

async fn patch_file(
    path: &str,
    instructions: &str,
    file_modifications: &mut Vec<FileModification>,
) -> Result<ToolResult> {
    let original_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            return Ok(ToolResult {
                success: false,
                output: format!("Failed to read file '{}': {}", path, e),
            });
        }
    };

    let patch = match diffy::Patch::from_str(instructions) {
        Ok(patch) => patch,
        Err(e) => {
            return Ok(ToolResult {
                success: false,
                output: format!("Failed to parse patch instructions: {}", e),
            });
        }
    };

    let patched_content = match diffy::apply(&original_content, &patch) {
        Ok(content) => content,
        Err(e) => {
            return Ok(ToolResult {
                success: false,
                output: format!("Failed to apply patch: {}", e),
            });
        }
    };

    let mut lines_added = 0;
    let mut lines_removed = 0;
    for hunk in patch.hunks() {
        for line in hunk.lines() {
            match line {
                diffy::Line::Insert(_) => lines_added += 1,
                diffy::Line::Delete(_) => lines_removed += 1,
                _ => {}
            }
        }
    }
    file_modifications.push(FileModification {
        path: path.to_string(),
        lines_added,
        lines_removed,
    });

    match fs::write(path, patched_content) {
        Ok(_) => Ok(ToolResult {
            success: true,
            output: format!("Successfully patched file '{}'.", path),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to write patched file '{}': {}", path, e),
        }),
    }
}

// Git-related functions
async fn git_clone(repo_url: &str, path: &str, config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.git_clone(repo_url, path).await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!("STDOUT: {}\nSTDERR: {}", result.stdout, result.stderr),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn git_status(config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.git_status().await {
        Ok(output) => Ok(ToolResult {
            success: true,
            output,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn git_add(files: &[&str], config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.git_add(files).await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!("STDOUT: {}\nSTDERR: {}", result.stdout, result.stderr),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn git_commit(message: &str, config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.git_commit(message).await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!("STDOUT: {}\nSTDERR: {}", result.stdout, result.stderr),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn git_push(config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.git_push().await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!("STDOUT: {}\nSTDERR: {}", result.stdout, result.stderr),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn grep_files(pattern: &str, path: Option<&str>) -> Result<ToolResult> {
    let search_path = path.unwrap_or(".");
    let mut results = String::new();

    for entry in WalkDir::new(search_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains(pattern) {
                        results.push_str(&format!(
                            "{}:{}:{}\n",
                            entry.path().display(),
                            line_num + 1,
                            line.trim()
                        ));
                    }
                }
            }
        }
    }

    if results.is_empty() {
        Ok(ToolResult {
            success: true,
            output: "No matches found.".to_string(),
        })
    } else {
        Ok(ToolResult {
            success: true,
            output: results,
        })
    }
}

async fn read_file(path: &str, config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.read_file(path).await {
        Ok(content) => Ok(ToolResult {
            success: true,
            output: content,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn write_file(
    path: &str,
    content: &str,
    config: &Config,
    file_modifications: &mut Vec<FileModification>,
) -> Result<ToolResult> {
    let original_content = fs::read_to_string(path).unwrap_or_default();
    let patch = diffy::create_patch(&original_content, content);
    let mut lines_added = 0;
    let mut lines_removed = 0;
    for hunk in patch.hunks() {
        for line in hunk.lines() {
            match line {
                diffy::Line::Insert(_) => lines_added += 1,
                diffy::Line::Delete(_) => lines_removed += 1,
                _ => {}
            }
        }
    }
    file_modifications.push(FileModification {
        path: path.to_string(),
        lines_added,
        lines_removed,
    });

    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.create_file(path, content).await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!("STDOUT: {}\nSTDERR: {}", result.stdout, result.stderr),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn delete_file(path: &str, config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.delete_file(path).await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!("STDOUT: {}\nSTDERR: {}", result.stdout, result.stderr),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn list_files(path: &str) -> Result<ToolResult> {
    let path = Path::new(path);
    if !path.is_dir() {
        return Ok(ToolResult {
            success: false,
            output: format!("'{}' is not a directory.", path.display()),
        });
    }

    match fs::read_dir(path) {
        Ok(entries) => {
            let mut file_list = String::new();
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_type = if entry.path().is_dir() { "[D]" } else { "[F]" };
                file_list.push_str(&format!("{} {}\n", file_type, file_name));
            }
            Ok(ToolResult {
                success: true,
                output: file_list,
            })
        }
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to list files in '{}': {}", path.display(), e),
        }),
    }
}

// Shell command execution functions
async fn execute_batch(commands: &[String], config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };
    let executor = ShellExecutor::new(shell_config);

    match executor.execute_batch(commands).await {
        Ok(results) => {
            let output = results
                .into_iter()
                .map(|r| {
                    format!(
                        "Command: {}\nSuccess: {}\nSTDOUT: {}\nSTDERR: {}",
                        r.command, r.success, r.stdout, r.stderr
                    )
                })
                .collect::<Vec<String>>()
                .join("\n---\n");
            Ok(ToolResult {
                success: true,
                output,
            })
        }
        Err(e) => Ok(ToolResult {
            success: false,
            output: e.to_string(),
        }),
    }
}

async fn execute_shell_command(
    command: &str,
    security_level: Option<&str>,
    config: &Config,
) -> Result<ToolResult> {
    let security_level = security_level
        .map(SecurityLevel::from_str)
        .unwrap_or_else(|| SecurityLevel::from_str(&config.security_level));

    let shell_config = ShellConfig {
        security_level,
        ..Default::default()
    };

    let executor = ShellExecutor::new(shell_config);

    match executor.execute(command).await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!(
                "Command: {}\nExit Code: {}\nDuration: {:?}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                result.command,
                result.exit_code,
                result.duration,
                result.stdout,
                result.stderr
            ),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to execute command '{}': {}", command, e),
        }),
    }
}

async fn install_package(package_name: &str, config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };

    let executor = ShellExecutor::new(shell_config);

    match executor.install_package(package_name).await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!(
                "Package Installation: {}\nExit Code: {}\nDuration: {:?}\n\nOutput:\n{}{}",
                package_name,
                result.exit_code,
                result.duration,
                result.stdout,
                if !result.stderr.is_empty() {
                    format!("\n\nErrors:\n{}", result.stderr)
                } else {
                    String::new()
                }
            ),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to install package '{}': {}", package_name, e),
        }),
    }
}

async fn run_tests(config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };

    let executor = ShellExecutor::new(shell_config);

    match executor.run_tests().await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!(
                "Test Execution\nExit Code: {}\nDuration: {:?}\n\nOutput:\n{}{}",
                result.exit_code,
                result.duration,
                result.stdout,
                if !result.stderr.is_empty() {
                    format!("\n\nErrors:\n{}", result.stderr)
                } else {
                    String::new()
                }
            ),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to run tests: {}", e),
        }),
    }
}

async fn build_project(config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };

    let executor = ShellExecutor::new(shell_config);

    match executor.build_project().await {
        Ok(result) => Ok(ToolResult {
            success: result.success,
            output: format!(
                "Build Execution\nExit Code: {}\nDuration: {:?}\n\nOutput:\n{}{}",
                result.exit_code,
                result.duration,
                result.stdout,
                if !result.stderr.is_empty() {
                    format!("\n\nErrors:\n{}", result.stderr)
                } else {
                    String::new()
                }
            ),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to build project: {}", e),
        }),
    }
}

async fn get_system_info(config: &Config) -> Result<ToolResult> {
    let shell_config = ShellConfig {
        security_level: SecurityLevel::from_str(&config.security_level),
        ..Default::default()
    };

    let executor = ShellExecutor::new(shell_config);

    match executor.get_system_info().await {
        Ok(info) => {
            let mut output = String::from("System Information:\n");
            for (key, value) in info {
                output.push_str(&format!("{}: {}\n", key, value));
            }
            Ok(ToolResult {
                success: true,
                output,
            })
        }
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to get system information: {}", e),
        }),
    }
}
