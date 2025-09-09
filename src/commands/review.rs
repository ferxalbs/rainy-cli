use crate::commands::chat::handle_chat_command;
use miette::Result;
use rainy_cli::{config::Config, ui};
use std::path::PathBuf;

use rainy_cli::utils::context;

pub async fn handle_review_command(
    paths: Vec<PathBuf>,
    focus: Option<String>,
    git: bool,
    _git_ref: String,
    config: &Config,
) -> Result<()> {
    ui::print_command_start(
        "REVIEW",
        &format!("{} Forwarding to Agentic Chat...", ui::FORWARD),
    );

    let mut initial_message = "Please review my code.".to_string();

    if git {
        initial_message = "Please review the changes in my current git context.".to_string();
    } else if !paths.is_empty() {
        let paths_str: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
        let full_context = context::collect_context_from_paths(&paths)
            .map_err(|e| rainy_cli::error::CliError::context_error("Failed to collect context from paths", e))?;
        initial_message = format!(
            "Please review the code in the following files: {}.\n\nHere is the combined content of the files:\n\n{}",
            paths_str.join(", "),
            full_context
        );
    }

    if let Some(f) = focus {
        initial_message = format!("{} My main focus is on {}.", initial_message, f);
    }

    ui::print_info("The `review` command is now handled by the agentic chat.");
    ui::print_info("You will be dropped into a chat session with your request pre-filled.");

    handle_chat_command(Some(initial_message), Some(paths), config).await
}
