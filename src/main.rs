use clap::{Parser, Subcommand};
use miette::Result;
use rainy_cli::{config, error, ui, utils};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "rainy-cli")]
#[command(about = "🚀 AI-powered code assistant CLI agent - Professional developer toolkit")]
#[command(version = "0.2.0")]
#[command(long_about = "
Rainy CLI is a premium AI-powered code assistant that helps you:
• Analyze code for security, performance, and quality issues
• Generate code from natural language descriptions
• Review code with intelligent suggestions
• Chat with AI about your codebase
• Generate project templates and documentation

Powered by advanced AI models and built for professional developers.
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Override the default model
    #[arg(short, long, global = true)]
    model: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze code in a file or directory
    Analyze {
        /// Path to file or directory to analyze
        #[arg(short, long)]
        path: PathBuf,

        /// Analysis type (security, performance, style, complexity, general)
        #[arg(short, long, default_value = "general")]
        analysis_type: String,

        /// Apply suggestions interactively
        #[arg(long)]
        apply: bool,
    },
    /// Generate code based on description
    Generate {
        /// Description of what to generate
        description: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Generate unit tests along with code
        #[arg(long)]
        with_tests: bool,

        /// Generate documentation
        #[arg(long)]
        with_docs: bool,
    },
    /// Generate project from template
    Template {
        /// Template type (rust-api, rust-cli, rust-lib, web-api, microservice)
        template: String,

        /// Project name
        name: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Review and suggest improvements for code
    Review {
        /// Path to file or directory to review
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Focus area (performance, security, readability, etc.)
        #[arg(short, long)]
        focus: Option<String>,

        /// Review only Git changes (staged, unstaged, or specific commit)
        #[arg(long)]
        git: bool,

        /// Git reference to compare against (default: HEAD)
        #[arg(long, default_value = "HEAD")]
        git_ref: String,
    },
    /// Interactive chat mode with the AI agent
    Chat {
        /// Initial message to start the conversation
        message: Option<String>,

        /// Load specific file context for the chat
        #[arg(long)]
        context_file: Option<PathBuf>,
    },
    /// Configure CLI settings
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,

        /// Set API key
        #[arg(long)]
        set_api_key: Option<String>,

        /// Set default model
        #[arg(long)]
        set_model: Option<String>,

        /// Reset configuration to defaults
        #[arg(long)]
        reset: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure rainy.md exists
    utils::rainy_md::ensure_rainy_md_exists()
        .map_err(|e| error::CliError::config_error(&e.to_string()))?;

    // Load configuration
    let mut config = config::Config::load()
        .map_err(|e| error::CliError::config_error(&format!("Failed to load configuration: {}", e)))?;

    let cli = Cli::parse();

    // Apply global flags
    if cli.verbose {
        config.verbose = true;
    }

    if let Some(model) = cli.model {
        config.default_model = model;
    }

    // Show welcome screen for interactive commands
    match &cli.command {
        Commands::Chat { .. } | Commands::Config { show: true, .. } => {
            ui::print_welcome();
        }
        _ => {}
    }

    // Handle config command early (before API key check)
    if let Commands::Config {
        show,
        set_api_key,
        set_model,
        reset,
    } = &cli.command
    {
        return handle_config_command(&mut config, *show, set_api_key, set_model, *reset).await;
    }

    // Check if API key is configured for commands that need it
    if !config.has_api_key() && !matches!(cli.command, Commands::Template { .. }) {
        ui::print_api_key_setup();

        let api_key = loop {
            let key = ui::prompt_api_key()
                .map_err(|e| error::CliError::file_error("Failed to read API key input", e))?;

            if key.is_empty() {
                ui::print_error("API key cannot be empty. Please try again.");
                continue;
            }

            break key;
        };

        config
            .set_api_key(api_key.clone())
            .map_err(|e| error::CliError::config_error(&format!("Failed to save API key: {}", e)))?;
        ui::print_success("API key saved successfully!");
        println!();
    }

    // Route to appropriate command handler
    match cli.command {
        Commands::Analyze {
            path,
            analysis_type,
            apply: _,
        } => commands::analyze::handle_analyze_command(path, analysis_type, &config).await,
        Commands::Generate {
            description,
            output,
            with_tests: _,
            with_docs: _,
        } => commands::generate::handle_generate_command(description, output, &config).await,
        Commands::Template {
            template,
            name,
            output,
        } => commands::template::handle_template_command(template, name, output).await,
        Commands::Review {
            path,
            focus,
            git,
            git_ref,
        } => commands::review::handle_review_command(path, focus, git, git_ref, &config).await,
        Commands::Chat {
            message,
            context_file: _,
        } => commands::chat::handle_chat_command(message, &config).await,
        Commands::Config { .. } => {
            // Already handled above
            Ok(())
        }
    }
}

async fn handle_config_command(
    config: &mut config::Config,
    show: bool,
    set_api_key: &Option<String>,
    set_model: &Option<String>,
    reset: bool,
) -> Result<()> {
    ui::print_command_start("CONFIG", &format!("{} Configuration Management", ui::GEAR));

    if reset {
        *config = config::Config::default();
        config
            .save()
            .map_err(|e| error::CliError::config_error(&format!("Failed to save configuration: {}", e)))?;
        ui::print_success("Configuration reset to defaults!");
        return Ok(());
    }

    if let Some(api_key) = set_api_key {
        config
            .set_api_key(api_key.clone())
            .map_err(|e| error::CliError::config_error(&format!("Failed to save API key: {}", e)))?;
        ui::print_success("API key updated successfully!");
    }

    if let Some(model) = set_model {
        config.default_model = model.clone();
        config
            .save()
            .map_err(|e| error::CliError::config_error(&format!("Failed to save configuration: {}", e)))?;
        ui::print_success(&format!("Default model set to: {}", model));
    }

    if show {
        ui::print_separator();
        let config_display = format!(
            "API Key: {}\nDefault Model: {}\nTheme: {}\nMax Tokens: {:?}\nTemperature: {:?}\nAuto Save: {}\nVerbose: {}",
            if config.has_api_key() { "✓ Configured" } else { "✗ Not set" },
            config.get_model(),
            config.theme,
            config.get_max_tokens(),
            config.get_temperature(),
            config.should_auto_save(),
            config.is_verbose()
        );
        ui::print_code_block("Current Configuration", &config_display);

        ui::print_info("Configuration file location:");
        if let Ok(config_path) = config::Config::config_file() {
            ui::print_info(&format!("  {}", config_path.display()));
        }
    }

    Ok(())
}