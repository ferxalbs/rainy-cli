use crate::commands::chat::handle_chat_command;
use miette::Result;
use rainy_cli::{config::Config, ui};
use std::path::PathBuf;

pub async fn handle_analyze_command(
    path: PathBuf,
    analysis_type: String,
    config: &Config,
) -> Result<()> {
    ui::print_command_start(
        "ANALYZE",
        &format!("{} Forwarding to Agentic Chat...", ui::FORWARD),
    );

    let initial_message = format!(
        "Please perform a {} analysis on the file located at: {}",
        analysis_type,
        path.display()
    );

    ui::print_info("The `analyze` command is now handled by the agentic chat.");
    ui::print_info("You will be dropped into a chat session with your request pre-filled.");
    
    // Call the chat handler with a pre-filled message
    handle_chat_command(Some(initial_message), config).await
}
