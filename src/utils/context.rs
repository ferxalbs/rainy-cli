use crate::utils::git;
use anyhow::Result;

pub fn load_project_context() -> Result<String> {
    let mut context = String::new();
    let current_dir = std::env::current_dir()?;

    // Read Cargo.toml for Rust projects
    if let Ok(cargo_content) = std::fs::read_to_string("Cargo.toml") {
        context.push_str("## Project Information (Cargo.toml)\n");
        context.push_str(&cargo_content);
        context.push_str("\n\n");
    }

    // Read package.json for Node.js projects
    if let Ok(package_content) = std::fs::read_to_string("package.json") {
        context.push_str("## Project Information (package.json)\n");
        context.push_str(&package_content);
        context.push_str("\n\n");
    }

    // Read pyproject.toml for Python projects
    if let Ok(pyproject_content) = std::fs::read_to_string("pyproject.toml") {
        context.push_str("## Project Information (pyproject.toml)\n");
        context.push_str(&pyproject_content);
        context.push_str("\n\n");
    }

    // Read main source files
    let main_files = [
        "src/main.rs",
        "src/lib.rs", 
        "src/index.js",
        "src/app.js",
        "lib/index.js",
        "index.js",
        "main.py",
        "app.py",
        "__init__.py",
        "README.md",
    ];

    context.push_str("## Main Source Files\n");
    for file_path in &main_files {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            if content.len() < 2000 { // Limit file size for context
                context.push_str(&format!("### {}\n```\n{}\n```\n\n", file_path, content));
            } else {
                context.push_str(&format!("### {} (truncated)\n```\n{}\n```\n\n",
                    file_path,
                    &content[..1000]));
            }
        }
    }

    // Get project structure
    context.push_str("## Project Structure\n");
    if let Ok(entries) = std::fs::read_dir(&current_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                let name = entry.file_name().to_string_lossy().to_string();
                // Skip common build/cache directories
                if matches!(name.as_str(), "target" | "node_modules" | ".git" | "__pycache__" | "dist" | "build") {
                    continue;
                }
                if file_type.is_dir() {
                    context.push_str(&format!("📁 {}\n", name));
                } else {
                    context.push_str(&format!("📄 {}\n", name));
                }
            }
        }
    }

    // Get Git status if available
    if let Ok(git_summary) = git::get_git_status_summary() {
        context.push_str("\n## Git Information\n");
        context.push_str(&git_summary);
        context.push('\n');
    }

    // Add current working directory info
    context.push_str(&format!("\n## Current Directory\n{}\n", current_dir.display()));

    if context.is_empty() {
        context = "No project context available.".to_string();
    }

    Ok(context)
}

pub fn get_file_context(file_path: &str) -> Result<String> {
    let content = std::fs::read_to_string(file_path)?;
    let mut context = String::new();
    
    context.push_str(&format!("## File: {}\n", file_path));
    
    // Basic file statistics
    let lines = content.lines().count();
    let chars = content.len();
    let words = content.split_whitespace().count();
    
    context.push_str(&format!("**Statistics:** {} lines, {} characters, {} words\n\n", lines, chars, words));
    
    // File extension analysis
    if let Some(ext) = std::path::Path::new(file_path).extension() {
        if let Some(ext_str) = ext.to_str() {
            context.push_str(&format!("**File Type:** {}\n\n", ext_str));
        }
    }
    
    context.push_str("**Content:**\n```\n");
    context.push_str(&content);
    context.push_str("\n```\n");
    
    Ok(context)
}

use std::path::PathBuf;
use walkdir::WalkDir;

pub fn collect_context_from_paths(paths: &[PathBuf]) -> Result<String> {
    let mut context = String::new();

    for path in paths {
        if path.is_file() {
            let content = std::fs::read_to_string(path)?;
            context.push_str(&format!("## File: {}\n", path.display()));
            context.push_str(&content);
            context.push_str("\n\n");
        } else if path.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        context.push_str(&format!("## File: {}\n", entry.path().display()));
                        context.push_str(&content);
                        context.push_str("\n\n");
                    }
                }
            }
        }
    }

    Ok(context)
}
