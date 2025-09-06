use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tool", content = "parameters", rename_all = "snake_case")]
pub enum ToolCall {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    PatchFile { path: String, instructions: String },
    DeleteFile { path: String },
    ListFiles { path: String },
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
        ToolCall::PatchFile { .. } => {
            // Patching is complex and will be implemented later.
            // For now, it returns a placeholder.
            Ok(ToolResult {
                success: false,
                output: "Patching is not yet implemented.".to_string(),
            })
        }
        ToolCall::DeleteFile { path } => delete_file(&path).await,
        ToolCall::ListFiles { path } => list_files(&path).await,
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
