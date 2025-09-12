use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use diffy;
use std::path::Path;

use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "tool", content = "parameters", rename_all = "snake_case")]
pub enum ToolCall {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    PatchFile { path: String, instructions: String },
    DeleteFile { path: String },
    ListFiles { path: String },
    Grep { pattern: String, path: Option<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

pub async fn execute_tool(tool_call: ToolCall) -> Result<ToolResult> {
    match tool_call {
        ToolCall::ReadFile { path } => read_file(&path).await,
        ToolCall::WriteFile { path, content } => write_file(&path, &content).await,
        ToolCall::PatchFile { path, instructions } => patch_file(&path, &instructions).await,
        ToolCall::DeleteFile { path } => delete_file(&path).await,
        ToolCall::ListFiles { path } => list_files(&path).await,
        ToolCall::Grep { pattern, path } => grep_files(&pattern, path.as_deref()).await,
    }
}

async fn patch_file(path: &str, instructions: &str) -> Result<ToolResult> {
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

async fn read_file(path: &str) -> Result<ToolResult> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(ToolResult {
            success: true,
            output: content,
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to read file '{}': {}", path, e),
        }),
    }
}

async fn write_file(path: &str, content: &str) -> Result<ToolResult> {
    match fs::write(path, content) {
        Ok(_) => Ok(ToolResult {
            success: true,
            output: format!("Successfully wrote to file '{}'.", path),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to write to file '{}': {}", path, e),
        }),
    }
}

async fn delete_file(path: &str) -> Result<ToolResult> {
    match fs::remove_file(path) {
        Ok(_) => Ok(ToolResult {
            success: true,
            output: format!("Successfully deleted file '{}'.", path),
        }),
        Err(e) => Ok(ToolResult {
            success: false,
            output: format!("Failed to delete file '{}': {}", path, e),
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
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let file_type = if entry.path().is_dir() { "[D]" } else { "[F]" };
                    file_list.push_str(&format!("{} {}\n", file_type, file_name));
                }
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
