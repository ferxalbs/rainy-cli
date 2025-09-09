use crate::commands::chat::handle_chat_command;
use miette::Result;
use rainy_cli::{config::Config, ui};
use std::path::PathBuf;

use rainy_cli::utils::context;

pub async fn handle_analyze_command(
    paths: Vec<PathBuf>,
    analysis_type: String,
    config: &Config,
) -> Result<()> {
    ui::print_command_start(
        "ANALYZE",
        &format!("{} Forwarding to Agentic Chat...", ui::FORWARD),
    );

    let full_context = context::collect_context_from_paths(&paths)
        .map_err(|e| rainy_cli::error::CliError::context_error("Failed to collect context from paths", e))?;
    let paths_str: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();

    let initial_message = format!(
        "Please perform a {} analysis on the files: {}.\n\nHere is the combined content of the files:\n\n{}",
        analysis_type,
        paths_str.join(", "),
        full_context,
    );

    ui::print_info("The `analyze` command is now handled by the agentic chat.");
    ui::print_info("You will be dropped into a chat session with your request pre-filled.");

    // Call the chat handler with a pre-filled message and the file paths for context
    handle_chat_command(Some(initial_message), Some(paths), false, config).await
}
