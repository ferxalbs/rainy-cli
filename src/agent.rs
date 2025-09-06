use anyhow::{anyhow, Result};
use rainy_sdk::{RainyClient, ChatRole, ChatMessage as RainyChatMessage, ChatCompletionRequest};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct AIAgent {
    client: RainyClient,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
}

impl AIAgent {
    pub fn new(api_key: String) -> Result<Self> {
        Self::with_config(api_key, "moonshotai/kimi-k2-instruct".to_string(), 0.7, None)
    }

    pub fn with_config(api_key: String, model: String, temperature: f32, max_tokens: Option<u32>) -> Result<Self> {
        Ok(Self {
            client: RainyClient::with_api_key(&api_key)?,
            model,
            temperature,
            max_tokens,
        })
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature;
    }

    pub fn set_max_tokens(&mut self, max_tokens: Option<u32>) {
        self.max_tokens = max_tokens;
    }


    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        // Convert our ChatMessage format to rainy-sdk format
        let rainy_messages: Vec<RainyChatMessage> = messages
            .into_iter()
            .map(|msg| RainyChatMessage {
                role: match msg.role.as_str() {
                    "user" => ChatRole::User,
                    "assistant" => ChatRole::Assistant,
                    "system" => ChatRole::System,
                    _ => ChatRole::User, // default to user
                },
                content: msg.content,
            })
            .collect();

        // Create chat completion request
        let request = ChatCompletionRequest {
            messages: rainy_messages,
            model: self.model.clone(),
            temperature: Some(self.temperature),
            max_tokens: self.max_tokens,
            stream: Some(false),
        };

        // Use rainy-sdk's chat completion method
        let response = self.client.create_chat_completion(request).await?;

        // Extract content from the first choice
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("No response from AI"))
        }
    }

    pub async fn analyze_code(&self, code: &str, analysis_type: &str) -> Result<String> {
        let prompt = match analysis_type {
            "security" => format!(
                "Perform a comprehensive security analysis of the following code. Look for:\n\
                - SQL injection vulnerabilities\n\
                - Cross-site scripting (XSS)\n\
                - Command injection\n\
                - Buffer overflows\n\
                - Authentication bypasses\n\
                - Input validation issues\n\
                - Cryptographic weaknesses\n\
                \nProvide specific recommendations with code examples for fixes.\n\n{}",
                code
            ),
            "performance" => format!(
                "Analyze the following code for performance bottlenecks and optimization opportunities:\n\
                - Memory usage patterns\n\
                - Algorithm complexity\n\
                - I/O operations\n\
                - Database queries\n\
                - Caching opportunities\n\
                - Parallelization potential\n\
                \nProvide concrete performance improvements with before/after code examples.\n\n{}",
                code
            ),
            "style" => format!(
                "Review the following code for style, readability, and maintainability:\n\
                - Naming conventions\n\
                - Code organization\n\
                - Documentation quality\n\
                - Error handling patterns\n\
                - Code duplication\n\
                - Complexity management\n\
                \nFollow Rust style guidelines and provide specific refactoring suggestions.\n\n{}",
                code
            ),
            "complexity" => format!(
                "Analyze the code complexity and maintainability:\n\
                - Cyclomatic complexity\n\
                - Function length and responsibilities\n\
                - Coupling and cohesion\n\
                - Testability\n\
                - Refactoring opportunities\n\
                \nProvide complexity metrics and specific recommendations to reduce complexity.\n\n{}",
                code
            ),
            "general" => format!(
                "Provide a comprehensive analysis of the following code covering:\n\
                - Functionality and correctness\n\
                - Code quality and structure\n\
                - Potential improvements\n\
                - Best practices compliance\n\n{}",
                code
            ),
            _ => format!(
                "Analyze the following code and provide insights:\n\n{}",
                code
            ),
        };

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        self.chat(messages).await
    }

    pub async fn generate_code(&self, description: &str) -> Result<String> {
        let prompt = format!(
            "Generate high-quality code based on this description: {}\n\nProvide only the code without explanation.",
            description
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        self.chat(messages).await
    }

    pub async fn review_code(&self, code: &str, focus: Option<&str>) -> Result<String> {
        let focus_text = focus.map(|f| format!(" Focus on: {}", f)).unwrap_or_default();
        let prompt = format!(
            "Review the following code and suggest improvements:{}\n\n{}",
            focus_text, code
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        self.chat(messages).await
    }
}

pub async fn read_file_content(path: &Path) -> Result<String> {
    if !path.exists() {
        return Err(anyhow!("File does not exist: {}", path.display()));
    }

    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| anyhow!("Failed to read file {}: {}", path.display(), e))
}
