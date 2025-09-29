use serde::{Deserialize, Serialize};

/// Security levels for command execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    /// Always require user approval
    Low,
    /// Require approval for sensitive operations
    Medium,
    /// Never require approval (high trust)
    High,
}

impl SecurityLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => SecurityLevel::Low,
            "medium" => SecurityLevel::Medium,
            "high" => SecurityLevel::High,
            _ => SecurityLevel::Low, // Default to most secure
        }
    }
}