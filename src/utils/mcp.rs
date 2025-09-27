use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

pub fn load_mcp_config() -> Result<McpConfig, anyhow::Error> {
    let mut merged_config = McpConfig {
        mcp_servers: HashMap::new(),
    };

    // Load Claude for Desktop config
    if let Some(claude_config_path) = get_claude_config_path() {
        if claude_config_path.exists() {
            let content = fs::read_to_string(claude_config_path)?;
            if let Ok(claude_config) = serde_json::from_str::<McpConfig>(&content) {
                merged_config.mcp_servers.extend(claude_config.mcp_servers);
            }
        }
    }

    // Load rainy-mcp.json from global config dir
    if let Some(mut global_rainy_path) = dirs::config_dir() {
        global_rainy_path.push("rainy-cli/rainy-mcp.json");
        if global_rainy_path.exists() {
            let content = fs::read_to_string(global_rainy_path)?;
            if let Ok(rainy_config) = serde_json::from_str::<McpConfig>(&content) {
                merged_config.mcp_servers.extend(rainy_config.mcp_servers);
            }
        }
    }

    // Load rainy-mcp.json from .rainy/rainy-mcp.json
    let local_rainy_path_in_dir = PathBuf::from(".rainy/rainy-mcp.json");
    if local_rainy_path_in_dir.exists() {
        let content = fs::read_to_string(local_rainy_path_in_dir)?;
        if let Ok(rainy_config) = serde_json::from_str::<McpConfig>(&content) {
            merged_config.mcp_servers.extend(rainy_config.mcp_servers);
        }
    }

    // Note: Removed loading from current project directory (rainy-mcp.json)
    // The configuration should only be managed from the global config directory

    Ok(merged_config)
}

fn get_claude_config_path() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        dirs::config_dir().map(|p| p.join("Claude/claude_desktop_config.json"))
    } else if cfg!(target_os = "macos") {
        dirs::home_dir().map(|p| p.join("Library/Application Support/Claude/claude_desktop_config.json"))
    } else {
        dirs::config_dir().map(|p| p.join("Claude/claude_desktop_config.json"))
    }
}

pub fn add_mcp_server(server_name: &str, command: &str, args: &[String]) -> Result<(), anyhow::Error> {
    let mut config = load_mcp_config()?;
    let new_server = McpServerConfig {
        command: command.to_string(),
        args: args.to_vec(),
        env: HashMap::new(),
    };
    config.mcp_servers.insert(server_name.to_string(), new_server);
    save_mcp_config(&config)
}

pub fn remove_mcp_server(server_name: &str) -> Result<(), anyhow::Error> {
    let mut config = load_mcp_config()?;
    config.mcp_servers.remove(server_name);
    save_mcp_config(&config)
}

fn save_mcp_config(config: &McpConfig) -> Result<(), anyhow::Error> {
    let mut path = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    path.push("rainy-cli");
    fs::create_dir_all(&path)?;
    path.push("rainy-mcp.json");
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
