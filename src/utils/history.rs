use crate::executor::ChatMessage;
use anyhow::Result;
use std::path::PathBuf;

pub fn get_history_file_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    path.push(".rainy-cli");
    std::fs::create_dir_all(&path).unwrap_or(());
    path.push("chat_history.json");
    path
}

pub fn save_conversation_history(messages: &[ChatMessage]) -> Result<()> {
    // Filter out system messages and limit history size more aggressively
    let history: Vec<&ChatMessage> = messages.iter()
        .filter(|msg| msg.role != "system")
        .rev()
        .take(20) // Reduced from 50 to 20 messages to save tokens
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let content = serde_json::to_string_pretty(&history)?;
    std::fs::write(get_history_file_path(), content)?;
    Ok(())
}

/// Load conversation history with content truncation to save tokens
pub fn load_conversation_history_truncated(max_chars_per_message: usize) -> Result<Vec<ChatMessage>> {
    let mut history = load_conversation_history()?;

    // Truncate long messages to save tokens
    for msg in &mut history {
        if msg.content.len() > max_chars_per_message {
            msg.content = format!("{}... [truncated]", &msg.content[..max_chars_per_message.saturating_sub(20)]);
        }
    }

    Ok(history)
}

pub fn load_conversation_history() -> Result<Vec<ChatMessage>> {
    let path = get_history_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)?;
    let history: Vec<ChatMessage> = serde_json::from_str(&content)?;
    Ok(history)
}

pub fn clear_conversation_history() -> Result<()> {
    let path = get_history_file_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn export_conversation_history(export_path: &str) -> Result<()> {
    let history = load_conversation_history()?;
    let content = serde_json::to_string_pretty(&history)?;
    std::fs::write(export_path, content)?;
    Ok(())
}

pub fn get_history_summary() -> Result<String> {
    let history = load_conversation_history()?;
    let user_messages = history.iter().filter(|msg| msg.role == "user").count();
    let assistant_messages = history.iter().filter(|msg| msg.role == "assistant").count();
    
    Ok(format!(
        "Chat History: {} user messages, {} assistant responses",
        user_messages, assistant_messages
    ))
}
