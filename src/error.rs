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

    #[error("Analysis failed: {message}")]
    #[diagnostic(code(rainy_cli::analysis))]
    Analysis {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Invalid input: {message}")]
    #[diagnostic(code(rainy_cli::input))]
    InvalidInput {
        message: String,
        #[help]
        help: Option<String>,
    },

    #[error("Command execution failed: {message}")]
    #[diagnostic(code(rainy_cli::command))]
    Command {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
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

    pub fn analysis_error(message: impl Into<String>) -> Self {
        Self::Analysis {
            message: message.into(),
            source: None,
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
            help: None,
        }
    }

    pub fn command_error(message: impl Into<String>) -> Self {
        Self::Command {
            message: message.into(),
            source: None,
        }
    }
}

pub type Result<T> = miette::Result<T, CliError>;
