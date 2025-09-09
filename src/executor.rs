use anyhow::{anyhow, Context, Result};
use rainy_sdk::{RainyClient, ChatRole, ChatMessage as RainyChatMessage, ChatCompletionRequest};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct AgenticExecutor {
    client: RainyClient,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
    system_prompt: String,
}

impl AgenticExecutor {
    pub async fn new(api_key: String, model: Option<String>) -> Result<Self> {
        let rainy_md_content = fs::read_to_string("rainy.md")
            .await
            .context("Failed to read rainy.md")?;

        let tool_definitions = r#"
[
    {
        "tool": "read_file",
        "description": "Reads the entire content of a file and returns it as a string.",
        "parameters": {
            "path": "The path to the file to read."
        }
    },
    {
        "tool": "write_file",
        "description": "Creates a new file or overwrites an existing file with the given content.",
        "parameters": {
            "path": "The path to the file to write.",
            "content": "The content to write to the file."
        }
    },
    {
        "tool": "delete_file",
        "description": "Deletes a file.",
        "parameters": {
            "path": "The path to the file to delete."
        }
    },
    {
        "tool": "list_files",
        "description": "Lists all files and directories in a given directory path.",
        "parameters": {
            "path": "The path to the directory to list."
        }
    }
]
"#;

        let system_prompt = format!(
r#"You are Rainy Coder. Help users with coding tasks by creating JSON plans of tool calls.

Available tools:
{}

Project instructions:
{}

Response: JSON array of tool calls only. Format: [{{"tool": "name", "parameters": {{...}}}}]"#,
            tool_definitions,
            rainy_md_content
        );

        Ok(Self {
            client: RainyClient::with_api_key(&api_key)?,
            model: model.unwrap_or_else(|| "moonshotai/kimi-k2-instruct-0905".to_string()),
            temperature: 0.7,
            max_tokens: Some(16000), // Max output tokens for the model
            system_prompt,
        })
    }

    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<(rainy_sdk::ChatCompletionResponse, std::time::Duration)> {
        let mut full_messages = vec![RainyChatMessage {
            role: ChatRole::System,
            content: self.system_prompt.clone(),
        }];

        let rainy_messages: Vec<RainyChatMessage> = messages
            .into_iter()
            .map(|msg| RainyChatMessage {
                role: match msg.role.as_str() {
                    "user" => ChatRole::User,
                    "assistant" => ChatRole::Assistant,
                    "system" => ChatRole::User,
                    _ => ChatRole::User,
                },
                content: msg.content,
            })
            .collect();

        full_messages.extend(rainy_messages);

        let request = ChatCompletionRequest {
            messages: full_messages,
            model: self.model.clone(),
            provider: None,
            temperature: Some(self.temperature),
            top_p: None,
            stream: Some(false),
            stop: None,
            max_tokens: self.max_tokens,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let start_time = std::time::Instant::now();
        let response = self.client.create_chat_completion(request).await?;
        let duration = start_time.elapsed();

        if response.choices.first().is_some() {
            Ok((response, duration))
        } else {
            Err(anyhow!("No response from AI"))
        }
    }
}

// Helper function to read file content, can be used by tools later
pub async fn read_file_content(path: &Path) -> Result<String> {
    if !path.exists() {
        return Err(anyhow!("File does not exist: {}", path.display()));
    }

    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| anyhow!("Failed to read file {}: {}", path.display(), e))
}
