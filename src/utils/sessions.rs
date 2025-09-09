use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub message_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

pub struct SessionManager {
    sessions_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let mut sessions_dir = dirs::home_dir()
            .context("Could not find home directory")?;
        sessions_dir.push(".rainy-cli");
        sessions_dir.push("sessions");

        fs::create_dir_all(&sessions_dir)
            .context("Failed to create sessions directory")?;

        Ok(Self { sessions_dir })
    }

    fn get_session_file_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", session_id))
    }

    pub fn create_session(&self, name: String, description: Option<String>) -> Result<Session> {
        let session_id = format!("session_{}", chrono::Utc::now().timestamp_millis());
        let now = Utc::now();

        let session = Session {
            id: session_id.clone(),
            name,
            description,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        };

        self.save_session(&session)?;
        Ok(session)
    }

    pub fn save_session(&self, session: &Session) -> Result<()> {
        let session_path = self.get_session_file_path(&session.id);
        let json = serde_json::to_string_pretty(session)
            .context("Failed to serialize session")?;
        fs::write(session_path, json)
            .context("Failed to save session to file")?;
        Ok(())
    }

    pub fn load_session(&self, session_id: &str) -> Result<Session> {
        let session_path = self.get_session_file_path(session_id);
        if !session_path.exists() {
            return Err(anyhow::anyhow!("Session '{}' not found", session_id));
        }

        let content = fs::read_to_string(session_path)
            .context("Failed to read session file")?;
        let session: Session = serde_json::from_str(&content)
            .context("Failed to parse session")?;

        Ok(session)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let mut sessions = Vec::new();

        for entry in fs::read_dir(&self.sessions_dir)
            .context("Failed to read sessions directory")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        sessions.push(SessionMetadata {
                            id: session.id,
                            name: session.name,
                            description: session.description,
                            message_count: session.messages.len(),
                            created_at: session.created_at,
                            updated_at: session.updated_at,
                            tags: session.tags,
                        });
                    }
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sessions)
    }

    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let session_path = self.get_session_file_path(session_id);
        if !session_path.exists() {
            return Err(anyhow::anyhow!("Session '{}' not found", session_id));
        }

        fs::remove_file(session_path)
            .context("Failed to delete session file")?;
        Ok(())
    }

    pub fn update_session_name(&self, session_id: &str, new_name: String) -> Result<()> {
        let mut session = self.load_session(session_id)?;
        session.name = new_name;
        session.updated_at = Utc::now();
        self.save_session(&session)?;
        Ok(())
    }

    pub fn update_session_description(&self, session_id: &str, description: Option<String>) -> Result<()> {
        let mut session = self.load_session(session_id)?;
        session.description = description;
        session.updated_at = Utc::now();
        self.save_session(&session)?;
        Ok(())
    }

    pub fn add_session_tag(&self, session_id: &str, tag: String) -> Result<()> {
        let mut session = self.load_session(session_id)?;
        if !session.tags.contains(&tag) {
            session.tags.push(tag);
        }
        session.updated_at = Utc::now();
        self.save_session(&session)?;
        Ok(())
    }

    pub fn remove_session_tag(&self, session_id: &str, tag: &str) -> Result<()> {
        let mut session = self.load_session(session_id)?;
        session.tags.retain(|t| t != tag);
        session.updated_at = Utc::now();
        self.save_session(&session)?;
        Ok(())
    }

    pub fn add_message_to_session(&self, session_id: &str, message: ChatMessage) -> Result<()> {
        let mut session = self.load_session(session_id)?;
        session.messages.push(message);
        session.updated_at = Utc::now();
        self.save_session(&session)?;
        Ok(())
    }

    pub fn save_session_messages(&self, session_id: &str, messages: &[ChatMessage]) -> Result<()> {
        let mut session = self.load_session(session_id)?;
        session.messages = messages.to_vec();
        session.updated_at = Utc::now();
        self.save_session(&session)?;
        Ok(())
    }

    pub fn clear_session_messages(&self, session_id: &str) -> Result<()> {
        let mut session = self.load_session(session_id)?;
        session.messages.clear();
        session.updated_at = Utc::now();
        self.save_session(&session)?;
        Ok(())
    }

    pub fn search_sessions(&self, query: &str) -> Result<Vec<SessionMetadata>> {
        let all_sessions = self.list_sessions()?;
        let query_lower = query.to_lowercase();

        Ok(all_sessions.into_iter()
            .filter(|session| {
                session.name.to_lowercase().contains(&query_lower) ||
                session.description.as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false) ||
                session.tags.iter()
                    .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect())
    }

    pub fn export_session(&self, session_id: &str, export_path: &str) -> Result<()> {
        let session = self.load_session(session_id)?;
        let json = serde_json::to_string_pretty(&session)
            .context("Failed to serialize session")?;
        fs::write(export_path, json)
            .context("Failed to export session")?;
        Ok(())
    }

    pub fn import_session(&self, import_path: &str, new_name: Option<String>) -> Result<Session> {
        let content = fs::read_to_string(import_path)
            .context("Failed to read import file")?;
        let mut session: Session = serde_json::from_str(&content)
            .context("Failed to parse session")?;

        // Generate new ID and update timestamps
        session.id = format!("session_{}", chrono::Utc::now().timestamp_millis());
        if let Some(name) = new_name {
            session.name = name;
        }
        session.created_at = Utc::now();
        session.updated_at = Utc::now();

        self.save_session(&session)?;
        Ok(session)
    }
}
