use clap::{Parser, Subcommand};
use miette::Result;
use std::path::PathBuf;

use crate::config::Config;

mod commands;
mod config;
mod error;
mod executor;
mod tools;
mod ui;
mod utils;

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
enum SessionAction {
    /// Create a new session
    Create {
        /// Name for the new session
        name: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all sessions
    List,
    /// Show details of a specific session
    Show {
        /// Session ID
        id: String,
    },
    /// Start chat with a specific session
    Chat {
        /// Session ID
        id: String,

        /// Initial message to start the conversation
        message: Option<String>,

        /// Load specific file contexts for the chat
        #[arg(long, num_args = 1..)]
        context_files: Vec<PathBuf>,

        /// Skip loading conversation history to save tokens
        #[arg(long)]
        no_history: bool,
    },
    /// Rename a session
    Rename {
        /// Session ID
        id: String,

        /// New name
        name: String,
    },
    /// Update session description
    UpdateDescription {
        /// Session ID
        id: String,

        /// New description
        description: String,
    },
    /// Add a tag to a session
    AddTag {
        /// Session ID
        id: String,

        /// Tag to add
        tag: String,
    },
    /// Remove a tag from a session
    RemoveTag {
        /// Session ID
        id: String,

        /// Tag to remove
        tag: String,
    },
    /// Delete a session
    Delete {
        /// Session ID
        id: String,
    },
    /// Clear all messages from a session
    Clear {
        /// Session ID
        id: String,
    },
    /// Search sessions by name, description, or tags
    Search {
        /// Search query
        query: String,
    },
    /// Export a session to a file
    Export {
        /// Session ID
        id: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },
    /// Import a session from a file
    Import {
        /// Input file path
        input: String,

        /// Optional new name for the imported session
        #[arg(short, long)]
        name: Option<String>,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze code in one or more files or directories
    Analyze {
        /// Paths to files or directories to analyze
        #[arg(short, long, num_args = 1..)]
        paths: Vec<PathBuf>,

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
        /// Paths to files or directories to review
        #[arg(short, long, num_args = 1..)]
        paths: Vec<PathBuf>,

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

        /// Load specific file contexts for the chat
        #[arg(long, num_args = 1..)]
        context_files: Vec<PathBuf>,

        /// Skip loading conversation history to save tokens
        #[arg(long)]
        no_history: bool,
    },
    /// Manage chat sessions
    Session {
        #[command(subcommand)]
        action: SessionAction,
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
            paths,
            analysis_type,
            apply: _,
        } => commands::analyze::handle_analyze_command(paths, analysis_type, &config).await,
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
            paths,
            focus,
            git,
            git_ref,
        } => commands::review::handle_review_command(paths, focus, git, git_ref, &config).await,
        Commands::Chat {
            message,
            context_files,
            no_history,
        } => commands::chat::handle_chat_command(message, Some(context_files), no_history, &config).await,
        Commands::Session { action } => handle_session_command(action, &config).await,
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

async fn handle_session_command(action: SessionAction, config: &Config) -> Result<()> {
    use crate::utils::sessions::SessionManager;

    let session_manager = SessionManager::new()
        .map_err(|e| error::CliError::api_error(&format!("Failed to initialize session manager: {}", e)))?;

    match action {
        SessionAction::Create { name, description } => {
            ui::print_command_start("SESSION", &format!("{} Creating new session", ui::ADD));

            match session_manager.create_session(name.clone(), description.clone()) {
                Ok(session) => {
                    ui::print_success(&format!("Session '{}' created with ID: {}", name, session.id));

                    if let Some(desc) = description {
                        println!("Description: {}", desc);
                    }

                    ui::print_info("Use the following commands to work with this session:");
                    ui::print_info(&format!("  rainy-cli session chat {}", session.id));
                    ui::print_info(&format!("  rainy-cli session show {}", session.id));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to create session: {}", e));
                    return Err(error::CliError::api_error(&format!("Session creation failed: {}", e)).into());
                }
            }
        }

        SessionAction::List => {
            ui::print_command_start("SESSION", &format!("{} Listing all sessions", ui::LIST));

            match session_manager.list_sessions() {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        ui::print_info("No sessions found. Create your first session with:");
                        ui::print_info("  rainy-cli session create \"My Session\"");
                        return Ok(());
                    }

                    ui::print_info(&format!("Found {} session(s):", sessions.len()));
                    println!();

                    for session in sessions {
                        let desc = session.description
                            .as_ref()
                            .map(|d| format!(" - {}", d))
                            .unwrap_or_default();

                        let tags = if session.tags.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", session.tags.join(", "))
                        };

                        println!("📝 {} ({})", session.name, session.id);
                        println!("   Messages: {}, Created: {}{}",
                                session.message_count,
                                session.created_at.format("%Y-%m-%d %H:%M"),
                                desc);
                        if !tags.is_empty() {
                            println!("   Tags:{}", tags);
                        }
                        println!();
                    }
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to list sessions: {}", e));
                    return Err(error::CliError::api_error(&format!("Session listing failed: {}", e)).into());
                }
            }
        }

