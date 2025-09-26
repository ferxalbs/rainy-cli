use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Module for Rainy SDK types and functionality
pub mod rainy_sdk {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// Represents a role in a chat message
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum MessageRole {
        /// System message
        System,
        /// User message
        User,
        /// Assistant message
        Assistant,
    }

    /// Represents a chat message
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChatMessage {
        /// The role of the message sender
        pub role: MessageRole,
        /// The content of the message
        pub content: String,
    }

    /// Request structure for chat completion API
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChatCompletionRequest {
        /// The model to use for completion (e.g., "gpt-3.5-turbo")
        pub model: String,
        /// List of messages in the conversation
        pub messages: Vec<ChatMessage>,
        /// Optional provider override
        pub provider: Option<String>,
        /// Controls randomness in output (0.0 to 2.0)
        pub temperature: Option<f32>,
        /// Maximum number of tokens to generate
        pub max_tokens: Option<u32>,
        /// Whether to stream the response
        pub stream: Option<bool>,
        /// Sequences where the API will stop generating
        pub stop: Option<Vec<String>>,
        /// Controls diversity via nucleus sampling (0.0 to 1.0)
        pub top_p: Option<f32>,
        /// Penalizes new tokens based on presence in text (-2.0 to 2.0)
        pub presence_penalty: Option<f32>,
        /// Penalizes new tokens based on frequency (-2.0 to 2.0)
        pub frequency_penalty: Option<f32>,
        /// Number of completions to generate
        pub n: Option<u32>,
        /// Modifies likelihood of specified tokens
        pub logit_bias: Option<HashMap<String, f32>>,
        /// Whether to return log probabilities
        pub logprobs: Option<bool>,
        /// An integer between 0 and 20 specifying the number of most likely tokens to return
        pub top_logprobs: Option<u32>,
        /// Unique identifier for the user
        pub user: Option<String>,
        /// An object specifying the format that the model must output
        pub response_format: Option<serde_json::Value>,
        /// A list of tools the model may call
        pub tools: Option<Vec<serde_json::Value>>,
        /// Controls which tool is called by the model
        pub tool_choice: Option<serde_json::Value>,
    }


    // Placeholder for RainyClient - would need full implementation
    pub struct RainyClient;

    impl RainyClient {
        pub fn with_api_key(_api_key: &str) -> Result<Self, String> {
            // Placeholder implementation
            Ok(RainyClient)
        }

        pub async fn create_chat_completion(&self, _request: ChatCompletionRequest) -> Result<ChatCompletionResponse, String> {
            // Placeholder implementation
            Err("Not implemented".to_string())
        }
    }

