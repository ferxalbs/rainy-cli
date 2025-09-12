use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum CliError {
    #[error("Configuration error: {message}")]
    #[diagnostic(code(rainy_cli::config))]
    Config {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("API error: {message}")]
    #[diagnostic(code(rainy_cli::api))]
    Api {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("File operation failed: {message}")]
    #[diagnostic(code(rainy_cli::file))]
    File {
        message: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command execution failed: {message}")]
    #[diagnostic(code(rainy_cli::command))]
    Command {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Context error: {message}")]
    #[diagnostic(code(rainy_cli::context))]
    Context {
        message: String,
        #[source]
        source: anyhow::Error,
    },
}

impl CliError {
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
        }
    }

    pub fn api_error(message: impl Into<String>) -> Self {
        Self::Api {
            message: message.into(),
            source: None,
        }
    }

    pub fn file_error(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::File {
            message: message.into(),
            source,
        }
    }


    pub fn command_error(message: impl Into<String>) -> Self {
        Self::Command {
            message: message.into(),
            source: None,
        }
    }

    pub fn context_error(message: impl Into<String>, source: anyhow::Error) -> Self {
        Self::Context {
            message: message.into(),
            source,
        }
    }
}