        SessionAction::Show { id } => {
            ui::print_command_start("SESSION", &format!("{} Showing session details", ui::INFO));

            match session_manager.load_session(&id) {
                Ok(session) => {
                    ui::print_info(&format!("Session: {}", session.name));
                    ui::print_info(&format!("ID: {}", session.id));

                    if let Some(desc) = &session.description {
                        ui::print_info(&format!("Description: {}", desc));
                    }

                    ui::print_info(&format!("Messages: {}", session.messages.len()));
                    ui::print_info(&format!("Created: {}", session.created_at.format("%Y-%m-%d %H:%M:%S")));
                    ui::print_info(&format!("Updated: {}", session.updated_at.format("%Y-%m-%d %H:%M:%S")));

                    if !session.tags.is_empty() {
                        ui::print_info(&format!("Tags: {}", session.tags.join(", ")));
                    }

                    println!();
                    ui::print_info("Recent messages:");
                    let recent_messages = session.messages.iter().rev().take(5).collect::<Vec<_>>();
                    for (i, msg) in recent_messages.into_iter().rev().enumerate() {
                        let role_icon = match msg.role.as_str() {
                            "user" => "👤",
                            "assistant" => "🤖",
                            _ => "💬"
                        };
                        println!("{}. {}: {}...", i + 1, role_icon, &msg.content.chars().take(100).collect::<String>());
                    }

                    if session.messages.len() > 5 {
                        ui::print_info(&format!("... and {} more messages", session.messages.len() - 5));
                    }
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to load session '{}': {}", id, e));
                    return Err(error::CliError::api_error(&format!("Session loading failed: {}", e)).into());
                }
            }
        }

        SessionAction::Chat { id, message, context_files, no_history } => {
            ui::print_command_start("SESSION", &format!("{} Starting chat with session", ui::CHAT));

            match session_manager.load_session(&id) {
                Ok(session) => {
                    ui::print_info(&format!("Loaded session: {}", session.name));
                    ui::print_info(&format!("Messages in session: {}", session.messages.len()));

                    // Convert session messages to chat messages format
                    let mut session_messages = session.messages.into_iter()
                        .map(|msg| executor::ChatMessage {
                            role: msg.role,
                            content: msg.content,
                        })
                        .collect::<Vec<_>>();

                    // Add initial message if provided
                    if let Some(initial_msg) = message {
                        session_messages.push(executor::ChatMessage {
                            role: "user".to_string(),
                            content: initial_msg,
                        });
                    }

                    // Call the chat handler with session messages
                    commands::chat::handle_chat_with_session(
                        session_messages,
                        Some(context_files),
                        no_history,
                        &id,
                        &session_manager,
                        config
                    ).await?;
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to load session '{}': {}", id, e));
                    return Err(error::CliError::api_error(&format!("Session loading failed: {}", e)).into());
                }
            }
        }

        SessionAction::Rename { id, name } => {
            ui::print_command_start("SESSION", &format!("{} Renaming session", ui::EDIT));

            match session_manager.update_session_name(&id, name.clone()) {
                Ok(_) => {
                    ui::print_success(&format!("Session '{}' renamed to '{}'", id, name));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to rename session: {}", e));
                    return Err(error::CliError::api_error(&format!("Session rename failed: {}", e)).into());
                }
            }
        }

        SessionAction::UpdateDescription { id, description } => {
            ui::print_command_start("SESSION", &format!("{} Updating session description", ui::EDIT));

            match session_manager.update_session_description(&id, Some(description.clone())) {
                Ok(_) => {
                    ui::print_success(&format!("Session '{}' description updated", id));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to update session description: {}", e));
                    return Err(error::CliError::api_error(&format!("Session description update failed: {}", e)).into());
                }
            }
        }

