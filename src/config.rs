use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs::home_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
    pub default_model: String,
    pub theme: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub auto_save: bool,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            default_model: "rainy-coder-1".to_string(),
            theme: "dark".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            auto_save: true,
            verbose: false,
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        let config_dir = home.join(".rainy-cli");
        Ok(config_dir)
    }

    pub fn config_file() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;

        if !config_file.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_file)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        let config_file = Self::config_file()?;

        // Create config directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_file, content)?;
        Ok(())
    }

    pub fn set_api_key(&mut self, api_key: String) -> Result<()> {
        self.api_key = Some(api_key);
        self.save()
    }

    pub fn get_api_key(&self) -> Result<&str> {
        self.api_key.as_deref().ok_or_else(|| anyhow!(
            "API key not found. Please run the CLI to set it up."
        ))
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    pub fn get_model(&self) -> &str {
        &self.default_model
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature.unwrap_or(0.7)
    }

    pub fn get_max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn should_auto_save(&self) -> bool {
        self.auto_save
    }
}