    // Placeholder for response types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChatCompletionResponse {
        pub choices: Vec<ChatCompletionChoice>,
        pub usage: Option<Usage>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChatCompletionChoice {
        pub message: ChatMessage,
        pub finish_reason: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Usage {
        pub prompt_tokens: u32,
        pub completion_tokens: u32,
        pub total_tokens: u32,
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    pub project_overview: String,
    pub tech_stack: Vec<String>,
    pub project_structure: String,
    pub key_files: Vec<String>,
    pub build_commands: Vec<String>,
    pub run_commands: Vec<String>,
    pub test_commands: Vec<String>,
}


use std::path::Path;
use std::collections::HashSet;
use git2::Repository;

use crate::config::Config;

pub async fn analyze_project(config: &Config) -> Result<ProjectContext> {
    let mut context = ProjectContext::default();
    let current_dir = std::env::current_dir()?;

    let files = get_file_list(&current_dir)?;

    let (tech_stack, build_commands, run_commands, test_commands) = analyze_files(&files)?;
    context.tech_stack = tech_stack;
    context.build_commands = build_commands;
    context.run_commands = run_commands;
    context.test_commands = test_commands;

    context.project_structure = generate_project_structure(&current_dir, &files)?;
    context.key_files = identify_key_files(&files);

    // AI Summarization
    let api_key = config.get_api_key().unwrap_or_default();
    if !api_key.is_empty() {
        let model = config.get_model();
        let client = rainy_sdk::RainyClient::with_api_key(api_key)
            .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?;

        let readme_content = files.iter()
            .find(|f| f.eq_ignore_ascii_case("README.md"))
            .and_then(|f| std::fs::read_to_string(f).ok())
            .unwrap_or_default();

        let prompt = format!(
            r#"Based on the following project information, please generate a concise, one-paragraph overview of the project's purpose and primary function. Describe what the project does from a high-level perspective.

**Tech Stack:**
{}

**Key Commands:**
- Build: {}
- Run: {}
- Test: {}

**Project Structure:**
{}

**README.md Content (first 1000 chars):**
---
{}
---

**Your Task:**
Generate a single paragraph summary. Respond ONLY with the summary text, without any additional titles or formatting."#,
            context.tech_stack.join(", "),
            context.build_commands.join(", "),
            context.run_commands.join(", "),
            context.test_commands.join(", "),
            context.project_structure,
            readme_content.chars().take(1000).collect::<String>()
        );

        let request = rainy_sdk::ChatCompletionRequest {
            messages: vec![rainy_sdk::ChatMessage {
                role: rainy_sdk::MessageRole::User,
                content: prompt,
            }],
            model: model.to_string(),
            provider: None,
            temperature: Some(0.5),
            max_tokens: Some(250),
            stream: Some(false),
            stop: None,
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            n: None,
            user: None,
            response_format: None,
            tools: None,
            tool_choice: None,
        };

        if let Ok(response) = client.create_chat_completion(request).await {
            if let Some(choice) = response.choices.first() {
                context.project_overview = choice.message.content.trim().to_string();
            }
        }
    }

    Ok(context)
}

fn get_file_list(path: &Path) -> Result<Vec<String>> {
    if let Ok(repo) = Repository::open(path) {
        let mut tracked_files = Vec::new();
        let index = repo.index()?;
        for entry in index.iter() {
            if let Ok(path_str) = String::from_utf8(entry.path.to_vec()) {
                tracked_files.push(path_str);
            }
        }
        if !tracked_files.is_empty() {
            return Ok(tracked_files);
        }
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_path = entry.path().strip_prefix(path).unwrap_or_else(|_| entry.path());
            if let Some(path_str) = file_path.to_str() {
                files.push(path_str.to_string());
            }
        }
    }
    Ok(files)
}

fn analyze_files(files: &[String]) -> Result<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)> {
    let mut tech_stack = HashSet::new();
    let mut build_commands = Vec::new();
    let mut run_commands = Vec::new();
    let mut test_commands = Vec::new();

    for file in files {
        // Tech stack from extension
        if file.ends_with(".rs") { tech_stack.insert("Rust".to_string()); }
        if file.ends_with(".js") { tech_stack.insert("JavaScript".to_string()); }
        if file.ends_with(".ts") { tech_stack.insert("TypeScript".to_string()); }
        if file.ends_with(".py") { tech_stack.insert("Python".to_string()); }
        if file.ends_with(".java") { tech_stack.insert("Java".to_string()); }
        if file.ends_with(".go") { tech_stack.insert("Go".to_string()); }
        if file.ends_with(".html") { tech_stack.insert("HTML".to_string()); }
        if file.ends_with(".css") { tech_stack.insert("CSS".to_string()); }
        if file.ends_with(".scss") || file.ends_with(".sass") { tech_stack.insert("Sass/SCSS".to_string()); }
        if file.ends_with(".jsx") || file.ends_with(".tsx") { tech_stack.insert("React".to_string()); }
        if file.ends_with(".vue") { tech_stack.insert("Vue".to_string()); }
        if file.ends_with(".svelte") { tech_stack.insert("Svelte".to_string()); }

        // Tech stack and commands from specific files
        if file.ends_with("package.json") {
            tech_stack.insert("Node.js".to_string());
            if let Ok(content) = std::fs::read_to_string(file) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
                        if scripts.contains_key("build") { build_commands.push("npm run build".to_string()); }
                        if scripts.contains_key("start") { run_commands.push("npm run start".to_string()); }
                        if scripts.contains_key("dev") { run_commands.push("npm run dev".to_string()); }
                        if scripts.contains_key("test") { test_commands.push("npm run test".to_string()); }
                    }
                    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                        if deps.contains_key("react") { tech_stack.insert("React".to_string()); }
                        if deps.contains_key("next") { tech_stack.insert("Next.js".to_string()); }
                        if deps.contains_key("vue") { tech_stack.insert("Vue".to_string()); }
                        if deps.contains_key("angular") { tech_stack.insert("Angular".to_string()); }
                        if deps.contains_key("express") { tech_stack.insert("Express".to_string()); }
                    }
                }
            }
        }

        if file.ends_with("Cargo.toml") {
            tech_stack.insert("Rust".to_string());
            build_commands.push("cargo build".to_string());
            run_commands.push("cargo run".to_string());
            test_commands.push("cargo test".to_string());
        }

        if file.ends_with("requirements.txt") || file.ends_with("pyproject.toml") {
            tech_stack.insert("Python".to_string());
            test_commands.push("pytest".to_string());
            run_commands.push("python main.py".to_string());
        }

        if file.ends_with("pom.xml") {
            tech_stack.insert("Java".to_string());
            tech_stack.insert("Maven".to_string());
            build_commands.push("mvn clean install".to_string());
            test_commands.push("mvn test".to_string());
        }

        if file.ends_with("Dockerfile") {
            tech_stack.insert("Docker".to_string());
        }
    }

    Ok((tech_stack.into_iter().collect(), build_commands, run_commands, test_commands))
}

fn generate_project_structure(_path: &Path, files: &[String]) -> Result<String> {
    let mut tree = String::new();
    let mut root_dirs = HashSet::new();
    let mut root_files = HashSet::new();

    for file_path in files {
        let path = Path::new(file_path);
        let mut components = path.components();
        if let Some(first) = components.next() {
            if components.next().is_some() {
                root_dirs.insert(first.as_os_str().to_string_lossy().to_string());
            } else {
                root_files.insert(first.as_os_str().to_string_lossy().to_string());
            }
        }
    }

    let mut sorted_dirs: Vec<_> = root_dirs.into_iter().collect();
    sorted_dirs.sort();
    for dir in sorted_dirs {
        tree.push_str(&format!("📁 {}/\n", dir));
    }

    let mut sorted_files: Vec<_> = root_files.into_iter().collect();
    sorted_files.sort();
    for file in sorted_files {
        tree.push_str(&format!("📄 {}\n", file));
    }

    Ok(tree)
}

fn identify_key_files(files: &[String]) -> Vec<String> {
    let mut key_files = Vec::new();
    let key_file_names = [
        "README.md", "readme.md",
        "CONTRIBUTING.md", "contributing.md",
        "LICENSE", "LICENSE.md",
        "main.rs", "lib.rs", "index.js", "main.py", "app.py",
        "package.json", "Cargo.toml", "pyproject.toml", "pom.xml",
        "webpack.config.js", "vite.config.js", "tailwind.config.js",
        "Dockerfile", "docker-compose.yml",
    ];

    for file in files {
        let path = Path::new(file);
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if key_file_names.contains(&file_name) {
                key_files.push(file.to_string());
            }
        }
    }
    key_files
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