        SessionAction::AddTag { id, tag } => {
            ui::print_command_start("SESSION", &format!("{} Adding tag to session", ui::TAG));

            match session_manager.add_session_tag(&id, tag.clone()) {
                Ok(_) => {
                    ui::print_success(&format!("Tag '{}' added to session '{}'", tag, id));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to add tag to session: {}", e));
                    return Err(error::CliError::api_error(&format!("Session tag addition failed: {}", e)).into());
                }
            }
        }

        SessionAction::RemoveTag { id, tag } => {
            ui::print_command_start("SESSION", &format!("{} Removing tag from session", ui::TAG));

            match session_manager.remove_session_tag(&id, &tag) {
                Ok(_) => {
                    ui::print_success(&format!("Tag '{}' removed from session '{}'", tag, id));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to remove tag from session: {}", e));
                    return Err(error::CliError::api_error(&format!("Session tag removal failed: {}", e)).into());
                }
            }
        }

        SessionAction::Delete { id } => {
            ui::print_command_start("SESSION", &format!("{} Deleting session", ui::DELETE));

            // Confirm deletion
            if !dialoguer::Confirm::new()
                .with_prompt(&format!("Are you sure you want to delete session '{}'?", id))
                .default(false)
                .interact()
                .unwrap_or(false)
            {
                ui::print_info("Session deletion cancelled.");
                return Ok(());
            }

            match session_manager.delete_session(&id) {
                Ok(_) => {
                    ui::print_success(&format!("Session '{}' deleted successfully", id));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to delete session: {}", e));
                    return Err(error::CliError::api_error(&format!("Session deletion failed: {}", e)).into());
                }
            }
        }

        SessionAction::Clear { id } => {
            ui::print_command_start("SESSION", &format!("{} Clearing session messages", ui::CLEAR));

            // Confirm clearing
            if !dialoguer::Confirm::new()
                .with_prompt(&format!("Are you sure you want to clear all messages from session '{}'?", id))
                .default(false)
                .interact()
                .unwrap_or(false)
            {
                ui::print_info("Session clearing cancelled.");
                return Ok(());
            }

            match session_manager.clear_session_messages(&id) {
                Ok(_) => {
                    ui::print_success(&format!("All messages cleared from session '{}'", id));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to clear session messages: {}", e));
                    return Err(error::CliError::api_error(&format!("Session clearing failed: {}", e)).into());
                }
            }
        }

        SessionAction::Search { query } => {
            ui::print_command_start("SESSION", &format!("{} Searching sessions", ui::SEARCH));

            match session_manager.search_sessions(&query) {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        ui::print_info(&format!("No sessions found matching '{}'", query));
                        return Ok(());
                    }

                    ui::print_info(&format!("Found {} session(s) matching '{}':", sessions.len(), query));
                    println!();

                    for session in sessions {
                        let desc = session.description
                            .as_ref()
                            .map(|d| format!(" - {}", d))
                            .unwrap_or_default();

                        println!("📝 {} ({})", session.name, session.id);
                        println!("   Messages: {}, Created: {}{}",
                                session.message_count,
                                session.created_at.format("%Y-%m-%d %H:%M"),
                                desc);
                        println!();
                    }
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to search sessions: {}", e));
                    return Err(error::CliError::api_error(&format!("Session search failed: {}", e)).into());
                }
            }
        }

        SessionAction::Export { id, output } => {
            ui::print_command_start("SESSION", &format!("{} Exporting session", ui::EXPORT));

            match session_manager.export_session(&id, &output) {
                Ok(_) => {
                    ui::print_success(&format!("Session '{}' exported to '{}'", id, output));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to export session: {}", e));
                    return Err(error::CliError::api_error(&format!("Session export failed: {}", e)).into());
                }
            }
        }

        SessionAction::Import { input, name } => {
            ui::print_command_start("SESSION", &format!("{} Importing session", ui::IMPORT));

            match session_manager.import_session(&input, name.clone()) {
                Ok(session) => {
                    let final_name = name.unwrap_or_else(|| session.name.clone());
                    ui::print_success(&format!("Session '{}' imported with ID: {}", final_name, session.id));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to import session: {}", e));
                    return Err(error::CliError::api_error(&format!("Session import failed: {}", e)).into());
                }
            }
        }
    }

    Ok(())
}